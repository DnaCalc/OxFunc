use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::unary_numeric::{
    UnaryNumericSurfaceError, eval_unary_numeric_surface, map_unary_numeric_error_to_ws,
};
use crate::resolver::ReferenceResolver;
use crate::value::{EvalValue, WorksheetErrorCode};

pub const CSC_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.CSC",
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

pub fn csc_kernel(n: f64) -> Result<f64, WorksheetErrorCode> {
    let sin = n.sin();
    if sin == 0.0 {
        return Err(WorksheetErrorCode::Div0);
    }
    Ok(1.0 / sin)
}

pub fn eval_csc_surface(
    args: &[crate::value::CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, UnaryNumericSurfaceError> {
    eval_unary_numeric_surface(args, resolver, csc_kernel)
}

pub fn map_csc_error_to_ws(e: &UnaryNumericSurfaceError) -> WorksheetErrorCode {
    map_unary_numeric_error_to_ws(e)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn csc_meta_function_id_is_stable() {
        assert_eq!(CSC_META.function_id, "FUNC.CSC");
    }

    #[test]
    fn csc_kernel_zero_is_div0() {
        assert_eq!(csc_kernel(0.0), Err(WorksheetErrorCode::Div0));
    }
}
