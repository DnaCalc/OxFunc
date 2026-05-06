use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{AggregatePreparedValue, expand_aggregate_arg};
use crate::functions::aggregate_common::extrema_a_argument_value;
use crate::resolver::ReferenceResolver;
use crate::value::{CallArgValue, EvalValue, WorksheetErrorCode};

pub const MAXA_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.MAXA",
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
pub enum MaxAEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
}

fn eval_maxa_aggregate(args: &[AggregatePreparedValue]) -> Result<EvalValue, MaxAEvalError> {
    let mut acc: Option<f64> = None;
    for arg in args {
        if let Some(value) = extrema_a_argument_value(arg).map_err(MaxAEvalError::Coercion)? {
            acc = Some(match acc {
                Some(current) => current.max(value),
                None => value,
            });
        }
    }
    Ok(EvalValue::Number(acc.unwrap_or(0.0)))
}

pub fn eval_maxa_surface(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, MaxAEvalError> {
    let argc = args.len();
    if !MAXA_META.arity.accepts(argc) {
        return Err(MaxAEvalError::ArityMismatch {
            expected_min: MAXA_META.arity.min,
            expected_max: MAXA_META.arity.max,
            actual: argc,
        });
    }

    let mut prepared = Vec::new();
    for arg in args {
        prepared.extend(expand_aggregate_arg(arg, resolver).map_err(MaxAEvalError::Coercion)?);
    }
    eval_maxa_aggregate(&prepared)
}

pub fn map_maxa_error_to_ws(e: &MaxAEvalError) -> WorksheetErrorCode {
    match e {
        MaxAEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        MaxAEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        MaxAEvalError::Coercion(_) => WorksheetErrorCode::Value,
    }
}
