use crate::function::{
    Arity, CoercionLiftProfile, DeterminismClass, ExcelRealPolicy, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::unary_numeric::{
    UnaryNumericExecSpec, UnaryNumericSurfaceError, eval_unary_numeric_via_executor,
    map_unary_numeric_error_to_ws,
};
use crate::resolver::ReferenceSystemProvider;
use crate::value::CalcValue;
use crate::value::WorksheetErrorCode;

pub const COTH_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.COTH",
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
    // BUG-FUNC-027 CLASS-C3.h: for large |n|, cosh/sinh = Inf/Inf = NaN; Excel saturates
    // COTH to `sign(n)` i.e. `±1`. Verified live Excel 16.0 b20026: COTH(800)=1.
    real_result_policy: ExcelRealPolicy::SATURATE_SIGN,
    error_collapse_profile: FunctionMeta::DEFAULT_ERROR_COLLAPSE_PROFILE,
};

pub fn coth_kernel(n: f64) -> Result<f64, WorksheetErrorCode> {
    let sinh = n.sinh();
    if sinh == 0.0 {
        return Err(WorksheetErrorCode::Div0);
    }
    // Non-finite cosh/sinh (large |n|) saturates to sign(n) under COTH's real policy.
    COTH_META.real_result_policy.publish(n, n.cosh() / sinh)
}

pub fn eval_coth_surface(
    args: &[crate::value::CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, UnaryNumericSurfaceError> {
    eval_unary_numeric_via_executor(
        args,
        resolver,
        UnaryNumericExecSpec::fallible(coth_kernel, COTH_META.real_result_policy),
    )
}

pub fn map_coth_error_to_ws(e: &UnaryNumericSurfaceError) -> WorksheetErrorCode {
    map_unary_numeric_error_to_ws(e)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coth_meta_function_id_is_stable() {
        assert_eq!(COTH_META.function_id, "FUNC.COTH");
    }

    #[test]
    fn coth_kernel_zero_is_div0() {
        assert_eq!(coth_kernel(0.0), Err(WorksheetErrorCode::Div0));
    }

    // BUG-FUNC-027 CLASS-C3.h: large |n| saturates to ±1 (live Excel COTH(800)=1).
    #[test]
    fn coth_kernel_large_argument_saturates() {
        assert_eq!(coth_kernel(800.0), Ok(1.0));
        assert_eq!(coth_kernel(-800.0), Ok(-1.0));
        // Interior values are unchanged.
        assert_eq!(coth_kernel(1.0), Ok(1.0_f64.cosh() / 1.0_f64.sinh()));
    }
}
