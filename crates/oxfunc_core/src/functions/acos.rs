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

pub const ACOS_META: FunctionMeta = function_spec! {
    function_id: "FUNC.ACOS",
    arity: Arity::exact(1),
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    coercion_lift_profile: CoercionLiftProfile::UnaryNumericScalarOrArrayElementwise,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
};

pub fn acos_kernel(n: f64) -> Result<f64, WorksheetErrorCode> {
    // Live Excel 16.0 b20326: ACOS(x)=PI()/2-ASIN(x) 34/34 including
    // the 0.75 row where ATAN2 is 1 ULP. libm acos is 1 ULP off ACOS(0.5).
    match crate::functions::asin::asin_kernel(n) {
        Ok(a) => Ok(std::f64::consts::FRAC_PI_2 - a),
        Err(_) => Err(WorksheetErrorCode::Num),
    }
}

pub fn eval_acos_surface(
    args: &[crate::value::CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, UnaryNumericSurfaceError> {
    eval_unary_numeric_via_executor(
        args,
        resolver,
        UnaryNumericExecSpec::fallible(acos_kernel, ACOS_META.real_result_policy),
    )
}

pub fn map_acos_error_to_ws(e: &UnaryNumericSurfaceError) -> WorksheetErrorCode {
    map_unary_numeric_error_to_ws(e)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn acos_meta_function_id_is_stable() {
        assert_eq!(ACOS_META.function_id, "FUNC.ACOS");
    }

    #[test]
    fn acos_kernel_rejects_out_of_domain() {
        assert_eq!(acos_kernel(2.0), Err(WorksheetErrorCode::Num));
    }

    #[test]
    fn acos_matches_live_excel_pi2_asin_pins() {
        assert_eq!(acos_kernel(0.5).unwrap().to_bits(), 0x3ff0c152382d7365);
        assert_eq!(acos_kernel(0.75).unwrap().to_bits(), 0x3fe720a392c1d954);
        assert_eq!(acos_kernel(0.0).unwrap().to_bits(), 0x3ff921fb54442d18);
        assert_eq!(acos_kernel(1.0).unwrap().to_bits(), 0x0000000000000000);
        assert_eq!(acos_kernel(-1.0).unwrap().to_bits(), 0x400921fb54442d18);
        assert_eq!(acos_kernel(-0.5).unwrap().to_bits(), 0x4000c152382d7366);
    }
}
