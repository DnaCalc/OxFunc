use crate::coercion::CoercionError;
use crate::function::{
    Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile, FunctionMeta,
    HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{
    apply_unary_numeric_scalar_prepared, expand_arg_values_only, prepare_arg_values_only,
};
use crate::resolver::ReferenceSystemProvider;
use crate::value::CalcValue;
use crate::value::{CalcArray, CoreValue, WorksheetErrorCode};

pub const ASIN_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.ASIN",
    arity: Arity::exact(1),
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: FunctionMeta::DEFAULT_ARG_PREPARATION_PROFILE,
    coercion_lift_profile: CoercionLiftProfile::UnaryNumericScalarOrArrayElementwise,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
    real_result_policy: FunctionMeta::DEFAULT_REAL_RESULT_POLICY,
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
    args: &[crate::value::CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, AsinEvalError> {
    if !ASIN_META.arity.accepts(args.len()) {
        return Err(AsinEvalError::ArityMismatch {
            expected: ASIN_META.arity.min,
            actual: args.len(),
        });
    }

    let prepared = prepare_arg_values_only(&args[0], resolver).map_err(AsinEvalError::Coercion)?;
    match prepared.core() {
        CoreValue::Array(array) => {
            let mapped = expand_arg_values_only(&args[0], resolver)
                .map_err(AsinEvalError::Coercion)?
                .into_iter()
                .map(|item| {
                    let n = apply_unary_numeric_scalar_prepared(&item, |x| x)
                        .map_err(AsinEvalError::Coercion);
                    match n {
                        Ok(n) => match asin_kernel(n) {
                            Ok(v) => Ok(CalcValue::number(v)),
                            Err(AsinEvalError::Domain) => {
                                Ok(CalcValue::error(WorksheetErrorCode::Num))
                            }
                            Err(other) => Err(other),
                        },
                        Err(AsinEvalError::Coercion(CoercionError::WorksheetError(code))) => {
                            Ok(CalcValue::error(code))
                        }
                        Err(AsinEvalError::Coercion(_)) => {
                            Ok(CalcValue::error(WorksheetErrorCode::Value))
                        }
                        Err(other) => Err(other),
                    }
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(CalcValue::array(
                CalcArray::new(array.shape(), mapped).expect("shape preserved"),
            ))
        }
        _ => {
            let n = apply_unary_numeric_scalar_prepared(&prepared, |x| x)
                .map_err(AsinEvalError::Coercion)?;
            Ok(CalcValue::number(asin_kernel(n)?))
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
    fn eval_asin_domain_error_is_num() {
        let got = eval_asin_surface(&[(CalcValue::number(2.0))], &NoResolver);
        assert_eq!(
            map_asin_error_to_ws(&got.unwrap_err()),
            WorksheetErrorCode::Num
        );
    }

    #[test]
    fn eval_asin_accepts_numeric_text() {
        let got = eval_asin_surface(
            &[(CalcValue::text(ExcelText::from_utf16_code_units(
                "1".encode_utf16().collect(),
            )))],
            &NoResolver,
        );
        assert_eq!(got, Ok(CalcValue::number(std::f64::consts::FRAC_PI_2)));
    }

    #[test]
    fn eval_asin_array_lifts_with_element_errors() {
        let got = eval_asin_surface(
            &[(CalcValue::array(
                CalcArray::from_rows(vec![vec![CalcValue::number(0.0), CalcValue::number(2.0)]])
                    .unwrap(),
            ))],
            &NoResolver,
        )
        .unwrap();
        assert_eq!(
            got,
            CalcValue::array(
                CalcArray::from_rows(vec![vec![
                    CalcValue::number(0.0),
                    CalcValue::error(WorksheetErrorCode::Num),
                ]])
                .unwrap()
            )
        );
    }
}
