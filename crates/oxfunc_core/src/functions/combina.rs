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

pub const COMBINA_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.COMBINA",
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

pub fn combina_kernel(n: f64, k: f64) -> Result<f64, WorksheetErrorCode> {
    let n = trunc_nonnegative(n)?;
    let k = trunc_nonnegative(k)?;
    if k == 0 {
        return Ok(1.0);
    }
    if n == 0 && k > 0 {
        return Err(WorksheetErrorCode::Num);
    }
    combinations_of_int(n + k - 1, k)
}

pub fn eval_combina_surface(
    args: &[crate::value::CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, BinaryNumericSurfaceError> {
    eval_binary_numeric_surface(args, resolver, combina_kernel)
}

pub fn map_combina_error_to_ws(e: &BinaryNumericSurfaceError) -> WorksheetErrorCode {
    map_binary_numeric_error_to_ws(e)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn combina_meta_function_id_is_stable() {
        assert_eq!(COMBINA_META.function_id, "FUNC.COMBINA");
    }

    #[test]
    fn combina_kernel_matches_excel_boundary_lanes() {
        assert_eq!(combina_kernel(4.0, 3.0), Ok(20.0));
        assert_eq!(combina_kernel(5.9, 2.2), Ok(15.0));
        assert_eq!(combina_kernel(0.0, 0.0), Ok(1.0));
        assert_eq!(combina_kernel(0.0, 1.0), Err(WorksheetErrorCode::Num));
    }
}
