use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::a1_refs::{
    A1Reference, A1ReferenceNotation, format_relative_target, parse_a1_reference,
};
use crate::functions::adapters::{expand_lookup_vector_arg, prepare_arg_values_only};
use crate::functions::xmatch::{XmatchEvalError, eval_xmatch_adapter_prepared};
use crate::resolver::{
    ReferenceSystemProvider, enumerate_reference_values, materialize_resolved_reference_values,
    resolve_eval_value,
};
use crate::value::CalcValue;
use crate::value::{
    ArrayShape, CalcArray, CoreValue, ReferenceKind, ReferenceLike, WorksheetErrorCode,
};

pub const XLOOKUP_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.XLOOKUP",
    arity: Arity { min: 3, max: 6 },
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::WorkbookState,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: ArgPreparationProfile::RefsVisibleInAdapter,
    coercion_lift_profile: CoercionLiftProfile::Custom,
    kernel_signature_class: KernelSignatureClass::LookupMatch,
    fec_dependency_profile: FecDependencyProfile::RefOnly,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
    real_result_policy: FunctionMeta::DEFAULT_REAL_RESULT_POLICY,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VectorOrientation {
    Horizontal,
    Vertical,
    Scalar,
}

#[derive(Debug, Clone, PartialEq)]
enum ReturnItem {
    Eval(CalcValue),
    Reference(ReferenceLike),
    MissingArg,
    EmptyCell,
}

#[derive(Debug, Clone, PartialEq)]
enum ReturnSelection {
    Scalar(ReturnItem),
    Vector {
        items: Vec<ReturnItem>,
        orientation: VectorOrientation,
    },
    ArrayValue(CalcArray),
    ReferenceArea(A1Reference),
}

#[derive(Debug, Clone, PartialEq)]
pub enum XlookupEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    LengthMismatch {
        lookup_len: usize,
        return_len: usize,
    },
    OrientationMismatch,
    Coercion(CoercionError),
    InvalidMatchMode(f64),
    InvalidSearchMode(f64),
    NotAvailable,
}

fn map_xmatch_error(err: XmatchEvalError) -> XlookupEvalError {
    match err {
        XmatchEvalError::ArityMismatch {
            expected_min,
            expected_max,
            actual,
        } => XlookupEvalError::ArityMismatch {
            expected_min,
            expected_max,
            actual,
        },
        XmatchEvalError::EmptyLookupArray => XlookupEvalError::NotAvailable,
        XmatchEvalError::MissingArg => XlookupEvalError::Coercion(CoercionError::MissingArg),
        XmatchEvalError::EmptyCell => XlookupEvalError::Coercion(CoercionError::EmptyCell),
        XmatchEvalError::Coercion(err) => XlookupEvalError::Coercion(err),
        XmatchEvalError::UnsupportedValueKind(kind) => {
            XlookupEvalError::Coercion(CoercionError::UnsupportedValueKind(kind))
        }
        XmatchEvalError::InvalidMatchMode(n) => XlookupEvalError::InvalidMatchMode(n),
        XmatchEvalError::InvalidSearchMode(n) => XlookupEvalError::InvalidSearchMode(n),
        XmatchEvalError::UnsupportedMatchModeForSeed(_)
        | XmatchEvalError::UnsupportedSearchModeForSeed(_) => {
            XlookupEvalError::Coercion(CoercionError::UnsupportedValueKind("xmatch_seed_mode"))
        }
        XmatchEvalError::NotAvailable => XlookupEvalError::NotAvailable,
    }
}

fn orientation_from_shape(rows: usize, cols: usize) -> Result<VectorOrientation, XlookupEvalError> {
    if rows > 1 && cols > 1 {
        return Err(XlookupEvalError::Coercion(
            CoercionError::UnsupportedValueKind("two_dimensional_array"),
        ));
    }
    Ok(if rows > 1 {
        VectorOrientation::Vertical
    } else if cols > 1 {
        VectorOrientation::Horizontal
    } else {
        VectorOrientation::Scalar
    })
}

fn cell_to_return_item(cell: &CalcValue) -> ReturnItem {
    match cell.core() {
        CoreValue::Number(n) => ReturnItem::Eval(CalcValue::number(*n)),
        CoreValue::Text(t) => ReturnItem::Eval(CalcValue::text(t.clone())),
        CoreValue::Logical(b) => ReturnItem::Eval(CalcValue::logical(*b)),
        CoreValue::Error(code) => ReturnItem::Eval(CalcValue::error(*code)),
        CoreValue::Empty | CoreValue::Missing => ReturnItem::EmptyCell,
        CoreValue::Array(_) | CoreValue::Reference(_) => {
            ReturnItem::Eval(CalcValue::error(WorksheetErrorCode::Value))
        }
    }
}

fn reference_from_cell(parsed: &A1Reference, row: usize, col: usize) -> ReferenceLike {
    let single = A1Reference {
        prefix: parsed.prefix.clone(),
        start_row: parsed.start_row + row,
        end_row: parsed.start_row + row,
        start_col: parsed.start_col + col,
        end_col: parsed.start_col + col,
        notation: A1ReferenceNotation::Rect,
    };
    let target = format_relative_target(&single).expect("single cell reference is formattable");
    ReferenceLike::new(ReferenceKind::A1, target)
}

