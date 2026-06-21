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

pub const EXP_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.EXP",
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
    // BUG-FUNC-027 / oxf-vgxs: EXP overflow is `#NUM!` in Excel, not `+Inf`.
    real_result_policy: ExcelRealPolicy::FINITE,
};

pub fn exp_kernel(n: f64) -> f64 {
    n.exp()
}

pub fn eval_exp_surface(
    args: &[crate::value::CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, UnaryNumericSurfaceError> {
    eval_unary_numeric_via_executor(
        args,
        resolver,
        UnaryNumericExecSpec::raw(exp_kernel, EXP_META.real_result_policy),
    )
}

pub fn map_exp_error_to_ws(e: &UnaryNumericSurfaceError) -> WorksheetErrorCode {
    map_unary_numeric_error_to_ws(e)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exp_meta_function_id_is_stable() {
        assert_eq!(EXP_META.function_id, "FUNC.EXP");
    }

    #[test]
    fn exp_kernel_matches_std() {
        assert_eq!(exp_kernel(1.0), std::f64::consts::E);
    }

    // BUG-FUNC-027 / oxf-vgxs: live Excel EXP(1000)=#NUM! (overflow guard on the surface).
    #[test]
    fn exp_overflow_maps_to_num() {
        assert_eq!(
            EXP_META
                .real_result_policy
                .publish(1000.0, exp_kernel(1000.0)),
            Err(WorksheetErrorCode::Num)
        );
        assert_eq!(
            EXP_META.real_result_policy.publish(1.0, exp_kernel(1.0)),
            Ok(std::f64::consts::E)
        );
    }
}
