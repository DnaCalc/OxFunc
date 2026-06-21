use crate::function::{
    Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile, FunctionMeta,
    HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::binary_numeric::{
    BinaryNumericSurfaceError, eval_binary_numeric_surface, map_binary_numeric_error_to_ws,
};
use crate::functions::bit_common::coerce_bit_operand;
use crate::resolver::ReferenceSystemProvider;
use crate::value::CalcValue;
use crate::value::WorksheetErrorCode;

pub const BITOR_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.BITOR",
    arity: Arity::exact(2),
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: FunctionMeta::DEFAULT_ARG_PREPARATION_PROFILE,
    coercion_lift_profile: CoercionLiftProfile::UnaryNumericScalarOnly,
    kernel_signature_class: KernelSignatureClass::NumsToNum,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
    real_result_policy: FunctionMeta::DEFAULT_REAL_RESULT_POLICY,
};

pub fn bitor_kernel(lhs: f64, rhs: f64) -> Result<f64, WorksheetErrorCode> {
    Ok((coerce_bit_operand(lhs)? | coerce_bit_operand(rhs)?) as f64)
}

pub fn eval_bitor_surface(
    args: &[crate::value::CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, BinaryNumericSurfaceError> {
    eval_binary_numeric_surface(args, resolver, bitor_kernel)
}

pub fn map_bitor_error_to_ws(e: &BinaryNumericSurfaceError) -> WorksheetErrorCode {
    map_binary_numeric_error_to_ws(e)
}
