use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::binary_numeric::{
    BinaryNumericSurfaceError, eval_binary_numeric_surface, map_binary_numeric_error_to_ws,
};
use crate::resolver::ReferenceResolver;
use crate::value::{EvalValue, WorksheetErrorCode};

pub const MOD_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.MOD",
    arity: Arity::exact(2),
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: ArgPreparationProfile::ValuesOnlyPreAdapter,
    coercion_lift_profile: CoercionLiftProfile::UnaryNumericScalarOnly,
    kernel_signature_class: KernelSignatureClass::NumsToNum,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
};

pub fn mod_kernel(number: f64, divisor: f64) -> Result<f64, WorksheetErrorCode> {
    if divisor == 0.0 {
        return Err(WorksheetErrorCode::Div0);
    }
    Ok(number - divisor * (number / divisor).floor())
}

pub fn eval_mod_surface(
    args: &[crate::value::CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, BinaryNumericSurfaceError> {
    eval_binary_numeric_surface(args, resolver, mod_kernel)
}

pub fn map_mod_error_to_ws(e: &BinaryNumericSurfaceError) -> WorksheetErrorCode {
    map_binary_numeric_error_to_ws(e)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mod_meta_function_id_is_stable() {
        assert_eq!(MOD_META.function_id, "FUNC.MOD");
    }

    #[test]
    fn mod_kernel_matches_excel_sign_behavior() {
        assert_eq!(mod_kernel(3.0, 2.0), Ok(1.0));
        assert_eq!(mod_kernel(-3.0, 2.0), Ok(1.0));
        assert_eq!(mod_kernel(3.0, -2.0), Ok(-1.0));
        assert_eq!(mod_kernel(-3.0, -2.0), Ok(-1.0));
        assert_eq!(mod_kernel(3.0, 0.0), Err(WorksheetErrorCode::Div0));
    }
}
