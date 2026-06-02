use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{
    PreparedValue, apply_unary_numeric_scalar_prepared, expand_arg_values_only,
    prepare_arg_values_only,
};
use crate::resolver::ReferenceSystemProvider;
use crate::value::{FunctionArray, FunctionArrayCell, FunctionValue, WorksheetErrorCode};

pub const ASIN_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.ASIN",
    arity: Arity::exact(1),
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: ArgPreparationProfile::ValuesOnlyPreAdapter,
    coercion_lift_profile: CoercionLiftProfile::UnaryNumericScalarOrArrayElementwise,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
};

#[derive(Debug, Clone, PartialEq)]
pub enum AsinEvalError {
    ArityMismatch { expected: usize, actual: usize },
    Coercion(CoercionError),
    Domain,
}

pub fn asin_kernel(n: f64) -> Result<f64, AsinEvalError> {
    if !(-1.0..=1.0).contains(&n) {
        return Err(AsinEvalError::Domain);
    }
    Ok(n.asin())
}

pub fn eval_asin_surface(
    args: &[crate::value::FunctionArg],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<FunctionValue, AsinEvalError> {
    if !ASIN_META.arity.accepts(args.len()) {
        return Err(AsinEvalError::ArityMismatch {
            expected: ASIN_META.arity.min,
            actual: args.len(),
        });
    }

    let prepared = prepare_arg_values_only(&args[0], resolver).map_err(AsinEvalError::Coercion)?;
    match prepared {
        PreparedValue::Eval(FunctionValue::Array(array)) => {
            let mapped = expand_arg_values_only(&args[0], resolver)
                .map_err(AsinEvalError::Coercion)?
                .into_iter()
                .map(|item| {
                    let n = apply_unary_numeric_scalar_prepared(&item, |x| x)
                        .map_err(AsinEvalError::Coercion);
                    match n {
                        Ok(n) => match asin_kernel(n) {
                            Ok(v) => Ok(FunctionArrayCell::Number(v)),
                            Err(AsinEvalError::Domain) => {
                                Ok(FunctionArrayCell::Error(WorksheetErrorCode::Num))
                            }
                            Err(other) => Err(other),
                        },
                        Err(AsinEvalError::Coercion(CoercionError::WorksheetError(code))) => {
                            Ok(FunctionArrayCell::Error(code))
                        }
                        Err(AsinEvalError::Coercion(_)) => {
                            Ok(FunctionArrayCell::Error(WorksheetErrorCode::Value))
                        }
                        Err(other) => Err(other),
                    }
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(FunctionValue::Array(
                FunctionArray::new(array.shape(), mapped).expect("shape preserved"),
            ))
        }
        other => {
            let n = apply_unary_numeric_scalar_prepared(&other, |x| x)
                .map_err(AsinEvalError::Coercion)?;
            Ok(FunctionValue::Number(asin_kernel(n)?))
        }
    }
}

pub fn map_asin_error_to_ws(e: &AsinEvalError) -> WorksheetErrorCode {
    match e {
        AsinEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        AsinEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        AsinEvalError::Coercion(_) => WorksheetErrorCode::Value,
        AsinEvalError::Domain => WorksheetErrorCode::Num,
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
    fn eval_asin_domain_error_is_num() {
        let got = eval_asin_surface(
            &[FunctionArg::Eval(FunctionValue::Number(2.0))],
            &NoResolver,
        );
        assert_eq!(
            map_asin_error_to_ws(&got.unwrap_err()),
            WorksheetErrorCode::Num
        );
    }

    #[test]
    fn eval_asin_accepts_numeric_text() {
        let got = eval_asin_surface(
            &[FunctionArg::Eval(FunctionValue::Text(
                ExcelText::from_utf16_code_units("1".encode_utf16().collect()),
            ))],
            &NoResolver,
        );
        assert_eq!(got, Ok(FunctionValue::Number(std::f64::consts::FRAC_PI_2)));
    }

    #[test]
    fn eval_asin_array_lifts_with_element_errors() {
        let got = eval_asin_surface(
            &[FunctionArg::Eval(FunctionValue::Array(
                FunctionArray::from_rows(vec![vec![
                    FunctionArrayCell::Number(0.0),
                    FunctionArrayCell::Number(2.0),
                ]])
                .unwrap(),
            ))],
            &NoResolver,
        )
        .unwrap();
        assert_eq!(
            got,
            FunctionValue::Array(
                FunctionArray::from_rows(vec![vec![
                    FunctionArrayCell::Number(0.0),
                    FunctionArrayCell::Error(WorksheetErrorCode::Num),
                ]])
                .unwrap()
            )
        );
    }
}
