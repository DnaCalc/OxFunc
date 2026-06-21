use crate::function::{
    Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile, FunctionMeta,
    HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::binary_numeric::{
    BinaryNumericSurfaceError, eval_binary_numeric_surface, map_binary_numeric_error_to_ws,
};
use crate::functions::combinatorics_common::combinations_of_int;
use crate::functions::factorial_common::trunc_nonnegative;
use crate::resolver::ReferenceSystemProvider;
use crate::value::CalcValue;
use crate::value::WorksheetErrorCode;

pub const COMBINA_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.COMBINA",
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
    precision_rounding_profile: FunctionMeta::DEFAULT_PRECISION_ROUNDING_PROFILE,
};

pub fn combina_kernel(n: f64, k: f64) -> Result<f64, WorksheetErrorCode> {
    let n = trunc_nonnegative(n)?;
    let k = trunc_nonnegative(k)?;
    if k == 0 {
        return Ok(1.0);
    }
    if n == 0 && k > 0 {
        return Err(WorksheetErrorCode::Num);
    }
    combinations_of_int(n + k - 1, k)
}

pub fn eval_combina_surface(
    args: &[crate::value::CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, BinaryNumericSurfaceError> {
    eval_binary_numeric_surface(args, resolver, combina_kernel)
}

pub fn map_combina_error_to_ws(e: &BinaryNumericSurfaceError) -> WorksheetErrorCode {
    map_binary_numeric_error_to_ws(e)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_bits(actual: f64, expected: f64) {
        assert_eq!(
            actual.to_bits(),
            expected.to_bits(),
            "{actual} vs {expected}"
        );
    }

    #[test]
    fn combina_meta_function_id_is_stable() {
        assert_eq!(COMBINA_META.function_id, "FUNC.COMBINA");
    }

    #[test]
    fn combina_kernel_matches_excel_boundary_lanes() {
        assert_eq!(combina_kernel(4.0, 3.0), Ok(20.0));
        assert_eq!(combina_kernel(5.9, 2.2), Ok(15.0));
        assert_eq!(combina_kernel(0.0, 0.0), Ok(1.0));
        assert_eq!(combina_kernel(0.0, 1.0), Err(WorksheetErrorCode::Num));
    }

    #[test]
    fn combina_exact_publication_controls_remain_exact() {
        assert_bits(combina_kernel(4.0, 3.0).expect("combina(4,3)"), 20.0_f64);
        assert_bits(combina_kernel(10.0, 3.0).expect("combina(10,3)"), 220.0_f64);
    }
}
