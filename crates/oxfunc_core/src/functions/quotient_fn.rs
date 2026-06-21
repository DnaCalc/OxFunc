use crate::function::{
    Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile, FunctionMeta,
    HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::binary_numeric::{
    BinaryNumericSurfaceError, eval_binary_numeric_surface, map_binary_numeric_error_to_ws,
};
use crate::resolver::ReferenceSystemProvider;
use crate::value::CalcValue;
use crate::value::WorksheetErrorCode;

pub const QUOTIENT_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.QUOTIENT",
    arity: Arity::exact(2),
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: FunctionMeta::DEFAULT_ARG_PREPARATION_PROFILE,
    coercion_lift_profile: CoercionLiftProfile::UnaryNumericScalarOnly,
    lift_broadcast_profile: FunctionMeta::DEFAULT_LIFT_BROADCAST_PROFILE,
    kernel_signature_class: KernelSignatureClass::NumsToNum,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
    real_result_policy: FunctionMeta::DEFAULT_REAL_RESULT_POLICY,
    error_collapse_profile: FunctionMeta::DEFAULT_ERROR_COLLAPSE_PROFILE,
};

pub fn quotient_kernel(numerator: f64, denominator: f64) -> Result<f64, WorksheetErrorCode> {
    if denominator == 0.0 {
        return Err(WorksheetErrorCode::Div0);
    }
    Ok((numerator / denominator).trunc())
}

pub fn eval_quotient_surface(
    args: &[crate::value::CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, BinaryNumericSurfaceError> {
    eval_binary_numeric_surface(args, resolver, quotient_kernel)
}

pub fn map_quotient_error_to_ws(e: &BinaryNumericSurfaceError) -> WorksheetErrorCode {
    map_binary_numeric_error_to_ws(e)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quotient_meta_function_id_is_stable() {
        assert_eq!(QUOTIENT_META.function_id, "FUNC.QUOTIENT");
    }

    #[test]
    fn quotient_kernel_truncates_toward_zero() {
        assert_eq!(quotient_kernel(7.0, 3.0), Ok(2.0));
        assert_eq!(quotient_kernel(-7.0, 3.0), Ok(-2.0));
        assert_eq!(quotient_kernel(7.0, -3.0), Ok(-2.0));
        assert_eq!(quotient_kernel(-7.0, -3.0), Ok(2.0));
        assert_eq!(quotient_kernel(1.0, 0.0), Err(WorksheetErrorCode::Div0));
    }
}
