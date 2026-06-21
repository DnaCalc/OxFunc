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

pub const COSH_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.COSH",
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
    // BUG-FUNC-027 CLASS-A3: COSH overflows to `#NUM!` in Excel, not `+Inf`.
    real_result_policy: ExcelRealPolicy::FINITE,
};

pub fn cosh_kernel(n: f64) -> f64 {
    n.cosh()
}

pub fn eval_cosh_surface(
    args: &[crate::value::CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, UnaryNumericSurfaceError> {
    eval_unary_numeric_via_executor(
        args,
        resolver,
        UnaryNumericExecSpec::raw(cosh_kernel, COSH_META.real_result_policy),
    )
}

pub fn map_cosh_error_to_ws(e: &UnaryNumericSurfaceError) -> WorksheetErrorCode {
    map_unary_numeric_error_to_ws(e)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cosh_meta_function_id_is_stable() {
        assert_eq!(COSH_META.function_id, "FUNC.COSH");
    }

    #[test]
    fn cosh_kernel_matches_std() {
        assert_eq!(cosh_kernel(1.0), 1.0f64.cosh());
    }

    // BUG-FUNC-027 CLASS-A3: live Excel 16.0 b20026 COSH(-24230)=#NUM!.
    #[test]
    fn cosh_overflow_maps_to_num() {
        assert_eq!(
            COSH_META
                .real_result_policy
                .publish(-24230.0, cosh_kernel(-24230.0)),
            Err(WorksheetErrorCode::Num)
        );
        assert_eq!(
            COSH_META.real_result_policy.publish(1.0, cosh_kernel(1.0)),
            Ok(1.0f64.cosh())
        );
    }
}
