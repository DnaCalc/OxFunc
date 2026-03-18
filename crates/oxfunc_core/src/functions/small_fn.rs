use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{
    AggregatePreparedValue, PreparedArgValue, coerce_prepared_to_number, expand_aggregate_arg,
    prepare_arg_values_only,
};
use crate::functions::aggregate_common::median_argument_value;
use crate::resolver::ReferenceResolver;
use crate::value::{CallArgValue, EvalValue, WorksheetErrorCode};

pub const SMALL_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.SMALL",
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
pub enum SmallEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
}

fn collect_values(args: &[AggregatePreparedValue]) -> Result<Vec<f64>, SmallEvalError> {
    let mut values = Vec::new();
    for arg in args {
        if let Some(value) = median_argument_value(arg).map_err(SmallEvalError::Coercion)? {
            values.push(value);
        }
    }
    Ok(values)
}

fn coerce_k(prepared: &PreparedArgValue) -> Result<usize, SmallEvalError> {
    let k = coerce_prepared_to_number(prepared)
        .map_err(SmallEvalError::Coercion)?
        .trunc();
    if k < 1.0 {
        return Err(SmallEvalError::Coercion(CoercionError::WorksheetError(
            WorksheetErrorCode::Num,
        )));
    }
    Ok(k as usize)
}

pub fn eval_small_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, SmallEvalError> {
    let argc = args.len();
    if !SMALL_META.arity.accepts(argc) {
        return Err(SmallEvalError::ArityMismatch {
            expected_min: SMALL_META.arity.min,
            expected_max: SMALL_META.arity.max,
            actual: argc,
        });
    }
    let expanded = expand_aggregate_arg(&args[0], resolver).map_err(SmallEvalError::Coercion)?;
    let mut values = collect_values(&expanded)?;
    let k_prepared =
        prepare_arg_values_only(&args[1], resolver).map_err(SmallEvalError::Coercion)?;
    let k = coerce_k(&k_prepared)?;
    if values.len() < k {
        return Ok(EvalValue::Error(WorksheetErrorCode::Num));
    }
    values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    Ok(EvalValue::Number(values[k - 1]))
}

pub fn map_small_error_to_ws(e: &SmallEvalError) -> WorksheetErrorCode {
    match e {
        SmallEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        SmallEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        SmallEvalError::Coercion(_) => WorksheetErrorCode::Value,
    }
}