fn flatten_reference_vector(
    reference: &ReferenceLike,
) -> Result<(Vec<ReturnItem>, VectorOrientation), XlookupEvalError> {
    if let Some(parsed) = parse_a1_reference(reference.target()) {
        let orientation = orientation_from_shape(parsed.height(), parsed.width())?;
        let mut items = Vec::with_capacity(parsed.height() * parsed.width());
        for row in 0..parsed.height() {
            for col in 0..parsed.width() {
                items.push(ReturnItem::Reference(reference_from_cell(
                    &parsed, row, col,
                )));
            }
        }
        Ok((items, orientation))
    } else {
        Ok((
            vec![ReturnItem::Reference(reference.clone())],
            VectorOrientation::Scalar,
        ))
    }
}

fn flatten_return_eval(
    value: &CalcValue,
) -> Result<(Vec<ReturnItem>, VectorOrientation), XlookupEvalError> {
    match value.core() {
        CoreValue::Array(array) => {
            let shape = array.shape();
            let orientation = orientation_from_shape(shape.rows, shape.cols)?;
            Ok((
                array.iter_row_major().map(cell_to_return_item).collect(),
                orientation,
            ))
        }
        CoreValue::Reference(reference) => flatten_reference_vector(reference),
        _ => Ok((
            vec![ReturnItem::Eval(value.clone())],
            VectorOrientation::Scalar,
        )),
    }
}

fn expand_return_arg(
    arg: &CalcValue,
) -> Result<(Vec<ReturnItem>, VectorOrientation), XlookupEvalError> {
    match arg.core() {
        CoreValue::Reference(reference) => flatten_reference_vector(reference),
        CoreValue::Missing => Ok((vec![ReturnItem::MissingArg], VectorOrientation::Scalar)),
        CoreValue::Empty => Ok((vec![ReturnItem::EmptyCell], VectorOrientation::Scalar)),
        _ => flatten_return_eval(arg),
    }
}

fn materialize_return_item(item: &ReturnItem) -> CalcValue {
    match item {
        ReturnItem::Eval(value) => value.clone(),
        ReturnItem::Reference(reference) => CalcValue::reference(reference.clone()),
        ReturnItem::MissingArg => CalcValue::error(WorksheetErrorCode::NA),
        ReturnItem::EmptyCell => CalcValue::number(0.0),
    }
}

fn materialize_fallback(arg: &CalcValue) -> CalcValue {
    match arg.core() {
        CoreValue::Reference(reference) => CalcValue::reference(reference.clone()),
        CoreValue::Missing => CalcValue::error(WorksheetErrorCode::NA),
        CoreValue::Empty => CalcValue::number(0.0),
        _ => arg.clone(),
    }
}

fn materialize_cell_value(cell: &CalcValue) -> CalcValue {
    match cell.core() {
        CoreValue::Number(n) => CalcValue::number(*n),
        CoreValue::Text(t) => CalcValue::text(t.clone()),
        CoreValue::Logical(b) => CalcValue::logical(*b),
        CoreValue::Error(code) => CalcValue::error(*code),
        CoreValue::Empty | CoreValue::Missing => CalcValue::number(0.0),
        CoreValue::Array(_) | CoreValue::Reference(_) => {
            CalcValue::error(WorksheetErrorCode::Value)
        }
    }
}

fn prepared_from_lookup_value_cell(cell: &CalcValue) -> CalcValue {
    match cell.core() {
        CoreValue::Number(n) => CalcValue::number(*n),
        CoreValue::Text(t) => CalcValue::text(t.clone()),
        CoreValue::Logical(b) => CalcValue::logical(*b),
        CoreValue::Error(code) => CalcValue::error(*code),
        CoreValue::Empty | CoreValue::Missing => CalcValue::empty(),
        CoreValue::Array(_) | CoreValue::Reference(_) => {
            CalcValue::error(WorksheetErrorCode::Value)
        }
    }
}

fn top_left_array_cell(array: &CalcArray) -> CalcValue {
    array
        .get(0, 0)
        .cloned()
        .unwrap_or(CalcValue::error(WorksheetErrorCode::Value))
}

fn materialized_eval_value_to_array_cell(value: CalcValue) -> CalcValue {
    match value.core() {
        CoreValue::Number(_) | CoreValue::Text(_) | CoreValue::Logical(_) | CoreValue::Error(_) => {
            value
        }
        CoreValue::Array(array) => top_left_array_cell(array),
        CoreValue::Reference(_) => CalcValue::error(WorksheetErrorCode::Value),
        _ => CalcValue::error(WorksheetErrorCode::Value),
    }
}

