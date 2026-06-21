use crate::function::{
    Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile, FunctionMeta,
    HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::excel_numeric::ExcelRealPolicy;
use crate::functions::unary_numeric::{
    UnaryNumericExecSpec, UnaryNumericSurfaceError, eval_unary_numeric_via_executor,
    map_unary_numeric_error_to_ws,
};
use crate::resolver::ReferenceSystemProvider;
use crate::value::CalcValue;
use crate::value::WorksheetErrorCode;

pub const CSC_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.CSC",
    arity: Arity::exact(1),
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: FunctionMeta::DEFAULT_ARG_PREPARATION_PROFILE,
    coercion_lift_profile: CoercionLiftProfile::UnaryNumericScalarOrArrayElementwise,
    lift_broadcast_profile: FunctionMeta::DEFAULT_LIFT_BROADCAST_PROFILE,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
    // BUG-FUNC-027 CLASS-B2: circular trig is `#NUM!` once `|x| >= 2^27`.
    real_result_policy: ExcelRealPolicy::CIRCULAR_TRIG,
    error_collapse_profile: FunctionMeta::DEFAULT_ERROR_COLLAPSE_PROFILE,
    precision_rounding_profile: FunctionMeta::DEFAULT_PRECISION_ROUNDING_PROFILE,
};

pub fn csc_kernel(n: f64) -> Result<f64, WorksheetErrorCode> {
    CSC_META.real_result_policy.check_arg(n)?;
    let sin = n.sin();
    if sin == 0.0 {
        return Err(WorksheetErrorCode::Div0);
    }
    Ok(1.0 / sin)
}

pub fn eval_csc_surface(
    args: &[crate::value::CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, UnaryNumericSurfaceError> {
    eval_unary_numeric_via_executor(
        args,
        resolver,
        UnaryNumericExecSpec::fallible(csc_kernel, CSC_META.real_result_policy),
    )
}

pub fn map_csc_error_to_ws(e: &UnaryNumericSurfaceError) -> WorksheetErrorCode {
    map_unary_numeric_error_to_ws(e)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn csc_meta_function_id_is_stable() {
        assert_eq!(CSC_META.function_id, "FUNC.CSC");
    }

    #[test]
    fn csc_kernel_zero_is_div0() {
        assert_eq!(csc_kernel(0.0), Err(WorksheetErrorCode::Div0));
    }
}
