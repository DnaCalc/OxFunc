use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::expand_aggregate_arg;
use crate::functions::variance_common::{
    VarianceDivisor, VarianceInclusionPolicy, collect_variance_values, stdev_from_values,
};
use crate::resolver::ReferenceResolver;
use crate::value::{CallArgValue, EvalValue, WorksheetErrorCode};

pub const STDEV_S_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.STDEV.S",
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
pub enum StdevSEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
}

pub fn eval_stdev_s_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, StdevSEvalError> {
    let argc = args.len();
    if !STDEV_S_META.arity.accepts(argc) {
        return Err(StdevSEvalError::ArityMismatch {
            expected_min: STDEV_S_META.arity.min,
            expected_max: STDEV_S_META.arity.max,
            actual: argc,
        });
    }

    let mut prepared = Vec::new();
    for arg in args {
        prepared.extend(expand_aggregate_arg(arg, resolver).map_err(StdevSEvalError::Coercion)?);
    }
    let values = collect_variance_values(&prepared, VarianceInclusionPolicy::AverageLike)
        .map_err(StdevSEvalError::Coercion)?;
    match stdev_from_values(&values, VarianceDivisor::Sample) {
        Ok(value) => Ok(EvalValue::Number(value)),
        Err(code) => Ok(EvalValue::Error(code)),
    }
}

pub fn map_stdev_s_error_to_ws(e: &StdevSEvalError) -> WorksheetErrorCode {
    match e {
        StdevSEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        StdevSEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        StdevSEvalError::Coercion(_) => WorksheetErrorCode::Value,
    }
}
