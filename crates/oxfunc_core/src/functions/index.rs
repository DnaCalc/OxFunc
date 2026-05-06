use crate::coercion::{CoercionError, coerce_arg_to_number};
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::a1_refs::{
    A1Reference, A1ReferenceNotation, format_relative_target, parse_a1_reference,
};
use crate::resolver::ReferenceResolver;
use crate::value::{
    ArrayCellValue, ArrayShape, CallArgValue, EvalArray, EvalValue, ExcelText, ReferenceKind,
    ReferenceLike, WorksheetErrorCode,
};

pub const INDEX_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.INDEX",
    arity: Arity { min: 2, max: 4 },
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::WorkbookState,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: ArgPreparationProfile::RefsVisibleInAdapter,
    coercion_lift_profile: CoercionLiftProfile::Custom,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::RefOnly,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
};

#[derive(Debug, Clone, PartialEq)]
pub enum IndexEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
    InvalidIndexNumber(f64),
    InvalidAreaNumber(f64),
    OutOfBounds {
        rows: usize,
        cols: usize,
        row: usize,
        col: usize,
    },
    UnsupportedSource(&'static str),
    ArrayPayloadUnavailable,
}

#[derive(Debug, Clone, PartialEq)]
enum ArrayIndexSelector {
    Scalar(usize),
    SelectorArray(EvalArray),
}

