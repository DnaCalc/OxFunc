use crate::coercion::CoercionError;
use crate::function::{
    Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile, FunctionMeta,
    HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::var_s_fn::{VarSEvalError, eval_var_s_surface};
use crate::resolver::ReferenceSystemProvider;
use crate::value::CalcValue;
use crate::value::WorksheetErrorCode;

pub const VAR_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.VAR",
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
pub enum VarEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
}

pub fn eval_var_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, VarEvalError> {
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
