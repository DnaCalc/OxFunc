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

pub const ACOSH_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.ACOSH",
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

pub fn acosh_kernel(n: f64) -> Result<f64, WorksheetErrorCode> {
    if n < 1.0 {
        return Err(WorksheetErrorCode::Num);
    }
    Ok(n.acosh())
}

pub fn eval_acosh_surface(
    args: &[crate::value::CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, UnaryNumericSurfaceError> {
    eval_unary_numeric_surface(args, resolver, acosh_kernel)
}

pub fn map_acosh_error_to_ws(e: &UnaryNumericSurfaceError) -> WorksheetErrorCode {
    map_unary_numeric_error_to_ws(e)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn acosh_meta_function_id_is_stable() {
        assert_eq!(ACOSH_META.function_id, "FUNC.ACOSH");
    }

    #[test]
    fn acosh_kernel_rejects_values_below_one() {
        assert_eq!(acosh_kernel(0.5), Err(WorksheetErrorCode::Num));
    }

    #[test]
    fn acosh_just_above_one_is_nonzero_matching_excel() {
        // BUG-FUNC-027 C5: the "Excel collapses to 0" near-1 witness was a
        // formula-literal artifact (the parser rounded 1+1e-15 down to 1.0). With
        // the exact input, live Excel 16.0 b20026 returns 4.712160905917527e-08,
        // which this kernel matches — so it must stay non-zero, not collapse to 0.
        let y = acosh_kernel(1.0 + 1e-15).expect("non-zero just above 1");
        assert!(y > 0.0, "got {y}");
        assert!(
            (y / 4.712_160_905_917_527e-8 - 1.0).abs() < 1e-12,
            "got {y}"
        );
    }
}
