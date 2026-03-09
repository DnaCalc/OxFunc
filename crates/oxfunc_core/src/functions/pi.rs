use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::value::{EvalError, Value};

pub const PI_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.PI",
    arity: Arity::exact(0),
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: ArgPreparationProfile::ValuesOnlyPreAdapter,
    coercion_lift_profile: CoercionLiftProfile::None,
    kernel_signature_class: KernelSignatureClass::NullaryConst,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::None,
};

pub fn eval_pi(args: &[Value]) -> Result<Value, EvalError> {
    if !PI_META.arity.accepts(args.len()) {
        return Err(EvalError::ArityMismatch {
            expected: PI_META.arity.min,
            actual: args.len(),
        });
    }

    Ok(Value::Number(std::f64::consts::PI))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pi_meta_thread_safety_class_is_safe_pure() {
        assert_eq!(PI_META.thread_safety, ThreadSafetyClass::SafePure);
    }

    #[test]
    fn test_pi_meta_arg_preparation_profile_values_only() {
        assert_eq!(
            PI_META.arg_preparation_profile,
            ArgPreparationProfile::ValuesOnlyPreAdapter
        );
    }

    #[test]
    fn test_pi_meta_coercion_and_kernel_profiles() {
        assert_eq!(PI_META.coercion_lift_profile, CoercionLiftProfile::None);
        assert_eq!(
            PI_META.kernel_signature_class,
            KernelSignatureClass::NullaryConst
        );
    }

    #[test]
    fn test_pi_meta_adapter_and_surface_fec_profiles_none() {
        assert_eq!(PI_META.fec_dependency_profile, FecDependencyProfile::None);
        assert_eq!(
            PI_META.surface_fec_dependency_profile,
            FecDependencyProfile::None
        );
    }

    #[test]
    fn test_eval_pi_returns_pi_constant() {
        match eval_pi(&[]) {
            Ok(Value::Number(n)) => assert_eq!(n.to_bits(), std::f64::consts::PI.to_bits()),
            other => panic!("unexpected eval_pi outcome: {:?}", other),
        }
    }

    #[test]
    fn test_eval_pi_rejects_nonzero_args() {
        let args = [Value::Number(1.0)];
        let got = eval_pi(&args);
        assert_eq!(
            got,
            Err(EvalError::ArityMismatch {
                expected: 0,
                actual: 1
            })
        );
    }

    #[test]
    fn test_eval_pi_is_bitwise_stable() {
        for _ in 0..32 {
            match eval_pi(&[]) {
                Ok(Value::Number(n)) => assert_eq!(n.to_bits(), std::f64::consts::PI.to_bits()),
                other => panic!("unexpected eval_pi outcome: {:?}", other),
            }
        }
    }
}
