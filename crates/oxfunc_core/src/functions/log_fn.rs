use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{coerce_prepared_to_number, run_values_only_prepared};
use crate::resolver::ReferenceSystemProvider;
use crate::value::{CalcArray, CalcValue, CoreValue, WorksheetErrorCode};

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
    real_result_policy: FunctionMeta::DEFAULT_REAL_RESULT_POLICY,
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
    // Match Excel bit-for-bit: Excel evaluates the common bases via dedicated
    // log10/log2 rather than ln(x)/ln(base) (which is ~1 ULP off, e.g. LOG(2)).
    Ok(match base {
        10.0 => number.log10(),
        2.0 => number.log2(),
        _ => number.ln() / base.ln(),
    })
}

fn log_array_cell(cell: &CalcValue, base: f64) -> CalcValue {
    match cell.core() {
        CoreValue::Number(number) => match log_kernel(*number, base) {
            Ok(value) => CalcValue::number(value),
            Err(code) => CalcValue::error(code),
        },
        CoreValue::Error(_) => cell.clone(),
        CoreValue::Text(_)
        | CoreValue::Logical(_)
        | CoreValue::Empty
        | CoreValue::Missing
        | CoreValue::Array(_)
        | CoreValue::Reference(_) => CalcValue::error(WorksheetErrorCode::Value),
    }
}

fn eval_log_prepared(args: &[CalcValue]) -> Result<CalcValue, LogEvalError> {
    if !LOG_META.arity.accepts(args.len()) {
        return Err(LogEvalError::ArityMismatch {
            expected_min: LOG_META.arity.min,
            expected_max: LOG_META.arity.max,
            actual: args.len(),
        });
    }
    match args[0].core() {
        CoreValue::Array(array) => {
            let base = if args.len() >= 2 {
                match args[1].core() {
                    CoreValue::Array(_) => {
                        return Err(LogEvalError::Coercion(CoercionError::UnsupportedValueKind(
                            "array_base",
                        )));
                    }
                    _ => coerce_prepared_to_number(&args[1]).map_err(LogEvalError::Coercion)?,
                }
            } else {
                10.0
            };
            let cells = array
                .iter_row_major()
                .map(|cell| log_array_cell(cell, base))
                .collect();
            Ok(CalcValue::array(
                CalcArray::new(array.shape(), cells).expect("input array shape is valid"),
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
                Ok(value) => Ok(CalcValue::number(value)),
                Err(code) => Ok(CalcValue::error(code)),
            }
        }
    }
}

pub fn eval_log_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, LogEvalError> {
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
    use crate::resolver::{ReferenceSystemCapabilities, ReferenceSystemProvider};

    struct NoResolver;

    impl ReferenceSystemProvider for NoResolver {
        fn capabilities(&self) -> ReferenceSystemCapabilities {
            ReferenceSystemCapabilities::permissive_local()
        }

        fn dereference(
            &self,
            request: &crate::resolver::ReferenceDereferenceRequest,
        ) -> Result<CalcValue, crate::resolver::ReferenceResolutionError> {
            let reference = &request.reference;
            Err(
                crate::resolver::ReferenceResolutionError::UnresolvedReference {
                    target: reference.target().to_string(),
                },
            )
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
                (CalcValue::array(
                    CalcArray::from_rows(vec![vec![
                        CalcValue::number(0.5),
                        CalcValue::number(0.25),
                        CalcValue::number(0.125),
                        CalcValue::number(0.125),
                    ]])
                    .unwrap(),
                )),
                (CalcValue::number(2.0)),
            ],
            &NoResolver,
        );
        assert_eq!(
            got,
            Ok(CalcValue::array(
                CalcArray::from_rows(vec![vec![
                    CalcValue::number(-1.0),
                    CalcValue::number(-2.0),
                    CalcValue::number(-3.0),
                    CalcValue::number(-3.0),
                ]])
                .unwrap()
            ))
        );
    }
}
