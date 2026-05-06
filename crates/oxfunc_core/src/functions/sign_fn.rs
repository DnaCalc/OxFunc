use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::unary_numeric::{
    UnaryNumericSurfaceError, eval_unary_numeric_surface, map_unary_numeric_error_to_ws,
};
use crate::resolver::ReferenceResolver;
use crate::value::{EvalValue, WorksheetErrorCode};

pub const SIGN_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.SIGN",
    arity: Arity::exact(1),
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: ArgPreparationProfile::ValuesOnlyPreAdapter,
    coercion_lift_profile: CoercionLiftProfile::UnaryNumericScalarOrArrayElementwise,
    kernel_signature_class: KernelSignatureClass::NumToNum,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
};

pub fn sign_kernel(n: f64) -> Result<f64, WorksheetErrorCode> {
    Ok(if n > 0.0 {
        1.0
    } else if n < 0.0 {
        -1.0
    } else {
        0.0
    })
}

pub fn eval_sign_surface(
    args: &[crate::value::CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, UnaryNumericSurfaceError> {
    eval_unary_numeric_surface(args, resolver, sign_kernel)
}

pub fn map_sign_error_to_ws(e: &UnaryNumericSurfaceError) -> WorksheetErrorCode {
    map_unary_numeric_error_to_ws(e)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sign_meta_function_id_is_stable() {
        assert_eq!(SIGN_META.function_id, "FUNC.SIGN");
    }

    #[test]
    fn sign_kernel_maps_signs() {
        assert_eq!(sign_kernel(-2.0), Ok(-1.0));
        assert_eq!(sign_kernel(0.0), Ok(0.0));
        assert_eq!(sign_kernel(2.0), Ok(1.0));
    }
}
