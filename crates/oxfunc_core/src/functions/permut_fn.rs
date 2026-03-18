use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::binary_numeric::{
    BinaryNumericSurfaceError, eval_binary_numeric_surface, map_binary_numeric_error_to_ws,
};
use crate::functions::factorial_common::{factorial_of_int, trunc_nonnegative};
use crate::resolver::ReferenceResolver;
use crate::value::{EvalValue, WorksheetErrorCode};

pub const PERMUT_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.PERMUT",
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

pub fn permut_kernel(n: f64, k: f64) -> Result<f64, WorksheetErrorCode> {
    let n = trunc_nonnegative(n)?;
    let k = trunc_nonnegative(k)?;
    if k > n {
        return Err(WorksheetErrorCode::Num);
    }
    Ok(factorial_of_int(n) / factorial_of_int(n - k))
}

pub fn eval_permut_surface(
    args: &[crate::value::CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, BinaryNumericSurfaceError> {
    eval_binary_numeric_surface(args, resolver, permut_kernel)
}

pub fn map_permut_error_to_ws(e: &BinaryNumericSurfaceError) -> WorksheetErrorCode {
    map_binary_numeric_error_to_ws(e)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn permut_meta_function_id_is_stable() {
        assert_eq!(PERMUT_META.function_id, "FUNC.PERMUT");
    }

    #[test]
    fn permut_kernel_matches_excel_lanes() {
        assert_eq!(permut_kernel(10.0, 3.0), Ok(720.0));
        assert_eq!(permut_kernel(10.9, 3.2), Ok(720.0));
        assert_eq!(permut_kernel(0.0, 0.0), Ok(1.0));
        assert_eq!(permut_kernel(3.0, 4.0), Err(WorksheetErrorCode::Num));
        assert_eq!(permut_kernel(-1.0, 1.0), Err(WorksheetErrorCode::Num));
    }
}
