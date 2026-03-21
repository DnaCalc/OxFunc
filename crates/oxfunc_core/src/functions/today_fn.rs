use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::value::{
    CallArgValue, EvalValue, ExtendedValue, NumberFormatHint, PresentationHint, WorksheetErrorCode,
};

pub const TODAY_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.TODAY",
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

pub trait TodayProvider {
    fn today_serial(&self) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub enum TodayEvalError {
    ArityMismatch { expected: usize, actual: usize },
    ProviderNonFinite(f64),
}

pub fn eval_today_surface(
    args: &[CallArgValue],
    provider: &impl TodayProvider,
) -> Result<EvalValue, TodayEvalError> {
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

    Ok(EvalValue::Number(serial.floor()))
}

pub fn eval_today_surface_extended(
    args: &[CallArgValue],
    provider: &impl TodayProvider,
) -> Result<ExtendedValue, TodayEvalError> {
    let value = eval_today_surface(args, provider)?;
    Ok(ExtendedValue::ValueWithPresentation {
        value,
        hint: PresentationHint::number_format(NumberFormatHint::DateLike),
    })
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
        assert_eq!(got, Ok(EvalValue::Number(46000.0)));
    }

    #[test]
    fn eval_today_extended_wraps_value_with_number_format_hint() {
        let got = eval_today_surface_extended(&[], &FixedTodayProvider { serial: 46000.75 });
        assert_eq!(
            got,
            Ok(ExtendedValue::ValueWithPresentation {
                value: EvalValue::Number(46000.0),
                hint: PresentationHint::number_format(NumberFormatHint::DateLike),
            })
        );
    }

    #[test]
    fn eval_today_rejects_args() {
        let got = eval_today_surface(
            &[CallArgValue::EmptyCell],
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
