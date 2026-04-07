use crate::coercion::CoercionError;
use crate::functions::adapters::{
    BroadcastPreparedPair, PreparedArgValue, coerce_prepared_to_number,
    expand_binary_broadcast_grid, prepare_args_values_only,
};
use crate::resolver::ReferenceResolver;
use crate::value::{ArrayCellValue, CallArgValue, EvalArray, EvalValue, WorksheetErrorCode};

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryNumericSurfaceError {
    ArityMismatch { expected: usize, actual: usize },
    Coercion(CoercionError),
    Domain(WorksheetErrorCode),
}

pub fn eval_binary_numeric_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
    kernel: impl Fn(f64, f64) -> Result<f64, WorksheetErrorCode> + Copy,
) -> Result<EvalValue, BinaryNumericSurfaceError> {
    let prepared =
        prepare_args_values_only(args, resolver).map_err(BinaryNumericSurfaceError::Coercion)?;
    eval_binary_numeric_prepared(&prepared, kernel)
}

pub fn eval_binary_numeric_prepared(
    args: &[PreparedArgValue],
    kernel: impl Fn(f64, f64) -> Result<f64, WorksheetErrorCode> + Copy,
) -> Result<EvalValue, BinaryNumericSurfaceError> {
    if args.len() != 2 {
        return Err(BinaryNumericSurfaceError::ArityMismatch {
            expected: 2,
            actual: args.len(),
        });
    }

    if let Some((shape, cells)) = expand_binary_broadcast_grid(&args[0], &args[1]) {
        let mapped = cells
            .into_iter()
            .map(|cell| match cell {
                BroadcastPreparedPair::Pair(lhs_value, rhs_value) => {
                    map_binary_numeric_item(&lhs_value, &rhs_value, kernel)
                }
                BroadcastPreparedPair::MissingCoordinate => {
                    ArrayCellValue::Error(WorksheetErrorCode::NA)
                }
            })
            .collect();
        Ok(EvalValue::Array(
            EvalArray::new(shape, mapped).expect("shape preserved"),
        ))
    } else {
        eval_binary_numeric_scalars(&args[0], &args[1], kernel)
    }
}

pub fn map_binary_numeric_error_to_ws(e: &BinaryNumericSurfaceError) -> WorksheetErrorCode {
    match e {
        BinaryNumericSurfaceError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        BinaryNumericSurfaceError::Coercion(CoercionError::WorksheetError(code)) => *code,
        BinaryNumericSurfaceError::Coercion(_) => WorksheetErrorCode::Value,
        BinaryNumericSurfaceError::Domain(code) => *code,
    }
}

fn map_binary_numeric_item(
    lhs: &PreparedArgValue,
    rhs: &PreparedArgValue,
    kernel: impl Fn(f64, f64) -> Result<f64, WorksheetErrorCode> + Copy,
) -> ArrayCellValue {
    let lhs = match coerce_prepared_to_number(lhs) {
        Ok(lhs) => lhs,
        Err(CoercionError::WorksheetError(code)) => return ArrayCellValue::Error(code),
        Err(_) => return ArrayCellValue::Error(WorksheetErrorCode::Value),
    };
    let rhs = match coerce_prepared_to_number(rhs) {
        Ok(rhs) => rhs,
        Err(CoercionError::WorksheetError(code)) => return ArrayCellValue::Error(code),
        Err(_) => return ArrayCellValue::Error(WorksheetErrorCode::Value),
    };

    match kernel(lhs, rhs) {
        Ok(value) => ArrayCellValue::Number(value),
        Err(code) => ArrayCellValue::Error(code),
    }
}

