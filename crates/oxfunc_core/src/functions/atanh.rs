use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::unary_numeric::{
    UnaryNumericSurfaceError, eval_unary_numeric_surface, map_unary_numeric_error_to_ws,
};
use crate::resolver::ReferenceResolver;
use crate::value::{EvalValue, WorksheetErrorCode};

pub const ATANH_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.ATANH",
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

pub fn atanh_kernel(n: f64) -> Result<f64, WorksheetErrorCode> {
    if n.abs() >= 1.0 {
        return Err(WorksheetErrorCode::Num);
    }
    Ok(n.atanh())
}

pub fn eval_atanh_surface(
    args: &[crate::value::CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, UnaryNumericSurfaceError> {
    eval_unary_numeric_surface(args, resolver, atanh_kernel)
}

pub fn map_atanh_error_to_ws(e: &UnaryNumericSurfaceError) -> WorksheetErrorCode {
    map_unary_numeric_error_to_ws(e)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn atanh_meta_function_id_is_stable() {
        assert_eq!(ATANH_META.function_id, "FUNC.ATANH");
    }

    #[test]
    fn atanh_kernel_rejects_abs_one() {
        assert_eq!(atanh_kernel(1.0), Err(WorksheetErrorCode::Num));
        assert_eq!(atanh_kernel(-1.0), Err(WorksheetErrorCode::Num));
    }
}
