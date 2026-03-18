use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{
    coerce_prepared_to_number, expand_aggregate_arg, prepare_arg_values_only,
};
use crate::functions::percentile_common::{collect_percentile_values, percentile_inc_kernel};
use crate::resolver::ReferenceResolver;
use crate::value::{CallArgValue, EvalValue, WorksheetErrorCode};

pub const PERCENTILE_INC_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.PERCENTILE.INC",
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
pub enum PercentileIncEvalError {
    ArityMismatch { expected: usize, actual: usize },
    Coercion(CoercionError),
}

pub fn eval_percentile_inc_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, PercentileIncEvalError> {
    if !PERCENTILE_INC_META.arity.accepts(args.len()) {
        return Err(PercentileIncEvalError::ArityMismatch {
            expected: PERCENTILE_INC_META.arity.min,
            actual: args.len(),
        });
    }
    let expanded =
        expand_aggregate_arg(&args[0], resolver).map_err(PercentileIncEvalError::Coercion)?;
    let mut values =
        collect_percentile_values(&expanded).map_err(PercentileIncEvalError::Coercion)?;
    let k = coerce_prepared_to_number(
        &prepare_arg_values_only(&args[1], resolver).map_err(PercentileIncEvalError::Coercion)?,
    )
    .map_err(PercentileIncEvalError::Coercion)?;
    match percentile_inc_kernel(&mut values, k) {
        Ok(v) => Ok(EvalValue::Number(v)),
        Err(code) => Ok(EvalValue::Error(code)),
    }
}

pub fn map_percentile_inc_error_to_ws(e: &PercentileIncEvalError) -> WorksheetErrorCode {
    match e {
        PercentileIncEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        PercentileIncEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        PercentileIncEvalError::Coercion(_) => WorksheetErrorCode::Value,
    }
}