fn eval_value_to_array_cell(
    value: CalcValue,
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> CalcValue {
    match value.core() {
        CoreValue::Reference(reference) => resolve_eval_value(resolver, reference)
            .map(materialized_eval_value_to_array_cell)
            .unwrap_or(CalcValue::error(WorksheetErrorCode::Value)),
        _ => materialized_eval_value_to_array_cell(value),
    }
}

fn infer_selection_orientation(shape: ArrayShape) -> Option<VectorOrientation> {
    if shape.rows == 1 && shape.cols == 1 {
        Some(VectorOrientation::Scalar)
    } else if shape.rows == 1 {
        Some(VectorOrientation::Horizontal)
    } else if shape.cols == 1 {
        Some(VectorOrientation::Vertical)
    } else {
        None
    }
}

fn return_selection_from_array(array: &CalcArray) -> ReturnSelection {
    let shape = array.shape();
    if infer_selection_orientation(shape).is_none() {
        ReturnSelection::ArrayValue(array.clone())
    } else {
        let items: Vec<ReturnItem> = array.iter_row_major().map(cell_to_return_item).collect();
        if items.len() == 1 {
            ReturnSelection::Scalar(items.into_iter().next().expect("len checked"))
        } else {
            ReturnSelection::Vector {
                items,
                orientation: infer_selection_orientation(shape)
                    .expect("non-matrix array has vector orientation"),
            }
        }
    }
}

fn prepare_return_selection(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<ReturnSelection, XlookupEvalError> {
    if args.len() != 1 {
        let mut items = Vec::new();
        for arg in args {
            let (expanded, _) = expand_return_arg(arg)?;
            items.extend(expanded);
        }
        return Ok(if items.len() == 1 {
            ReturnSelection::Scalar(items.into_iter().next().expect("len checked"))
        } else {
            ReturnSelection::Vector {
                items,
                orientation: VectorOrientation::Horizontal,
            }
        });
    }

    match args[0].core() {
        CoreValue::Reference(reference) => {
            if let Some(values) = enumerate_reference_values(resolver, reference)
                .map_err(CoercionError::RefResolution)
                .map_err(XlookupEvalError::Coercion)?
            {
                let array = materialize_resolved_reference_values(&values)
                    .map_err(CoercionError::RefResolution)
                    .map_err(XlookupEvalError::Coercion)?;
                return Ok(return_selection_from_array(&array));
            }
            if let Some(parsed) = parse_a1_reference(reference.target()) {
                if parsed.height() > 1 && parsed.width() > 1 {
                    Ok(ReturnSelection::ReferenceArea(parsed))
                } else {
                    let (items, _) = flatten_reference_vector(reference)?;
                    Ok(if items.len() == 1 {
                        ReturnSelection::Scalar(items.into_iter().next().expect("len checked"))
                    } else {
                        ReturnSelection::Vector {
                            items,
                            orientation: infer_selection_orientation(ArrayShape {
                                rows: parsed.height(),
                                cols: parsed.width(),
                            })
                            .expect("non-matrix reference has vector orientation"),
                        }
                    })
                }
            } else {
                Ok(ReturnSelection::Scalar(ReturnItem::Reference(
                    reference.clone(),
                )))
            }
        }
        CoreValue::Array(array) => Ok(return_selection_from_array(array)),
        CoreValue::Missing => Ok(ReturnSelection::Scalar(ReturnItem::MissingArg)),
        CoreValue::Empty => Ok(ReturnSelection::Scalar(ReturnItem::EmptyCell)),
        _ => Ok(ReturnSelection::Scalar(ReturnItem::Eval(args[0].clone()))),
    }
}

fn selection_len(
    selection: &ReturnSelection,
    lookup_orientation: Option<VectorOrientation>,
) -> Result<usize, XlookupEvalError> {
    match selection {
        ReturnSelection::Scalar(_) => Ok(1),
        ReturnSelection::Vector { items, orientation } => {
            if let Some(lookup_orientation) = lookup_orientation
                && lookup_orientation != VectorOrientation::Scalar
                && *orientation != VectorOrientation::Scalar
                && *orientation != lookup_orientation
            {
                return Err(XlookupEvalError::OrientationMismatch);
            }
            Ok(items.len())
        }
        ReturnSelection::ArrayValue(array) => {
            let shape = array.shape();
            match lookup_orientation {
                Some(VectorOrientation::Vertical) => Ok(shape.rows),
                Some(VectorOrientation::Horizontal) => Ok(shape.cols),
                Some(VectorOrientation::Scalar) => Ok(1),
                None => Err(XlookupEvalError::OrientationMismatch),
            }
        }
        ReturnSelection::ReferenceArea(reference) => match lookup_orientation {
            Some(VectorOrientation::Vertical) => Ok(reference.height()),
            Some(VectorOrientation::Horizontal) => Ok(reference.width()),
            Some(VectorOrientation::Scalar) => Ok(1),
            None => Err(XlookupEvalError::OrientationMismatch),
        },
    }
}

fn column_array(array: &CalcArray, col_index: usize) -> CalcArray {
    let shape = array.shape();
    let mut cells = Vec::with_capacity(shape.rows);
    for row in 0..shape.rows {
        cells.push(
            array
                .get(row, col_index)
                .cloned()
                .expect("validated column selection"),
        );
    }
    CalcArray::new(
        ArrayShape {
            rows: shape.rows,
            cols: 1,
        },
        cells,
    )
    .expect("column slice dimensions validated")
}

fn select_reference_slice(
    reference: &A1Reference,
    lookup_orientation: Option<VectorOrientation>,
    index: usize,
) -> Result<CalcValue, XlookupEvalError> {
    let selected = match lookup_orientation.ok_or(XlookupEvalError::OrientationMismatch)? {
        VectorOrientation::Vertical => A1Reference {
            prefix: reference.prefix.clone(),
            start_row: reference.start_row + index,
            end_row: reference.start_row + index,
            start_col: reference.start_col,
            end_col: reference.end_col,
            notation: A1ReferenceNotation::Rect,
        },
        VectorOrientation::Horizontal => A1Reference {
            prefix: reference.prefix.clone(),
            start_row: reference.start_row,
            end_row: reference.end_row,
            start_col: reference.start_col + index,
            end_col: reference.start_col + index,
            notation: A1ReferenceNotation::Rect,
        },
        VectorOrientation::Scalar => reference.clone(),
    };
    let target = format_relative_target(&selected).expect("selected reference remains formattable");
    Ok(CalcValue::reference(ReferenceLike::new(
        if selected.height() == 1 && selected.width() == 1 {
            ReferenceKind::A1
        } else {
            ReferenceKind::Area
        },
        target,
    )))
}

fn select_return_value(
    selection: &ReturnSelection,
    lookup_orientation: Option<VectorOrientation>,
    index: usize,
) -> Result<CalcValue, XlookupEvalError> {
    match selection {
        ReturnSelection::Scalar(item) => Ok(materialize_return_item(item)),
        ReturnSelection::Vector { items, .. } => items
            .get(index)
            .map(materialize_return_item)
            .ok_or(XlookupEvalError::NotAvailable),
        ReturnSelection::ArrayValue(array) => {
            let shape = array.shape();
            match lookup_orientation.ok_or(XlookupEvalError::OrientationMismatch)? {
                VectorOrientation::Vertical => {
                    if shape.cols == 1 {
                        Ok(materialize_cell_value(
                            array.get(index, 0).expect("validated row selection"),
                        ))
                    } else {
                        Ok(CalcValue::array(
                            CalcArray::new(
                                ArrayShape {
                                    rows: 1,
                                    cols: shape.cols,
                                },
                                array
                                    .row_slice(index)
                                    .expect("validated row selection")
                                    .to_vec(),
                            )
                            .expect("row slice dimensions validated"),
                        ))
                    }
                }
                VectorOrientation::Horizontal => {
                    if shape.rows == 1 {
                        Ok(materialize_cell_value(
                            array.get(0, index).expect("validated column selection"),
                        ))
                    } else {
                        Ok(CalcValue::array(column_array(array, index)))
                    }
                }
                VectorOrientation::Scalar => Ok(materialize_cell_value(
                    array.get(0, 0).expect("scalar selection"),
                )),
            }
        }
        ReturnSelection::ReferenceArea(reference) => {
            select_reference_slice(reference, lookup_orientation, index)
        }
    }
}

fn select_reference_scalar_cell(
    reference: &A1Reference,
    lookup_orientation: VectorOrientation,
    index: usize,
) -> ReferenceLike {
    let (row, col) = match lookup_orientation {
        VectorOrientation::Vertical => (index, 0),
        VectorOrientation::Horizontal => (0, index),
        VectorOrientation::Scalar => (0, 0),
    };
    reference_from_cell(reference, row, col)
}

fn select_return_array_cell(
    selection: &ReturnSelection,
    lookup_orientation: Option<VectorOrientation>,
    index: usize,
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> CalcValue {
    match selection {
        ReturnSelection::Scalar(item) => {
            eval_value_to_array_cell(materialize_return_item(item), resolver)
        }
        ReturnSelection::Vector { items, .. } => items
            .get(index)
            .map(materialize_return_item)
            .map(|value| eval_value_to_array_cell(value, resolver))
            .unwrap_or(CalcValue::error(WorksheetErrorCode::NA)),
        ReturnSelection::ArrayValue(array) => {
            let selected = match lookup_orientation.unwrap_or(VectorOrientation::Scalar) {
                VectorOrientation::Vertical => array.get(index, 0),
                VectorOrientation::Horizontal => array.get(0, index),
                VectorOrientation::Scalar => array.get(0, 0),
            };
            selected
                .cloned()
                .unwrap_or(CalcValue::error(WorksheetErrorCode::Value))
        }
        ReturnSelection::ReferenceArea(reference) => {
            let Some(lookup_orientation) = lookup_orientation else {
                return CalcValue::error(WorksheetErrorCode::Value);
            };
            let selected = select_reference_scalar_cell(reference, lookup_orientation, index);
            eval_value_to_array_cell(CalcValue::reference(selected), resolver)
        }
    }
}

fn prepare_lookup_vector(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<(Vec<CalcValue>, Option<VectorOrientation>), XlookupEvalError> {
    let mut prepared = Vec::new();
    let mut orientation = None;
    if args.len() == 1 {
        match args.first().expect("len checked").core() {
            CoreValue::Array(array) => {
                let shape = array.shape();
                orientation = Some(orientation_from_shape(shape.rows, shape.cols)?);
            }
            CoreValue::Reference(reference) => {
                if let Some(values) = enumerate_reference_values(resolver, reference)
                    .map_err(CoercionError::RefResolution)
                    .map_err(XlookupEvalError::Coercion)?
                {
                    let array = materialize_resolved_reference_values(&values)
                        .map_err(CoercionError::RefResolution)
                        .map_err(XlookupEvalError::Coercion)?;
                    let shape = array.shape();
                    orientation = Some(orientation_from_shape(shape.rows, shape.cols)?);
                } else {
                    let resolved = resolve_eval_value(resolver, reference)
                        .map_err(CoercionError::RefResolution)
                        .map_err(XlookupEvalError::Coercion)?;
                    if let CoreValue::Array(array) = resolved.core() {
                        let shape = array.shape();
                        orientation = Some(orientation_from_shape(shape.rows, shape.cols)?);
                    }
                }
            }
            _ => {}
        }
    }

    for arg in args {
        prepared
            .extend(expand_lookup_vector_arg(arg, resolver).map_err(XlookupEvalError::Coercion)?);
    }
    Ok((prepared, orientation))
}

fn xlookup_lookup_value_array_result_to_cell(
    result: Result<f64, XmatchEvalError>,
    return_selection: &ReturnSelection,
    lookup_orientation: Option<VectorOrientation>,
    if_not_found: Option<&CalcValue>,
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> CalcValue {
    match result {
        Ok(index) => {
            let index = index as usize - 1;
            select_return_array_cell(return_selection, lookup_orientation, index, resolver)
        }
        Err(XmatchEvalError::NotAvailable) => {
            if let Some(if_not_found) = if_not_found {
                return eval_value_to_array_cell(materialize_fallback(if_not_found), resolver);
            }
            CalcValue::error(WorksheetErrorCode::NA)
        }
        Err(err) => CalcValue::error(map_xlookup_error_to_ws(&map_xmatch_error(err))),
    }
}

fn eval_xlookup_lookup_value_array(
    lookup_value_array: &CalcArray,
    lookup_array: &[CalcValue],
    return_selection: &ReturnSelection,
    lookup_orientation: Option<VectorOrientation>,
    if_not_found: Option<&CalcValue>,
    match_mode: Option<&CalcValue>,
    search_mode: Option<&CalcValue>,
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> CalcValue {
    let cells = lookup_value_array
        .iter_row_major()
        .map(prepared_from_lookup_value_cell)
        .map(|lookup_value| {
            let result =
                eval_xmatch_adapter_prepared(&lookup_value, lookup_array, match_mode, search_mode);
            xlookup_lookup_value_array_result_to_cell(
                result,
                return_selection,
                lookup_orientation,
                if_not_found,
                resolver,
            )
        })
        .collect();
    CalcValue::array(
        CalcArray::new(lookup_value_array.shape(), cells)
            .expect("lookup-value array result preserves input shape"),
    )
}

pub fn eval_xlookup_surface(
    lookup_value: &CalcValue,
    lookup_array: &[CalcValue],
    return_array: &[CalcValue],
    if_not_found: Option<&CalcValue>,
    match_mode: Option<&CalcValue>,
    search_mode: Option<&CalcValue>,
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, XlookupEvalError> {
    let argc = 3
        + usize::from(if_not_found.is_some())
        + usize::from(match_mode.is_some())
        + usize::from(search_mode.is_some());
    if !XLOOKUP_META.arity.accepts(argc) {
        return Err(XlookupEvalError::ArityMismatch {
            expected_min: XLOOKUP_META.arity.min,
            expected_max: XLOOKUP_META.arity.max,
            actual: argc,
        });
    }

    let prepared_lookup_value =
        prepare_arg_values_only(lookup_value, resolver).map_err(XlookupEvalError::Coercion)?;
    let (prepared_lookup_array, lookup_orientation) =
        prepare_lookup_vector(lookup_array, resolver)?;
    let return_selection = prepare_return_selection(return_array, resolver)?;
    let return_len = selection_len(&return_selection, lookup_orientation)?;

    if prepared_lookup_array.len() != return_len {
        return Err(XlookupEvalError::LengthMismatch {
            lookup_len: prepared_lookup_array.len(),
            return_len,
        });
    }

    let prepared_match_mode = match_mode
        .map(|arg| prepare_arg_values_only(arg, resolver))
        .transpose()
        .map_err(XlookupEvalError::Coercion)?;
    let prepared_search_mode = search_mode
        .map(|arg| prepare_arg_values_only(arg, resolver))
        .transpose()
        .map_err(XlookupEvalError::Coercion)?;

    if let CoreValue::Array(lookup_value_array) = prepared_lookup_value.core() {
        return Ok(eval_xlookup_lookup_value_array(
            lookup_value_array,
            &prepared_lookup_array,
            &return_selection,
            lookup_orientation,
            if_not_found,
            prepared_match_mode.as_ref(),
            prepared_search_mode.as_ref(),
            resolver,
        ));
    }

    let index = match eval_xmatch_adapter_prepared(
        &prepared_lookup_value,
        &prepared_lookup_array,
        prepared_match_mode.as_ref(),
        prepared_search_mode.as_ref(),
    ) {
        Ok(index) => index as usize - 1,
        Err(XmatchEvalError::NotAvailable) => {
            if let Some(if_not_found) = if_not_found {
                return Ok(materialize_fallback(if_not_found));
            }
            return Err(XlookupEvalError::NotAvailable);
        }
        Err(err) => return Err(map_xmatch_error(err)),
    };

    select_return_value(&return_selection, lookup_orientation, index)
}

pub fn map_xlookup_error_to_ws(e: &XlookupEvalError) -> WorksheetErrorCode {
    match e {
        XlookupEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        XlookupEvalError::LengthMismatch { .. } => WorksheetErrorCode::Value,
        XlookupEvalError::OrientationMismatch => WorksheetErrorCode::Value,
        XlookupEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        XlookupEvalError::InvalidMatchMode(_) => WorksheetErrorCode::Value,
        XlookupEvalError::InvalidSearchMode(_) => WorksheetErrorCode::Value,
        XlookupEvalError::NotAvailable => WorksheetErrorCode::NA,
        XlookupEvalError::Coercion(_) => WorksheetErrorCode::Value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::ReferenceSystemCapabilities;
    use crate::value::{CalcArray, ExcelText};

    struct NoResolver;
    impl ReferenceSystemProvider for NoResolver {
        fn capabilities(&self) -> ReferenceSystemCapabilities {
            ReferenceSystemCapabilities::permissive_local()
        }

        fn dereference(
            &self,
            request: &crate::resolver::ReferenceDereferenceRequest,
        ) -> Result<CalcValue, crate::resolver::ReferenceResolutionError> {
            let reference = &request.reference;
            Err(
                crate::resolver::ReferenceResolutionError::UnresolvedReference {
                    target: reference.target().to_string(),
                },
            )
        }
    }

    fn text_arg(s: &str) -> CalcValue {
        CalcValue::text(ExcelText::from_utf16_code_units(s.encode_utf16().collect()))
    }

    #[test]
    fn eval_xlookup_exact_forward_returns_value() {
        let got = eval_xlookup_surface(
            &(CalcValue::number(2.0)),
            &[(CalcValue::number(1.0)), (CalcValue::number(2.0))],
            &[text_arg("one"), text_arg("two")],
            None,
            None,
            None,
            &NoResolver,
        );
        assert_eq!(got, Ok(text_arg("two").into_eval().unwrap()));
    }

    #[test]
    fn eval_xlookup_supports_wildcard_and_reverse_search() {
        let got = eval_xlookup_surface(
            &text_arg("a*"),
            &[text_arg("abc"), text_arg("ade")],
            &[text_arg("first"), text_arg("last")],
            None,
            Some(&(CalcValue::number(2.0))),
            Some(&(CalcValue::number(-1.0))),
            &NoResolver,
        );
        assert_eq!(got, Ok(text_arg("last").into_eval().unwrap()));
    }

    #[test]
    fn eval_xlookup_returns_if_not_found_fallback() {
        let got = eval_xlookup_surface(
            &(CalcValue::number(9.0)),
            &[
                (CalcValue::number(1.0)),
                (CalcValue::number(2.0)),
                (CalcValue::number(3.0)),
            ],
            &[
                (CalcValue::number(10.0)),
                (CalcValue::number(20.0)),
                (CalcValue::number(30.0)),
            ],
            Some(&text_arg("nf")),
            None,
            None,
            &NoResolver,
        );
        assert_eq!(got, Ok(text_arg("nf").into_eval().unwrap()));
    }

    #[test]
    fn eval_xlookup_treats_missing_optional_modes_as_omitted() {
        let got = eval_xlookup_surface(
            &(CalcValue::number(2.0)),
            &[(CalcValue::array(
                CalcArray::from_rows(vec![vec![
                    CalcValue::number(1.0),
                    CalcValue::number(2.0),
                    CalcValue::number(3.0),
                ]])
                .unwrap(),
            ))],
            &[(CalcValue::array(
                CalcArray::from_rows(vec![vec![
                    CalcValue::number(10.0),
                    CalcValue::number(20.0),
                    CalcValue::number(30.0),
                ]])
                .unwrap(),
            ))],
            Some(&CalcValue::missing()),
            Some(&(CalcValue::number(0.0))),
            Some(&CalcValue::missing()),
            &NoResolver,
        );
        assert_eq!(got, Ok(CalcValue::number(20.0)));
    }

    #[test]
    fn eval_xlookup_omitted_or_blank_lookup_matches_true_blank_cells() {
        let lookup_array = [(CalcValue::array(
            CalcArray::from_rows(vec![vec![CalcValue::empty(), CalcValue::number(1.0)]]).unwrap(),
        ))];
        let return_array = [(CalcValue::array(
            CalcArray::from_rows(vec![vec![CalcValue::number(10.0), CalcValue::number(20.0)]])
                .unwrap(),
        ))];

        let omitted = eval_xlookup_surface(
            &CalcValue::missing(),
            &lookup_array,
            &return_array,
            None,
            None,
            None,
            &NoResolver,
        );
        assert_eq!(omitted, Ok(CalcValue::number(10.0)));

        let blank = eval_xlookup_surface(
            &CalcValue::empty(),
            &lookup_array,
            &return_array,
            None,
            None,
            None,
            &NoResolver,
        );
        assert_eq!(blank, Ok(CalcValue::number(10.0)));
    }

    #[test]
    fn eval_xlookup_empty_string_matches_formula_empty_not_true_blank() {
        let lookup_array = [(CalcValue::array(
            CalcArray::from_rows(vec![vec![
                CalcValue::text(ExcelText::from_utf16_code_units(Vec::new())),
                CalcValue::empty(),
            ]])
            .unwrap(),
        ))];
        let return_array = [(CalcValue::array(
            CalcArray::from_rows(vec![vec![CalcValue::number(10.0), CalcValue::number(20.0)]])
                .unwrap(),
        ))];

        let got = eval_xlookup_surface(
            &text_arg(""),
            &lookup_array,
            &return_array,
            None,
            None,
            None,
            &NoResolver,
        );
        assert_eq!(got, Ok(CalcValue::number(10.0)));
    }

    #[test]
    fn eval_xlookup_returns_zero_for_true_blank_return_cells() {
        let got = eval_xlookup_surface(
            &(CalcValue::number(1.0)),
            &[(CalcValue::array(
                CalcArray::from_rows(vec![vec![CalcValue::number(1.0), CalcValue::number(2.0)]])
                    .unwrap(),
            ))],
            &[(CalcValue::array(
                CalcArray::from_rows(vec![vec![CalcValue::empty(), CalcValue::number(20.0)]])
                    .unwrap(),
            ))],
            None,
            None,
            None,
            &NoResolver,
        );
        assert_eq!(got, Ok(CalcValue::number(0.0)));
    }

    #[test]
    fn eval_xlookup_binary_unsorted_exact_matches_empirical_lane() {
        let got = eval_xlookup_surface(
            &(CalcValue::number(2.0)),
            &[(CalcValue::array(
                CalcArray::from_rows(vec![vec![
                    CalcValue::number(3.0),
                    CalcValue::number(1.0),
                    CalcValue::number(4.0),
                    CalcValue::number(2.0),
                ]])
                .unwrap(),
            ))],
            &[(CalcValue::array(
                CalcArray::from_rows(vec![vec![
                    CalcValue::number(30.0),
                    CalcValue::number(10.0),
                    CalcValue::number(40.0),
                    CalcValue::number(20.0),
                ]])
                .unwrap(),
            ))],
            None,
            Some(&(CalcValue::number(0.0))),
            Some(&(CalcValue::number(2.0))),
            &NoResolver,
        );
        assert_eq!(got, Err(XlookupEvalError::NotAvailable));
    }

    #[test]
    fn eval_xlookup_supports_reference_return_from_single_area_argument() {
        let got = eval_xlookup_surface(
            &(CalcValue::number(2.0)),
            &[
                (CalcValue::number(1.0)),
                (CalcValue::number(2.0)),
                (CalcValue::number(3.0)),
            ],
            &[CalcValue::reference(ReferenceLike::new(
                ReferenceKind::Area,
                "B1:D1".to_string(),
            ))],
            None,
            None,
            None,
            &NoResolver,
        );
        assert_eq!(
            got,
            Ok(CalcValue::reference(ReferenceLike::new(
                ReferenceKind::A1,
                "C1".to_string()
            )))
        );
    }

    #[test]
    fn eval_xlookup_rejects_orientation_mismatch() {
        let got = eval_xlookup_surface(
            &(CalcValue::number(2.0)),
            &[(CalcValue::array(
                CalcArray::from_rows(vec![
                    vec![CalcValue::number(1.0)],
                    vec![CalcValue::number(2.0)],
                    vec![CalcValue::number(3.0)],
                ])
                .unwrap(),
            ))],
            &[(CalcValue::array(
                CalcArray::from_rows(vec![vec![
                    CalcValue::number(10.0),
                    CalcValue::number(20.0),
                    CalcValue::number(30.0),
                ]])
                .unwrap(),
            ))],
            None,
            None,
            None,
            &NoResolver,
        );
        assert_eq!(got, Err(XlookupEvalError::OrientationMismatch));
    }

    #[test]
    fn eval_xlookup_vertical_lookup_returns_matching_row_from_matrix() {
        let got = eval_xlookup_surface(
            &(CalcValue::number(2.0)),
            &[(CalcValue::array(
                CalcArray::from_rows(vec![
                    vec![CalcValue::number(1.0)],
                    vec![CalcValue::number(2.0)],
                    vec![CalcValue::number(3.0)],
                ])
                .unwrap(),
            ))],
            &[(CalcValue::array(
                CalcArray::from_rows(vec![
                    vec![CalcValue::number(10.0), CalcValue::number(11.0)],
                    vec![CalcValue::number(20.0), CalcValue::number(21.0)],
                    vec![CalcValue::number(30.0), CalcValue::number(31.0)],
                ])
                .unwrap(),
            ))],
            None,
            None,
            None,
            &NoResolver,
        );
        assert_eq!(
            got,
            Ok(CalcValue::array(
                CalcArray::from_rows(vec![
                    vec![CalcValue::number(20.0), CalcValue::number(21.0),]
                ])
                .unwrap()
            ))
        );
    }

    #[test]
    fn eval_xlookup_horizontal_lookup_returns_matching_reference_column() {
        let got = eval_xlookup_surface(
            &(CalcValue::number(2.0)),
            &[(CalcValue::array(
                CalcArray::from_rows(vec![vec![
                    CalcValue::number(1.0),
                    CalcValue::number(2.0),
                    CalcValue::number(3.0),
                ]])
                .unwrap(),
            ))],
            &[CalcValue::reference(ReferenceLike::new(
                ReferenceKind::Area,
                "B2:D4".to_string(),
            ))],
            None,
            None,
            None,
            &NoResolver,
        );
        assert_eq!(
            got,
            Ok(CalcValue::reference(ReferenceLike::new(
                ReferenceKind::Area,
                "C2:C4".to_string()
            )))
        );
    }

    #[test]
    fn eval_xlookup_spills_array_lookup_value_results() {
        let got = eval_xlookup_surface(
            &(CalcValue::array(
                CalcArray::from_rows(vec![vec![
                    CalcValue::number(1.0),
                    CalcValue::number(2.0),
                    CalcValue::number(3.0),
                ]])
                .unwrap(),
            )),
            &[(CalcValue::array(
                CalcArray::from_rows(vec![vec![
                    CalcValue::number(2.0),
                    CalcValue::number(4.0),
                    CalcValue::number(6.0),
                    CalcValue::number(8.0),
                ]])
                .unwrap(),
            ))],
            &[(CalcValue::array(
                CalcArray::from_rows(vec![vec![
                    CalcValue::number(20.0),
                    CalcValue::number(40.0),
                    CalcValue::number(60.0),
                    CalcValue::number(80.0),
                ]])
                .unwrap(),
            ))],
            None,
            None,
            None,
            &NoResolver,
        );
        assert_eq!(
            got,
            Ok(CalcValue::array(
                CalcArray::from_rows(vec![vec![
                    CalcValue::error(WorksheetErrorCode::NA),
                    CalcValue::number(20.0),
                    CalcValue::error(WorksheetErrorCode::NA),
                ]])
                .unwrap()
            ))
        );
    }

    #[test]
    fn eval_xlookup_array_lookup_value_preserves_shape_and_uses_fallback_top_left() {
        let got = eval_xlookup_surface(
            &(CalcValue::array(
                CalcArray::from_rows(vec![
                    vec![CalcValue::number(1.0), CalcValue::number(2.0)],
                    vec![CalcValue::number(3.0), CalcValue::number(4.0)],
                ])
                .unwrap(),
            )),
            &[(CalcValue::array(
                CalcArray::from_rows(vec![vec![
                    CalcValue::number(2.0),
                    CalcValue::number(4.0),
                    CalcValue::number(6.0),
                    CalcValue::number(8.0),
                ]])
                .unwrap(),
            ))],
            &[(CalcValue::array(
                CalcArray::from_rows(vec![vec![
                    CalcValue::number(20.0),
                    CalcValue::number(40.0),
                    CalcValue::number(60.0),
                    CalcValue::number(80.0),
                ]])
                .unwrap(),
            ))],
            Some(
                &(CalcValue::array(
                    CalcArray::from_rows(vec![vec![
                        CalcValue::text(ExcelText::from_interop_assignment("NF")),
                        CalcValue::text(ExcelText::from_interop_assignment("ignored")),
                    ]])
                    .unwrap(),
                )),
            ),
            None,
            None,
            &NoResolver,
        );
        assert_eq!(
            got,
            Ok(CalcValue::array(
                CalcArray::from_rows(vec![
                    vec![
                        CalcValue::text(ExcelText::from_interop_assignment("NF")),
                        CalcValue::number(20.0),
                    ],
                    vec![
                        CalcValue::text(ExcelText::from_interop_assignment("NF")),
                        CalcValue::number(40.0),
                    ],
                ])
                .unwrap()
            ))
        );
    }

    #[test]
    fn eval_xlookup_array_lookup_value_from_matrix_return_selects_first_cell() {
        let vertical_lookup = eval_xlookup_surface(
            &(CalcValue::array(
                CalcArray::from_rows(vec![vec![CalcValue::number(2.0), CalcValue::number(4.0)]])
                    .unwrap(),
            )),
            &[(CalcValue::array(
                CalcArray::from_rows(vec![
                    vec![CalcValue::number(2.0)],
                    vec![CalcValue::number(4.0)],
                    vec![CalcValue::number(6.0)],
                ])
                .unwrap(),
            ))],
            &[(CalcValue::array(
                CalcArray::from_rows(vec![
                    vec![CalcValue::number(20.0), CalcValue::number(21.0)],
                    vec![CalcValue::number(40.0), CalcValue::number(41.0)],
                    vec![CalcValue::number(60.0), CalcValue::number(61.0)],
                ])
                .unwrap(),
            ))],
            None,
            None,
            None,
            &NoResolver,
        );
        assert_eq!(
            vertical_lookup,
            Ok(CalcValue::array(
                CalcArray::from_rows(vec![
                    vec![CalcValue::number(20.0), CalcValue::number(40.0),]
                ])
                .unwrap()
            ))
        );

        let horizontal_lookup = eval_xlookup_surface(
            &(CalcValue::array(
                CalcArray::from_rows(vec![vec![CalcValue::number(2.0), CalcValue::number(4.0)]])
                    .unwrap(),
            )),
            &[(CalcValue::array(
                CalcArray::from_rows(vec![vec![
                    CalcValue::number(2.0),
                    CalcValue::number(4.0),
                    CalcValue::number(6.0),
                ]])
                .unwrap(),
            ))],
            &[(CalcValue::array(
                CalcArray::from_rows(vec![
                    vec![
                        CalcValue::number(20.0),
                        CalcValue::number(40.0),
                        CalcValue::number(60.0),
                    ],
                    vec![
                        CalcValue::number(21.0),
                        CalcValue::number(41.0),
                        CalcValue::number(61.0),
                    ],
                ])
                .unwrap(),
            ))],
            None,
            None,
            None,
            &NoResolver,
        );
        assert_eq!(
            horizontal_lookup,
            Ok(CalcValue::array(
                CalcArray::from_rows(vec![
                    vec![CalcValue::number(20.0), CalcValue::number(40.0),]
                ])
                .unwrap()
            ))
        );
    }

    trait IntoEval {
        fn into_eval(self) -> Option<CalcValue>;
    }

    impl IntoEval for CalcValue {
        fn into_eval(self) -> Option<CalcValue> {
            Some(self)
        }
    }
}
