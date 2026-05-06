use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::unary_numeric::{
    UnaryNumericSurfaceError, eval_unary_numeric_surface, map_unary_numeric_error_to_ws,
};
use crate::resolver::ReferenceResolver;
use crate::value::{EvalValue, WorksheetErrorCode};

pub const CSCH_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.CSCH",
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
};

pub fn csch_kernel(n: f64) -> Result<f64, WorksheetErrorCode> {
    let sinh = n.sinh();
    if sinh == 0.0 {
        return Err(WorksheetErrorCode::Div0);
    }
    Ok(1.0 / sinh)
}

pub fn eval_csch_surface(
    args: &[crate::value::CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, UnaryNumericSurfaceError> {
    eval_unary_numeric_surface(args, resolver, csch_kernel)
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
}
