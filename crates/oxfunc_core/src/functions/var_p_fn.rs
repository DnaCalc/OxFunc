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

pub const VAR_P_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.VAR.P",
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
    precision_rounding_profile: FunctionMeta::DEFAULT_PRECISION_ROUNDING_PROFILE,
};

#[derive(Debug, Clone, PartialEq)]
pub enum VarPEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
}

pub fn eval_var_p_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, VarPEvalError> {
    let argc = args.len();
    if !VAR_P_META.arity.accepts(argc) {
        return Err(VarPEvalError::ArityMismatch {
            expected_min: VAR_P_META.arity.min,
            expected_max: VAR_P_META.arity.max,
            actual: argc,
        });
    }

    let mut prepared = Vec::new();
    for arg in args {
        prepared.extend(expand_aggregate_arg(arg, resolver).map_err(VarPEvalError::Coercion)?);
    }
    let values = collect_variance_values(&prepared, VarianceInclusionPolicy::AverageLike)
        .map_err(VarPEvalError::Coercion)?;
    match variance_from_values(&values, VarianceDivisor::Population) {
        Ok(value) => Ok(CalcValue::number(value)),
        Err(code) => Ok(CalcValue::error(code)),
    }
}

pub fn map_var_p_error_to_ws(e: &VarPEvalError) -> WorksheetErrorCode {
    match e {
        VarPEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        VarPEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        VarPEvalError::Coercion(_) => WorksheetErrorCode::Value,
    }
}
