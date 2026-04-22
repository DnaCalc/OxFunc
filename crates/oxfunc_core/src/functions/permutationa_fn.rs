use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::binary_numeric::{
    eval_binary_numeric_surface, map_binary_numeric_error_to_ws, BinaryNumericSurfaceError,
};
use crate::functions::factorial_common::trunc_nonnegative;
use crate::resolver::ReferenceResolver;
use crate::value::{EvalValue, WorksheetErrorCode};

pub const PERMUTATIONA_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.PERMUTATIONA",
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

pub fn permutationa_kernel(n: f64, k: f64) -> Result<f64, WorksheetErrorCode> {
    let n = trunc_nonnegative(n)?;
    let k = trunc_nonnegative(k)?;
    Ok((n as f64).powi(k as i32))
}

pub fn eval_permutationa_surface(
    args: &[crate::value::CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, BinaryNumericSurfaceError> {
    eval_binary_numeric_surface(args, resolver, permutationa_kernel)
}

pub fn map_permutationa_error_to_ws(e: &BinaryNumericSurfaceError) -> WorksheetErrorCode {
    map_binary_numeric_error_to_ws(e)
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
    fn permutationa_meta_function_id_is_stable() {
        assert_eq!(PERMUTATIONA_META.function_id, "FUNC.PERMUTATIONA");
    }

    #[test]
    fn permutationa_kernel_matches_excel_lanes() {
        assert_eq!(permutationa_kernel(3.0, 2.0), Ok(9.0));
        assert_eq!(permutationa_kernel(3.9, 2.1), Ok(9.0));
        assert_eq!(permutationa_kernel(0.0, 0.0), Ok(1.0));
        assert_eq!(permutationa_kernel(0.0, 1.0), Ok(0.0));
        assert_eq!(permutationa_kernel(-1.0, 1.0), Err(WorksheetErrorCode::Num));
    }

    #[test]
    fn permutationa_exact_publication_controls_remain_exact() {
        assert_bits(
            permutationa_kernel(3.0, 2.0).expect("permutationa(3,2)"),
            9.0_f64,
        );
        assert_bits(
            permutationa_kernel(4.0, 3.0).expect("permutationa(4,3)"),
            64.0_f64,
        );
    }
}
