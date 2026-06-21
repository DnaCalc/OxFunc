use crate::coercion::CoercionError;
use crate::function::{
    Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile, FunctionMeta,
    HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::expand_aggregate_arg;
use crate::functions::rank_common::{
    collect_rank_values, prepare_rank_number, prepare_rank_order, rank_avg,
};
use crate::resolver::ReferenceSystemProvider;
use crate::value::CalcValue;
use crate::value::WorksheetErrorCode;

pub const RANK_AVG_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.RANK.AVG",
    arity: Arity { min: 2, max: 3 },
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
pub enum RankAvgEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
}

pub fn eval_rank_avg_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, RankAvgEvalError> {
    let argc = args.len();
    if !RANK_AVG_META.arity.accepts(argc) {
        return Err(RankAvgEvalError::ArityMismatch {
            expected_min: RANK_AVG_META.arity.min,
            expected_max: RANK_AVG_META.arity.max,
            actual: argc,
        });
    }

    let Some(number) =
        prepare_rank_number(&args[0], resolver).map_err(RankAvgEvalError::Coercion)?
    else {
        return Ok(CalcValue::error(WorksheetErrorCode::NA));
    };
    let expanded = expand_aggregate_arg(&args[1], resolver).map_err(RankAvgEvalError::Coercion)?;
    let values = collect_rank_values(&expanded).map_err(RankAvgEvalError::Coercion)?;
    let order = prepare_rank_order(args.get(2), resolver).map_err(RankAvgEvalError::Coercion)?;
    match rank_avg(number, &values, order) {
        Ok(value) => Ok(CalcValue::number(value)),
        Err(code) => Ok(CalcValue::error(code)),
    }
}

pub fn map_rank_avg_error_to_ws(e: &RankAvgEvalError) -> WorksheetErrorCode {
    match e {
        RankAvgEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        RankAvgEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        RankAvgEvalError::Coercion(_) => WorksheetErrorCode::Value,
    }
}
