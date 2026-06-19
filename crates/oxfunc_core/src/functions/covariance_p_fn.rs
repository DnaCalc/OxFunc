use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::expand_aggregate_arg;
use crate::functions::paired_stats_common::{
    CovarianceDivisor, collect_paired_values, covariance_from_pairs,
};
use crate::resolver::ReferenceSystemProvider;
use crate::value::CalcValue;
use crate::value::WorksheetErrorCode;

pub const COVARIANCE_P_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.COVARIANCE.P",
    arity: Arity::exact(2),
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: ArgPreparationProfile::ValuesOnlyPreAdapter,
    coercion_lift_profile: CoercionLiftProfile::Custom,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
    real_result_policy: FunctionMeta::DEFAULT_REAL_RESULT_POLICY,
};

#[derive(Debug, Clone, PartialEq)]
pub enum CovariancePEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
}

pub fn eval_covariance_p_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, CovariancePEvalError> {
    let argc = args.len();
    if !COVARIANCE_P_META.arity.accepts(argc) {
        return Err(CovariancePEvalError::ArityMismatch {
            expected_min: COVARIANCE_P_META.arity.min,
            expected_max: COVARIANCE_P_META.arity.max,
            actual: argc,
        });
    }
    let xs = expand_aggregate_arg(&args[0], resolver).map_err(CovariancePEvalError::Coercion)?;
    let ys = expand_aggregate_arg(&args[1], resolver).map_err(CovariancePEvalError::Coercion)?;
    let pairs = collect_paired_values(&xs, &ys).map_err(CovariancePEvalError::Coercion)?;
    match covariance_from_pairs(&pairs, CovarianceDivisor::Population) {
        Ok(value) => Ok(CalcValue::number(value)),
        Err(code) => Ok(CalcValue::error(code)),
    }
}

pub fn map_covariance_p_error_to_ws(e: &CovariancePEvalError) -> WorksheetErrorCode {
    match e {
        CovariancePEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        CovariancePEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        CovariancePEvalError::Coercion(_) => WorksheetErrorCode::Value,
    }
}
