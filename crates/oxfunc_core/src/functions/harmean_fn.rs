use crate::coercion::CoercionError;
use crate::function::{
    Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile, FunctionMeta,
    HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{AggregatePreparedItem, expand_aggregate_arg};
use crate::functions::aggregate_common::average_argument_value;
use crate::resolver::ReferenceSystemProvider;
use crate::value::CalcValue;
use crate::value::WorksheetErrorCode;

pub const HARMEAN_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.HARMEAN",
    arity: Arity { min: 1, max: 255 },
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: FunctionMeta::DEFAULT_ARG_PREPARATION_PROFILE,
    coercion_lift_profile: CoercionLiftProfile::AggregateDirectAndRangeDualPolicy,
    lift_broadcast_profile: FunctionMeta::DEFAULT_LIFT_BROADCAST_PROFILE,
    kernel_signature_class: KernelSignatureClass::NumsToNum,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
    real_result_policy: FunctionMeta::DEFAULT_REAL_RESULT_POLICY,
    error_collapse_profile: FunctionMeta::DEFAULT_ERROR_COLLAPSE_PROFILE,
};

#[derive(Debug, Clone, PartialEq)]
pub enum HarMeanEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
}

fn eval_harmean_aggregate(args: &[AggregatePreparedItem]) -> Result<CalcValue, HarMeanEvalError> {
    let mut reciprocal_sum = 0.0;
    let mut count = 0usize;
    for arg in args {
        if let Some(value) = average_argument_value(arg).map_err(HarMeanEvalError::Coercion)? {
            if value <= 0.0 {
                return Ok(CalcValue::error(WorksheetErrorCode::Num));
            }
            reciprocal_sum += 1.0 / value;
            count += 1;
        }
    }
    if count == 0 {
        return Ok(CalcValue::error(WorksheetErrorCode::NA));
    }
    Ok(CalcValue::number(count as f64 / reciprocal_sum))
}

pub fn eval_harmean_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, HarMeanEvalError> {
    let argc = args.len();
    if !HARMEAN_META.arity.accepts(argc) {
        return Err(HarMeanEvalError::ArityMismatch {
            expected_min: HARMEAN_META.arity.min,
            expected_max: HARMEAN_META.arity.max,
            actual: argc,
        });
    }
    let mut prepared = Vec::new();
    for arg in args {
        prepared.extend(expand_aggregate_arg(arg, resolver).map_err(HarMeanEvalError::Coercion)?);
    }
    eval_harmean_aggregate(&prepared)
}

pub fn map_harmean_error_to_ws(e: &HarMeanEvalError) -> WorksheetErrorCode {
    match e {
        HarMeanEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        HarMeanEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        HarMeanEvalError::Coercion(_) => WorksheetErrorCode::Value,
    }
}
