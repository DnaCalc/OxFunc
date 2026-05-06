use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::unary_numeric::{
    UnaryNumericSurfaceError, eval_unary_numeric_surface, map_unary_numeric_error_to_ws,
};
use crate::resolver::ReferenceResolver;
use crate::value::{EvalValue, WorksheetErrorCode};

pub const ASINH_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.ASINH",
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

pub fn asinh_kernel(n: f64) -> Result<f64, WorksheetErrorCode> {
    // Current-baseline Excel publication aligns with sign(x) * ln(|x| + hypot(x, 1))
    // on the disputed lanes where platform libm `asinh` differs by 1+ ULP.
    Ok(n.signum() * (n.abs() + n.hypot(1.0)).ln())
}

pub fn eval_asinh_surface(
    args: &[crate::value::CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, UnaryNumericSurfaceError> {
    eval_unary_numeric_surface(args, resolver, asinh_kernel)
}

pub fn map_asinh_error_to_ws(e: &UnaryNumericSurfaceError) -> WorksheetErrorCode {
    map_unary_numeric_error_to_ws(e)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn asinh_meta_function_id_is_stable() {
        assert_eq!(ASINH_META.function_id, "FUNC.ASINH");
    }

    #[test]
    fn asinh_kernel_matches_std() {
        assert_eq!(asinh_kernel(0.5), Ok(0.5f64.asinh()));
    }

    #[test]
    fn asinh_kernel_matches_pinned_excel_publication_on_disputed_rows() {
        assert_eq!(asinh_kernel(1.0), Ok(0.8813735870195429));
        assert_eq!(asinh_kernel(-1.0), Ok(-0.8813735870195429));
        assert_eq!(asinh_kernel(1.0e-10), Ok(1.000000082690371e-10));
        assert_eq!(asinh_kernel(-1.0e-10), Ok(-1.000000082690371e-10));
    }
}
