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

pub const TANH_META: FunctionMeta = function_spec! {
    function_id: "FUNC.TANH",
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

pub fn tanh_kernel(n: f64) -> f64 {
    // Live Excel 16.0 b20326: TANH(x)=SINH(x)/COSH(x) 8/8. libm tanh is
    // 1 ULP off the first pin.
    crate::functions::sinh::sinh_kernel(n) / crate::functions::cosh::cosh_kernel(n)
}

pub fn eval_tanh_surface(
    args: &[crate::value::CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, UnaryNumericSurfaceError> {
    eval_unary_numeric_via_executor(
        args,
        resolver,
        UnaryNumericExecSpec::raw(tanh_kernel, TANH_META.real_result_policy),
    )
}

pub fn map_tanh_error_to_ws(e: &UnaryNumericSurfaceError) -> WorksheetErrorCode {
    map_unary_numeric_error_to_ws(e)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tanh_meta_function_id_is_stable() {
        assert_eq!(TANH_META.function_id, "FUNC.TANH");
    }

    #[test]
    fn tanh_matches_live_excel_sinh_cosh_pins() {
        // Live Excel 16.0 b20326: TANH(x)=SINH(x)/COSH(x) 8/8.
        assert_eq!(tanh_kernel(0.5).to_bits(), 0x3fdd9353d7568af4);
        assert_eq!(tanh_kernel(1.0).to_bits(), 0x3fe85efab514f394);
        assert_eq!(tanh_kernel(-0.5).to_bits(), 0xbfdd9353d7568af4);
    }

    #[test]
    fn tanh_kernel_matches_std() {
        assert_eq!(tanh_kernel(1.0), 1.0f64.tanh());
    }
}
