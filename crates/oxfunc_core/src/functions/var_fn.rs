use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::var_s_fn::{VarSEvalError, eval_var_s_surface};
use crate::resolver::ReferenceResolver;
use crate::value::{CallArgValue, EvalValue, WorksheetErrorCode};

pub const VAR_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.VAR",
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
pub enum VarEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
}

pub fn eval_var_surface(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, VarEvalError> {
    eval_var_s_surface(args, resolver).map_err(|err| match err {
        VarSEvalError::ArityMismatch {
            expected_min,
            expected_max,
            actual,
        } => VarEvalError::ArityMismatch {
            expected_min,
            expected_max,
            actual,
        },
        VarSEvalError::Coercion(err) => VarEvalError::Coercion(err),
    })
}

pub fn map_var_error_to_ws(e: &VarEvalError) -> WorksheetErrorCode {
    match e {
        VarEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        VarEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        VarEvalError::Coercion(_) => WorksheetErrorCode::Value,
    }
}
