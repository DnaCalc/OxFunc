use crate::function::{
    Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile, FunctionMeta,
    HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::value::CalcValue;
use crate::value::{CoreValue, NumberFormatHint, PresentationHint, WorksheetErrorCode};

pub const TODAY_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.TODAY",
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
    error_collapse_profile: FunctionMeta::DEFAULT_ERROR_COLLAPSE_PROFILE,
};

pub trait TodayProvider {
    fn today_serial(&self) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub enum TodayEvalError {
    ArityMismatch { expected: usize, actual: usize },
    ProviderNonFinite(f64),
}

pub fn eval_today_surface(
    args: &[CalcValue],
    provider: &impl TodayProvider,
) -> Result<CalcValue, TodayEvalError> {
    if !TODAY_META.arity.accepts(args.len()) {
        return Err(TodayEvalError::ArityMismatch {
            expected: TODAY_META.arity.min,
            actual: args.len(),
        });
    }

    let serial = provider.today_serial();
    if !serial.is_finite() {
        return Err(TodayEvalError::ProviderNonFinite(serial));
    }

    Ok(CalcValue::number(serial.floor()))
}

pub fn eval_today_calc_surface(
    args: &[CalcValue],
    provider: &impl TodayProvider,
) -> Result<CalcValue, TodayEvalError> {
    if !TODAY_META.arity.accepts(args.len()) {
        return Err(TodayEvalError::ArityMismatch {
            expected: TODAY_META.arity.min,
            actual: args.len(),
        });
    }

    let serial = provider.today_serial();
    if !serial.is_finite() {
        return Err(TodayEvalError::ProviderNonFinite(serial));
    }

    Ok(CalcValue::with_presentation(
        CoreValue::Number(serial.floor()),
        PresentationHint::number_format(NumberFormatHint::DateLike),
    ))
}

pub fn eval_today_surface_rich(
    args: &[CalcValue],
    provider: &impl TodayProvider,
) -> Result<CalcValue, TodayEvalError> {
    eval_today_calc_surface(args, provider)
}

pub fn map_today_error_to_ws(e: &TodayEvalError) -> WorksheetErrorCode {
    match e {
        TodayEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        TodayEvalError::ProviderNonFinite(_) => WorksheetErrorCode::Value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FixedTodayProvider {
        serial: f64,
    }

    impl TodayProvider for FixedTodayProvider {
        fn today_serial(&self) -> f64 {
            self.serial
        }
    }

    #[test]
    fn eval_today_floors_provider_serial() {
        let got = eval_today_surface(&[], &FixedTodayProvider { serial: 46000.75 });
        assert_eq!(got, Ok(CalcValue::number(46000.0)));
    }

    #[test]
    fn eval_today_rich_wraps_value_with_number_format_hint() {
        let got = eval_today_surface_rich(&[], &FixedTodayProvider { serial: 46000.75 });
        assert_eq!(
            got,
            Ok(CalcValue::with_presentation(
                CoreValue::Number(46000.0),
                PresentationHint::number_format(NumberFormatHint::DateLike),
            ))
        );
    }

    #[test]
    fn eval_today_rejects_args() {
        let got = eval_today_surface(
            &[CalcValue::empty()],
            &FixedTodayProvider { serial: 46000.0 },
        );
        assert_eq!(
            got,
            Err(TodayEvalError::ArityMismatch {
                expected: 0,
                actual: 1,
            })
        );
    }
}
