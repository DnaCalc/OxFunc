use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::expand_aggregate_arg;
use crate::functions::paired_stats_common::{collect_paired_values, correlation_from_pairs};
use crate::resolver::ReferenceResolver;
use crate::value::{CallArgValue, EvalValue, WorksheetErrorCode};

pub const PEARSON_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.PEARSON",
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
pub enum PearsonEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
}

pub fn eval_pearson_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, PearsonEvalError> {
    let argc = args.len();
    if !PEARSON_META.arity.accepts(argc) {
        return Err(PearsonEvalError::ArityMismatch {
            expected_min: PEARSON_META.arity.min,
            expected_max: PEARSON_META.arity.max,
            actual: argc,
        });
    }
    let xs = expand_aggregate_arg(&args[0], resolver).map_err(PearsonEvalError::Coercion)?;
    let ys = expand_aggregate_arg(&args[1], resolver).map_err(PearsonEvalError::Coercion)?;
    let pairs = collect_paired_values(&xs, &ys).map_err(PearsonEvalError::Coercion)?;
    match correlation_from_pairs(&pairs) {
        Ok(value) => Ok(EvalValue::Number(value)),
        Err(code) => Ok(EvalValue::Error(code)),
    }
}

pub fn map_pearson_error_to_ws(e: &PearsonEvalError) -> WorksheetErrorCode {
    match e {
        PearsonEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        PearsonEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        PearsonEvalError::Coercion(_) => WorksheetErrorCode::Value,
    }
}
