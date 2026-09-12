use crate::function::{
    Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile, FunctionMeta,
    HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::unary_numeric::{
    UnaryNumericExecSpec, UnaryNumericSurfaceError, eval_unary_numeric_via_executor,
    map_unary_numeric_error_to_ws,
};
use crate::resolver::ReferenceSystemProvider;
use crate::value::CalcValue;
use crate::value::WorksheetErrorCode;

pub const SECH_META: FunctionMeta = function_spec! {
    function_id: "FUNC.SECH",
    arity: Arity::exact(1),
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    coercion_lift_profile: CoercionLiftProfile::UnaryNumericScalarOrArrayElementwise,
    kernel_signature_class: KernelSignatureClass::NumToNum,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
};

pub fn sech_kernel(n: f64) -> Result<f64, WorksheetErrorCode> {
    // Live Excel 16.0 b20326: SECH(x)=1/COSH(x) 40/40 on the COSH EXP-pair grid.
    Ok(1.0 / crate::functions::cosh::cosh_kernel(n))
}

pub fn eval_sech_surface(
    args: &[crate::value::CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, UnaryNumericSurfaceError> {
    eval_unary_numeric_via_executor(
        args,
        resolver,
        UnaryNumericExecSpec::fallible(sech_kernel, SECH_META.real_result_policy),
    )
}

pub fn map_sech_error_to_ws(e: &UnaryNumericSurfaceError) -> WorksheetErrorCode {
    map_unary_numeric_error_to_ws(e)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sech_meta_function_id_is_stable() {
        assert_eq!(SECH_META.function_id, "FUNC.SECH");
    }

    #[test]
    fn sech_kernel_zero_is_one() {
        assert_eq!(sech_kernel(0.0), Ok(1.0));
    }

    #[test]
    fn sech_matches_live_excel_one_over_cosh_pins() {
        // Live Excel 16.0 b20326: SECH(x)=1/COSH(x) 5/5.
        assert_eq!(sech_kernel(0.5).unwrap().to_bits(), 0x3fec60d1ff040dd1);
        assert_eq!(sech_kernel(1.0).unwrap().to_bits(), 0x3fe4bcdc50ed6be8);
    }
}
