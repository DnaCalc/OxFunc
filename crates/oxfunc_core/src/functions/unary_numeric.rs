use crate::coercion::{CoercionError, coerce_calc_scalar_to_number};
use crate::functions::adapters::{
    PreparedValue, apply_unary_numeric_scalar_prepared, expand_arg_values_only,
    prepare_arg_values_only, prepare_calc_value_values_only,
};
use crate::resolver::ReferenceSystemProvider;
use crate::value::{
    CalcArray, CalcValue, CoreValue, FunctionArray, FunctionArrayCell, FunctionValue,
    WorksheetErrorCode,
};

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryNumericSurfaceError {
    ArityMismatch { expected: usize, actual: usize },
    Coercion(CoercionError),
    Domain(WorksheetErrorCode),
}

pub fn eval_unary_numeric_surface(
    args: &[crate::value::FunctionArg],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
    kernel: impl Fn(f64) -> Result<f64, WorksheetErrorCode> + Copy,
) -> Result<FunctionValue, UnaryNumericSurfaceError> {
    if args.len() != 1 {
        return Err(UnaryNumericSurfaceError::ArityMismatch {
            expected: 1,
            actual: args.len(),
        });
    }

    let prepared =
        prepare_arg_values_only(&args[0], resolver).map_err(UnaryNumericSurfaceError::Coercion)?;

    match prepared {
        PreparedValue::Eval(FunctionValue::Array(array)) => {
            let mapped = expand_arg_values_only(&args[0], resolver)
                .map_err(UnaryNumericSurfaceError::Coercion)?
                .into_iter()
                .map(|item| map_unary_numeric_item(&item, kernel))
                .collect::<Vec<_>>();
            Ok(FunctionValue::Array(
                FunctionArray::new(array.shape(), mapped).expect("shape preserved"),
            ))
        }
        other => match apply_unary_numeric_scalar_prepared(&other, |n| n) {
            Ok(n) => kernel(n)
                .map(FunctionValue::Number)
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
    item: &PreparedValue,
    kernel: impl Fn(f64) -> Result<f64, WorksheetErrorCode> + Copy,
) -> FunctionArrayCell {
    match apply_unary_numeric_scalar_prepared(item, |n| n) {
        Ok(n) => match kernel(n) {
            Ok(v) => FunctionArrayCell::Number(v),
            Err(code) => FunctionArrayCell::Error(code),
        },
        Err(CoercionError::WorksheetError(code)) => FunctionArrayCell::Error(code),
        Err(_) => FunctionArrayCell::Error(WorksheetErrorCode::Value),
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
    use crate::value::{ExcelText, FunctionArg};

    struct NoResolver;

    impl ReferenceSystemProvider for NoResolver {
        fn capabilities(&self) -> ReferenceSystemCapabilities {
            ReferenceSystemCapabilities::permissive_local()
        }

        fn dereference(
            &self,
            request: &crate::resolver::ReferenceDereferenceRequest,
        ) -> Result<FunctionValue, crate::resolver::ReferenceResolutionError> {
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
            &[FunctionArg::Eval(FunctionValue::Text(
                ExcelText::from_utf16_code_units("1".encode_utf16().collect()),
            ))],
            &NoResolver,
            |n| Ok(n + 1.0),
        )
        .unwrap();
        assert_eq!(got, FunctionValue::Number(2.0));
    }

    #[test]
    fn unary_numeric_surface_lifts_arrays_elementwise() {
        let got = eval_unary_numeric_surface(
            &[FunctionArg::Eval(FunctionValue::Array(
                FunctionArray::from_rows(vec![vec![
                    FunctionArrayCell::Number(1.0),
                    FunctionArrayCell::Text(ExcelText::from_utf16_code_units(
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
            FunctionValue::Array(
                FunctionArray::from_rows(vec![vec![
                    FunctionArrayCell::Number(2.0),
                    FunctionArrayCell::Error(WorksheetErrorCode::Value),
                ]])
                .unwrap()
            )
        );
    }

    #[test]
    fn unary_numeric_surface_maps_domain_errors() {
        let got = eval_unary_numeric_surface(
            &[FunctionArg::Eval(FunctionValue::Number(-1.0))],
            &NoResolver,
            |_| Err(WorksheetErrorCode::Num),
        );
        assert_eq!(
            got,
            Err(UnaryNumericSurfaceError::Domain(WorksheetErrorCode::Num))
        );
    }
}
