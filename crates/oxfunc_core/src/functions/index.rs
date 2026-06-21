use crate::coercion::{CoercionError, coerce_arg_to_number, coerce_calc_scalar_to_number};
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::resolver::{
    ReferenceSystemError, ReferenceSystemOperation, ReferenceSystemProvider,
    ReferenceTransformKind, ReferenceTransformRequest, enumerate_reference_values,
    materialize_resolved_reference_values,
};
use crate::value::CalcValue;
use crate::value::{ArrayShape, CalcArray, CoreValue, ExcelText, WorksheetErrorCode};

pub const INDEX_META: FunctionMeta = function_spec! {
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
    SelectorArray(CalcArray),
}

fn coerce_index_number(
    arg: &CalcValue,
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<usize, IndexEvalError> {
    let n = coerce_arg_to_number(arg, resolver).map_err(IndexEvalError::Coercion)?;
    if !n.is_finite() || n < 0.0 || n.fract() != 0.0 {
        return Err(IndexEvalError::InvalidIndexNumber(n));
    }
    Ok(n as usize)
}

fn coerce_optional_index_number(
    arg: Option<&CalcValue>,
    resolver: &(impl ReferenceSystemProvider + ?Sized),
    omitted_default: usize,
    blank_default: usize,
) -> Result<usize, IndexEvalError> {
    match arg {
        None => Ok(omitted_default),
        Some(value) if matches!(value.core(), CoreValue::Missing | CoreValue::Empty) => {
            Ok(blank_default)
        }
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
    arg: Option<&CalcValue>,
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<usize, IndexEvalError> {
    match arg {
        None => Ok(1),
        Some(value) if matches!(value.core(), CoreValue::Missing | CoreValue::Empty) => Ok(1),
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
    arg: Option<&CalcValue>,
    resolver: &(impl ReferenceSystemProvider + ?Sized),
    omitted_default: usize,
    blank_default: usize,
) -> Result<ArrayIndexSelector, IndexEvalError> {
    match arg {
        None => Ok(ArrayIndexSelector::Scalar(omitted_default)),
        Some(value) if matches!(value.core(), CoreValue::Missing | CoreValue::Empty) => {
            Ok(ArrayIndexSelector::Scalar(blank_default))
        }
        Some(value) if matches!(value.core(), CoreValue::Array(_)) => {
            let CoreValue::Array(array) = value.core() else {
                unreachable!("guarded array")
            };
            Ok(ArrayIndexSelector::SelectorArray(array.clone()))
        }
        Some(other) => coerce_index_number(other, resolver).map(ArrayIndexSelector::Scalar),
    }
}

fn cell_to_eval_value(cell: &CalcValue) -> CalcValue {
    if cell.rich().is_some() {
        return cell.clone();
    }

    match cell.core() {
        CoreValue::Number(n) => CalcValue::number(*n),
        CoreValue::Text(t) => CalcValue::text(t.clone()),
        CoreValue::Logical(b) => CalcValue::logical(*b),
        CoreValue::Error(code) => CalcValue::error(*code),
        CoreValue::Empty | CoreValue::Missing => {
            CalcValue::text(ExcelText::from_utf16_code_units(Vec::new()))
        }
        CoreValue::Array(_) | CoreValue::Reference(_) => cell.clone(),
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

fn scalar_array_from_eval_value(value: &CalcValue) -> Option<CalcArray> {
    let cell = match value.core() {
        CoreValue::Number(n) => CalcValue::number(*n),
        CoreValue::Text(t) => CalcValue::text(t.clone()),
        CoreValue::Logical(b) => CalcValue::logical(*b),
        CoreValue::Error(code) => CalcValue::error(*code),
        CoreValue::Array(_) | CoreValue::Reference(_) => return None,
        _ => return None,
    };
    CalcArray::from_scalar(cell)
}

fn slice_array(array: &CalcArray, row: usize, col: usize) -> Result<CalcValue, IndexEvalError> {
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
                CalcArray::new(
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
            CalcArray::new(
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

fn coerce_selector_cell_to_index(cell: &CalcValue) -> Result<usize, IndexEvalError> {
    match cell {
        value if matches!(value.core(), CoreValue::Number(_)) => {
            let CoreValue::Number(n) = value.core() else {
                unreachable!("guarded number")
            };
            if !n.is_finite() || *n < 1.0 || n.fract() != 0.0 {
                return Err(IndexEvalError::InvalidIndexNumber(*n));
            }
            Ok(*n as usize)
        }
        value if matches!(value.core(), CoreValue::Error(_)) => {
            let CoreValue::Error(code) = value.core() else {
                unreachable!("guarded error")
            };
            Err(IndexEvalError::Coercion(CoercionError::WorksheetError(
                *code,
            )))
        }
        _ => Err(IndexEvalError::Coercion(
            CoercionError::UnsupportedValueKind("array_index_selector"),
        )),
    }
}

fn vector_cell_at_position(
    array: &CalcArray,
    position: usize,
) -> Result<CalcValue, IndexEvalError> {
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
    array: &CalcArray,
    selector: &CalcArray,
) -> Result<CalcValue, IndexEvalError> {
    let cells = selector
        .iter_row_major()
        .map(|cell| {
            coerce_selector_cell_to_index(cell).and_then(|idx| vector_cell_at_position(array, idx))
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(CalcValue::array(
        CalcArray::new(selector.shape(), cells).expect("selector shape preserved"),
    ))
}

fn try_slice_vector_with_selector_array(
    array: &CalcArray,
    row_selector: &ArrayIndexSelector,
    col_selector: &ArrayIndexSelector,
    row_arg: Option<&CalcValue>,
    col_arg: Option<&CalcValue>,
) -> Result<Option<CalcValue>, IndexEvalError> {
    let shape = array.shape();
    let row_omitted = matches!(
        row_arg.map(CalcValue::core),
        None | Some(CoreValue::Missing | CoreValue::Empty)
    );
    let col_omitted = matches!(
        col_arg.map(CalcValue::core),
        None | Some(CoreValue::Missing | CoreValue::Empty)
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
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, IndexEvalError> {
    let argc = args.len();
    if !INDEX_META.arity.accepts(argc) {
        return Err(IndexEvalError::ArityMismatch {
            expected_min: INDEX_META.arity.min,
            expected_max: INDEX_META.arity.max,
            actual: argc,
        });
    }

    match args[0].core() {
        CoreValue::Reference(r) => {
            let row = coerce_optional_index_number(args.get(1), resolver, 0, 0)?;
            let col = coerce_optional_index_number(args.get(2), resolver, 1, 0)?;
            let area = coerce_area_number(args.get(3), resolver)?;
            if area != 1 && !matches!(r.kind(), crate::value::ReferenceKind::MultiArea) {
                return Err(IndexEvalError::InvalidAreaNumber(area as f64));
            }
            let enumerated = enumerate_reference_values(resolver, r)
                .map_err(|e| IndexEvalError::Coercion(CoercionError::RefResolution(e)))?;
            if let Some(values) = &enumerated {
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
            match resolver.transform_reference(&ReferenceTransformRequest {
                reference: r.clone(),
                transform: ReferenceTransformKind::Index { row, col, area },
            }) {
                Ok(reference) => Ok(CalcValue::reference(reference)),
                Err(ReferenceSystemError::Unsupported {
                    operation: ReferenceSystemOperation::Transform,
                }) => {
                    let Some(values) = enumerated else {
                        return Err(IndexEvalError::ReferenceSystem(
                            ReferenceSystemError::Unsupported {
                                operation: ReferenceSystemOperation::Transform,
                            },
                        ));
                    };
                    let array = materialize_resolved_reference_values(&values)
                        .map_err(|e| IndexEvalError::Coercion(CoercionError::RefResolution(e)))?;
                    let (row, col) =
                        normalize_array_indices_for_vector_position(&array, row, col, args.get(2));
                    slice_array(&array, row, col)
                }
                Err(err) => Err(IndexEvalError::ReferenceSystem(err)),
            }
        }
        CoreValue::Array(array) => {
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
        CoreValue::Number(_) | CoreValue::Text(_) | CoreValue::Logical(_) | CoreValue::Error(_) => {
            let value = &args[0];
            let row = coerce_optional_index_number(args.get(1), resolver, 0, 0)?;
            let col = coerce_optional_index_number(args.get(2), resolver, 1, 0)?;
            if let CoreValue::Error(code) = value.core() {
                return Ok(CalcValue::error(*code));
            }
            if row == 1
                && col == 0
                && scalar_array_from_eval_value(value).is_some()
                && matches!(args.get(2), Some(arg) if !matches!(arg.core(), CoreValue::Missing | CoreValue::Empty))
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
        CoreValue::Missing => Err(IndexEvalError::UnsupportedSource("missing_arg_source")),
        CoreValue::Empty => Err(IndexEvalError::UnsupportedSource("empty_cell_source")),
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
    use crate::resolver::{
        ReferenceEnumerationRequest, ReferenceResolutionError, ReferenceSystemCapabilities,
        ResolvedReferenceCell, ResolvedReferenceExtent, ResolvedReferenceValues,
    };
    use crate::value::{ReferenceKind, ReferenceLike};

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

        fn transform_reference(
            &self,
            request: &ReferenceTransformRequest,
        ) -> Result<ReferenceLike, ReferenceSystemError> {
            let ReferenceTransformKind::Index { row, col, area } = request.transform else {
                return Err(ReferenceSystemError::Unsupported {
                    operation: crate::resolver::ReferenceSystemOperation::Transform,
                });
            };
            match (request.reference.target(), row, col, area) {
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

    struct EnumeratingOnlyResolver;
    impl ReferenceSystemProvider for EnumeratingOnlyResolver {
        fn capabilities(&self) -> ReferenceSystemCapabilities {
            ReferenceSystemCapabilities::permissive_local()
        }

        fn enumerate_values(
            &self,
            request: &ReferenceEnumerationRequest,
        ) -> Result<Option<ResolvedReferenceValues>, ReferenceResolutionError> {
            if request.reference.target() != "A1:B3" {
                return Ok(None);
            }
            Ok(Some(ResolvedReferenceValues::new(
                ResolvedReferenceExtent::new(3, 2),
                vec![
                    ResolvedReferenceCell::new(1, 1, CalcValue::number(10.0)),
                    ResolvedReferenceCell::new(1, 2, CalcValue::number(20.0)),
                    ResolvedReferenceCell::new(2, 1, CalcValue::number(30.0)),
                    ResolvedReferenceCell::new(2, 2, CalcValue::number(40.0)),
                    ResolvedReferenceCell::new(3, 1, CalcValue::number(50.0)),
                    ResolvedReferenceCell::new(3, 2, CalcValue::number(60.0)),
                ],
                None,
            )))
        }
    }

    #[test]
    fn eval_index_reference_projection_projects_actual_a1_target() {
        let args = [
            CalcValue::reference(ReferenceLike::new(ReferenceKind::Area, "A1:C3".to_string())),
            (CalcValue::number(2.0)),
            (CalcValue::number(1.0)),
        ];
        let got = eval_index_surface(&args, &NoResolver);
        assert_eq!(
            got,
            Ok(CalcValue::reference(ReferenceLike::new(
                ReferenceKind::A1,
                "A2".to_string()
            )))
        );
    }

    #[test]
    fn eval_index_reference_value_context_materializes_when_transform_unsupported() {
        let args = [
            CalcValue::reference(ReferenceLike::new(ReferenceKind::Area, "A1:B3".to_string())),
            CalcValue::number(2.0),
            CalcValue::number(2.0),
        ];
        let got = eval_index_surface(&args, &EnumeratingOnlyResolver);
        assert_eq!(got, Ok(CalcValue::number(40.0)));
    }

    #[test]
    fn eval_index_array_position_returns_payload_value() {
        let args = [
            (CalcValue::array(
                CalcArray::from_rows(vec![
                    vec![CalcValue::number(1.0), CalcValue::number(2.0)],
                    vec![CalcValue::number(3.0), CalcValue::number(4.0)],
                    vec![CalcValue::number(5.0), CalcValue::number(6.0)],
                ])
                .unwrap(),
            )),
            (CalcValue::number(1.0)),
            (CalcValue::number(1.0)),
        ];
        let got = eval_index_surface(&args, &NoResolver);
        assert_eq!(got, Ok(CalcValue::number(1.0)));
    }

    #[test]
    fn eval_index_row_vector_single_position_uses_vector_semantics() {
        let args = [
            (CalcValue::array(
                CalcArray::from_rows(vec![vec![
                    CalcValue::number(10.0),
                    CalcValue::number(20.0),
                    CalcValue::number(30.0),
                ]])
                .unwrap(),
            )),
            (CalcValue::number(2.0)),
        ];
        let got = eval_index_surface(&args, &NoResolver);
        assert_eq!(got, Ok(CalcValue::number(20.0)));
    }

    #[test]
    fn eval_index_column_vector_single_position_uses_vector_semantics() {
        let args = [
            (CalcValue::array(
                CalcArray::from_rows(vec![
                    vec![CalcValue::number(10.0)],
                    vec![CalcValue::number(20.0)],
                    vec![CalcValue::number(30.0)],
                ])
                .unwrap(),
            )),
            (CalcValue::number(2.0)),
        ];
        let got = eval_index_surface(&args, &NoResolver);
        assert_eq!(got, Ok(CalcValue::number(20.0)));
    }

    #[test]
    fn eval_index_scalar_source_treated_as_single_cell_array() {
        let args = [(CalcValue::number(42.0)), (CalcValue::number(1.0))];
        let got = eval_index_surface(&args, &NoResolver);
        assert_eq!(got, Ok(CalcValue::number(42.0)));
    }

    #[test]
    fn ftc_1032_index_scalar_source_with_explicit_zero_column_returns_scalar() {
        let args = [
            (CalcValue::number(0.0)),
            (CalcValue::number(1.0)),
            (CalcValue::number(0.0)),
        ];
        let got = eval_index_surface(&args, &NoResolver);
        assert_eq!(got, Ok(CalcValue::number(0.0)));
    }

    #[test]
    fn ftc_0930_index_error_source_propagates_value_error() {
        let args = [
            (CalcValue::error(WorksheetErrorCode::Value)),
            (CalcValue::number(3.0)),
        ];
        let got = eval_index_surface(&args, &NoResolver);
        assert_eq!(got, Ok(CalcValue::error(WorksheetErrorCode::Value)));
    }

    #[test]
    fn ftc_0833_index_row_vector_selector_array_direct_call_returns_first_three_values() {
        let args = [
            (CalcValue::array(
                CalcArray::from_rows(vec![vec![
                    CalcValue::number(10.0),
                    CalcValue::number(20.0),
                    CalcValue::number(30.0),
                    CalcValue::number(40.0),
                    CalcValue::number(50.0),
                ]])
                .unwrap(),
            )),
            (CalcValue::array(
                CalcArray::from_rows(vec![
                    vec![CalcValue::number(1.0)],
                    vec![CalcValue::number(2.0)],
                    vec![CalcValue::number(3.0)],
                ])
                .unwrap(),
            )),
        ];
        let got = eval_index_surface(&args, &NoResolver);
        assert_eq!(
            got,
            Ok(CalcValue::array(
                CalcArray::from_rows(vec![
                    vec![CalcValue::number(10.0)],
                    vec![CalcValue::number(20.0)],
                    vec![CalcValue::number(30.0)],
                ])
                .unwrap()
            ))
        );
    }

    #[test]
    fn ftc_0910_index_row_vector_omitted_row_vector_column_selector_returns_first_five_values() {
        let args = [
            (CalcValue::array(
                CalcArray::from_rows(vec![vec![
                    CalcValue::number(10.0),
                    CalcValue::number(20.0),
                    CalcValue::number(30.0),
                    CalcValue::number(40.0),
                    CalcValue::number(50.0),
                    CalcValue::number(60.0),
                    CalcValue::number(70.0),
                    CalcValue::number(80.0),
                    CalcValue::number(90.0),
                    CalcValue::number(100.0),
                ]])
                .unwrap(),
            )),
            CalcValue::missing(),
            (CalcValue::array(
                CalcArray::from_rows(vec![
                    vec![CalcValue::number(1.0)],
                    vec![CalcValue::number(2.0)],
                    vec![CalcValue::number(3.0)],
                    vec![CalcValue::number(4.0)],
                    vec![CalcValue::number(5.0)],
                ])
                .unwrap(),
            )),
        ];
        let got = eval_index_surface(&args, &NoResolver);
        assert_eq!(
            got,
            Ok(CalcValue::array(
                CalcArray::from_rows(vec![
                    vec![CalcValue::number(10.0)],
                    vec![CalcValue::number(20.0)],
                    vec![CalcValue::number(30.0)],
                    vec![CalcValue::number(40.0)],
                    vec![CalcValue::number(50.0)],
                ])
                .unwrap()
            ))
        );
    }

    #[test]
    fn ftc_0910_index_row_vector_omitted_row_vector_column_selector_returns_last_five_values() {
        let args = [
            (CalcValue::array(
                CalcArray::from_rows(vec![vec![
                    CalcValue::number(10.0),
                    CalcValue::number(20.0),
                    CalcValue::number(30.0),
                    CalcValue::number(40.0),
                    CalcValue::number(50.0),
                    CalcValue::number(60.0),
                    CalcValue::number(70.0),
                    CalcValue::number(80.0),
                    CalcValue::number(90.0),
                    CalcValue::number(100.0),
                ]])
                .unwrap(),
            )),
            CalcValue::missing(),
            (CalcValue::array(
                CalcArray::from_rows(vec![
                    vec![CalcValue::number(6.0)],
                    vec![CalcValue::number(7.0)],
                    vec![CalcValue::number(8.0)],
                    vec![CalcValue::number(9.0)],
                    vec![CalcValue::number(10.0)],
                ])
                .unwrap(),
            )),
        ];
        let got = eval_index_surface(&args, &NoResolver);
        assert_eq!(
            got,
            Ok(CalcValue::array(
                CalcArray::from_rows(vec![
                    vec![CalcValue::number(60.0)],
                    vec![CalcValue::number(70.0)],
                    vec![CalcValue::number(80.0)],
                    vec![CalcValue::number(90.0)],
                    vec![CalcValue::number(100.0)],
                ])
                .unwrap()
            ))
        );
    }

    #[test]
    fn eval_index_array_bounds_checked() {
        let args = [
            (CalcValue::array(
                CalcArray::from_rows(vec![
                    vec![CalcValue::number(1.0), CalcValue::number(2.0)],
                    vec![CalcValue::number(3.0), CalcValue::number(4.0)],
                ])
                .unwrap(),
            )),
            (CalcValue::number(3.0)),
            (CalcValue::number(1.0)),
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
            (CalcValue::array(
                CalcArray::from_rows(vec![
                    vec![CalcValue::number(1.0), CalcValue::number(2.0)],
                    vec![CalcValue::number(3.0), CalcValue::number(4.0)],
                ])
                .unwrap(),
            )),
            (CalcValue::number(0.0)),
            (CalcValue::number(2.0)),
        ];
        let got = eval_index_surface(&args, &NoResolver);
        assert_eq!(
            got,
            Ok(CalcValue::array(
                CalcArray::from_rows(vec![
                    vec![CalcValue::number(2.0)],
                    vec![CalcValue::number(4.0)],
                ])
                .unwrap()
            ))
        );
    }

    #[test]
    fn eval_index_invalid_area_num_rejected() {
        let args = [
            CalcValue::reference(ReferenceLike::new(ReferenceKind::Area, "A1:C3".to_string())),
            (CalcValue::number(1.0)),
            (CalcValue::number(1.0)),
            (CalcValue::number(2.0)),
        ];
        let got = eval_index_surface(&args, &NoResolver);
        assert_eq!(got, Err(IndexEvalError::InvalidAreaNumber(2.0)));
    }

    #[test]
    fn eval_index_missing_row_and_col_follow_excel_defaults() {
        let args = [
            CalcValue::reference(ReferenceLike::new(ReferenceKind::Area, "B1:C2".to_string())),
            CalcValue::missing(),
            (CalcValue::number(2.0)),
        ];
        let got = eval_index_surface(&args, &NoResolver);
        assert_eq!(
            got,
            Ok(CalcValue::reference(ReferenceLike::new(
                ReferenceKind::Area,
                "C1:C2".to_string()
            )))
        );

        let args = [
            CalcValue::reference(ReferenceLike::new(ReferenceKind::Area, "B1:C2".to_string())),
            (CalcValue::number(2.0)),
            CalcValue::missing(),
        ];
        let got = eval_index_surface(&args, &NoResolver);
        assert_eq!(
            got,
            Ok(CalcValue::reference(ReferenceLike::new(
                ReferenceKind::Area,
                "B2:C2".to_string()
            )))
        );
    }

    #[test]
    fn eval_index_multi_area_reference_selects_area_num() {
        let args = [
            CalcValue::reference(ReferenceLike::new(
                ReferenceKind::MultiArea,
                "(A1:A2,G1:G2)".to_string(),
            )),
            (CalcValue::number(2.0)),
            (CalcValue::number(1.0)),
            (CalcValue::number(2.0)),
        ];
        let got = eval_index_surface(&args, &NoResolver);
        assert_eq!(
            got,
            Ok(CalcValue::reference(ReferenceLike::new(
                ReferenceKind::A1,
                "G2".to_string()
            )))
        );
    }

    #[test]
    fn eval_index_multi_area_transform_failure_is_reported_from_provider() {
        let args = [
            CalcValue::reference(ReferenceLike::new(
                ReferenceKind::MultiArea,
                "(Sheet1!A1:A2,Sheet2!G1:G2)".to_string(),
            )),
            (CalcValue::number(1.0)),
            (CalcValue::number(1.0)),
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
            CalcValue::reference(ReferenceLike::new(
                ReferenceKind::Area,
                "(A1:A2,G1:G2)".to_string(),
            )),
            (CalcValue::number(2.0)),
            (CalcValue::number(1.0)),
            (CalcValue::number(2.0)),
        ];
        let got = eval_index_surface(&args, &NoResolver);
        assert_eq!(got, Err(IndexEvalError::InvalidAreaNumber(2.0)));
    }
}
