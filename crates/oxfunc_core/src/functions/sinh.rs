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

pub const SINH_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.SINH",
    arity: Arity::exact(1),
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: FunctionMeta::DEFAULT_ARG_PREPARATION_PROFILE,
    coercion_lift_profile: CoercionLiftProfile::UnaryNumericScalarOrArrayElementwise,
    lift_broadcast_profile: FunctionMeta::DEFAULT_LIFT_BROADCAST_PROFILE,
    kernel_signature_class: KernelSignatureClass::NumToNum,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
    // BUG-FUNC-027 CLASS-A3: SINH overflows to `#NUM!` in Excel, not `±Inf`.
    real_result_policy: ExcelRealPolicy::FINITE,
    error_collapse_profile: FunctionMeta::DEFAULT_ERROR_COLLAPSE_PROFILE,
    precision_rounding_profile: FunctionMeta::DEFAULT_PRECISION_ROUNDING_PROFILE,
};

pub fn sinh_kernel(n: f64) -> f64 {
    n.sinh()
}

pub fn eval_sinh_surface(
    args: &[crate::value::CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, UnaryNumericSurfaceError> {
    eval_unary_numeric_via_executor(
        args,
        resolver,
        UnaryNumericExecSpec::raw(sinh_kernel, SINH_META.real_result_policy),
    )
}

pub fn map_sinh_error_to_ws(e: &UnaryNumericSurfaceError) -> WorksheetErrorCode {
    map_unary_numeric_error_to_ws(e)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sinh_meta_function_id_is_stable() {
        assert_eq!(SINH_META.function_id, "FUNC.SINH");
    }

    #[test]
    fn sinh_kernel_matches_std() {
        assert_eq!(sinh_kernel(1.0), 1.0f64.sinh());
    }

    // BUG-FUNC-027 CLASS-A3: live Excel 16.0 b20026 SINH(-326648.33)=#NUM!.
    #[test]
    fn sinh_overflow_maps_to_num() {
        assert_eq!(
            SINH_META
                .real_result_policy
                .publish(-326648.33, sinh_kernel(-326648.33)),
            Err(WorksheetErrorCode::Num)
        );
        assert_eq!(
            SINH_META.real_result_policy.publish(1.0, sinh_kernel(1.0)),
            Ok(1.0f64.sinh())
        );
    }
}
