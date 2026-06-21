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

pub const DEGREES_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.DEGREES",
    arity: Arity::exact(1),
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: FunctionMeta::DEFAULT_ARG_PREPARATION_PROFILE,
    coercion_lift_profile: CoercionLiftProfile::UnaryNumericScalarOrArrayElementwise,
    kernel_signature_class: KernelSignatureClass::NumToNum,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
    // BUG-FUNC-027 / oxf-vgxs: DEGREES overflow is `#NUM!` in Excel, not `±Inf`.
    real_result_policy: ExcelRealPolicy::FINITE,
};

pub fn degrees_kernel(n: f64) -> f64 {
    n * (180.0 / std::f64::consts::PI)
}

pub fn eval_degrees_surface(
    args: &[crate::value::CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, UnaryNumericSurfaceError> {
    eval_unary_numeric_via_executor(
        args,
        resolver,
        UnaryNumericExecSpec::raw(degrees_kernel, DEGREES_META.real_result_policy),
    )
}

pub fn map_degrees_error_to_ws(e: &UnaryNumericSurfaceError) -> WorksheetErrorCode {
    map_unary_numeric_error_to_ws(e)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn degrees_meta_function_id_is_stable() {
        assert_eq!(DEGREES_META.function_id, "FUNC.DEGREES");
    }

    #[test]
    fn degrees_kernel_maps_pi_to_180() {
        assert_eq!(degrees_kernel(std::f64::consts::PI), 180.0);
    }

    // BUG-FUNC-027 / oxf-vgxs: live Excel DEGREES(1E307)=#NUM! (overflow guard on surface).
    #[test]
    fn degrees_overflow_maps_to_num() {
        assert_eq!(
            DEGREES_META
                .real_result_policy
                .publish(1e307, degrees_kernel(1e307)),
            Err(WorksheetErrorCode::Num)
        );
        assert_eq!(
            DEGREES_META
                .real_result_policy
                .publish(std::f64::consts::PI, degrees_kernel(std::f64::consts::PI)),
            Ok(180.0)
        );
    }
}
