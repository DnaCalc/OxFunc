use crate::coercion::CoercionError;
use crate::function::{
    Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile, FunctionMeta,
    HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::expand_aggregate_arg;
use crate::functions::paired_stats_common::{
    CovarianceDivisor, collect_paired_values, covariance_from_pairs,
};
use crate::resolver::ReferenceSystemProvider;
use crate::value::CalcValue;
use crate::value::WorksheetErrorCode;

pub const COVARIANCE_S_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.COVARIANCE.S",
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
pub enum CovarianceSEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
}

pub fn eval_covariance_s_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, CovarianceSEvalError> {
    let argc = args.len();
    if !COVARIANCE_S_META.arity.accepts(argc) {
        return Err(CovarianceSEvalError::ArityMismatch {
            expected_min: COVARIANCE_S_META.arity.min,
            expected_max: COVARIANCE_S_META.arity.max,
            actual: argc,
        });
    }
    let xs = expand_aggregate_arg(&args[0], resolver).map_err(CovarianceSEvalError::Coercion)?;
    let ys = expand_aggregate_arg(&args[1], resolver).map_err(CovarianceSEvalError::Coercion)?;
    let pairs = collect_paired_values(&xs, &ys).map_err(CovarianceSEvalError::Coercion)?;
    match covariance_from_pairs(&pairs, CovarianceDivisor::Sample) {
        Ok(value) => Ok(CalcValue::number(value)),
        Err(code) => Ok(CalcValue::error(code)),
    }
}

pub fn map_covariance_s_error_to_ws(e: &CovarianceSEvalError) -> WorksheetErrorCode {
    match e {
        CovarianceSEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        CovarianceSEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        CovarianceSEvalError::Coercion(_) => WorksheetErrorCode::Value,
    }
}
