use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::stdev_s_fn::{StdevSEvalError, eval_stdev_s_surface};
use crate::resolver::ReferenceResolver;
use crate::value::{CallArgValue, EvalValue, WorksheetErrorCode};

pub const STDEV_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.STDEV",
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
pub enum StdevEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
}

pub fn eval_stdev_surface(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, StdevEvalError> {
    eval_stdev_s_surface(args, resolver).map_err(|err| match err {
        StdevSEvalError::ArityMismatch {
            expected_min,
            expected_max,
            actual,
        } => StdevEvalError::ArityMismatch {
            expected_min,
            expected_max,
            actual,
        },
        StdevSEvalError::Coercion(err) => StdevEvalError::Coercion(err),
    })
}

pub fn map_stdev_error_to_ws(e: &StdevEvalError) -> WorksheetErrorCode {
    match e {
        StdevEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        StdevEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        StdevEvalError::Coercion(_) => WorksheetErrorCode::Value,
    }
}
