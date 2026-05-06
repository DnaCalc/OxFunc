use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::var_p_fn::{VarPEvalError, eval_var_p_surface};
use crate::resolver::ReferenceResolver;
use crate::value::{CallArgValue, EvalValue, WorksheetErrorCode};

pub const VARP_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.VARP",
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
pub enum VarpEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
}

pub fn eval_varp_surface(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, VarpEvalError> {
    eval_var_p_surface(args, resolver).map_err(|err| match err {
        VarPEvalError::ArityMismatch {
            expected_min,
            expected_max,
            actual,
        } => VarpEvalError::ArityMismatch {
            expected_min,
            expected_max,
            actual,
        },
        VarPEvalError::Coercion(err) => VarpEvalError::Coercion(err),
    })
}

pub fn map_varp_error_to_ws(e: &VarpEvalError) -> WorksheetErrorCode {
    match e {
        VarpEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        VarpEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        VarpEvalError::Coercion(_) => WorksheetErrorCode::Value,
    }
}
