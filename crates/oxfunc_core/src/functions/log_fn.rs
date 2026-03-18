use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{
    PreparedArgValue, coerce_prepared_to_number, run_values_only_prepared,
};
use crate::resolver::ReferenceResolver;
use crate::value::{CallArgValue, EvalValue, WorksheetErrorCode};

pub const LOG_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.LOG",
    arity: Arity { min: 1, max: 2 },
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
pub enum LogEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
}

pub fn log_kernel(number: f64, base: f64) -> Result<f64, WorksheetErrorCode> {
    if number <= 0.0 || base <= 0.0 {
        return Err(WorksheetErrorCode::Num);
    }
    if base == 1.0 {
        return Err(WorksheetErrorCode::Div0);
    }
    Ok(number.ln() / base.ln())
}

fn eval_log_prepared(args: &[PreparedArgValue]) -> Result<EvalValue, LogEvalError> {
    if !LOG_META.arity.accepts(args.len()) {
        return Err(LogEvalError::ArityMismatch {
            expected_min: LOG_META.arity.min,
            expected_max: LOG_META.arity.max,
            actual: args.len(),
        });
    }
    let number = coerce_prepared_to_number(&args[0]).map_err(LogEvalError::Coercion)?;
    let base = if args.len() >= 2 {
        coerce_prepared_to_number(&args[1]).map_err(LogEvalError::Coercion)?
    } else {
        10.0
    };
    match log_kernel(number, base) {
        Ok(value) => Ok(EvalValue::Number(value)),
        Err(code) => Ok(EvalValue::Error(code)),
    }
}

pub fn eval_log_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, LogEvalError> {
    run_values_only_prepared(args, resolver, eval_log_prepared, LogEvalError::Coercion)
}

pub fn map_log_error_to_ws(e: &LogEvalError) -> WorksheetErrorCode {
    match e {
        LogEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        LogEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        LogEvalError::Coercion(_) => WorksheetErrorCode::Value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn log_kernel_seed_lanes_match_excel_probe() {
        assert_eq!(log_kernel(8.0, 2.0), Ok(3.0));
        assert_eq!(log_kernel(8.0, 1.0), Err(WorksheetErrorCode::Div0));
        assert_eq!(log_kernel(8.0, -2.0), Err(WorksheetErrorCode::Num));
    }
}
