use crate::function::{
    Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile, FunctionMeta,
    HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::value::CalcValue;
use crate::value::{CoreValue, NumberFormatHint, PresentationHint, WorksheetErrorCode};

pub const NOW_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.NOW",
    arity: Arity::exact(0),
    determinism: DeterminismClass::TimeDependent,
    volatility: VolatilityClass::VolatileFull,
    host_interaction: HostInteractionClass::ApplicationState,
    thread_safety: ThreadSafetyClass::HostSerialized,
    arg_preparation_profile: FunctionMeta::DEFAULT_ARG_PREPARATION_PROFILE,
    coercion_lift_profile: CoercionLiftProfile::None,
    lift_broadcast_profile: FunctionMeta::DEFAULT_LIFT_BROADCAST_PROFILE,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::TimeProvider,
    surface_fec_dependency_profile: FecDependencyProfile::TimeProvider,
    real_result_policy: FunctionMeta::DEFAULT_REAL_RESULT_POLICY,
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
    args: &[CalcValue],
    provider: &impl NowProvider,
) -> Result<CalcValue, NowEvalError> {
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

    Ok(CalcValue::number(serial))
}

pub fn eval_now_calc_surface(
    args: &[CalcValue],
    provider: &impl NowProvider,
) -> Result<CalcValue, NowEvalError> {
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

    Ok(CalcValue::with_presentation(
        CoreValue::Number(serial),
        PresentationHint::number_format(NumberFormatHint::DateLike),
    ))
}

pub fn eval_now_surface_rich(
    args: &[CalcValue],
    provider: &impl NowProvider,
) -> Result<CalcValue, NowEvalError> {
    eval_now_calc_surface(args, provider)
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
        assert_eq!(got, Ok(CalcValue::number(46000.25)));
    }

    #[test]
    fn eval_now_rich_wraps_value_with_number_format_hint() {
        let provider = FixedNowProvider { serial: 46000.25 };
        let got = eval_now_surface_rich(&[], &provider);
        assert_eq!(
            got,
            Ok(CalcValue::with_presentation(
                CoreValue::Number(46000.25),
                PresentationHint::number_format(NumberFormatHint::DateLike),
            ))
        );
    }

    #[test]
    fn eval_now_rejects_args() {
        let provider = FixedNowProvider { serial: 46000.25 };
        let got = eval_now_surface(&[CalcValue::empty()], &provider);
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
