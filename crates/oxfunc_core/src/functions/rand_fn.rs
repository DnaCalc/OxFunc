use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::value::{CallArgValue, EvalValue, WorksheetErrorCode};

pub const RAND_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.RAND",
    arity: Arity::exact(0),
    determinism: DeterminismClass::PseudoRandom,
    volatility: VolatilityClass::VolatileFull,
    host_interaction: HostInteractionClass::ApplicationState,
    thread_safety: ThreadSafetyClass::HostSerialized,
    arg_preparation_profile: ArgPreparationProfile::ValuesOnlyPreAdapter,
    coercion_lift_profile: CoercionLiftProfile::None,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::RandomProvider,
    surface_fec_dependency_profile: FecDependencyProfile::RandomProvider,
};

pub trait RandomProvider {
    fn random_unit(&self) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub enum RandEvalError {
    ArityMismatch { expected: usize, actual: usize },
    ProviderOutOfRange(f64),
}

pub fn eval_rand_surface(
    args: &[CallArgValue],
    provider: &impl RandomProvider,
) -> Result<EvalValue, RandEvalError> {
    if !RAND_META.arity.accepts(args.len()) {
        return Err(RandEvalError::ArityMismatch {
            expected: RAND_META.arity.min,
            actual: args.len(),
        });
    }

    let value = provider.random_unit();
    if !value.is_finite() || !(0.0..1.0).contains(&value) {
        return Err(RandEvalError::ProviderOutOfRange(value));
    }

    Ok(EvalValue::Number(value))
}

pub fn map_rand_error_to_ws(e: &RandEvalError) -> WorksheetErrorCode {
    match e {
        RandEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        RandEvalError::ProviderOutOfRange(_) => WorksheetErrorCode::Value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FixedRandomProvider {
        value: f64,
    }

    impl RandomProvider for FixedRandomProvider {
        fn random_unit(&self) -> f64 {
            self.value
        }
    }

    #[test]
    fn eval_rand_uses_provider_value() {
        let got = eval_rand_surface(&[], &FixedRandomProvider { value: 0.25 });
        assert_eq!(got, Ok(EvalValue::Number(0.25)));
    }

    #[test]
    fn eval_rand_rejects_out_of_range_provider_value() {
        let got = eval_rand_surface(&[], &FixedRandomProvider { value: 1.5 });
        assert_eq!(got, Err(RandEvalError::ProviderOutOfRange(1.5)));
    }

    #[test]
    fn eval_rand_rejects_non_finite_and_negative_values() {
        let got_nan = eval_rand_surface(&[], &FixedRandomProvider { value: f64::NAN });
        assert!(matches!(got_nan, Err(RandEvalError::ProviderOutOfRange(v)) if v.is_nan()));

        let got_negative = eval_rand_surface(&[], &FixedRandomProvider { value: -0.1 });
        assert_eq!(got_negative, Err(RandEvalError::ProviderOutOfRange(-0.1)));
    }
}
