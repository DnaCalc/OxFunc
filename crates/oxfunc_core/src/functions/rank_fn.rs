use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::rank_eq_fn::{RankEqEvalError, eval_rank_eq_surface};
use crate::resolver::ReferenceResolver;
use crate::value::{CallArgValue, EvalValue, WorksheetErrorCode};

pub const RANK_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.RANK",
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
pub enum RankEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
}

pub fn eval_rank_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, RankEvalError> {
    eval_rank_eq_surface(args, resolver).map_err(|err| match err {
        RankEqEvalError::ArityMismatch {
            expected_min,
            expected_max,
            actual,
        } => RankEvalError::ArityMismatch {
            expected_min,
            expected_max,
            actual,
        },
        RankEqEvalError::Coercion(err) => RankEvalError::Coercion(err),
    })
}

pub fn map_rank_error_to_ws(e: &RankEvalError) -> WorksheetErrorCode {
    match e {
        RankEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        RankEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        RankEvalError::Coercion(_) => WorksheetErrorCode::Value,
    }
}
