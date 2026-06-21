use crate::coercion::{CoercionError, coerce_calc_scalar_to_number};
use crate::function::{
    Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile, FunctionMeta,
    HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::expand_aggregate_arg;
use crate::functions::factorial_common::trunc_nonnegative;
use crate::functions::gcd_lcm_common::gcd_int;
use crate::resolver::ReferenceSystemProvider;
use crate::value::CalcValue;
use crate::value::WorksheetErrorCode;

pub const GCD_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.GCD",
    arity: Arity { min: 1, max: 255 },
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: FunctionMeta::DEFAULT_ARG_PREPARATION_PROFILE,
    coercion_lift_profile: CoercionLiftProfile::Custom,
    lift_broadcast_profile: FunctionMeta::DEFAULT_LIFT_BROADCAST_PROFILE,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
    real_result_policy: FunctionMeta::DEFAULT_REAL_RESULT_POLICY,
};

#[derive(Debug, Clone, PartialEq)]
pub enum GcdEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
    Domain(WorksheetErrorCode),
}

fn coerce_calc_to_nonnegative_int(arg: &CalcValue) -> Result<i64, GcdEvalError> {
    let n = coerce_calc_scalar_to_number(arg).map_err(GcdEvalError::Coercion)?;
    trunc_nonnegative(n).map_err(GcdEvalError::Domain)
}

pub fn gcd_kernel(items: &[i64]) -> f64 {
    items.iter().copied().fold(0, gcd_int) as f64
}

pub fn eval_gcd_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, GcdEvalError> {
    let argc = args.len();
    if !GCD_META.arity.accepts(argc) {
        return Err(GcdEvalError::ArityMismatch {
            expected_min: GCD_META.arity.min,
            expected_max: GCD_META.arity.max,
            actual: argc,
        });
    }
    let mut items = Vec::new();
    for arg in args {
        let expanded = expand_aggregate_arg(arg, resolver).map_err(GcdEvalError::Coercion)?;
        for item in expanded {
            items.push(coerce_calc_to_nonnegative_int(&item.0)?);
        }
    }
    Ok(CalcValue::number(gcd_kernel(&items)))
}

pub fn map_gcd_error_to_ws(e: &GcdEvalError) -> WorksheetErrorCode {
    match e {
        GcdEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        GcdEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        GcdEvalError::Coercion(_) => WorksheetErrorCode::Value,
        GcdEvalError::Domain(code) => *code,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::{ReferenceSystemCapabilities, ReferenceSystemProvider};
    use crate::value::CalcArray;

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
    fn gcd_meta_function_id_is_stable() {
        assert_eq!(GCD_META.function_id, "FUNC.GCD");
    }

    #[test]
    fn gcd_kernel_matches_excel_seed_rows() {
        assert_eq!(gcd_kernel(&[24, 36]), 12.0);
        assert_eq!(gcd_kernel(&[0, 5]), 5.0);
        assert_eq!(gcd_kernel(&[0, 0]), 0.0);
    }

    #[test]
    fn ftc_0959_gcd_array_input_reduces_literal_vector_and_scalar_to_one() {
        let got = eval_gcd_surface(
            &[
                (CalcValue::array(
                    CalcArray::from_rows(vec![vec![
                        CalcValue::number(1.0),
                        CalcValue::number(2.0),
                        CalcValue::number(3.0),
                        CalcValue::number(4.0),
                        CalcValue::number(5.0),
                        CalcValue::number(6.0),
                        CalcValue::number(7.0),
                        CalcValue::number(8.0),
                        CalcValue::number(9.0),
                        CalcValue::number(10.0),
                        CalcValue::number(11.0),
                        CalcValue::number(12.0),
                    ]])
                    .unwrap(),
                )),
                (CalcValue::number(12.0)),
            ],
            &NoResolver,
        );
        assert_eq!(got, Ok(CalcValue::number(1.0)));
    }
}
