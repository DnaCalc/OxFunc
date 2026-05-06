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

pub const STANDARDIZE_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.STANDARDIZE",
    arity: Arity::exact(3),
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
pub enum StandardizeEvalError {
    ArityMismatch { expected: usize, actual: usize },
    Coercion(CoercionError),
}

pub fn standardize_kernel(x: f64, mean: f64, stdev: f64) -> Result<f64, WorksheetErrorCode> {
    if stdev <= 0.0 {
        return Err(WorksheetErrorCode::Num);
    }
    Ok((x - mean) / stdev)
}

fn eval_standardize_prepared(args: &[PreparedArgValue]) -> Result<EvalValue, StandardizeEvalError> {
    if args.len() != 3 {
        return Err(StandardizeEvalError::ArityMismatch {
            expected: 3,
            actual: args.len(),
        });
    }
    let x = coerce_prepared_to_number(&args[0]).map_err(StandardizeEvalError::Coercion)?;
    let mean = coerce_prepared_to_number(&args[1]).map_err(StandardizeEvalError::Coercion)?;
    let stdev = coerce_prepared_to_number(&args[2]).map_err(StandardizeEvalError::Coercion)?;
    match standardize_kernel(x, mean, stdev) {
        Ok(value) => Ok(EvalValue::Number(value)),
        Err(code) => Ok(EvalValue::Error(code)),
    }
}

pub fn eval_standardize_surface(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, StandardizeEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        eval_standardize_prepared,
        StandardizeEvalError::Coercion,
    )
}

pub fn map_standardize_error_to_ws(e: &StandardizeEvalError) -> WorksheetErrorCode {
    match e {
        StandardizeEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        StandardizeEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        StandardizeEvalError::Coercion(_) => WorksheetErrorCode::Value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standardize_kernel_seed_lane_matches_excel_probe() {
        assert_eq!(standardize_kernel(42.0, 40.0, 1.5), Ok(4.0 / 3.0));
        assert_eq!(
            standardize_kernel(42.0, 40.0, 0.0),
            Err(WorksheetErrorCode::Num)
        );
    }
}
