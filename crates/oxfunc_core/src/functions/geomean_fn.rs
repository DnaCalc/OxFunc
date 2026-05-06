use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{AggregatePreparedValue, expand_aggregate_arg};
use crate::functions::aggregate_common::average_argument_value;
use crate::resolver::ReferenceResolver;
use crate::value::{CallArgValue, EvalValue, WorksheetErrorCode};

pub const GEOMEAN_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.GEOMEAN",
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
pub enum GeoMeanEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
}

fn eval_geomean_aggregate(args: &[AggregatePreparedValue]) -> Result<EvalValue, GeoMeanEvalError> {
    let mut acc_ln = 0.0;
    let mut count = 0usize;
    for arg in args {
        if let Some(value) = average_argument_value(arg).map_err(GeoMeanEvalError::Coercion)? {
            if value <= 0.0 {
                return Ok(EvalValue::Error(WorksheetErrorCode::Num));
            }
            acc_ln += value.ln();
            count += 1;
        }
    }
    if count == 0 {
        return Ok(EvalValue::Error(WorksheetErrorCode::Num));
    }
    Ok(EvalValue::Number((acc_ln / count as f64).exp()))
}

pub fn eval_geomean_surface(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, GeoMeanEvalError> {
    let argc = args.len();
    if !GEOMEAN_META.arity.accepts(argc) {
        return Err(GeoMeanEvalError::ArityMismatch {
            expected_min: GEOMEAN_META.arity.min,
            expected_max: GEOMEAN_META.arity.max,
            actual: argc,
        });
    }
    let mut prepared = Vec::new();
    for arg in args {
        prepared.extend(expand_aggregate_arg(arg, resolver).map_err(GeoMeanEvalError::Coercion)?);
    }
    eval_geomean_aggregate(&prepared)
}

pub fn map_geomean_error_to_ws(e: &GeoMeanEvalError) -> WorksheetErrorCode {
    match e {
        GeoMeanEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        GeoMeanEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        GeoMeanEvalError::Coercion(_) => WorksheetErrorCode::Value,
    }
}
