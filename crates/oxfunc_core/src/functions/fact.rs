use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, ExcelRealPolicy,
    FecDependencyProfile, FunctionMeta, HostInteractionClass, KernelSignatureClass,
    ThreadSafetyClass, VolatilityClass,
};
use crate::functions::factorial_common::{factorial_of_int, trunc_nonnegative_or_minus_one};
use crate::functions::unary_numeric::{
    UnaryNumericSurfaceError, eval_unary_numeric_surface, map_unary_numeric_error_to_ws,
};
use crate::resolver::ReferenceSystemProvider;
use crate::value::CalcValue;
use crate::value::WorksheetErrorCode;

pub const FACT_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.FACT",
    arity: Arity::exact(1),
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: ArgPreparationProfile::ValuesOnlyPreAdapter,
    coercion_lift_profile: CoercionLiftProfile::UnaryNumericScalarOrArrayElementwise,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
    // BUG-FUNC-027 / oxf-vgxs: 171! overflows to `+Inf`; Excel returns `#NUM!`.
    real_result_policy: ExcelRealPolicy::FINITE,
};

pub fn fact_kernel(n: f64) -> Result<f64, WorksheetErrorCode> {
    let truncated = trunc_nonnegative_or_minus_one(n)?;
    if truncated < 0 {
        return Err(WorksheetErrorCode::Num);
    }
    FACT_META
        .real_result_policy
        .publish(n, factorial_of_int(truncated))
}

pub fn eval_fact_surface(
    args: &[crate::value::CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, UnaryNumericSurfaceError> {
    eval_unary_numeric_surface(args, resolver, fact_kernel)
}

pub fn map_fact_error_to_ws(e: &UnaryNumericSurfaceError) -> WorksheetErrorCode {
    map_unary_numeric_error_to_ws(e)
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
    fn fact_meta_function_id_is_stable() {
        assert_eq!(FACT_META.function_id, "FUNC.FACT");
    }

    #[test]
    fn fact_kernel_truncates_and_rejects_negative() {
        assert_eq!(fact_kernel(5.9), Ok(120.0));
        assert_eq!(fact_kernel(-1.0), Err(WorksheetErrorCode::Num));
    }

    // BUG-FUNC-027 / oxf-vgxs: live Excel FACT(171)=#NUM!, FACT(170) finite.
    #[test]
    fn fact_kernel_overflow_maps_to_num() {
        assert_eq!(fact_kernel(171.0), Err(WorksheetErrorCode::Num));
        assert!(fact_kernel(170.0).is_ok());
    }

    #[test]
    fn fact_exact_publication_controls_remain_exact() {
        assert_bits(fact_kernel(9.0).expect("fact(9)"), 362880.0_f64);
        assert_bits(fact_kernel(2.0).expect("fact(2)"), 2.0_f64);
    }
}
