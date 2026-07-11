use crate::function::{
    Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile, FunctionMeta,
    HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::excel_numeric::ExcelRealPolicy;
use crate::functions::unary_numeric::{
    UnaryNumericExecSpec, UnaryNumericSurfaceError, eval_unary_numeric_via_executor,
    map_unary_numeric_error_to_ws,
};
use crate::resolver::ReferenceSystemProvider;
use crate::value::CalcValue;
use crate::value::WorksheetErrorCode;

pub const SEC_META: FunctionMeta = function_spec! {
    function_id: "FUNC.SEC",
    arity: Arity::exact(1),
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    coercion_lift_profile: CoercionLiftProfile::UnaryNumericScalarOrArrayElementwise,
    kernel_signature_class: KernelSignatureClass::NumToNum,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
    // BUG-FUNC-027 CLASS-B2: circular trig is `#NUM!` once `|x| >= 2^27`.
    real_result_policy: ExcelRealPolicy::CIRCULAR_TRIG,
};

pub fn sec_kernel(n: f64) -> Result<f64, WorksheetErrorCode> {
    SEC_META.real_result_policy.check_arg(n)?;
    // W109 G4-01: SEC = RN53(RN64(1/cos)) over the RAW fFCOS chain (the
    // published-COS small-argument 1.0 shortcut composes identically here:
    // recip-dr of 1-ulp ties back to exactly 1.0).
    Ok(crate::excel_numeric::excel_x87_recip(
        crate::excel_numeric::excel_cos(n),
    ))
}

pub fn eval_sec_surface(
    args: &[crate::value::CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, UnaryNumericSurfaceError> {
    eval_unary_numeric_via_executor(
        args,
        resolver,
        UnaryNumericExecSpec::fallible(sec_kernel, SEC_META.real_result_policy),
    )
}

pub fn map_sec_error_to_ws(e: &UnaryNumericSurfaceError) -> WorksheetErrorCode {
    map_unary_numeric_error_to_ws(e)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sec_meta_function_id_is_stable() {
        assert_eq!(SEC_META.function_id, "FUNC.SEC");
    }

    #[test]
    fn sec_kernel_zero_is_one() {
        assert_eq!(sec_kernel(0.0), Ok(1.0));
    }
}
