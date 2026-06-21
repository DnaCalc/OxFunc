use crate::function::{
    Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile, FunctionMeta,
    HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::unary_numeric::{
    UnaryNumericExecSpec, UnaryNumericSurfaceError, eval_unary_numeric_via_executor,
    map_unary_numeric_error_to_ws,
};
use crate::resolver::ReferenceSystemProvider;
use crate::value::CalcValue;
use crate::value::WorksheetErrorCode;

pub const SQRT_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.SQRT",
    arity: Arity::exact(1),
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: FunctionMeta::DEFAULT_ARG_PREPARATION_PROFILE,
    coercion_lift_profile: CoercionLiftProfile::UnaryNumericScalarOrArrayElementwise,
    lift_broadcast_profile: FunctionMeta::DEFAULT_LIFT_BROADCAST_PROFILE,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
    real_result_policy: FunctionMeta::DEFAULT_REAL_RESULT_POLICY,
    error_collapse_profile: FunctionMeta::DEFAULT_ERROR_COLLAPSE_PROFILE,
    precision_rounding_profile: FunctionMeta::DEFAULT_PRECISION_ROUNDING_PROFILE,
};

pub fn sqrt_kernel(n: f64) -> Result<f64, WorksheetErrorCode> {
    if n < 0.0 {
        return Err(WorksheetErrorCode::Num);
    }
    Ok(n.sqrt())
}

pub fn eval_sqrt_surface(
    args: &[crate::value::CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, UnaryNumericSurfaceError> {
    eval_unary_numeric_via_executor(
        args,
        resolver,
        UnaryNumericExecSpec::fallible(sqrt_kernel, SQRT_META.real_result_policy),
    )
}

pub fn map_sqrt_error_to_ws(e: &UnaryNumericSurfaceError) -> WorksheetErrorCode {
    map_unary_numeric_error_to_ws(e)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sqrt_meta_function_id_is_stable() {
        assert_eq!(SQRT_META.function_id, "FUNC.SQRT");
    }

    #[test]
    fn sqrt_kernel_negative_is_num() {
        assert_eq!(sqrt_kernel(-1.0), Err(WorksheetErrorCode::Num));
    }
}
