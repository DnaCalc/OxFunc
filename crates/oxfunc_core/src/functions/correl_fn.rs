use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::expand_aggregate_arg;
use crate::functions::paired_stats_common::{collect_paired_values, correlation_from_pairs};
use crate::resolver::ReferenceResolver;
use crate::value::{CallArgValue, EvalValue, WorksheetErrorCode};

pub const CORREL_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.CORREL",
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
};

#[derive(Debug, Clone, PartialEq)]
pub enum CorrelEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
}

pub fn eval_correl_surface(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, CorrelEvalError> {
    let argc = args.len();
    if !CORREL_META.arity.accepts(argc) {
        return Err(CorrelEvalError::ArityMismatch {
            expected_min: CORREL_META.arity.min,
            expected_max: CORREL_META.arity.max,
            actual: argc,
        });
    }
    let xs = expand_aggregate_arg(&args[0], resolver).map_err(CorrelEvalError::Coercion)?;
    let ys = expand_aggregate_arg(&args[1], resolver).map_err(CorrelEvalError::Coercion)?;
    let pairs = collect_paired_values(&xs, &ys).map_err(CorrelEvalError::Coercion)?;
    match correlation_from_pairs(&pairs) {
        Ok(value) => Ok(EvalValue::Number(value)),
        Err(code) => Ok(EvalValue::Error(code)),
    }
}

pub fn map_correl_error_to_ws(e: &CorrelEvalError) -> WorksheetErrorCode {
    match e {
        CorrelEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        CorrelEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        CorrelEvalError::Coercion(_) => WorksheetErrorCode::Value,
    }
}
