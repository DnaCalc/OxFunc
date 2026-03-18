use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::expand_aggregate_arg;
use crate::functions::rank_common::{
    collect_rank_values, prepare_rank_number, prepare_rank_order, rank_eq,
};
use crate::resolver::ReferenceResolver;
use crate::value::{CallArgValue, EvalValue, WorksheetErrorCode};

pub const RANK_EQ_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.RANK.EQ",
    arity: Arity { min: 2, max: 3 },
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: ArgPreparationProfile::ValuesOnlyPreAdapter,
    coercion_lift_profile: CoercionLiftProfile::Custom,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
};

#[derive(Debug, Clone, PartialEq)]
pub enum RankEqEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
}

pub fn eval_rank_eq_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, RankEqEvalError> {
    let argc = args.len();
    if !RANK_EQ_META.arity.accepts(argc) {
        return Err(RankEqEvalError::ArityMismatch {
            expected_min: RANK_EQ_META.arity.min,
            expected_max: RANK_EQ_META.arity.max,
            actual: argc,
        });
    }

    let Some(number) =
        prepare_rank_number(&args[0], resolver).map_err(RankEqEvalError::Coercion)?
    else {
        return Ok(EvalValue::Error(WorksheetErrorCode::NA));
    };
    let expanded = expand_aggregate_arg(&args[1], resolver).map_err(RankEqEvalError::Coercion)?;
    let values = collect_rank_values(&expanded).map_err(RankEqEvalError::Coercion)?;
    let order = prepare_rank_order(args.get(2), resolver).map_err(RankEqEvalError::Coercion)?;
    match rank_eq(number, &values, order) {
        Ok(value) => Ok(EvalValue::Number(value)),
        Err(code) => Ok(EvalValue::Error(code)),
    }
}

pub fn map_rank_eq_error_to_ws(e: &RankEqEvalError) -> WorksheetErrorCode {
    match e {
        RankEqEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        RankEqEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        RankEqEvalError::Coercion(_) => WorksheetErrorCode::Value,
    }
}
