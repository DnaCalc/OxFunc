use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{AggregatePreparedValue, expand_aggregate_arg};
use crate::functions::aggregate_common::extrema_a_argument_value;
use crate::resolver::ReferenceResolver;
use crate::value::{CallArgValue, EvalValue, WorksheetErrorCode};

pub const MINA_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.MINA",
    arity: Arity { min: 1, max: 255 },
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: ArgPreparationProfile::ValuesOnlyPreAdapter,
    coercion_lift_profile: CoercionLiftProfile::AggregateDirectAndRangeDualPolicy,
    kernel_signature_class: KernelSignatureClass::NumsToNum,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
};

#[derive(Debug, Clone, PartialEq)]
pub enum MinAEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
}

fn eval_mina_aggregate(args: &[AggregatePreparedValue]) -> Result<EvalValue, MinAEvalError> {
    let mut acc: Option<f64> = None;
    for arg in args {
        if let Some(value) = extrema_a_argument_value(arg).map_err(MinAEvalError::Coercion)? {
            acc = Some(match acc {
                Some(current) => current.min(value),
                None => value,
            });
        }
    }
    Ok(EvalValue::Number(acc.unwrap_or(0.0)))
}

pub fn eval_mina_surface(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, MinAEvalError> {
    let argc = args.len();
    if !MINA_META.arity.accepts(argc) {
        return Err(MinAEvalError::ArityMismatch {
            expected_min: MINA_META.arity.min,
            expected_max: MINA_META.arity.max,
            actual: argc,
        });
    }

    let mut prepared = Vec::new();
    for arg in args {
        prepared.extend(expand_aggregate_arg(arg, resolver).map_err(MinAEvalError::Coercion)?);
    }
    eval_mina_aggregate(&prepared)
}

pub fn map_mina_error_to_ws(e: &MinAEvalError) -> WorksheetErrorCode {
    match e {
        MinAEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        MinAEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        MinAEvalError::Coercion(_) => WorksheetErrorCode::Value,
    }
}
