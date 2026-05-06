use crate::coercion::CoercionError;
use crate::functions::adapters::{
    PreparedArgValue, apply_unary_numeric_scalar_prepared, expand_arg_values_only,
    prepare_arg_values_only,
};
use crate::resolver::ReferenceResolver;
use crate::value::{ArrayCellValue, EvalArray, EvalValue, WorksheetErrorCode};

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryNumericSurfaceError {
    ArityMismatch { expected: usize, actual: usize },
    Coercion(CoercionError),
    Domain(WorksheetErrorCode),
}

pub fn eval_unary_numeric_surface(
    args: &[crate::value::CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
    kernel: impl Fn(f64) -> Result<f64, WorksheetErrorCode> + Copy,
) -> Result<EvalValue, UnaryNumericSurfaceError> {
    if args.len() != 1 {
        return Err(UnaryNumericSurfaceError::ArityMismatch {
            expected: 1,
            actual: args.len(),
        });
    }

    let prepared =
        prepare_arg_values_only(&args[0], resolver).map_err(UnaryNumericSurfaceError::Coercion)?;

    match prepared {
        PreparedArgValue::Eval(EvalValue::Array(array)) => {
            let mapped = expand_arg_values_only(&args[0], resolver)
                .map_err(UnaryNumericSurfaceError::Coercion)?
                .into_iter()
                .map(|item| map_unary_numeric_item(&item, kernel))
                .collect::<Vec<_>>();
            Ok(EvalValue::Array(
                EvalArray::new(array.shape(), mapped).expect("shape preserved"),
            ))
        }
        other => match apply_unary_numeric_scalar_prepared(&other, |n| n) {
            Ok(n) => kernel(n)
                .map(EvalValue::Number)
                .map_err(UnaryNumericSurfaceError::Domain),
            Err(err) => Err(UnaryNumericSurfaceError::Coercion(err)),
        },
    }
}

fn map_unary_numeric_item(
    item: &PreparedArgValue,
    kernel: impl Fn(f64) -> Result<f64, WorksheetErrorCode> + Copy,
) -> ArrayCellValue {
    match apply_unary_numeric_scalar_prepared(item, |n| n) {
        Ok(n) => match kernel(n) {
            Ok(v) => ArrayCellValue::Number(v),
            Err(code) => ArrayCellValue::Error(code),
        },
        Err(CoercionError::WorksheetError(code)) => ArrayCellValue::Error(code),
        Err(_) => ArrayCellValue::Error(WorksheetErrorCode::Value),
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
    use crate::resolver::{RefResolutionError, ResolverCapabilities};
    use crate::value::{CallArgValue, ExcelText, ReferenceLike};

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
    fn unary_numeric_surface_accepts_numeric_text() {
        let got = eval_unary_numeric_surface(
            &[CallArgValue::Eval(EvalValue::Text(
                ExcelText::from_utf16_code_units("1".encode_utf16().collect()),
            ))],
            &NoResolver,
            |n| Ok(n + 1.0),
        )
        .unwrap();
        assert_eq!(got, EvalValue::Number(2.0));
    }

    #[test]
    fn unary_numeric_surface_lifts_arrays_elementwise() {
        let got = eval_unary_numeric_surface(
            &[CallArgValue::Eval(EvalValue::Array(
                EvalArray::from_rows(vec![vec![
                    ArrayCellValue::Number(1.0),
                    ArrayCellValue::Text(ExcelText::from_utf16_code_units(
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
            EvalValue::Array(
                EvalArray::from_rows(vec![vec![
                    ArrayCellValue::Number(2.0),
                    ArrayCellValue::Error(WorksheetErrorCode::Value),
                ]])
                .unwrap()
            )
        );
    }

    #[test]
    fn unary_numeric_surface_maps_domain_errors() {
        let got = eval_unary_numeric_surface(
            &[CallArgValue::Eval(EvalValue::Number(-1.0))],
            &NoResolver,
            |_| Err(WorksheetErrorCode::Num),
        );
        assert_eq!(
            got,
            Err(UnaryNumericSurfaceError::Domain(WorksheetErrorCode::Num))
        );
    }
}
