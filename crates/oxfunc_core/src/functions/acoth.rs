use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::unary_numeric::{
    UnaryNumericSurfaceError, eval_unary_numeric_surface, map_unary_numeric_error_to_ws,
};
use crate::resolver::ReferenceSystemProvider;
use crate::value::CalcValue;
use crate::value::WorksheetErrorCode;

pub const ACOTH_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.ACOTH",
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
    real_result_policy: FunctionMeta::DEFAULT_REAL_RESULT_POLICY,
};

pub fn acoth_kernel(n: f64) -> Result<f64, WorksheetErrorCode> {
    if n.abs() <= 1.0 {
        return Err(WorksheetErrorCode::Num);
    }
    Ok(0.5 * ((n + 1.0) / (n - 1.0)).ln())
}

pub fn eval_acoth_surface(
    args: &[crate::value::CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, UnaryNumericSurfaceError> {
    eval_unary_numeric_surface(args, resolver, acoth_kernel)
}

pub fn map_acoth_error_to_ws(e: &UnaryNumericSurfaceError) -> WorksheetErrorCode {
    map_unary_numeric_error_to_ws(e)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn acoth_meta_function_id_is_stable() {
        assert_eq!(ACOTH_META.function_id, "FUNC.ACOTH");
    }

    #[test]
    fn acoth_kernel_rejects_abs_one() {
        assert_eq!(acoth_kernel(1.0), Err(WorksheetErrorCode::Num));
        assert_eq!(acoth_kernel(-1.0), Err(WorksheetErrorCode::Num));
    }

    #[test]
    fn acoth_just_above_one_is_finite_matching_excel() {
        // BUG-FUNC-027 C5: the "Excel #NUM!" near-1 witness was a formula-literal
        // artifact (the parser rounded 1+ULP down to 1.0). With the exact input
        // 1 + 2^-52, live Excel 16.0 b20026 returns 18.36840028483855, which this
        // kernel matches bit-for-bit — so it must stay finite, not collapse to #NUM!.
        let y = acoth_kernel(1.0 + f64::EPSILON).expect("finite just above 1");
        assert!((y - 18.36840028483855).abs() < 1e-10, "got {y}");
    }
}
