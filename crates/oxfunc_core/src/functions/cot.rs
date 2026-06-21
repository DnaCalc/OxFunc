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

pub const COT_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.COT",
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
};

pub fn cot_kernel(n: f64) -> Result<f64, WorksheetErrorCode> {
    COT_META.real_result_policy.check_arg(n)?;
    let tan = n.tan();
    if tan == 0.0 {
        return Err(WorksheetErrorCode::Div0);
    }
    Ok(1.0 / tan)
}

pub fn eval_cot_surface(
    args: &[crate::value::CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, UnaryNumericSurfaceError> {
    eval_unary_numeric_via_executor(
        args,
        resolver,
        UnaryNumericExecSpec::fallible(cot_kernel, COT_META.real_result_policy),
    )
}

pub fn map_cot_error_to_ws(e: &UnaryNumericSurfaceError) -> WorksheetErrorCode {
    map_unary_numeric_error_to_ws(e)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cot_meta_function_id_is_stable() {
        assert_eq!(COT_META.function_id, "FUNC.COT");
    }

    #[test]
    fn cot_kernel_zero_is_div0() {
        assert_eq!(cot_kernel(0.0), Err(WorksheetErrorCode::Div0));
    }

    // BUG-FUNC-027 CLASS-B2: |x| >= 2^27 -> #NUM! (live Excel COT(2^27)=#NUM!).
    #[test]
    fn cot_kernel_large_argument_is_num() {
        assert_eq!(cot_kernel(134_217_728.0), Err(WorksheetErrorCode::Num));
        assert_eq!(cot_kernel(-134_217_728.0), Err(WorksheetErrorCode::Num));
        assert!(cot_kernel(1.0).is_ok());
    }
}
