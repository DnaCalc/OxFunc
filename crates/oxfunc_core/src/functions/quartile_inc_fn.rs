use crate::coercion::CoercionError;
use crate::function::{
    Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile, FunctionMeta,
    HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{expand_aggregate_arg, prepare_arg_values_only};
use crate::functions::percentile_common::{
    collect_percentile_values, percentile_inc_kernel, quartile_k,
};
use crate::resolver::ReferenceSystemProvider;
use crate::value::CalcValue;
use crate::value::WorksheetErrorCode;

pub const QUARTILE_INC_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.QUARTILE.INC",
    arity: Arity::exact(2),
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: FunctionMeta::DEFAULT_ARG_PREPARATION_PROFILE,
    coercion_lift_profile: CoercionLiftProfile::Custom,
    lift_broadcast_profile: FunctionMeta::DEFAULT_LIFT_BROADCAST_PROFILE,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
    real_result_policy: FunctionMeta::DEFAULT_REAL_RESULT_POLICY,
    error_collapse_profile: FunctionMeta::DEFAULT_ERROR_COLLAPSE_PROFILE,
    precision_rounding_profile: FunctionMeta::DEFAULT_PRECISION_ROUNDING_PROFILE,
};

#[derive(Debug, Clone, PartialEq)]
pub enum QuartileIncEvalError {
    ArityMismatch { expected: usize, actual: usize },
    Coercion(CoercionError),
}

pub fn eval_quartile_inc_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, QuartileIncEvalError> {
    if !QUARTILE_INC_META.arity.accepts(args.len()) {
        return Err(QuartileIncEvalError::ArityMismatch {
            expected: QUARTILE_INC_META.arity.min,
            actual: args.len(),
        });
    }
    let expanded =
        expand_aggregate_arg(&args[0], resolver).map_err(QuartileIncEvalError::Coercion)?;
    let mut values =
        collect_percentile_values(&expanded).map_err(QuartileIncEvalError::Coercion)?;
    let q = quartile_k(
        &prepare_arg_values_only(&args[1], resolver).map_err(QuartileIncEvalError::Coercion)?,
    )
    .map_err(QuartileIncEvalError::Coercion)?;
    if !(0..=4).contains(&q) {
        return Ok(CalcValue::error(WorksheetErrorCode::Num));
    }
    match percentile_inc_kernel(&mut values, q as f64 / 4.0) {
        Ok(v) => Ok(CalcValue::number(v)),
        Err(code) => Ok(CalcValue::error(code)),
    }
}

pub fn map_quartile_inc_error_to_ws(e: &QuartileIncEvalError) -> WorksheetErrorCode {
    match e {
        QuartileIncEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        QuartileIncEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        QuartileIncEvalError::Coercion(_) => WorksheetErrorCode::Value,
    }
}
