use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::binary_numeric::{
    BinaryNumericSurfaceError, eval_binary_numeric_surface, map_binary_numeric_error_to_ws,
};
use crate::functions::combinatorics_common::combinations_of_int;
use crate::functions::factorial_common::trunc_nonnegative;
use crate::resolver::ReferenceResolver;
use crate::value::{EvalValue, WorksheetErrorCode};

pub const COMBIN_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.COMBIN",
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

pub fn combin_kernel(n: f64, k: f64) -> Result<f64, WorksheetErrorCode> {
    combinations_of_int(trunc_nonnegative(n)?, trunc_nonnegative(k)?)
}

pub fn eval_combin_surface(
    args: &[crate::value::CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, BinaryNumericSurfaceError> {
    eval_binary_numeric_surface(args, resolver, combin_kernel)
}

pub fn map_combin_error_to_ws(e: &BinaryNumericSurfaceError) -> WorksheetErrorCode {
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
    fn combin_meta_function_id_is_stable() {
        assert_eq!(COMBIN_META.function_id, "FUNC.COMBIN");
    }

    #[test]
    fn combin_kernel_matches_excel_truncation_and_num_lanes() {
        assert_eq!(combin_kernel(5.0, 2.0), Ok(10.0));
        assert_eq!(combin_kernel(5.9, 2.2), Ok(10.0));
        assert_eq!(combin_kernel(5.0, 6.0), Err(WorksheetErrorCode::Num));
        assert_eq!(combin_kernel(-1.0, 1.0), Err(WorksheetErrorCode::Num));
    }

    #[test]
    fn combin_exact_publication_controls_remain_exact() {
        assert_bits(combin_kernel(10.0, 3.0).expect("combin(10,3)"), 120.0_f64);
        assert_bits(combin_kernel(9.0, 2.0).expect("combin(9,2)"), 36.0_f64);
    }
}
