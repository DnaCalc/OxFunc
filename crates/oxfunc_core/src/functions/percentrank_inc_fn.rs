use crate::coercion::CoercionError;
use crate::function::{
    Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile, FunctionMeta,
    HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{
    coerce_prepared_to_number, expand_aggregate_arg, prepare_arg_values_only,
};
use crate::functions::percentile_common::collect_percentile_values;
use crate::functions::percentrank_common::{PercentRankMode, percentrank};
use crate::resolver::ReferenceSystemProvider;
use crate::value::CalcValue;
use crate::value::WorksheetErrorCode;

pub const PERCENTRANK_INC_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.PERCENTRANK.INC",
    arity: Arity { min: 2, max: 3 },
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: FunctionMeta::DEFAULT_ARG_PREPARATION_PROFILE,
    coercion_lift_profile: CoercionLiftProfile::Custom,
    lift_broadcast_profile: FunctionMeta::DEFAULT_LIFT_BROADCAST_PROFILE,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
    real_result_policy: FunctionMeta::DEFAULT_REAL_RESULT_POLICY,
    error_collapse_profile: FunctionMeta::DEFAULT_ERROR_COLLAPSE_PROFILE,
};

#[derive(Debug, Clone, PartialEq)]
pub enum PercentRankIncEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
}

pub fn eval_percentrank_inc_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, PercentRankIncEvalError> {
    if !PERCENTRANK_INC_META.arity.accepts(args.len()) {
        return Err(PercentRankIncEvalError::ArityMismatch {
            expected_min: PERCENTRANK_INC_META.arity.min,
            expected_max: PERCENTRANK_INC_META.arity.max,
            actual: args.len(),
        });
    }
    let expanded =
        expand_aggregate_arg(&args[0], resolver).map_err(PercentRankIncEvalError::Coercion)?;
    let mut values =
        collect_percentile_values(&expanded).map_err(PercentRankIncEvalError::Coercion)?;
    let x = coerce_prepared_to_number(
        &prepare_arg_values_only(&args[1], resolver).map_err(PercentRankIncEvalError::Coercion)?,
    )
    .map_err(PercentRankIncEvalError::Coercion)?;
    let significance = if args.len() > 2 {
        coerce_prepared_to_number(
            &prepare_arg_values_only(&args[2], resolver)
                .map_err(PercentRankIncEvalError::Coercion)?,
        )
        .map_err(PercentRankIncEvalError::Coercion)?
        .trunc() as i64
    } else {
        3
    };
    match percentrank(&mut values, x, significance, PercentRankMode::Inclusive) {
        Ok(v) => Ok(CalcValue::number(v)),
        Err(code) => Ok(CalcValue::error(code)),
    }
}

pub fn map_percentrank_inc_error_to_ws(e: &PercentRankIncEvalError) -> WorksheetErrorCode {
    match e {
        PercentRankIncEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        PercentRankIncEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        PercentRankIncEvalError::Coercion(_) => WorksheetErrorCode::Value,
    }
}
