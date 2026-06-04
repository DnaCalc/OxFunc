use crate::coercion::CoercionError;
use crate::functions::adapters::{
    BroadcastPreparedPair, coerce_prepared_to_number, expand_binary_broadcast_grid,
    prepare_args_values_only, prepare_calc_values_only, prepared_from_calc_value,
};
use crate::resolver::ReferenceSystemProvider;
use crate::value::CalcValue;
use crate::value::{CalcArray, WorksheetErrorCode};

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryNumericSurfaceError {
    ArityMismatch { expected: usize, actual: usize },
    Coercion(CoercionError),
    Domain(WorksheetErrorCode),
}

pub fn eval_binary_numeric_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
    kernel: impl Fn(f64, f64) -> Result<f64, WorksheetErrorCode> + Copy,
) -> Result<CalcValue, BinaryNumericSurfaceError> {
    let prepared =
        prepare_args_values_only(args, resolver).map_err(BinaryNumericSurfaceError::Coercion)?;
    eval_binary_numeric_prepared(&prepared, kernel)
}

pub fn eval_binary_numeric_calc_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
    kernel: impl Fn(f64, f64) -> Result<f64, WorksheetErrorCode> + Copy,
) -> Result<CalcValue, BinaryNumericSurfaceError> {
    let prepared_calc =
        prepare_calc_values_only(args, resolver).map_err(BinaryNumericSurfaceError::Coercion)?;
    let prepared = prepared_calc
        .iter()
        .map(prepared_from_calc_value)
        .collect::<Vec<_>>();
    eval_binary_numeric_prepared(&prepared, kernel).map(CalcValue::from)
}

