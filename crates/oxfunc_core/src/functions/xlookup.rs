use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::a1_refs::{
    A1Reference, A1ReferenceNotation, format_relative_target, parse_a1_reference,
};
use crate::functions::adapters::{
    PreparedArgValue, expand_lookup_vector_arg, prepare_arg_values_only,
};
use crate::functions::xmatch::{XmatchEvalError, eval_xmatch_adapter_prepared};
use crate::resolver::{ReferenceResolver, resolve_eval_value};
use crate::value::{
    ArrayCellValue, ArrayShape, CallArgValue, EvalArray, EvalValue, ReferenceKind, ReferenceLike,
    WorksheetErrorCode,
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
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VectorOrientation {
    Horizontal,
    Vertical,
    Scalar,
}

#[derive(Debug, Clone, PartialEq)]
enum ReturnItem {
    Eval(EvalValue),
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
    ArrayValue(EvalArray),
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
        return Err(XlookupEvalError::Coercion(CoercionError::UnsupportedValueKind(
            "two_dimensional_array",
        )));
    }
    Ok(if rows > 1 {
        VectorOrientation::Vertical
    } else if cols > 1 {
        VectorOrientation::Horizontal
    } else {
        VectorOrientation::Scalar
    })
}

fn cell_to_return_item(cell: &ArrayCellValue) -> ReturnItem {
    match cell {
        ArrayCellValue::Number(n) => ReturnItem::Eval(EvalValue::Number(*n)),
        ArrayCellValue::Text(t) => ReturnItem::Eval(EvalValue::Text(t.clone())),
        ArrayCellValue::Logical(b) => ReturnItem::Eval(EvalValue::Logical(*b)),
        ArrayCellValue::Error(code) => ReturnItem::Eval(EvalValue::Error(*code)),
        ArrayCellValue::EmptyCell => ReturnItem::EmptyCell,
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
    ReferenceLike {
        kind: ReferenceKind::A1,
        target,
    }
}

fn flatten_reference_vector(
    reference: &ReferenceLike,
) -> Result<(Vec<ReturnItem>, VectorOrientation), XlookupEvalError> {
    if let Some(parsed) = parse_a1_reference(&reference.target) {
        let orientation = orientation_from_shape(parsed.height(), parsed.width())?;
        let mut items = Vec::with_capacity(parsed.height() * parsed.width());
        for row in 0..parsed.height() {
            for col in 0..parsed.width() {
                items.push(ReturnItem::Reference(reference_from_cell(&parsed, row, col)));
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
    value: &EvalValue,
) -> Result<(Vec<ReturnItem>, VectorOrientation), XlookupEvalError> {
    match value {
        EvalValue::Array(array) => {
            let shape = array.shape();
            let orientation = orientation_from_shape(shape.rows, shape.cols)?;
            Ok((
                array.iter_row_major().map(cell_to_return_item).collect(),
                orientation,
            ))
        }
        EvalValue::Reference(reference) => flatten_reference_vector(reference),
        _ => Ok((vec![ReturnItem::Eval(value.clone())], VectorOrientation::Scalar)),
    }
}

fn expand_return_arg(arg: &CallArgValue) -> Result<(Vec<ReturnItem>, VectorOrientation), XlookupEvalError> {
    match arg {
        CallArgValue::Reference(reference) => flatten_reference_vector(reference),
        CallArgValue::Eval(EvalValue::Reference(reference)) => flatten_reference_vector(reference),
        CallArgValue::Eval(value) => flatten_return_eval(value),
        CallArgValue::MissingArg => Ok((vec![ReturnItem::MissingArg], VectorOrientation::Scalar)),
        CallArgValue::EmptyCell => Ok((vec![ReturnItem::EmptyCell], VectorOrientation::Scalar)),
    }
}

fn materialize_return_item(item: &ReturnItem) -> EvalValue {
    match item {
        ReturnItem::Eval(value) => value.clone(),
        ReturnItem::Reference(reference) => EvalValue::Reference(reference.clone()),
        ReturnItem::MissingArg => EvalValue::Error(WorksheetErrorCode::NA),
        ReturnItem::EmptyCell => EvalValue::Number(0.0),
    }
}

fn materialize_fallback(
    arg: &CallArgValue,
) -> EvalValue {
    match arg {
        CallArgValue::Reference(reference) => EvalValue::Reference(reference.clone()),
        CallArgValue::Eval(EvalValue::Reference(reference)) => {
            EvalValue::Reference(reference.clone())
        }
        CallArgValue::Eval(value) => value.clone(),
        CallArgValue::MissingArg => EvalValue::Error(WorksheetErrorCode::NA),
        CallArgValue::EmptyCell => EvalValue::Number(0.0),
    }
}

fn materialize_cell_value(cell: &ArrayCellValue) -> EvalValue {
    match cell {
        ArrayCellValue::Number(n) => EvalValue::Number(*n),
        ArrayCellValue::Text(t) => EvalValue::Text(t.clone()),
        ArrayCellValue::Logical(b) => EvalValue::Logical(*b),
        ArrayCellValue::Error(code) => EvalValue::Error(*code),
        ArrayCellValue::EmptyCell => EvalValue::Number(0.0),
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

fn prepare_return_selection(
    args: &[CallArgValue],
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

    match &args[0] {
        CallArgValue::Reference(reference) | CallArgValue::Eval(EvalValue::Reference(reference)) => {
            if let Some(parsed) = parse_a1_reference(&reference.target) {
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
                Ok(ReturnSelection::Scalar(ReturnItem::Reference(reference.clone())))
            }
        }
        CallArgValue::Eval(EvalValue::Array(array)) => {
            let shape = array.shape();
            if infer_selection_orientation(shape).is_none() {
                Ok(ReturnSelection::ArrayValue(array.clone()))
            } else {
                let items: Vec<ReturnItem> = array.iter_row_major().map(cell_to_return_item).collect();
                Ok(if items.len() == 1 {
                    ReturnSelection::Scalar(items.into_iter().next().expect("len checked"))
                } else {
                    ReturnSelection::Vector {
                        items,
                        orientation: infer_selection_orientation(shape)
                            .expect("non-matrix array has vector orientation"),
                    }
                })
            }
        }
        CallArgValue::Eval(value) => Ok(ReturnSelection::Scalar(ReturnItem::Eval(value.clone()))),
        CallArgValue::MissingArg => Ok(ReturnSelection::Scalar(ReturnItem::MissingArg)),
        CallArgValue::EmptyCell => Ok(ReturnSelection::Scalar(ReturnItem::EmptyCell)),
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

fn column_array(array: &EvalArray, col_index: usize) -> EvalArray {
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
    EvalArray::new(
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
) -> Result<EvalValue, XlookupEvalError> {
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
    Ok(EvalValue::Reference(ReferenceLike {
        kind: if selected.height() == 1 && selected.width() == 1 {
            ReferenceKind::A1
        } else {
            ReferenceKind::Area
        },
        target,
    }))
}

fn select_return_value(
    selection: &ReturnSelection,
    lookup_orientation: Option<VectorOrientation>,
    index: usize,
) -> Result<EvalValue, XlookupEvalError> {
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
                        Ok(EvalValue::Array(
                            EvalArray::new(
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
                        Ok(EvalValue::Array(column_array(array, index)))
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

fn prepare_lookup_vector(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<(Vec<PreparedArgValue>, Option<VectorOrientation>), XlookupEvalError> {
    let mut prepared = Vec::new();
    let mut orientation = None;
    if args.len() == 1 {
        match args.first().expect("len checked") {
            CallArgValue::Eval(EvalValue::Array(array)) => {
                let shape = array.shape();
                orientation = Some(orientation_from_shape(shape.rows, shape.cols)?);
            }
            CallArgValue::Reference(reference) => {
                let resolved = resolve_eval_value(resolver, reference)
                    .map_err(CoercionError::RefResolution)
                    .map_err(XlookupEvalError::Coercion)?;
                if let EvalValue::Array(array) = resolved {
                    let shape = array.shape();
                    orientation = Some(orientation_from_shape(shape.rows, shape.cols)?);
                }
            }
            _ => {}
        }
    }

    for arg in args {
        prepared.extend(
            expand_lookup_vector_arg(arg, resolver).map_err(XlookupEvalError::Coercion)?,
        );
    }
    Ok((prepared, orientation))
}

pub fn eval_xlookup_surface(
    lookup_value: &CallArgValue,
    lookup_array: &[CallArgValue],
    return_array: &[CallArgValue],
    if_not_found: Option<&CallArgValue>,
    match_mode: Option<&CallArgValue>,
    search_mode: Option<&CallArgValue>,
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, XlookupEvalError> {
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
    let (prepared_lookup_array, lookup_orientation) = prepare_lookup_vector(lookup_array, resolver)?;
    let return_selection = prepare_return_selection(return_array)?;
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
    use crate::resolver::{RefResolutionError, ResolverCapabilities};
    use crate::value::{EvalArray, ExcelText};

    struct NoResolver;
    impl ReferenceResolver for NoResolver {
        fn capabilities(&self) -> ResolverCapabilities {
            ResolverCapabilities::permissive_local()
        }

        fn resolve_reference(
            &self,
            reference: &ReferenceLike,
        ) -> Result<EvalValue, RefResolutionError> {
            Err(RefResolutionError::UnresolvedReference {
                target: reference.target.clone(),
            })
        }
    }

    fn text_arg(s: &str) -> CallArgValue {
        CallArgValue::Eval(EvalValue::Text(ExcelText::from_utf16_code_units(
            s.encode_utf16().collect(),
        )))
    }

    #[test]
    fn eval_xlookup_exact_forward_returns_value() {
        let got = eval_xlookup_surface(
            &CallArgValue::Eval(EvalValue::Number(2.0)),
            &[
                CallArgValue::Eval(EvalValue::Number(1.0)),
                CallArgValue::Eval(EvalValue::Number(2.0)),
            ],
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
            Some(&CallArgValue::Eval(EvalValue::Number(2.0))),
            Some(&CallArgValue::Eval(EvalValue::Number(-1.0))),
            &NoResolver,
        );
        assert_eq!(got, Ok(text_arg("last").into_eval().unwrap()));
    }

    #[test]
    fn eval_xlookup_returns_if_not_found_fallback() {
        let got = eval_xlookup_surface(
            &CallArgValue::Eval(EvalValue::Number(9.0)),
            &[
                CallArgValue::Eval(EvalValue::Number(1.0)),
                CallArgValue::Eval(EvalValue::Number(2.0)),
                CallArgValue::Eval(EvalValue::Number(3.0)),
            ],
            &[
                CallArgValue::Eval(EvalValue::Number(10.0)),
                CallArgValue::Eval(EvalValue::Number(20.0)),
                CallArgValue::Eval(EvalValue::Number(30.0)),
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
            &CallArgValue::Eval(EvalValue::Number(2.0)),
            &[CallArgValue::Eval(EvalValue::Array(
                EvalArray::from_rows(vec![vec![
                    ArrayCellValue::Number(1.0),
                    ArrayCellValue::Number(2.0),
                    ArrayCellValue::Number(3.0),
                ]])
                .unwrap(),
            ))],
            &[CallArgValue::Eval(EvalValue::Array(
                EvalArray::from_rows(vec![vec![
                    ArrayCellValue::Number(10.0),
                    ArrayCellValue::Number(20.0),
                    ArrayCellValue::Number(30.0),
                ]])
                .unwrap(),
            ))],
            Some(&CallArgValue::MissingArg),
            Some(&CallArgValue::Eval(EvalValue::Number(0.0))),
            Some(&CallArgValue::MissingArg),
            &NoResolver,
        );
        assert_eq!(got, Ok(EvalValue::Number(20.0)));
    }

    #[test]
    fn eval_xlookup_omitted_or_blank_lookup_matches_true_blank_cells() {
        let lookup_array = [CallArgValue::Eval(EvalValue::Array(
            EvalArray::from_rows(vec![vec![
                ArrayCellValue::EmptyCell,
                ArrayCellValue::Number(1.0),
            ]])
            .unwrap(),
        ))];
        let return_array = [CallArgValue::Eval(EvalValue::Array(
            EvalArray::from_rows(vec![vec![
                ArrayCellValue::Number(10.0),
                ArrayCellValue::Number(20.0),
            ]])
            .unwrap(),
        ))];

        let omitted = eval_xlookup_surface(
            &CallArgValue::MissingArg,
            &lookup_array,
            &return_array,
            None,
            None,
            None,
            &NoResolver,
        );
        assert_eq!(omitted, Ok(EvalValue::Number(10.0)));

        let blank = eval_xlookup_surface(
            &CallArgValue::EmptyCell,
            &lookup_array,
            &return_array,
            None,
            None,
            None,
            &NoResolver,
        );
        assert_eq!(blank, Ok(EvalValue::Number(10.0)));
    }

    #[test]
    fn eval_xlookup_empty_string_matches_formula_empty_not_true_blank() {
        let lookup_array = [CallArgValue::Eval(EvalValue::Array(
            EvalArray::from_rows(vec![vec![
                ArrayCellValue::Text(ExcelText::from_utf16_code_units(Vec::new())),
                ArrayCellValue::EmptyCell,
            ]])
            .unwrap(),
        ))];
        let return_array = [CallArgValue::Eval(EvalValue::Array(
            EvalArray::from_rows(vec![vec![
                ArrayCellValue::Number(10.0),
                ArrayCellValue::Number(20.0),
            ]])
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
        assert_eq!(got, Ok(EvalValue::Number(10.0)));
    }

    #[test]
    fn eval_xlookup_returns_zero_for_true_blank_return_cells() {
        let got = eval_xlookup_surface(
            &CallArgValue::Eval(EvalValue::Number(1.0)),
            &[CallArgValue::Eval(EvalValue::Array(
                EvalArray::from_rows(vec![vec![
                    ArrayCellValue::Number(1.0),
                    ArrayCellValue::Number(2.0),
                ]])
                .unwrap(),
            ))],
            &[CallArgValue::Eval(EvalValue::Array(
                EvalArray::from_rows(vec![vec![
                    ArrayCellValue::EmptyCell,
                    ArrayCellValue::Number(20.0),
                ]])
                .unwrap(),
            ))],
            None,
            None,
            None,
            &NoResolver,
        );
        assert_eq!(got, Ok(EvalValue::Number(0.0)));
    }

    #[test]
    fn eval_xlookup_binary_unsorted_exact_matches_empirical_lane() {
        let got = eval_xlookup_surface(
            &CallArgValue::Eval(EvalValue::Number(2.0)),
            &[CallArgValue::Eval(EvalValue::Array(
                EvalArray::from_rows(vec![vec![
                    ArrayCellValue::Number(3.0),
                    ArrayCellValue::Number(1.0),
                    ArrayCellValue::Number(4.0),
                    ArrayCellValue::Number(2.0),
                ]])
                .unwrap(),
            ))],
            &[CallArgValue::Eval(EvalValue::Array(
                EvalArray::from_rows(vec![vec![
                    ArrayCellValue::Number(30.0),
                    ArrayCellValue::Number(10.0),
                    ArrayCellValue::Number(40.0),
                    ArrayCellValue::Number(20.0),
                ]])
                .unwrap(),
            ))],
            None,
            Some(&CallArgValue::Eval(EvalValue::Number(0.0))),
            Some(&CallArgValue::Eval(EvalValue::Number(2.0))),
            &NoResolver,
        );
        assert_eq!(got, Err(XlookupEvalError::NotAvailable));
    }

    #[test]
    fn eval_xlookup_supports_reference_return_from_single_area_argument() {
        let got = eval_xlookup_surface(
            &CallArgValue::Eval(EvalValue::Number(2.0)),
            &[
                CallArgValue::Eval(EvalValue::Number(1.0)),
                CallArgValue::Eval(EvalValue::Number(2.0)),
                CallArgValue::Eval(EvalValue::Number(3.0)),
            ],
            &[CallArgValue::Reference(ReferenceLike {
                kind: ReferenceKind::Area,
                target: "B1:D1".to_string(),
            })],
            None,
            None,
            None,
            &NoResolver,
        );
        assert_eq!(
            got,
            Ok(EvalValue::Reference(ReferenceLike {
                kind: ReferenceKind::A1,
                target: "C1".to_string(),
            }))
        );
    }

    #[test]
    fn eval_xlookup_rejects_orientation_mismatch() {
        let got = eval_xlookup_surface(
            &CallArgValue::Eval(EvalValue::Number(2.0)),
            &[CallArgValue::Eval(EvalValue::Array(
                EvalArray::from_rows(vec![
                    vec![ArrayCellValue::Number(1.0)],
                    vec![ArrayCellValue::Number(2.0)],
                    vec![ArrayCellValue::Number(3.0)],
                ])
                .unwrap(),
            ))],
            &[CallArgValue::Eval(EvalValue::Array(
                EvalArray::from_rows(vec![vec![
                    ArrayCellValue::Number(10.0),
                    ArrayCellValue::Number(20.0),
                    ArrayCellValue::Number(30.0),
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
            &CallArgValue::Eval(EvalValue::Number(2.0)),
            &[CallArgValue::Eval(EvalValue::Array(
                EvalArray::from_rows(vec![
                    vec![ArrayCellValue::Number(1.0)],
                    vec![ArrayCellValue::Number(2.0)],
                    vec![ArrayCellValue::Number(3.0)],
                ])
                .unwrap(),
            ))],
            &[CallArgValue::Eval(EvalValue::Array(
                EvalArray::from_rows(vec![
                    vec![ArrayCellValue::Number(10.0), ArrayCellValue::Number(11.0)],
                    vec![ArrayCellValue::Number(20.0), ArrayCellValue::Number(21.0)],
                    vec![ArrayCellValue::Number(30.0), ArrayCellValue::Number(31.0)],
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
            Ok(EvalValue::Array(
                EvalArray::from_rows(vec![vec![
                    ArrayCellValue::Number(20.0),
                    ArrayCellValue::Number(21.0),
                ]])
                .unwrap()
            ))
        );
    }

    #[test]
    fn eval_xlookup_horizontal_lookup_returns_matching_reference_column() {
        let got = eval_xlookup_surface(
            &CallArgValue::Eval(EvalValue::Number(2.0)),
            &[CallArgValue::Eval(EvalValue::Array(
                EvalArray::from_rows(vec![vec![
                    ArrayCellValue::Number(1.0),
                    ArrayCellValue::Number(2.0),
                    ArrayCellValue::Number(3.0),
                ]])
                .unwrap(),
            ))],
            &[CallArgValue::Reference(ReferenceLike {
                kind: ReferenceKind::Area,
                target: "B2:D4".to_string(),
            })],
            None,
            None,
            None,
            &NoResolver,
        );
        assert_eq!(
            got,
            Ok(EvalValue::Reference(ReferenceLike {
                kind: ReferenceKind::Area,
                target: "C2:C4".to_string(),
            }))
        );
    }

    trait IntoEval {
        fn into_eval(self) -> Option<EvalValue>;
    }

    impl IntoEval for CallArgValue {
        fn into_eval(self) -> Option<EvalValue> {
            match self {
                CallArgValue::Eval(v) => Some(v),
                _ => None,
            }
        }
    }
}