fn eval_binary_numeric_scalars(
    lhs: &PreparedArgValue,
    rhs: &PreparedArgValue,
    kernel: impl Fn(f64, f64) -> Result<f64, WorksheetErrorCode> + Copy,
) -> Result<EvalValue, BinaryNumericSurfaceError> {
    let lhs = coerce_prepared_to_number(lhs).map_err(BinaryNumericSurfaceError::Coercion)?;
    let rhs = coerce_prepared_to_number(rhs).map_err(BinaryNumericSurfaceError::Coercion)?;
    kernel(lhs, rhs)
        .map(EvalValue::Number)
        .map_err(BinaryNumericSurfaceError::Domain)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::{RefResolutionError, ResolverCapabilities};
    use crate::value::{ExcelText, ReferenceLike};

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
    fn binary_numeric_surface_accepts_numeric_text() {
        let got = eval_binary_numeric_surface(
            &[
                CallArgValue::Eval(EvalValue::Text(ExcelText::from_utf16_code_units(
                    "2".encode_utf16().collect(),
                ))),
                CallArgValue::Eval(EvalValue::Number(3.0)),
            ],
            &NoResolver,
            |lhs, rhs| Ok(lhs + rhs),
        )
        .unwrap();
        assert_eq!(got, EvalValue::Number(5.0));
    }

    #[test]
    fn binary_numeric_surface_maps_domain_errors() {
        let got = eval_binary_numeric_surface(
            &[
                CallArgValue::Eval(EvalValue::Number(1.0)),
                CallArgValue::Eval(EvalValue::Number(0.0)),
            ],
            &NoResolver,
            |_lhs, _rhs| Err(WorksheetErrorCode::Div0),
        );
        assert_eq!(
            got,
            Err(BinaryNumericSurfaceError::Domain(WorksheetErrorCode::Div0))
        );
    }

    #[test]
    fn binary_numeric_surface_lifts_array_scalar_elementwise() {
        let got = eval_binary_numeric_surface(
            &[
                CallArgValue::Eval(EvalValue::Array(
                    EvalArray::from_rows(vec![vec![
                        ArrayCellValue::Number(1.0),
                        ArrayCellValue::Text(ExcelText::from_utf16_code_units(
                            "2".encode_utf16().collect(),
                        )),
                    ]])
                    .unwrap(),
                )),
                CallArgValue::Eval(EvalValue::Number(10.0)),
            ],
            &NoResolver,
            |lhs, rhs| Ok(lhs + rhs),
        )
        .unwrap();

        assert_eq!(
            got,
            EvalValue::Array(
                EvalArray::from_rows(vec![vec![
                    ArrayCellValue::Number(11.0),
                    ArrayCellValue::Number(12.0),
                ]])
                .unwrap()
            )
        );
    }

    #[test]
    fn binary_numeric_surface_lifts_scalar_array_elementwise() {
        let got = eval_binary_numeric_surface(
            &[
                CallArgValue::Eval(EvalValue::Number(2.0)),
                CallArgValue::Eval(EvalValue::Array(
                    EvalArray::from_rows(vec![
                        vec![ArrayCellValue::Number(3.0), ArrayCellValue::Logical(true)],
                        vec![ArrayCellValue::EmptyCell, ArrayCellValue::Number(4.0)],
                    ])
                    .unwrap(),
                )),
            ],
            &NoResolver,
            |lhs, rhs| Ok(lhs * rhs),
        )
        .unwrap();

        assert_eq!(
            got,
            EvalValue::Array(
                EvalArray::from_rows(vec![
                    vec![ArrayCellValue::Number(6.0), ArrayCellValue::Number(2.0)],
                    vec![
                        ArrayCellValue::Error(WorksheetErrorCode::Value),
                        ArrayCellValue::Number(8.0)
                    ],
                ])
                .unwrap()
            )
        );
    }

    #[test]
    fn binary_numeric_surface_lifts_same_shape_arrays_elementwise() {
        let got = eval_binary_numeric_surface(
            &[
                CallArgValue::Eval(EvalValue::Array(
                    EvalArray::from_rows(vec![
                        vec![ArrayCellValue::Number(1.0), ArrayCellValue::Number(2.0)],
                        vec![ArrayCellValue::Number(6.0), ArrayCellValue::Number(8.0)],
                    ])
                    .unwrap(),
                )),
                CallArgValue::Eval(EvalValue::Array(
                    EvalArray::from_rows(vec![
                        vec![ArrayCellValue::Number(1.0), ArrayCellValue::Number(0.0)],
                        vec![ArrayCellValue::Number(3.0), ArrayCellValue::Number(2.0)],
                    ])
                    .unwrap(),
                )),
            ],
            &NoResolver,
            |lhs, rhs| {
                if rhs == 0.0 {
                    Err(WorksheetErrorCode::Div0)
                } else {
                    Ok(lhs / rhs)
                }
            },
        )
        .unwrap();

        assert_eq!(
            got,
            EvalValue::Array(
                EvalArray::from_rows(vec![
                    vec![
                        ArrayCellValue::Number(1.0),
                        ArrayCellValue::Error(WorksheetErrorCode::Div0)
                    ],
                    vec![ArrayCellValue::Number(2.0), ArrayCellValue::Number(4.0)],
                ])
                .unwrap()
            )
        );
    }

    #[test]
    fn binary_numeric_surface_broadcasts_row_and_column_arrays() {
        let got = eval_binary_numeric_surface(
            &[
                CallArgValue::Eval(EvalValue::Array(
                    EvalArray::from_rows(vec![vec![
                        ArrayCellValue::Number(1.0),
                        ArrayCellValue::Number(2.0),
                    ]])
                    .unwrap(),
                )),
                CallArgValue::Eval(EvalValue::Array(
                    EvalArray::from_rows(vec![
                        vec![ArrayCellValue::Number(1.0)],
                        vec![ArrayCellValue::Number(2.0)],
                    ])
                    .unwrap(),
                )),
            ],
            &NoResolver,
            |lhs, rhs| Ok(lhs + rhs),
        )
        .unwrap();

        assert_eq!(
            got,
            EvalValue::Array(
                EvalArray::from_rows(vec![
                    vec![ArrayCellValue::Number(2.0), ArrayCellValue::Number(3.0)],
                    vec![ArrayCellValue::Number(3.0), ArrayCellValue::Number(4.0)],
                ])
                .unwrap()
            )
        );
    }

    #[test]
    fn binary_numeric_surface_marks_non_broadcastable_cells_as_na() {
        let got = eval_binary_numeric_surface(
            &[
                CallArgValue::Eval(EvalValue::Array(
                    EvalArray::from_rows(vec![vec![
                        ArrayCellValue::Number(1.0),
                        ArrayCellValue::Number(2.0),
                    ]])
                    .unwrap(),
                )),
                CallArgValue::Eval(EvalValue::Array(
                    EvalArray::from_rows(vec![vec![
                        ArrayCellValue::Number(1.0),
                        ArrayCellValue::Number(2.0),
                        ArrayCellValue::Number(3.0),
                    ]])
                    .unwrap(),
                )),
            ],
            &NoResolver,
            |lhs, rhs| Ok(lhs + rhs),
        );

        assert_eq!(
            got,
            Ok(EvalValue::Array(
                EvalArray::from_rows(vec![vec![
                    ArrayCellValue::Number(2.0),
                    ArrayCellValue::Number(4.0),
                    ArrayCellValue::Error(WorksheetErrorCode::NA),
                ]])
                .unwrap()
            ))
        );
    }
}
