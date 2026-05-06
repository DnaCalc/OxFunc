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

pub const STDEVPA_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.STDEVPA",
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
pub enum StdevPAEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
}

pub fn eval_stdevpa_surface(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, StdevPAEvalError> {
    let argc = args.len();
    if !STDEVPA_META.arity.accepts(argc) {
        return Err(StdevPAEvalError::ArityMismatch {
            expected_min: STDEVPA_META.arity.min,
            expected_max: STDEVPA_META.arity.max,
            actual: argc,
        });
    }

    let mut prepared = Vec::new();
    for arg in args {
        prepared.extend(expand_aggregate_arg(arg, resolver).map_err(StdevPAEvalError::Coercion)?);
    }
    let values = collect_variance_values(&prepared, VarianceInclusionPolicy::AverageALike)
        .map_err(StdevPAEvalError::Coercion)?;
    match stdev_from_values(&values, VarianceDivisor::Population) {
        Ok(value) => Ok(EvalValue::Number(value)),
        Err(code) => Ok(EvalValue::Error(code)),
    }
}

pub fn map_stdevpa_error_to_ws(e: &StdevPAEvalError) -> WorksheetErrorCode {
    match e {
        StdevPAEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        StdevPAEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        StdevPAEvalError::Coercion(_) => WorksheetErrorCode::Value,
    }
}
