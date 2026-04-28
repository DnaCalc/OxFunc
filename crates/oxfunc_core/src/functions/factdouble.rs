use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::factorial_common::{double_factorial_of_int, trunc_nonnegative_or_minus_one};
use crate::functions::unary_numeric::{
    UnaryNumericSurfaceError, eval_unary_numeric_surface, map_unary_numeric_error_to_ws,
};
use crate::resolver::ReferenceResolver;
use crate::value::{EvalValue, WorksheetErrorCode};

pub const FACTDOUBLE_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.FACTDOUBLE",
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

pub fn factdouble_kernel(n: f64) -> Result<f64, WorksheetErrorCode> {
    let truncated = trunc_nonnegative_or_minus_one(n)?;
    Ok(double_factorial_of_int(truncated))
}

pub fn eval_factdouble_surface(
    args: &[crate::value::CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, UnaryNumericSurfaceError> {
    eval_unary_numeric_surface(args, resolver, factdouble_kernel)
}

pub fn map_factdouble_error_to_ws(e: &UnaryNumericSurfaceError) -> WorksheetErrorCode {
    map_unary_numeric_error_to_ws(e)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_bits(actual: f64, expected: f64) {
        assert_eq!(
            actual.to_bits(),
            expected.to_bits(),
            "{actual} vs {expected}"
        );
    }

    #[test]
    fn factdouble_meta_function_id_is_stable() {
        assert_eq!(FACTDOUBLE_META.function_id, "FUNC.FACTDOUBLE");
    }

    #[test]
    fn factdouble_kernel_matches_excel_negative_boundary() {
        assert_eq!(factdouble_kernel(6.9), Ok(48.0));
        assert_eq!(factdouble_kernel(-1.0), Ok(1.0));
        assert_eq!(factdouble_kernel(-1.1), Err(WorksheetErrorCode::Num));
    }

    #[test]
    fn factdouble_exact_publication_controls_remain_exact() {
        assert_bits(factdouble_kernel(9.0).expect("factdouble(9)"), 945.0_f64);
        assert_bits(factdouble_kernel(6.0).expect("factdouble(6)"), 48.0_f64);
    }
}
