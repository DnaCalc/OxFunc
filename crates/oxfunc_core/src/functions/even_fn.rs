use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::unary_numeric::{
    UnaryNumericSurfaceError, eval_unary_numeric_surface, map_unary_numeric_error_to_ws,
};
use crate::resolver::ReferenceResolver;
use crate::value::{EvalValue, WorksheetErrorCode};

pub const EVEN_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.EVEN",
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

pub fn even_kernel(n: f64) -> Result<f64, WorksheetErrorCode> {
    if n == 0.0 {
        return Ok(0.0);
    }
    let magnitude = n.abs().ceil();
    let rounded = if (magnitude as i64) % 2 == 0 {
        magnitude
    } else {
        magnitude + 1.0
    };
    Ok(if n.is_sign_negative() {
        -rounded
    } else {
        rounded
    })
}

pub fn eval_even_surface(
    args: &[crate::value::CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, UnaryNumericSurfaceError> {
    eval_unary_numeric_surface(args, resolver, even_kernel)
}

pub fn map_even_error_to_ws(e: &UnaryNumericSurfaceError) -> WorksheetErrorCode {
    map_unary_numeric_error_to_ws(e)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn even_meta_function_id_is_stable() {
        assert_eq!(EVEN_META.function_id, "FUNC.EVEN");
    }

    #[test]
    fn even_kernel_rounds_away_from_zero_to_even() {
        assert_eq!(even_kernel(1.5), Ok(2.0));
        assert_eq!(even_kernel(-1.5), Ok(-2.0));
        assert_eq!(even_kernel(2.0), Ok(2.0));
    }
}
