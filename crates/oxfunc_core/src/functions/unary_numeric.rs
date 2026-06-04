use crate::coercion::{CoercionError, coerce_calc_scalar_to_number};
use crate::functions::adapters::{
    apply_unary_numeric_scalar_prepared, expand_arg_values_only, prepare_arg_values_only,
    prepare_calc_value_values_only,
};
use crate::resolver::ReferenceSystemProvider;
use crate::value::CalcValue;
use crate::value::{CalcArray, CoreValue, WorksheetErrorCode};

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryNumericSurfaceError {
    ArityMismatch { expected: usize, actual: usize },
    Coercion(CoercionError),
    Domain(WorksheetErrorCode),
}

pub fn eval_unary_numeric_surface(
    args: &[crate::value::CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
    kernel: impl Fn(f64) -> Result<f64, WorksheetErrorCode> + Copy,
) -> Result<CalcValue, UnaryNumericSurfaceError> {
    if args.len() != 1 {
        return Err(UnaryNumericSurfaceError::ArityMismatch {
            expected: 1,
            actual: args.len(),
        });
    }

    let prepared =
        prepare_arg_values_only(&args[0], resolver).map_err(UnaryNumericSurfaceError::Coercion)?;

    match prepared.core() {
        CoreValue::Array(array) => {
            let mapped = expand_arg_values_only(&args[0], resolver)
                .map_err(UnaryNumericSurfaceError::Coercion)?
                .into_iter()
                .map(|item| map_unary_numeric_item(&item, kernel))
                .collect::<Vec<_>>();
            Ok(CalcValue::array(
                CalcArray::new(array.shape(), mapped).expect("shape preserved"),
            ))
        }
        _ => match apply_unary_numeric_scalar_prepared(&prepared, |n| n) {
            Ok(n) => kernel(n)
                .map(CalcValue::number)
                .map_err(UnaryNumericSurfaceError::Domain),
            Err(err) => Err(UnaryNumericSurfaceError::Coercion(err)),
        },
    }
}

pub fn eval_unary_numeric_calc_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
    kernel: impl Fn(f64) -> Result<f64, WorksheetErrorCode> + Copy,
) -> Result<CalcValue, UnaryNumericSurfaceError> {
    if args.len() != 1 {
        return Err(UnaryNumericSurfaceError::ArityMismatch {
            expected: 1,
            actual: args.len(),
        });
    }

    let prepared = prepare_calc_value_values_only(&args[0], resolver)
        .map_err(UnaryNumericSurfaceError::Coercion)?;
    match prepared.core() {
        CoreValue::Array(array) => {
            let cells = array
                .iter_row_major()
                .map(|item| map_unary_numeric_calc_item(item, kernel))
                .collect::<Vec<_>>();
            Ok(CalcValue::array(
                CalcArray::new(array.shape(), cells).expect("shape preserved"),
            ))
        }
        _ => match coerce_calc_scalar_to_number(&prepared) {
            Ok(n) => kernel(n)
                .map(CalcValue::number)
                .map_err(UnaryNumericSurfaceError::Domain),
            Err(err) => Err(UnaryNumericSurfaceError::Coercion(err)),
        },
    }
}

fn map_unary_numeric_item(
    item: &CalcValue,
    kernel: impl Fn(f64) -> Result<f64, WorksheetErrorCode> + Copy,
) -> CalcValue {
    match apply_unary_numeric_scalar_prepared(item, |n| n) {
        Ok(n) => match kernel(n) {
            Ok(v) => CalcValue::number(v),
            Err(code) => CalcValue::error(code),
        },
        Err(CoercionError::WorksheetError(code)) => CalcValue::error(code),
        Err(_) => CalcValue::error(WorksheetErrorCode::Value),
    }
}

fn map_unary_numeric_calc_item(
    item: &CalcValue,
    kernel: impl Fn(f64) -> Result<f64, WorksheetErrorCode> + Copy,
) -> CalcValue {
    match coerce_calc_scalar_to_number(item) {
        Ok(n) => match kernel(n) {
            Ok(v) => CalcValue::number(v),
            Err(code) => CalcValue::error(code),
        },
        Err(CoercionError::WorksheetError(code)) => CalcValue::error(code),
        Err(_) => CalcValue::error(WorksheetErrorCode::Value),
    }
}

pub fn map_unary_numeric_error_to_ws(e: &UnaryNumericSurfaceError) -> WorksheetErrorCode {
    match e {
        UnaryNumericSurfaceError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        UnaryNumericSurfaceError::Coercion(CoercionError::WorksheetError(code)) => *code,
        UnaryNumericSurfaceError::Coercion(_) => WorksheetErrorCode::Value,
        UnaryNumericSurfaceError::Domain(code) => *code,
    }
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
    fn unary_numeric_surface_accepts_numeric_text() {
        let got = eval_unary_numeric_surface(
            &[(CalcValue::text(ExcelText::from_utf16_code_units(
                "1".encode_utf16().collect(),
            )))],
            &NoResolver,
            |n| Ok(n + 1.0),
        )
        .unwrap();
        assert_eq!(got, CalcValue::number(2.0));
    }

    #[test]
    fn unary_numeric_surface_lifts_arrays_elementwise() {
        let got = eval_unary_numeric_surface(
            &[(CalcValue::array(
                CalcArray::from_rows(vec![vec![
                    CalcValue::number(1.0),
                    CalcValue::text(ExcelText::from_utf16_code_units(
                        "bad".encode_utf16().collect(),
                    )),
                ]])
                .unwrap(),
            ))],
            &NoResolver,
            |n| Ok(n * 2.0),
        )
        .unwrap();
        assert_eq!(
            got,
            CalcValue::array(
                CalcArray::from_rows(vec![vec![
                    CalcValue::number(2.0),
                    CalcValue::error(WorksheetErrorCode::Value),
                ]])
                .unwrap()
            )
        );
    }

    #[test]
    fn unary_numeric_surface_maps_domain_errors() {
        let got = eval_unary_numeric_surface(&[(CalcValue::number(-1.0))], &NoResolver, |_| {
            Err(WorksheetErrorCode::Num)
        });
        assert_eq!(
            got,
            Err(UnaryNumericSurfaceError::Domain(WorksheetErrorCode::Num))
        );
    }
}
