use crate::function::{
    Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile, FunctionMeta,
    HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::unary_numeric::{
    eval_unary_numeric_via_executor, map_unary_numeric_error_to_ws, UnaryNumericExecSpec,
    UnaryNumericSurfaceError,
};
use crate::resolver::ReferenceSystemProvider;
use crate::value::CalcValue;
use crate::value::WorksheetErrorCode;

pub const ACOT_META: FunctionMeta = function_spec! {
    function_id: "FUNC.ACOT",
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

pub fn acot_kernel(n: f64) -> Result<f64, WorksheetErrorCode> {
    // W111 distinguishes this reciprocal/angle graph from direct atan2 and
    // from a binary64-only reciprocal. It is an observed arithmetic model,
    // not a claim about Excel internals or universal FPATAN parity.
    if n == 0.0 {
        return Ok(std::f64::consts::FRAC_PI_2);
    }
    let reciprocal = crate::excel_numeric::excel_x87_recip(n);
    let angle = crate::excel_numeric::excel_atan(reciprocal);
    let result = if n < 0.0 {
        angle + std::f64::consts::PI
    } else {
        angle
    };
    Ok(if result.abs() < f64::MIN_POSITIVE {
        0.0
    } else {
        result
    })
}

pub fn eval_acot_surface(
    args: &[crate::value::CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, UnaryNumericSurfaceError> {
    eval_unary_numeric_via_executor(
        args,
        resolver,
        UnaryNumericExecSpec::fallible(acot_kernel, ACOT_META.real_result_policy),
    )
}

pub fn map_acot_error_to_ws(e: &UnaryNumericSurfaceError) -> WorksheetErrorCode {
    map_unary_numeric_error_to_ws(e)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn acot_meta_function_id_is_stable() {
        assert_eq!(ACOT_META.function_id, "FUNC.ACOT");
    }

    #[test]
    fn acot_kernel_zero_is_pi_over_two() {
        assert_eq!(acot_kernel(0.0), Ok(std::f64::consts::FRAC_PI_2));
    }

    #[test]
    fn acot_matches_live_excel_pins() {
        // Live Excel 16.0 b20326 ATAN2(x,1) identity.
        assert_eq!(acot_kernel(0.5).unwrap().to_bits(), 0x3ff1b6e192ebbe44);
        assert_eq!(acot_kernel(1.0).unwrap().to_bits(), 0x3fe921fb54442d18);
        assert_eq!(acot_kernel(2.0).unwrap().to_bits(), 0x3fddac670561bb4f);
        assert_eq!(acot_kernel(-2.0).unwrap().to_bits(), 0x40056c6e7397f5ae);
        assert_eq!(acot_kernel(4.0).unwrap().to_bits(), 0x3fcf5b75f92c80dd);
    }
}
