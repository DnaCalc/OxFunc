use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::value::{EvalValue, WorksheetErrorCode};

pub const FALSE_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.FALSE",
    arity: Arity::exact(0),
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: ArgPreparationProfile::ValuesOnlyPreAdapter,
    coercion_lift_profile: CoercionLiftProfile::None,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::None,
};

pub fn eval_false_surface(
    args: &[crate::value::CallArgValue],
) -> Result<EvalValue, WorksheetErrorCode> {
    if !args.is_empty() {
        return Err(WorksheetErrorCode::Value);
    }
    Ok(EvalValue::Logical(false))
}
