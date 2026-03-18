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

pub const LARGE_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.LARGE",
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
pub enum LargeEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
}

fn collect_values(args: &[AggregatePreparedValue]) -> Result<Vec<f64>, LargeEvalError> {
    let mut values = Vec::new();
    for arg in args {
        if let Some(value) = median_argument_value(arg).map_err(LargeEvalError::Coercion)? {
            values.push(value);
        }
    }
    Ok(values)
}

fn coerce_k(prepared: &PreparedArgValue) -> Result<usize, LargeEvalError> {
    let k = coerce_prepared_to_number(prepared)
        .map_err(LargeEvalError::Coercion)?
        .trunc();
    if k < 1.0 {
        return Err(LargeEvalError::Coercion(CoercionError::WorksheetError(
            WorksheetErrorCode::Num,
        )));
    }
    Ok(k as usize)
}

pub fn eval_large_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, LargeEvalError> {
    let argc = args.len();
    if !LARGE_META.arity.accepts(argc) {
        return Err(LargeEvalError::ArityMismatch {
            expected_min: LARGE_META.arity.min,
            expected_max: LARGE_META.arity.max,
            actual: argc,
        });
    }
    let expanded = expand_aggregate_arg(&args[0], resolver).map_err(LargeEvalError::Coercion)?;
    let mut values = collect_values(&expanded)?;
    let k_prepared =
        prepare_arg_values_only(&args[1], resolver).map_err(LargeEvalError::Coercion)?;
    let k = coerce_k(&k_prepared)?;
    if values.len() < k {
        return Ok(EvalValue::Error(WorksheetErrorCode::Num));
    }
    values.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
    Ok(EvalValue::Number(values[k - 1]))
}

pub fn map_large_error_to_ws(e: &LargeEvalError) -> WorksheetErrorCode {
    match e {
        LargeEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        LargeEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        LargeEvalError::Coercion(_) => WorksheetErrorCode::Value,
    }
}
