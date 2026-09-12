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

pub const CSCH_META: FunctionMeta = function_spec! {
    function_id: "FUNC.CSCH",
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

pub fn csch_kernel(n: f64) -> Result<f64, WorksheetErrorCode> {
    let sinh = n.sinh();
    if sinh == 0.0 {
        return Err(WorksheetErrorCode::Div0);
    }
    Ok(1.0 / sinh)
}

pub fn eval_csch_surface(
    args: &[crate::value::CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, UnaryNumericSurfaceError> {
    eval_unary_numeric_via_executor(
        args,
        resolver,
        UnaryNumericExecSpec::fallible(csch_kernel, CSCH_META.real_result_policy),
    )
}

pub fn map_csch_error_to_ws(e: &UnaryNumericSurfaceError) -> WorksheetErrorCode {
    map_unary_numeric_error_to_ws(e)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn csch_meta_function_id_is_stable() {
        assert_eq!(CSCH_META.function_id, "FUNC.CSCH");
    }

    #[test]
    fn csch_kernel_zero_is_div0() {
        assert_eq!(csch_kernel(0.0), Err(WorksheetErrorCode::Div0));
    }

    #[test]
    fn csch_matches_live_excel_one_over_sinh_pins() {
        // Live Excel 16.0 b20326: CSCH(x)=1/SINH(x) 5/5.
        assert_eq!(csch_kernel(0.5).unwrap().to_bits(), 0x3ffeb45dc88defed);
        assert_eq!(csch_kernel(1.0).unwrap().to_bits(), 0x3feb3ab8a78b90c1);
    }
}
