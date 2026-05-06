use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::binary_numeric::{
    BinaryNumericSurfaceError, eval_binary_numeric_surface, map_binary_numeric_error_to_ws,
};
use crate::resolver::ReferenceResolver;
use crate::value::{EvalValue, WorksheetErrorCode};

pub const MROUND_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.MROUND",
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

pub fn mround_kernel(number: f64, multiple: f64) -> Result<f64, WorksheetErrorCode> {
    if multiple == 0.0 {
        return Ok(0.0);
    }
    if number.signum() != multiple.signum() {
        return Err(WorksheetErrorCode::Num);
    }
    Ok((number / multiple).round() * multiple)
}

pub fn eval_mround_surface(
    args: &[crate::value::CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, BinaryNumericSurfaceError> {
    eval_binary_numeric_surface(args, resolver, mround_kernel)
}

pub fn map_mround_error_to_ws(e: &BinaryNumericSurfaceError) -> WorksheetErrorCode {
    map_binary_numeric_error_to_ws(e)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mround_meta_function_id_is_stable() {
        assert_eq!(MROUND_META.function_id, "FUNC.MROUND");
    }

    #[test]
    fn mround_kernel_matches_excel_sign_and_midpoint_lanes() {
        assert_eq!(mround_kernel(10.0, 3.0), Ok(9.0));
        assert_eq!(mround_kernel(-10.0, -3.0), Ok(-9.0));
        assert_eq!(mround_kernel(10.0, -3.0), Err(WorksheetErrorCode::Num));
        assert_eq!(mround_kernel(-10.0, 3.0), Err(WorksheetErrorCode::Num));
        assert_eq!(mround_kernel(1.25, 0.5), Ok(1.5));
        assert_eq!(mround_kernel(5.0, 0.0), Ok(0.0));
    }
}
