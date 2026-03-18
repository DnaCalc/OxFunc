use crate::coercion::CoercionError;
use crate::functions::adapters::{
    PreparedArgValue, coerce_prepared_to_number, run_values_only_prepared,
};
use crate::resolver::ReferenceResolver;
use crate::value::{CallArgValue, EvalValue, WorksheetErrorCode};

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
    run_values_only_prepared(
        args,
        resolver,
        |prepared| eval_binary_numeric_prepared(prepared, kernel),
        BinaryNumericSurfaceError::Coercion,
    )
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

    let lhs = coerce_prepared_to_number(&args[0]).map_err(BinaryNumericSurfaceError::Coercion)?;
    let rhs = coerce_prepared_to_number(&args[1]).map_err(BinaryNumericSurfaceError::Coercion)?;
    kernel(lhs, rhs)
        .map(EvalValue::Number)
        .map_err(BinaryNumericSurfaceError::Domain)
}

pub fn map_binary_numeric_error_to_ws(e: &BinaryNumericSurfaceError) -> WorksheetErrorCode {
    match e {
        BinaryNumericSurfaceError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        BinaryNumericSurfaceError::Coercion(CoercionError::WorksheetError(code)) => *code,
        BinaryNumericSurfaceError::Coercion(_) => WorksheetErrorCode::Value,
        BinaryNumericSurfaceError::Domain(code) => *code,
    }
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
}
