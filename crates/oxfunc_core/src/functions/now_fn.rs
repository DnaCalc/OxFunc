use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::value::{
    CallArgValue, EvalValue, ExtendedValue, NumberFormatHint, PresentationHint, WorksheetErrorCode,
};

pub const NOW_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.NOW",
    arity: Arity::exact(0),
    determinism: DeterminismClass::TimeDependent,
    volatility: VolatilityClass::VolatileFull,
    host_interaction: HostInteractionClass::ApplicationState,
    thread_safety: ThreadSafetyClass::HostSerialized,
    arg_preparation_profile: ArgPreparationProfile::ValuesOnlyPreAdapter,
    coercion_lift_profile: CoercionLiftProfile::None,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::TimeProvider,
    surface_fec_dependency_profile: FecDependencyProfile::TimeProvider,
};

pub trait NowProvider {
    fn now_serial(&self) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub enum NowEvalError {
    ArityMismatch { expected: usize, actual: usize },
    ProviderNonFinite(f64),
}

pub fn eval_now_surface(
    args: &[CallArgValue],
    provider: &impl NowProvider,
) -> Result<EvalValue, NowEvalError> {
    if !NOW_META.arity.accepts(args.len()) {
        return Err(NowEvalError::ArityMismatch {
            expected: NOW_META.arity.min,
            actual: args.len(),
        });
    }

    let serial = provider.now_serial();
    if !serial.is_finite() {
        return Err(NowEvalError::ProviderNonFinite(serial));
    }

    Ok(EvalValue::Number(serial))
}

pub fn eval_now_surface_extended(
    args: &[CallArgValue],
    provider: &impl NowProvider,
) -> Result<ExtendedValue, NowEvalError> {
    let value = eval_now_surface(args, provider)?;
    Ok(ExtendedValue::ValueWithPresentation {
        value,
        hint: PresentationHint::number_format(NumberFormatHint::DateLike),
    })
}

pub fn map_now_error_to_ws(e: &NowEvalError) -> WorksheetErrorCode {
    match e {
        NowEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        NowEvalError::ProviderNonFinite(_) => WorksheetErrorCode::Value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FixedNowProvider {
        serial: f64,
    }

    impl NowProvider for FixedNowProvider {
        fn now_serial(&self) -> f64 {
            self.serial
        }
    }

    #[test]
    fn eval_now_uses_provider_serial_value() {
        let provider = FixedNowProvider { serial: 46000.25 };
        let got = eval_now_surface(&[], &provider);
        assert_eq!(got, Ok(EvalValue::Number(46000.25)));
    }

    #[test]
    fn eval_now_extended_wraps_value_with_number_format_hint() {
        let provider = FixedNowProvider { serial: 46000.25 };
        let got = eval_now_surface_extended(&[], &provider);
        assert_eq!(
            got,
            Ok(ExtendedValue::ValueWithPresentation {
                value: EvalValue::Number(46000.25),
                hint: PresentationHint::number_format(NumberFormatHint::DateLike),
            })
        );
    }

    #[test]
    fn eval_now_rejects_args() {
        let provider = FixedNowProvider { serial: 46000.25 };
        let got = eval_now_surface(&[CallArgValue::EmptyCell], &provider);
        assert_eq!(
            got,
            Err(NowEvalError::ArityMismatch {
                expected: 0,
                actual: 1,
            })
        );
    }

    #[test]
    fn eval_now_rejects_non_finite_provider_payload() {
        let provider = FixedNowProvider { serial: f64::NAN };
        let got = eval_now_surface(&[], &provider);
        assert!(matches!(got, Err(NowEvalError::ProviderNonFinite(_))));
    }
}
