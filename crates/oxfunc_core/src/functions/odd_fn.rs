use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::unary_numeric::{
    UnaryNumericSurfaceError, eval_unary_numeric_surface, map_unary_numeric_error_to_ws,
};
use crate::resolver::ReferenceResolver;
use crate::value::{EvalValue, WorksheetErrorCode};

pub const ODD_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.ODD",
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

pub fn odd_kernel(n: f64) -> Result<f64, WorksheetErrorCode> {
    if n == 0.0 {
        return Ok(1.0);
    }
    let magnitude = n.abs().ceil();
    let rounded = if (magnitude as i64) % 2 == 1 {
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

pub fn eval_odd_surface(
    args: &[crate::value::CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, UnaryNumericSurfaceError> {
    eval_unary_numeric_surface(args, resolver, odd_kernel)
}

pub fn map_odd_error_to_ws(e: &UnaryNumericSurfaceError) -> WorksheetErrorCode {
    map_unary_numeric_error_to_ws(e)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn odd_meta_function_id_is_stable() {
        assert_eq!(ODD_META.function_id, "FUNC.ODD");
    }

    #[test]
    fn odd_kernel_rounds_away_from_zero_to_odd() {
        assert_eq!(odd_kernel(1.5), Ok(3.0));
        assert_eq!(odd_kernel(-1.5), Ok(-3.0));
        assert_eq!(odd_kernel(2.0), Ok(3.0));
    }
}
