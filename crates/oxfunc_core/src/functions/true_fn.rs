use crate::function::{
    Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile, FunctionMeta,
    HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::value::CalcValue;
use crate::value::WorksheetErrorCode;

pub const TRUE_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.TRUE",
    arity: Arity::exact(0),
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: FunctionMeta::DEFAULT_ARG_PREPARATION_PROFILE,
    coercion_lift_profile: CoercionLiftProfile::None,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::None,
    real_result_policy: FunctionMeta::DEFAULT_REAL_RESULT_POLICY,
};

pub fn eval_true_surface(
    args: &[crate::value::CalcValue],
) -> Result<CalcValue, WorksheetErrorCode> {
    if !args.is_empty() {
        return Err(WorksheetErrorCode::Value);
    }
    Ok(CalcValue::logical(true))
}