pub fn eval_binary_numeric_prepared(
    args: &[CalcValue],
    kernel: impl Fn(f64, f64) -> Result<f64, WorksheetErrorCode> + Copy,
) -> Result<CalcValue, BinaryNumericSurfaceError> {
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
                    CalcValue::error(WorksheetErrorCode::NA)
                }
            })
            .collect();
        Ok(CalcValue::array(
            CalcArray::new(shape, mapped).expect("shape preserved"),
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
    lhs: &CalcValue,
    rhs: &CalcValue,
    kernel: impl Fn(f64, f64) -> Result<f64, WorksheetErrorCode> + Copy,
) -> CalcValue {
    let lhs = match coerce_prepared_to_number(lhs) {
        Ok(lhs) => lhs,
        Err(CoercionError::WorksheetError(code)) => return CalcValue::error(code),
        Err(_) => return CalcValue::error(WorksheetErrorCode::Value),
    };
    let rhs = match coerce_prepared_to_number(rhs) {
        Ok(rhs) => rhs,
        Err(CoercionError::WorksheetError(code)) => return CalcValue::error(code),
        Err(_) => return CalcValue::error(WorksheetErrorCode::Value),
    };

    match kernel(lhs, rhs) {
        Ok(value) => CalcValue::number(value),
        Err(code) => CalcValue::error(code),
    }
}

fn eval_binary_numeric_scalars(
    lhs: &CalcValue,
    rhs: &CalcValue,
    kernel: impl Fn(f64, f64) -> Result<f64, WorksheetErrorCode> + Copy,
) -> Result<CalcValue, BinaryNumericSurfaceError> {
    let lhs = coerce_prepared_to_number(lhs).map_err(BinaryNumericSurfaceError::Coercion)?;
    let rhs = coerce_prepared_to_number(rhs).map_err(BinaryNumericSurfaceError::Coercion)?;
    kernel(lhs, rhs)
        .map(CalcValue::number)
        .map_err(BinaryNumericSurfaceError::Domain)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::ReferenceSystemCapabilities;
    use crate::value::ExcelText;

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

    #[test]
    fn binary_numeric_surface_accepts_numeric_text() {
        let got = eval_binary_numeric_surface(
            &[
                (CalcValue::text(ExcelText::from_utf16_code_units(
                    "2".encode_utf16().collect(),
                ))),
                (CalcValue::number(3.0)),
            ],
            &NoResolver,
            |lhs, rhs| Ok(lhs + rhs),
        )
        .unwrap();
        assert_eq!(got, CalcValue::number(5.0));
    }

    #[test]
    fn binary_numeric_surface_maps_domain_errors() {
        let got = eval_binary_numeric_surface(
            &[(CalcValue::number(1.0)), (CalcValue::number(0.0))],
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
                (CalcValue::array(
                    CalcArray::from_rows(vec![vec![
                        CalcValue::number(1.0),
                        CalcValue::text(ExcelText::from_utf16_code_units(
                            "2".encode_utf16().collect(),
                        )),
                    ]])
                    .unwrap(),
                )),
                (CalcValue::number(10.0)),
            ],
            &NoResolver,
            |lhs, rhs| Ok(lhs + rhs),
        )
        .unwrap();

        assert_eq!(
            got,
            CalcValue::array(
                CalcArray::from_rows(vec![
                    vec![CalcValue::number(11.0), CalcValue::number(12.0),]
                ])
                .unwrap()
            )
        );
    }

    #[test]
    fn binary_numeric_surface_lifts_scalar_array_elementwise() {
        let got = eval_binary_numeric_surface(
            &[
                (CalcValue::number(2.0)),
                (CalcValue::array(
                    CalcArray::from_rows(vec![
                        vec![CalcValue::number(3.0), CalcValue::logical(true)],
                        vec![CalcValue::empty(), CalcValue::number(4.0)],
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
            CalcValue::array(
                CalcArray::from_rows(vec![
                    vec![CalcValue::number(6.0), CalcValue::number(2.0)],
                    vec![
                        CalcValue::error(WorksheetErrorCode::Value),
                        CalcValue::number(8.0)
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
                (CalcValue::array(
                    CalcArray::from_rows(vec![
                        vec![CalcValue::number(1.0), CalcValue::number(2.0)],
                        vec![CalcValue::number(6.0), CalcValue::number(8.0)],
                    ])
                    .unwrap(),
                )),
                (CalcValue::array(
                    CalcArray::from_rows(vec![
                        vec![CalcValue::number(1.0), CalcValue::number(0.0)],
                        vec![CalcValue::number(3.0), CalcValue::number(2.0)],
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
            CalcValue::array(
                CalcArray::from_rows(vec![
                    vec![
                        CalcValue::number(1.0),
                        CalcValue::error(WorksheetErrorCode::Div0)
                    ],
                    vec![CalcValue::number(2.0), CalcValue::number(4.0)],
                ])
                .unwrap()
            )
        );
    }

    #[test]
    fn binary_numeric_surface_broadcasts_row_and_column_arrays() {
        let got = eval_binary_numeric_surface(
            &[
                (CalcValue::array(
                    CalcArray::from_rows(vec![vec![
                        CalcValue::number(1.0),
                        CalcValue::number(2.0),
                    ]])
                    .unwrap(),
                )),
                (CalcValue::array(
                    CalcArray::from_rows(vec![
                        vec![CalcValue::number(1.0)],
                        vec![CalcValue::number(2.0)],
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
            CalcValue::array(
                CalcArray::from_rows(vec![
                    vec![CalcValue::number(2.0), CalcValue::number(3.0)],
                    vec![CalcValue::number(3.0), CalcValue::number(4.0)],
                ])
                .unwrap()
            )
        );
    }

    #[test]
    fn binary_numeric_surface_marks_non_broadcastable_cells_as_na() {
        let got = eval_binary_numeric_surface(
            &[
                (CalcValue::array(
                    CalcArray::from_rows(vec![vec![
                        CalcValue::number(1.0),
                        CalcValue::number(2.0),
                    ]])
                    .unwrap(),
                )),
                (CalcValue::array(
                    CalcArray::from_rows(vec![vec![
                        CalcValue::number(1.0),
                        CalcValue::number(2.0),
                        CalcValue::number(3.0),
                    ]])
                    .unwrap(),
                )),
            ],
            &NoResolver,
            |lhs, rhs| Ok(lhs + rhs),
        );

        assert_eq!(
            got,
            Ok(CalcValue::array(
                CalcArray::from_rows(vec![vec![
                    CalcValue::number(2.0),
                    CalcValue::number(4.0),
                    CalcValue::error(WorksheetErrorCode::NA),
                ]])
                .unwrap()
            ))
        );
    }
}
