use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::unary_numeric::{
    UnaryNumericSurfaceError, eval_unary_numeric_surface, map_unary_numeric_error_to_ws,
};
use crate::resolver::ReferenceResolver;
use crate::value::{EvalValue, WorksheetErrorCode};

pub const ACOTH_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.ACOTH",
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

pub fn acoth_kernel(n: f64) -> Result<f64, WorksheetErrorCode> {
    if n.abs() <= 1.0 {
        return Err(WorksheetErrorCode::Num);
    }
    Ok(0.5 * ((n + 1.0) / (n - 1.0)).ln())
}

pub fn eval_acoth_surface(
    args: &[crate::value::CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, UnaryNumericSurfaceError> {
    eval_unary_numeric_surface(args, resolver, acoth_kernel)
}

pub fn map_acoth_error_to_ws(e: &UnaryNumericSurfaceError) -> WorksheetErrorCode {
    map_unary_numeric_error_to_ws(e)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn acoth_meta_function_id_is_stable() {
        assert_eq!(ACOTH_META.function_id, "FUNC.ACOTH");
    }

    #[test]
    fn acoth_kernel_rejects_abs_one() {
        assert_eq!(acoth_kernel(1.0), Err(WorksheetErrorCode::Num));
        assert_eq!(acoth_kernel(-1.0), Err(WorksheetErrorCode::Num));
    }
}
