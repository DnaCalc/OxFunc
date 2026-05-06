use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::stdev_p_fn::{StdevPEvalError, eval_stdev_p_surface};
use crate::resolver::ReferenceResolver;
use crate::value::{CallArgValue, EvalValue, WorksheetErrorCode};

pub const STDEVP_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.STDEVP",
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
pub enum StdevPExtEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
}

pub fn eval_stdevp_surface(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, StdevPExtEvalError> {
    eval_stdev_p_surface(args, resolver).map_err(|err| match err {
        StdevPEvalError::ArityMismatch {
            expected_min,
            expected_max,
            actual,
        } => StdevPExtEvalError::ArityMismatch {
            expected_min,
            expected_max,
            actual,
        },
        StdevPEvalError::Coercion(err) => StdevPExtEvalError::Coercion(err),
    })
}

pub fn map_stdevp_error_to_ws(e: &StdevPExtEvalError) -> WorksheetErrorCode {
    match e {
        StdevPExtEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        StdevPExtEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        StdevPExtEvalError::Coercion(_) => WorksheetErrorCode::Value,
    }
}
