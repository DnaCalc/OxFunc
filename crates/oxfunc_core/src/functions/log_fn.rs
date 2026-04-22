use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{
    coerce_prepared_to_number, run_values_only_prepared, PreparedArgValue,
};
use crate::resolver::ReferenceResolver;
use crate::value::{ArrayCellValue, CallArgValue, EvalArray, EvalValue, WorksheetErrorCode};

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

fn log_array_cell(cell: &ArrayCellValue, base: f64) -> ArrayCellValue {
    match cell {
        ArrayCellValue::Number(number) => match log_kernel(*number, base) {
            Ok(value) => ArrayCellValue::Number(value),
            Err(code) => ArrayCellValue::Error(code),
        },
        ArrayCellValue::Error(code) => ArrayCellValue::Error(*code),
        ArrayCellValue::Text(_) | ArrayCellValue::Logical(_) | ArrayCellValue::EmptyCell => {
            ArrayCellValue::Error(WorksheetErrorCode::Value)
        }
    }
}

fn eval_log_prepared(args: &[PreparedArgValue]) -> Result<EvalValue, LogEvalError> {
    if !LOG_META.arity.accepts(args.len()) {
        return Err(LogEvalError::ArityMismatch {
            expected_min: LOG_META.arity.min,
            expected_max: LOG_META.arity.max,
            actual: args.len(),
        });
    }
    match &args[0] {
        PreparedArgValue::Eval(EvalValue::Array(array)) => {
            let base = if args.len() >= 2 {
                match &args[1] {
                    PreparedArgValue::Eval(EvalValue::Array(_)) => {
                        return Err(LogEvalError::Coercion(CoercionError::UnsupportedValueKind(
                            "array_base",
                        )));
                    }
                    other => coerce_prepared_to_number(other).map_err(LogEvalError::Coercion)?,
                }
            } else {
                10.0
            };
            let cells = array
                .iter_row_major()
                .map(|cell| log_array_cell(cell, base))
                .collect();
            Ok(EvalValue::Array(
                EvalArray::new(array.shape(), cells).expect("input array shape is valid"),
            ))
        }
        _ => {
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
    use crate::resolver::{RefResolutionError, ReferenceResolver, ResolverCapabilities};
    use crate::value::ReferenceLike;

    struct NoResolver;

    impl ReferenceResolver for NoResolver {
        fn capabilities(&self) -> ResolverCapabilities {
            ResolverCapabilities::permissive_local()
        }

        fn resolve_reference(
            &self,
            reference: &ReferenceLike,
        ) -> Result<EvalValue, RefResolutionError> {
            Err(RefResolutionError::UnresolvedReference {
                target: reference.target.clone(),
            })
        }
    }

    #[test]
    fn log_kernel_seed_lanes_match_excel_probe() {
        assert_eq!(log_kernel(8.0, 2.0), Ok(3.0));
        assert_eq!(log_kernel(8.0, 1.0), Err(WorksheetErrorCode::Div0));
        assert_eq!(log_kernel(8.0, -2.0), Err(WorksheetErrorCode::Num));
    }

    #[test]
    fn ftc_0966_log_array_input_lifts_first_argument_against_scalar_base() {
        let got = eval_log_surface(
            &[
                CallArgValue::Eval(EvalValue::Array(
                    EvalArray::from_rows(vec![vec![
                        ArrayCellValue::Number(0.5),
                        ArrayCellValue::Number(0.25),
                        ArrayCellValue::Number(0.125),
                        ArrayCellValue::Number(0.125),
                    ]])
                    .unwrap(),
                )),
                CallArgValue::Eval(EvalValue::Number(2.0)),
            ],
            &NoResolver,
        );
        assert_eq!(
            got,
            Ok(EvalValue::Array(
                EvalArray::from_rows(vec![vec![
                    ArrayCellValue::Number(-1.0),
                    ArrayCellValue::Number(-2.0),
                    ArrayCellValue::Number(-3.0),
                    ArrayCellValue::Number(-3.0),
                ]])
                .unwrap()
            ))
        );
    }
}
