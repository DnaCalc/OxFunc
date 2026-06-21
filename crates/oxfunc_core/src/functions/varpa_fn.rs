use crate::coercion::CoercionError;
use crate::function::{
    Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile, FunctionMeta,
    HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::expand_aggregate_arg;
use crate::functions::variance_common::{
    VarianceDivisor, VarianceInclusionPolicy, collect_variance_values, variance_from_values,
};
use crate::resolver::ReferenceSystemProvider;
use crate::value::CalcValue;
use crate::value::WorksheetErrorCode;

pub const VARPA_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.VARPA",
    arity: Arity { min: 1, max: 255 },
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: FunctionMeta::DEFAULT_ARG_PREPARATION_PROFILE,
    coercion_lift_profile: CoercionLiftProfile::AggregateDirectAndRangeDualPolicy,
    kernel_signature_class: KernelSignatureClass::NumsToNum,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
    real_result_policy: FunctionMeta::DEFAULT_REAL_RESULT_POLICY,
};

#[derive(Debug, Clone, PartialEq)]
pub enum VarPAEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
}

pub fn eval_varpa_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, VarPAEvalError> {
    let argc = args.len();
    if !VARPA_META.arity.accepts(argc) {
        return Err(VarPAEvalError::ArityMismatch {
            expected_min: VARPA_META.arity.min,
            expected_max: VARPA_META.arity.max,
            actual: argc,
        });
    }

    let mut prepared = Vec::new();
    for arg in args {
        prepared.extend(expand_aggregate_arg(arg, resolver).map_err(VarPAEvalError::Coercion)?);
    }
    let values = collect_variance_values(&prepared, VarianceInclusionPolicy::AverageALike)
        .map_err(VarPAEvalError::Coercion)?;
    match variance_from_values(&values, VarianceDivisor::Population) {
        Ok(value) => Ok(CalcValue::number(value)),
        Err(code) => Ok(CalcValue::error(code)),
    }
}

pub fn map_varpa_error_to_ws(e: &VarPAEvalError) -> WorksheetErrorCode {
    match e {
        VarPAEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        VarPAEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        VarPAEvalError::Coercion(_) => WorksheetErrorCode::Value,
    }
}
