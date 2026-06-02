use crate::coercion::{CoercionError, coerce_arg_to_number, coerce_calc_scalar_to_number};
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::resolver::{
    ReferenceSystemError, ReferenceSystemProvider, ReferenceTransformKind,
    ReferenceTransformRequest, enumerate_reference_values,
};
use crate::value::{
    ArrayCellValue, ArrayShape, CalcArray, CalcValue, CallArgValue, CoreValue, EvalArray,
    EvalValue, ExcelText, WorksheetErrorCode,
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
    ReferenceSystem(ReferenceSystemError),
}

#[derive(Debug, Clone, PartialEq)]
enum ArrayIndexSelector {
    Scalar(usize),
    SelectorArray(EvalArray),
}

fn coerce_index_number(
    arg: &CallArgValue,
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<usize, IndexEvalError> {
    let n = coerce_arg_to_number(arg, resolver).map_err(IndexEvalError::Coercion)?;
    if !n.is_finite() || n < 0.0 || n.fract() != 0.0 {
        return Err(IndexEvalError::InvalidIndexNumber(n));
    }
    Ok(n as usize)
}

fn coerce_optional_index_number(
    arg: Option<&CallArgValue>,
    resolver: &(impl ReferenceSystemProvider + ?Sized),
    omitted_default: usize,
    blank_default: usize,
) -> Result<usize, IndexEvalError> {
    match arg {
        None => Ok(omitted_default),
        Some(CallArgValue::MissingArg | CallArgValue::EmptyCell) => Ok(blank_default),
        Some(other) => coerce_index_number(other, resolver),
    }
}

fn coerce_calc_index_number(arg: &CalcValue) -> Result<usize, IndexEvalError> {
    let n = coerce_calc_scalar_to_number(arg).map_err(IndexEvalError::Coercion)?;
    if !n.is_finite() || n < 0.0 || n.fract() != 0.0 {
        return Err(IndexEvalError::InvalidIndexNumber(n));
    }
    Ok(n as usize)
}

fn coerce_optional_calc_index_number(
    arg: Option<&CalcValue>,
    omitted_default: usize,
    blank_default: usize,
) -> Result<usize, IndexEvalError> {
    match arg {
        None => Ok(omitted_default),
        Some(value) if matches!(value.core(), CoreValue::Missing | CoreValue::Empty) => {
            Ok(blank_default)
        }
        Some(other) => coerce_calc_index_number(other),
    }
}

fn coerce_area_number(
    arg: Option<&CallArgValue>,
    resolver: &(impl ReferenceSystemProvider + ?Sized),
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
    resolver: &(impl ReferenceSystemProvider + ?Sized),
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

fn cell_to_eval_value(cell: &ArrayCellValue) -> EvalValue {
    match cell {
        ArrayCellValue::Number(n) => EvalValue::Number(*n),
        ArrayCellValue::Text(t) => EvalValue::Text(t.clone()),
        ArrayCellValue::Logical(b) => EvalValue::Logical(*b),
        ArrayCellValue::Error(code) => EvalValue::Error(*code),
        ArrayCellValue::EmptyCell => EvalValue::Text(ExcelText::from_utf16_code_units(Vec::new())),
    }
}

fn calc_cell_to_scalar_value(cell: &CalcValue) -> CalcValue {
    match cell.core() {
        CoreValue::Empty => CalcValue::text(ExcelText::from_utf16_code_units(Vec::new())),
        _ => cell.clone(),
    }
}

fn scalar_calc_array_from_value(value: &CalcValue) -> Option<CalcArray> {
    match value.core() {
        CoreValue::Number(_)
        | CoreValue::Text(_)
        | CoreValue::Logical(_)
        | CoreValue::Error(_)
        | CoreValue::Empty => CalcArray::from_scalar(value.clone()),
        CoreValue::Missing | CoreValue::Array(_) | CoreValue::Reference(_) => None,
    }
}

fn scalar_array_from_eval_value(value: &EvalValue) -> Option<EvalArray> {
    let cell = match value {
        EvalValue::Number(n) => ArrayCellValue::Number(*n),
        EvalValue::Text(t) => ArrayCellValue::Text(t.clone()),
        EvalValue::Logical(b) => ArrayCellValue::Logical(*b),
        EvalValue::Error(code) => ArrayCellValue::Error(*code),
        EvalValue::Array(_) | EvalValue::Reference(_) => return None,
        _ => return None,
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

fn slice_calc_array(
    array: &CalcArray,
    row: usize,
    col: usize,
) -> Result<CalcValue, IndexEvalError> {
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
        (0, 0) => Ok(CalcValue::array(array.clone())),
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
            Ok(CalcValue::array(
                CalcArray::from_cells_iter(
                    ArrayShape {
                        rows: shape.rows,
                        cols: 1,
                    },
                    cells,
                )
                .expect("slice dimensions validated"),
            ))
        }
        (r, 0) => Ok(CalcValue::array(
            CalcArray::from_cells_iter(
                ArrayShape {
                    rows: 1,
                    cols: shape.cols,
                },
                array
                    .row_slice(r - 1)
                    .expect("row bounds validated")
                    .iter()
                    .cloned(),
            )
            .expect("slice dimensions validated"),
        )),
        (r, c) => Ok(calc_cell_to_scalar_value(
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

fn normalize_calc_array_indices_for_vector_position(
    array: &CalcArray,
    row: usize,
    col: usize,
    col_arg: Option<&CalcValue>,
) -> (usize, usize) {
    let shape = array.shape();
    let col_omitted = matches!(
        col_arg.map(CalcValue::core),
        None | Some(CoreValue::Missing | CoreValue::Empty)
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
    resolver: &(impl ReferenceSystemProvider + ?Sized),
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
            if area != 1 && !matches!(r.kind, crate::value::ReferenceKind::MultiArea) {
                return Err(IndexEvalError::InvalidAreaNumber(area as f64));
            }
            if let Some(values) = enumerate_reference_values(resolver, r)
                .map_err(|e| IndexEvalError::Coercion(CoercionError::RefResolution(e)))?
            {
                let rows = values.declared_extent.rows;
                let cols = values.declared_extent.cols;
                if (row > 0 && row > rows) || (col > 0 && col > cols) {
                    return Err(IndexEvalError::OutOfBounds {
                        rows,
                        cols,
                        row,
                        col,
                    });
                }
            }
            resolver
                .transform_reference(&ReferenceTransformRequest {
                    reference: r.clone(),
                    transform: ReferenceTransformKind::Index { row, col, area },
                })
                .map(EvalValue::Reference)
                .map_err(IndexEvalError::ReferenceSystem)
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

pub fn eval_index_calc_surface(args: &[CalcValue]) -> Result<CalcValue, IndexEvalError> {
    let argc = args.len();
    if !INDEX_META.arity.accepts(argc) {
        return Err(IndexEvalError::ArityMismatch {
            expected_min: INDEX_META.arity.min,
            expected_max: INDEX_META.arity.max,
            actual: argc,
        });
    }

    match args[0].core() {
        CoreValue::Reference(_) => Err(IndexEvalError::UnsupportedSource("reference_source")),
        CoreValue::Array(array) => {
            let row = coerce_optional_calc_index_number(args.get(1), 0, 0)?;
            let col = coerce_optional_calc_index_number(args.get(2), 1, 0)?;
            let (row, col) =
                normalize_calc_array_indices_for_vector_position(array, row, col, args.get(2));
            slice_calc_array(array, row, col)
        }
        CoreValue::Missing => Err(IndexEvalError::UnsupportedSource("missing_arg_source")),
        CoreValue::Empty => Err(IndexEvalError::UnsupportedSource("empty_cell_source")),
        _ => {
            let source = args[0].clone();
            let row = coerce_optional_calc_index_number(args.get(1), 0, 0)?;
            let col = coerce_optional_calc_index_number(args.get(2), 1, 0)?;
            if let CoreValue::Error(_) = source.core() {
                return Ok(source);
            }
            if row == 1
                && col == 0
                && scalar_calc_array_from_value(&source).is_some()
                && matches!(args.get(2), Some(arg) if !matches!(arg.core(), CoreValue::Missing | CoreValue::Empty))
            {
                return Ok(source);
            }
            let Some(array) = scalar_calc_array_from_value(&source) else {
                return Err(IndexEvalError::UnsupportedSource("non_array_non_reference"));
            };
            let (row, col) =
                normalize_calc_array_indices_for_vector_position(&array, row, col, args.get(2));
            slice_calc_array(&array, row, col)
        }
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
        IndexEvalError::ReferenceSystem(_) => WorksheetErrorCode::Ref,
        IndexEvalError::Coercion(_) => WorksheetErrorCode::Value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::ReferenceSystemCapabilities;
    use crate::value::{ReferenceKind, ReferenceLike};

    struct NoResolver;
    impl ReferenceSystemProvider for NoResolver {
        fn capabilities(&self) -> ReferenceSystemCapabilities {
            ReferenceSystemCapabilities::permissive_local()
        }

        fn dereference(
            &self,
            request: &crate::resolver::ReferenceDereferenceRequest,
        ) -> Result<EvalValue, crate::resolver::ReferenceResolutionError> {
            let reference = &request.reference;
            Err(
                crate::resolver::ReferenceResolutionError::UnresolvedReference {
                    target: reference.target.clone(),
                },
            )
        }

        fn transform_reference(
            &self,
            request: &ReferenceTransformRequest,
        ) -> Result<ReferenceLike, ReferenceSystemError> {
            let ReferenceTransformKind::Index { row, col, area } = request.transform else {
                return Err(ReferenceSystemError::Unsupported {
                    operation: crate::resolver::ReferenceSystemOperation::Transform,
                });
            };
            match (request.reference.target.as_str(), row, col, area) {
                ("A1:C3", 2, 1, 1) => Ok(ReferenceLike::new(ReferenceKind::A1, "A2")),
                ("B1:C2", 0, 2, 1) => Ok(ReferenceLike::new(ReferenceKind::Area, "C1:C2")),
                ("B1:C2", 2, 0, 1) => Ok(ReferenceLike::new(ReferenceKind::Area, "B2:C2")),
                ("(A1:A2,G1:G2)", 2, 1, 2) => Ok(ReferenceLike::new(ReferenceKind::A1, "G2")),
                _ => Err(ReferenceSystemError::Unsupported {
                    operation: crate::resolver::ReferenceSystemOperation::Transform,
                }),
            }
        }
    }

    #[test]
    fn eval_index_reference_projection_projects_actual_a1_target() {
        let args = [
            CallArgValue::Reference(ReferenceLike::new(ReferenceKind::Area, "A1:C3".to_string())),
            CallArgValue::Eval(EvalValue::Number(2.0)),
            CallArgValue::Eval(EvalValue::Number(1.0)),
        ];
        let got = eval_index_surface(&args, &NoResolver);
        assert_eq!(
            got,
            Ok(EvalValue::Reference(ReferenceLike::new(
                ReferenceKind::A1,
                "A2".to_string()
            )))
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
            CallArgValue::Reference(ReferenceLike::new(ReferenceKind::Area, "A1:C3".to_string())),
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
            CallArgValue::Reference(ReferenceLike::new(ReferenceKind::Area, "B1:C2".to_string())),
            CallArgValue::MissingArg,
            CallArgValue::Eval(EvalValue::Number(2.0)),
        ];
        let got = eval_index_surface(&args, &NoResolver);
        assert_eq!(
            got,
            Ok(EvalValue::Reference(ReferenceLike::new(
                ReferenceKind::Area,
                "C1:C2".to_string()
            )))
        );

        let args = [
            CallArgValue::Reference(ReferenceLike::new(ReferenceKind::Area, "B1:C2".to_string())),
            CallArgValue::Eval(EvalValue::Number(2.0)),
            CallArgValue::MissingArg,
        ];
        let got = eval_index_surface(&args, &NoResolver);
        assert_eq!(
            got,
            Ok(EvalValue::Reference(ReferenceLike::new(
                ReferenceKind::Area,
                "B2:C2".to_string()
            )))
        );
    }

    #[test]
    fn eval_index_multi_area_reference_selects_area_num() {
        let args = [
            CallArgValue::Reference(ReferenceLike::new(
                ReferenceKind::MultiArea,
                "(A1:A2,G1:G2)".to_string(),
            )),
            CallArgValue::Eval(EvalValue::Number(2.0)),
            CallArgValue::Eval(EvalValue::Number(1.0)),
            CallArgValue::Eval(EvalValue::Number(2.0)),
        ];
        let got = eval_index_surface(&args, &NoResolver);
        assert_eq!(
            got,
            Ok(EvalValue::Reference(ReferenceLike::new(
                ReferenceKind::A1,
                "G2".to_string()
            )))
        );
    }

    #[test]
    fn eval_index_multi_area_transform_failure_is_reported_from_provider() {
        let args = [
            CallArgValue::Reference(ReferenceLike::new(
                ReferenceKind::MultiArea,
                "(Sheet1!A1:A2,Sheet2!G1:G2)".to_string(),
            )),
            CallArgValue::Eval(EvalValue::Number(1.0)),
            CallArgValue::Eval(EvalValue::Number(1.0)),
        ];
        let got = eval_index_surface(&args, &NoResolver);
        assert_eq!(
            got,
            Err(IndexEvalError::ReferenceSystem(
                ReferenceSystemError::Unsupported {
                    operation: crate::resolver::ReferenceSystemOperation::Transform,
                }
            ))
        );
    }

    #[test]
    fn eval_index_rejects_legacy_parenthesized_area_carrier() {
        let args = [
            CallArgValue::Reference(ReferenceLike::new(
                ReferenceKind::Area,
                "(A1:A2,G1:G2)".to_string(),
            )),
            CallArgValue::Eval(EvalValue::Number(2.0)),
            CallArgValue::Eval(EvalValue::Number(1.0)),
            CallArgValue::Eval(EvalValue::Number(2.0)),
        ];
        let got = eval_index_surface(&args, &NoResolver);
        assert_eq!(got, Err(IndexEvalError::InvalidAreaNumber(2.0)));
    }
}