fn coerce_index_number(
    arg: &CallArgValue,
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<usize, IndexEvalError> {
    let n = coerce_arg_to_number(arg, resolver).map_err(IndexEvalError::Coercion)?;
    if !n.is_finite() || n < 0.0 || n.fract() != 0.0 {
        return Err(IndexEvalError::InvalidIndexNumber(n));
    }
    Ok(n as usize)
}

fn coerce_optional_index_number(
    arg: Option<&CallArgValue>,
    resolver: &(impl ReferenceResolver + ?Sized),
    omitted_default: usize,
    blank_default: usize,
) -> Result<usize, IndexEvalError> {
    match arg {
        None => Ok(omitted_default),
        Some(CallArgValue::MissingArg | CallArgValue::EmptyCell) => Ok(blank_default),
        Some(other) => coerce_index_number(other, resolver),
    }
}

fn coerce_area_number(
    arg: Option<&CallArgValue>,
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<usize, IndexEvalError> {
    match arg {
        None => Ok(1),
        Some(CallArgValue::MissingArg | CallArgValue::EmptyCell) => Ok(1),
        Some(other) => {
            let n = coerce_arg_to_number(other, resolver).map_err(IndexEvalError::Coercion)?;
            if !n.is_finite() || n < 1.0 || n.fract() != 0.0 {
                return Err(IndexEvalError::InvalidAreaNumber(n));
            }
            Ok(n as usize)
        }
    }
}

fn coerce_array_index_selector(
    arg: Option<&CallArgValue>,
    resolver: &(impl ReferenceResolver + ?Sized),
    omitted_default: usize,
    blank_default: usize,
) -> Result<ArrayIndexSelector, IndexEvalError> {
    match arg {
        None => Ok(ArrayIndexSelector::Scalar(omitted_default)),
        Some(CallArgValue::MissingArg | CallArgValue::EmptyCell) => {
            Ok(ArrayIndexSelector::Scalar(blank_default))
        }
        Some(CallArgValue::Eval(EvalValue::Array(array))) => {
            Ok(ArrayIndexSelector::SelectorArray(array.clone()))
        }
        Some(other) => coerce_index_number(other, resolver).map(ArrayIndexSelector::Scalar),
    }
}

fn project_reference(base: &ReferenceLike, row: usize, col: usize) -> EvalValue {
    EvalValue::Reference(ReferenceLike {
        kind: base.kind,
        target: format!("{}#INDEX({row},{col})", base.target),
    })
}

fn has_legacy_multi_area_carrier(reference: &ReferenceLike) -> bool {
    !matches!(reference.kind, ReferenceKind::MultiArea)
        && reference.target.trim().starts_with('(')
        && reference.target.trim().ends_with(')')
}

fn parse_reference_areas(
    reference: &ReferenceLike,
) -> Result<Option<Vec<A1Reference>>, IndexEvalError> {
    let parts: Vec<String> = if matches!(reference.kind, ReferenceKind::MultiArea) {
        reference
            .multi_area_targets()
            .ok_or(IndexEvalError::UnsupportedSource(
                "invalid_multi_area_reference",
            ))?
    } else {
        return Ok(None);
    };
    if parts.is_empty() {
        return Ok(None);
    }

    let mut areas = Vec::with_capacity(parts.len());
    for part in parts {
        let Some(area) = parse_a1_reference(&part) else {
            return Ok(None);
        };
        areas.push(area);
    }

    if areas.len() > 1 {
        let first_prefix = areas[0].prefix.clone();
        if areas.iter().any(|area| area.prefix != first_prefix) {
            return Err(IndexEvalError::UnsupportedSource("mixed_sheet_multi_area"));
        }
    }

    Ok(Some(areas))
}

fn select_a1_reference(
    base: &A1Reference,
    row: usize,
    col: usize,
) -> Result<A1Reference, IndexEvalError> {
    let height = base.height();
    let width = base.width();

    if row > height || col > width {
        return Err(IndexEvalError::OutOfBounds {
            rows: height,
            cols: width,
            row,
            col,
        });
    }

    let mut selected = match (row, col) {
        (0, 0) => base.clone(),
        (0, c) => A1Reference {
            prefix: base.prefix.clone(),
            start_row: base.start_row,
            end_row: base.end_row,
            start_col: base.start_col + c - 1,
            end_col: base.start_col + c - 1,
            notation: A1ReferenceNotation::Rect,
        },
        (r, 0) => A1Reference {
            prefix: base.prefix.clone(),
            start_row: base.start_row + r - 1,
            end_row: base.start_row + r - 1,
            start_col: base.start_col,
            end_col: base.end_col,
            notation: A1ReferenceNotation::Rect,
        },
        (r, c) => A1Reference {
            prefix: base.prefix.clone(),
            start_row: base.start_row + r - 1,
            end_row: base.start_row + r - 1,
            start_col: base.start_col + c - 1,
            end_col: base.start_col + c - 1,
            notation: A1ReferenceNotation::Rect,
        },
    };

    selected.notation = if selected.start_row == 1
        && selected.end_row == crate::functions::a1_refs::EXCEL_MAX_ROWS
    {
        A1ReferenceNotation::WholeColumn
    } else if selected.start_col == 1
        && selected.end_col == crate::functions::a1_refs::EXCEL_MAX_COLS
    {
        A1ReferenceNotation::WholeRow
    } else {
        A1ReferenceNotation::Rect
    };

    Ok(selected)
}

fn reference_from_a1(reference: A1Reference) -> Result<EvalValue, IndexEvalError> {
    let target = format_relative_target(&reference)
        .ok_or(IndexEvalError::UnsupportedSource("unformattable_reference"))?;
    Ok(EvalValue::Reference(ReferenceLike {
        kind: if reference.width() == 1 && reference.height() == 1 {
            ReferenceKind::A1
        } else {
            ReferenceKind::Area
        },
        target,
    }))
}

fn cell_to_eval_value(cell: &ArrayCellValue) -> EvalValue {
    match cell {
        ArrayCellValue::Number(n) => EvalValue::Number(*n),
        ArrayCellValue::Text(t) => EvalValue::Text(t.clone()),
        ArrayCellValue::Logical(b) => EvalValue::Logical(*b),
        ArrayCellValue::Error(code) => EvalValue::Error(*code),
        ArrayCellValue::EmptyCell => EvalValue::Text(ExcelText::from_utf16_code_units(Vec::new())),
    }
}

fn scalar_array_from_eval_value(value: &EvalValue) -> Option<EvalArray> {
    let cell = match value {
        EvalValue::Number(n) => ArrayCellValue::Number(*n),
        EvalValue::Text(t) => ArrayCellValue::Text(t.clone()),
        EvalValue::Logical(b) => ArrayCellValue::Logical(*b),
        EvalValue::Error(code) => ArrayCellValue::Error(*code),
        EvalValue::Array(_) | EvalValue::Reference(_) | EvalValue::Lambda(_) => return None,
    };
    Some(EvalArray::from_scalar(cell))
}

fn slice_array(array: &EvalArray, row: usize, col: usize) -> Result<EvalValue, IndexEvalError> {
    let shape = array.shape();
    if row > shape.rows || col > shape.cols {
        return Err(IndexEvalError::OutOfBounds {
            rows: shape.rows,
            cols: shape.cols,
            row,
            col,
        });
    }

    match (row, col) {
        (0, 0) => Ok(EvalValue::Array(array.clone())),
        (0, c) => {
            let mut cells = Vec::with_capacity(shape.rows);
            for r in 0..shape.rows {
                cells.push(
                    array
                        .get(r, c - 1)
                        .cloned()
                        .expect("column bounds validated"),
                );
            }
            Ok(EvalValue::Array(
                EvalArray::new(
                    ArrayShape {
                        rows: shape.rows,
                        cols: 1,
                    },
                    cells,
                )
                .expect("slice dimensions validated"),
            ))
        }
        (r, 0) => Ok(EvalValue::Array(
            EvalArray::new(
                ArrayShape {
                    rows: 1,
                    cols: shape.cols,
                },
                array
                    .row_slice(r - 1)
                    .expect("row bounds validated")
                    .to_vec(),
            )
            .expect("slice dimensions validated"),
        )),
        (r, c) => Ok(cell_to_eval_value(
            array.get(r - 1, c - 1).expect("cell bounds validated"),
        )),
    }
}

fn normalize_array_indices_for_vector_position(
    array: &EvalArray,
    row: usize,
    col: usize,
    col_arg: Option<&CallArgValue>,
) -> (usize, usize) {
    let shape = array.shape();
    let col_omitted = matches!(
        col_arg,
        None | Some(CallArgValue::MissingArg | CallArgValue::EmptyCell)
    );

    if !col_omitted || row == 0 {
        return (row, col);
    }

    if shape.rows == 1 && shape.cols >= 1 {
        return (1, row);
    }

    (row, col)
}

fn coerce_selector_cell_to_index(cell: &ArrayCellValue) -> Result<usize, IndexEvalError> {
    match cell {
        ArrayCellValue::Number(n) => {
            if !n.is_finite() || *n < 1.0 || n.fract() != 0.0 {
                return Err(IndexEvalError::InvalidIndexNumber(*n));
            }
            Ok(*n as usize)
        }
        ArrayCellValue::Error(code) => Err(IndexEvalError::Coercion(
            CoercionError::WorksheetError(*code),
        )),
        _ => Err(IndexEvalError::Coercion(
            CoercionError::UnsupportedValueKind("array_index_selector"),
        )),
    }
}

fn vector_cell_at_position(
    array: &EvalArray,
    position: usize,
) -> Result<ArrayCellValue, IndexEvalError> {
    let shape = array.shape();
    if shape.rows == 1 {
        if position == 0 || position > shape.cols {
            return Err(IndexEvalError::OutOfBounds {
                rows: shape.rows,
                cols: shape.cols,
                row: 1,
                col: position,
            });
        }
        return Ok(array
            .get(0, position - 1)
            .expect("validated vector position")
            .clone());
    }
    if position == 0 || position > shape.rows {
        return Err(IndexEvalError::OutOfBounds {
            rows: shape.rows,
            cols: shape.cols,
            row: position,
            col: 1,
        });
    }
    Ok(array
        .get(position - 1, 0)
        .expect("validated vector position")
        .clone())
}

fn select_vector_positions(
    array: &EvalArray,
    selector: &EvalArray,
) -> Result<EvalValue, IndexEvalError> {
    let cells = selector
        .iter_row_major()
        .map(|cell| {
            coerce_selector_cell_to_index(cell).and_then(|idx| vector_cell_at_position(array, idx))
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(EvalValue::Array(
        EvalArray::new(selector.shape(), cells).expect("selector shape preserved"),
    ))
}

fn try_slice_vector_with_selector_array(
    array: &EvalArray,
    row_selector: &ArrayIndexSelector,
    col_selector: &ArrayIndexSelector,
    row_arg: Option<&CallArgValue>,
    col_arg: Option<&CallArgValue>,
) -> Result<Option<EvalValue>, IndexEvalError> {
    let shape = array.shape();
    let row_omitted = matches!(
        row_arg,
        None | Some(CallArgValue::MissingArg | CallArgValue::EmptyCell)
    );
    let col_omitted = matches!(
        col_arg,
        None | Some(CallArgValue::MissingArg | CallArgValue::EmptyCell)
    );

    if ((shape.rows == 1 && shape.cols >= 1) || (shape.cols == 1 && shape.rows >= 1)) && col_omitted
    {
        if let ArrayIndexSelector::SelectorArray(selector) = row_selector {
            return select_vector_positions(array, selector).map(Some);
        }
    }

    if shape.rows == 1 && shape.cols >= 1 && row_omitted {
        if let ArrayIndexSelector::SelectorArray(selector) = col_selector {
            return select_vector_positions(array, selector).map(Some);
        }
    }

    Ok(None)
}

pub fn eval_index_surface(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, IndexEvalError> {
    let argc = args.len();
    if !INDEX_META.arity.accepts(argc) {
        return Err(IndexEvalError::ArityMismatch {
            expected_min: INDEX_META.arity.min,
            expected_max: INDEX_META.arity.max,
            actual: argc,
        });
    }

    match &args[0] {
        CallArgValue::Reference(r) | CallArgValue::Eval(EvalValue::Reference(r)) => {
            let row = coerce_optional_index_number(args.get(1), resolver, 0, 0)?;
            let col = coerce_optional_index_number(args.get(2), resolver, 1, 0)?;
            let area = coerce_area_number(args.get(3), resolver)?;
            if let Some(areas) = parse_reference_areas(r)? {
                let Some(selected_area) = areas.get(area - 1) else {
                    return Err(IndexEvalError::InvalidAreaNumber(area as f64));
                };
                reference_from_a1(select_a1_reference(selected_area, row, col)?)
            } else if has_legacy_multi_area_carrier(r) {
                Err(IndexEvalError::UnsupportedSource(
                    "legacy_multi_area_carrier_removed",
                ))
            } else if area != 1 {
                Err(IndexEvalError::InvalidAreaNumber(area as f64))
            } else if let Some(parsed) = parse_a1_reference(&r.target) {
                reference_from_a1(select_a1_reference(&parsed, row, col)?)
            } else if row == 0 && col == 0 {
                Ok(EvalValue::Reference(r.clone()))
            } else {
                Ok(project_reference(r, row, col))
            }
        }
        CallArgValue::Eval(EvalValue::Array(array)) => {
            let row_selector = coerce_array_index_selector(args.get(1), resolver, 0, 0)?;
            let col_selector = coerce_array_index_selector(args.get(2), resolver, 1, 0)?;
            if let Some(selected) = try_slice_vector_with_selector_array(
                array,
                &row_selector,
                &col_selector,
                args.get(1),
                args.get(2),
            )? {
                return Ok(selected);
            }
            let ArrayIndexSelector::Scalar(row) = row_selector else {
                return Err(IndexEvalError::UnsupportedSource("array_index_selector"));
            };
            let ArrayIndexSelector::Scalar(col) = col_selector else {
                return Err(IndexEvalError::UnsupportedSource("array_index_selector"));
            };
            let (row, col) =
                normalize_array_indices_for_vector_position(array, row, col, args.get(2));
            slice_array(array, row, col)
        }
        CallArgValue::Eval(value) => {
            let row = coerce_optional_index_number(args.get(1), resolver, 0, 0)?;
            let col = coerce_optional_index_number(args.get(2), resolver, 1, 0)?;
            if let EvalValue::Error(code) = value {
                return Ok(EvalValue::Error(*code));
            }
            if row == 1
                && col == 0
                && scalar_array_from_eval_value(value).is_some()
                && matches!(args.get(2), Some(arg) if !matches!(arg, CallArgValue::MissingArg | CallArgValue::EmptyCell))
            {
                return Ok(value.clone());
            }
            let Some(array) = scalar_array_from_eval_value(value) else {
                return Err(IndexEvalError::UnsupportedSource("non_array_non_reference"));
            };
            let (row, col) =
                normalize_array_indices_for_vector_position(&array, row, col, args.get(2));
            slice_array(&array, row, col)
        }
        CallArgValue::MissingArg => Err(IndexEvalError::UnsupportedSource("missing_arg_source")),
        CallArgValue::EmptyCell => Err(IndexEvalError::UnsupportedSource("empty_cell_source")),
    }
}

pub fn map_index_error_to_ws(e: &IndexEvalError) -> WorksheetErrorCode {
    match e {
        IndexEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        IndexEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        IndexEvalError::InvalidIndexNumber(_) => WorksheetErrorCode::Value,
        IndexEvalError::InvalidAreaNumber(_) => WorksheetErrorCode::Ref,
        IndexEvalError::OutOfBounds { .. } => WorksheetErrorCode::Ref,
        IndexEvalError::UnsupportedSource(_) => WorksheetErrorCode::Value,
        IndexEvalError::ArrayPayloadUnavailable => WorksheetErrorCode::Calc,
        IndexEvalError::Coercion(_) => WorksheetErrorCode::Value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::{RefResolutionError, ResolverCapabilities};

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

    #[test]
    fn eval_index_reference_projection_projects_actual_a1_target() {
        let args = [
            CallArgValue::Reference(ReferenceLike {
                kind: ReferenceKind::Area,
                target: "A1:C3".to_string(),
            }),
            CallArgValue::Eval(EvalValue::Number(2.0)),
            CallArgValue::Eval(EvalValue::Number(1.0)),
        ];
        let got = eval_index_surface(&args, &NoResolver);
        assert_eq!(
            got,
            Ok(EvalValue::Reference(ReferenceLike {
                kind: ReferenceKind::A1,
                target: "A2".to_string(),
            }))
        );
    }

    #[test]
    fn eval_index_array_position_returns_payload_value() {
        let args = [
            CallArgValue::Eval(EvalValue::Array(
                EvalArray::from_rows(vec![
                    vec![ArrayCellValue::Number(1.0), ArrayCellValue::Number(2.0)],
                    vec![ArrayCellValue::Number(3.0), ArrayCellValue::Number(4.0)],
                    vec![ArrayCellValue::Number(5.0), ArrayCellValue::Number(6.0)],
                ])
                .unwrap(),
            )),
            CallArgValue::Eval(EvalValue::Number(1.0)),
            CallArgValue::Eval(EvalValue::Number(1.0)),
        ];
        let got = eval_index_surface(&args, &NoResolver);
        assert_eq!(got, Ok(EvalValue::Number(1.0)));
    }

    #[test]
    fn eval_index_row_vector_single_position_uses_vector_semantics() {
        let args = [
            CallArgValue::Eval(EvalValue::Array(
                EvalArray::from_rows(vec![vec![
                    ArrayCellValue::Number(10.0),
                    ArrayCellValue::Number(20.0),
                    ArrayCellValue::Number(30.0),
                ]])
                .unwrap(),
            )),
            CallArgValue::Eval(EvalValue::Number(2.0)),
        ];
        let got = eval_index_surface(&args, &NoResolver);
        assert_eq!(got, Ok(EvalValue::Number(20.0)));
    }

    #[test]
    fn eval_index_column_vector_single_position_uses_vector_semantics() {
        let args = [
            CallArgValue::Eval(EvalValue::Array(
                EvalArray::from_rows(vec![
                    vec![ArrayCellValue::Number(10.0)],
                    vec![ArrayCellValue::Number(20.0)],
                    vec![ArrayCellValue::Number(30.0)],
                ])
                .unwrap(),
            )),
            CallArgValue::Eval(EvalValue::Number(2.0)),
        ];
        let got = eval_index_surface(&args, &NoResolver);
        assert_eq!(got, Ok(EvalValue::Number(20.0)));
    }

    #[test]
    fn eval_index_scalar_source_treated_as_single_cell_array() {
        let args = [
            CallArgValue::Eval(EvalValue::Number(42.0)),
            CallArgValue::Eval(EvalValue::Number(1.0)),
        ];
        let got = eval_index_surface(&args, &NoResolver);
        assert_eq!(got, Ok(EvalValue::Number(42.0)));
    }

    #[test]
    fn ftc_1032_index_scalar_source_with_explicit_zero_column_returns_scalar() {
        let args = [
            CallArgValue::Eval(EvalValue::Number(0.0)),
            CallArgValue::Eval(EvalValue::Number(1.0)),
            CallArgValue::Eval(EvalValue::Number(0.0)),
        ];
        let got = eval_index_surface(&args, &NoResolver);
        assert_eq!(got, Ok(EvalValue::Number(0.0)));
    }

    #[test]
    fn ftc_0930_index_error_source_propagates_value_error() {
        let args = [
            CallArgValue::Eval(EvalValue::Error(WorksheetErrorCode::Value)),
            CallArgValue::Eval(EvalValue::Number(3.0)),
        ];
        let got = eval_index_surface(&args, &NoResolver);
        assert_eq!(got, Ok(EvalValue::Error(WorksheetErrorCode::Value)));
    }

    #[test]
    fn ftc_0833_index_row_vector_selector_array_direct_call_returns_first_three_values() {
        let args = [
            CallArgValue::Eval(EvalValue::Array(
                EvalArray::from_rows(vec![vec![
                    ArrayCellValue::Number(10.0),
                    ArrayCellValue::Number(20.0),
                    ArrayCellValue::Number(30.0),
                    ArrayCellValue::Number(40.0),
                    ArrayCellValue::Number(50.0),
                ]])
                .unwrap(),
            )),
            CallArgValue::Eval(EvalValue::Array(
                EvalArray::from_rows(vec![
                    vec![ArrayCellValue::Number(1.0)],
                    vec![ArrayCellValue::Number(2.0)],
                    vec![ArrayCellValue::Number(3.0)],
                ])
                .unwrap(),
            )),
        ];
        let got = eval_index_surface(&args, &NoResolver);
        assert_eq!(
            got,
            Ok(EvalValue::Array(
                EvalArray::from_rows(vec![
                    vec![ArrayCellValue::Number(10.0)],
                    vec![ArrayCellValue::Number(20.0)],
                    vec![ArrayCellValue::Number(30.0)],
                ])
                .unwrap()
            ))
        );
    }

    #[test]
    fn ftc_0910_index_row_vector_omitted_row_vector_column_selector_returns_first_five_values() {
        let args = [
            CallArgValue::Eval(EvalValue::Array(
                EvalArray::from_rows(vec![vec![
                    ArrayCellValue::Number(10.0),
                    ArrayCellValue::Number(20.0),
                    ArrayCellValue::Number(30.0),
                    ArrayCellValue::Number(40.0),
                    ArrayCellValue::Number(50.0),
                    ArrayCellValue::Number(60.0),
                    ArrayCellValue::Number(70.0),
                    ArrayCellValue::Number(80.0),
                    ArrayCellValue::Number(90.0),
                    ArrayCellValue::Number(100.0),
                ]])
                .unwrap(),
            )),
            CallArgValue::MissingArg,
            CallArgValue::Eval(EvalValue::Array(
                EvalArray::from_rows(vec![
                    vec![ArrayCellValue::Number(1.0)],
                    vec![ArrayCellValue::Number(2.0)],
                    vec![ArrayCellValue::Number(3.0)],
                    vec![ArrayCellValue::Number(4.0)],
                    vec![ArrayCellValue::Number(5.0)],
                ])
                .unwrap(),
            )),
        ];
        let got = eval_index_surface(&args, &NoResolver);
        assert_eq!(
            got,
            Ok(EvalValue::Array(
                EvalArray::from_rows(vec![
                    vec![ArrayCellValue::Number(10.0)],
                    vec![ArrayCellValue::Number(20.0)],
                    vec![ArrayCellValue::Number(30.0)],
                    vec![ArrayCellValue::Number(40.0)],
                    vec![ArrayCellValue::Number(50.0)],
                ])
                .unwrap()
            ))
        );
    }

    #[test]
    fn ftc_0910_index_row_vector_omitted_row_vector_column_selector_returns_last_five_values() {
        let args = [
            CallArgValue::Eval(EvalValue::Array(
                EvalArray::from_rows(vec![vec![
                    ArrayCellValue::Number(10.0),
                    ArrayCellValue::Number(20.0),
                    ArrayCellValue::Number(30.0),
                    ArrayCellValue::Number(40.0),
                    ArrayCellValue::Number(50.0),
                    ArrayCellValue::Number(60.0),
                    ArrayCellValue::Number(70.0),
                    ArrayCellValue::Number(80.0),
                    ArrayCellValue::Number(90.0),
                    ArrayCellValue::Number(100.0),
                ]])
                .unwrap(),
            )),
            CallArgValue::MissingArg,
            CallArgValue::Eval(EvalValue::Array(
                EvalArray::from_rows(vec![
                    vec![ArrayCellValue::Number(6.0)],
                    vec![ArrayCellValue::Number(7.0)],
                    vec![ArrayCellValue::Number(8.0)],
                    vec![ArrayCellValue::Number(9.0)],
                    vec![ArrayCellValue::Number(10.0)],
                ])
                .unwrap(),
            )),
        ];
        let got = eval_index_surface(&args, &NoResolver);
        assert_eq!(
            got,
            Ok(EvalValue::Array(
                EvalArray::from_rows(vec![
                    vec![ArrayCellValue::Number(60.0)],
                    vec![ArrayCellValue::Number(70.0)],
                    vec![ArrayCellValue::Number(80.0)],
                    vec![ArrayCellValue::Number(90.0)],
                    vec![ArrayCellValue::Number(100.0)],
                ])
                .unwrap()
            ))
        );
    }

    #[test]
    fn eval_index_array_bounds_checked() {
        let args = [
            CallArgValue::Eval(EvalValue::Array(
                EvalArray::from_rows(vec![
                    vec![ArrayCellValue::Number(1.0), ArrayCellValue::Number(2.0)],
                    vec![ArrayCellValue::Number(3.0), ArrayCellValue::Number(4.0)],
                ])
                .unwrap(),
            )),
            CallArgValue::Eval(EvalValue::Number(3.0)),
            CallArgValue::Eval(EvalValue::Number(1.0)),
        ];
        let got = eval_index_surface(&args, &NoResolver);
        assert_eq!(
            got,
            Err(IndexEvalError::OutOfBounds {
                rows: 2,
                cols: 2,
                row: 3,
                col: 1,
            })
        );
    }

    #[test]
    fn eval_index_array_zero_row_returns_column_array() {
        let args = [
            CallArgValue::Eval(EvalValue::Array(
                EvalArray::from_rows(vec![
                    vec![ArrayCellValue::Number(1.0), ArrayCellValue::Number(2.0)],
                    vec![ArrayCellValue::Number(3.0), ArrayCellValue::Number(4.0)],
                ])
                .unwrap(),
            )),
            CallArgValue::Eval(EvalValue::Number(0.0)),
            CallArgValue::Eval(EvalValue::Number(2.0)),
        ];
        let got = eval_index_surface(&args, &NoResolver);
        assert_eq!(
            got,
            Ok(EvalValue::Array(
                EvalArray::from_rows(vec![
                    vec![ArrayCellValue::Number(2.0)],
                    vec![ArrayCellValue::Number(4.0)],
                ])
                .unwrap()
            ))
        );
    }

    #[test]
    fn eval_index_invalid_area_num_rejected() {
        let args = [
            CallArgValue::Reference(ReferenceLike {
                kind: ReferenceKind::Area,
                target: "A1:C3".to_string(),
            }),
            CallArgValue::Eval(EvalValue::Number(1.0)),
            CallArgValue::Eval(EvalValue::Number(1.0)),
            CallArgValue::Eval(EvalValue::Number(2.0)),
        ];
        let got = eval_index_surface(&args, &NoResolver);
        assert_eq!(got, Err(IndexEvalError::InvalidAreaNumber(2.0)));
    }

    #[test]
    fn eval_index_missing_row_and_col_follow_excel_defaults() {
        let args = [
            CallArgValue::Reference(ReferenceLike {
                kind: ReferenceKind::Area,
                target: "B1:C2".to_string(),
            }),
            CallArgValue::MissingArg,
            CallArgValue::Eval(EvalValue::Number(2.0)),
        ];
        let got = eval_index_surface(&args, &NoResolver);
        assert_eq!(
            got,
            Ok(EvalValue::Reference(ReferenceLike {
                kind: ReferenceKind::Area,
                target: "C1:C2".to_string(),
            }))
        );

        let args = [
            CallArgValue::Reference(ReferenceLike {
                kind: ReferenceKind::Area,
                target: "B1:C2".to_string(),
            }),
            CallArgValue::Eval(EvalValue::Number(2.0)),
            CallArgValue::MissingArg,
        ];
        let got = eval_index_surface(&args, &NoResolver);
        assert_eq!(
            got,
            Ok(EvalValue::Reference(ReferenceLike {
                kind: ReferenceKind::Area,
                target: "B2:C2".to_string(),
            }))
        );
    }

    #[test]
    fn eval_index_multi_area_reference_selects_area_num() {
        let args = [
            CallArgValue::Reference(ReferenceLike {
                kind: ReferenceKind::MultiArea,
                target: "(A1:A2,G1:G2)".to_string(),
            }),
            CallArgValue::Eval(EvalValue::Number(2.0)),
            CallArgValue::Eval(EvalValue::Number(1.0)),
            CallArgValue::Eval(EvalValue::Number(2.0)),
        ];
        let got = eval_index_surface(&args, &NoResolver);
        assert_eq!(
            got,
            Ok(EvalValue::Reference(ReferenceLike {
                kind: ReferenceKind::A1,
                target: "G2".to_string(),
            }))
        );
    }

    #[test]
    fn eval_index_mixed_sheet_multi_area_is_rejected() {
        let args = [
            CallArgValue::Reference(ReferenceLike {
                kind: ReferenceKind::MultiArea,
                target: "(Sheet1!A1:A2,Sheet2!G1:G2)".to_string(),
            }),
            CallArgValue::Eval(EvalValue::Number(1.0)),
            CallArgValue::Eval(EvalValue::Number(1.0)),
        ];
        let got = eval_index_surface(&args, &NoResolver);
        assert_eq!(
            got,
            Err(IndexEvalError::UnsupportedSource("mixed_sheet_multi_area"))
        );
    }

    #[test]
    fn eval_index_rejects_legacy_parenthesized_area_carrier() {
        let args = [
            CallArgValue::Reference(ReferenceLike {
                kind: ReferenceKind::Area,
                target: "(A1:A2,G1:G2)".to_string(),
            }),
            CallArgValue::Eval(EvalValue::Number(2.0)),
            CallArgValue::Eval(EvalValue::Number(1.0)),
            CallArgValue::Eval(EvalValue::Number(2.0)),
        ];
        let got = eval_index_surface(&args, &NoResolver);
        assert_eq!(
            got,
            Err(IndexEvalError::UnsupportedSource(
                "legacy_multi_area_carrier_removed"
            ))
        );
    }
}
