use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{
    coerce_prepared_to_number, expand_aggregate_arg, prepare_arg_values_only,
};
use crate::functions::percentile_common::collect_percentile_values;
use crate::functions::percentrank_common::{PercentRankMode, percentrank};
use crate::resolver::ReferenceResolver;
use crate::value::{CallArgValue, EvalValue, WorksheetErrorCode};

pub const PERCENTRANK_EXC_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.PERCENTRANK.EXC",
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
pub enum PercentRankExcEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
}

pub fn eval_percentrank_exc_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, PercentRankExcEvalError> {
    if !PERCENTRANK_EXC_META.arity.accepts(args.len()) {
        return Err(PercentRankExcEvalError::ArityMismatch {
            expected_min: PERCENTRANK_EXC_META.arity.min,
            expected_max: PERCENTRANK_EXC_META.arity.max,
            actual: args.len(),
        });
    }
    let expanded =
        expand_aggregate_arg(&args[0], resolver).map_err(PercentRankExcEvalError::Coercion)?;
    let mut values =
        collect_percentile_values(&expanded).map_err(PercentRankExcEvalError::Coercion)?;
    let x = coerce_prepared_to_number(
        &prepare_arg_values_only(&args[1], resolver).map_err(PercentRankExcEvalError::Coercion)?,
    )
    .map_err(PercentRankExcEvalError::Coercion)?;
    let significance = if args.len() > 2 {
        coerce_prepared_to_number(
            &prepare_arg_values_only(&args[2], resolver)
                .map_err(PercentRankExcEvalError::Coercion)?,
        )
        .map_err(PercentRankExcEvalError::Coercion)?
        .trunc() as i64
    } else {
        3
    };
    match percentrank(&mut values, x, significance, PercentRankMode::Exclusive) {
        Ok(v) => Ok(EvalValue::Number(v)),
        Err(code) => Ok(EvalValue::Error(code)),
    }
}

pub fn map_percentrank_exc_error_to_ws(e: &PercentRankExcEvalError) -> WorksheetErrorCode {
    match e {
        PercentRankExcEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        PercentRankExcEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        PercentRankExcEvalError::Coercion(_) => WorksheetErrorCode::Value,
    }
}
