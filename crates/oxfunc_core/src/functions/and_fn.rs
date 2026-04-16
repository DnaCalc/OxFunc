use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::expand_aggregate_arg;
use crate::functions::aggregate_common::and_argument_truth;
use crate::resolver::ReferenceResolver;
use crate::value::{ArrayCellValue, CallArgValue, EvalArray, EvalValue, WorksheetErrorCode};

pub const AND_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.AND",
    arity: Arity { min: 1, max: 255 },
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
pub enum AndEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
}

enum DirectElementwiseArg {
    Scalar(CallArgValue),
    Array(EvalArray),
}

fn truth_from_direct_scalar(arg: &CallArgValue) -> Result<Option<bool>, CoercionError> {
    match arg {
        CallArgValue::MissingArg | CallArgValue::EmptyCell => Ok(None),
        CallArgValue::Eval(EvalValue::Logical(b)) => Ok(Some(*b)),
        CallArgValue::Eval(EvalValue::Number(n)) => Ok(Some(*n != 0.0)),
        CallArgValue::Eval(EvalValue::Error(code)) => Err(CoercionError::WorksheetError(*code)),
        CallArgValue::Eval(EvalValue::Text(_)) => {
            Err(CoercionError::NonNumericText("direct_text".to_string()))
        }
        CallArgValue::Eval(EvalValue::Array(_))
        | CallArgValue::Eval(EvalValue::Reference(_))
        | CallArgValue::Eval(EvalValue::Lambda(_))
        | CallArgValue::Reference(_) => Err(CoercionError::UnsupportedValueKind(
            "direct_elementwise_scalar",
        )),
    }
}

fn truth_from_direct_array_cell(cell: &ArrayCellValue) -> Result<Option<bool>, CoercionError> {
    match cell {
        ArrayCellValue::Logical(b) => Ok(Some(*b)),
        ArrayCellValue::Number(n) => Ok(Some(*n != 0.0)),
        ArrayCellValue::Error(code) => Err(CoercionError::WorksheetError(*code)),
        ArrayCellValue::Text(_) => Err(CoercionError::NonNumericText("direct_text".to_string())),
        ArrayCellValue::EmptyCell => Ok(None),
    }
}

fn try_eval_and_direct_elementwise(args: &[CallArgValue]) -> Result<Option<EvalValue>, CoercionError> {
    let mut saw_array = false;
    let mut target_shape = None;
    let mut prepared_args = Vec::with_capacity(args.len());

    for arg in args {
        match arg {
            CallArgValue::Eval(EvalValue::Array(array)) => {
                saw_array = true;
                match target_shape {
                    None => target_shape = Some(array.shape()),
                    Some(shape) if shape != array.shape() => {
                        return Err(CoercionError::UnsupportedValueKind("mismatched_array_shape"));
                    }
                    _ => {}
                }
                prepared_args.push(DirectElementwiseArg::Array(array.clone()));
            }
            CallArgValue::Eval(_)
            | CallArgValue::MissingArg
            | CallArgValue::EmptyCell => {
                prepared_args.push(DirectElementwiseArg::Scalar(arg.clone()));
            }
            CallArgValue::Reference(_) => return Ok(None),
        }
    }

    let Some(shape) = target_shape else {
        return Ok(None);
    };
    if !saw_array {
        return Ok(None);
    }

    let mut cells = Vec::with_capacity(shape.rows * shape.cols);
    for row in 0..shape.rows {
        for col in 0..shape.cols {
            let mut saw_value = false;
            let mut cell_result = ArrayCellValue::Logical(true);
            for arg in &prepared_args {
                let truth = match arg {
                    DirectElementwiseArg::Scalar(arg) => truth_from_direct_scalar(arg)?,
                    DirectElementwiseArg::Array(array) => {
                        truth_from_direct_array_cell(
                            array.get(row, col).expect("validated elementwise shape"),
                        )?
                    }
                };
                match truth {
                    Some(false) => {
                        cell_result = ArrayCellValue::Logical(false);
                        saw_value = true;
                        break;
                    }
                    Some(true) => saw_value = true,
                    None => {}
                }
            }
            if !saw_value {
                cell_result = ArrayCellValue::Error(WorksheetErrorCode::Value);
            }
            cells.push(cell_result);
        }
    }

    Ok(Some(EvalValue::Array(
        EvalArray::new(shape, cells).expect("validated elementwise shape"),
    )))
}

pub fn eval_and_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, AndEvalError> {
    let argc = args.len();
    if !AND_META.arity.accepts(argc) {
        return Err(AndEvalError::ArityMismatch {
            expected_min: AND_META.arity.min,
            expected_max: AND_META.arity.max,
            actual: argc,
        });
    }

    if let Some(elementwise) =
        try_eval_and_direct_elementwise(args).map_err(AndEvalError::Coercion)?
    {
        return Ok(elementwise);
    }

    let mut saw_value = false;
    for arg in args {
        for item in expand_aggregate_arg(arg, resolver).map_err(AndEvalError::Coercion)? {
            match and_argument_truth(&item).map_err(AndEvalError::Coercion)? {
                Some(false) => return Ok(EvalValue::Logical(false)),
                Some(true) => saw_value = true,
                None => {}
            }
        }
    }

    if !saw_value {
        return Ok(EvalValue::Error(WorksheetErrorCode::Value));
    }

    Ok(EvalValue::Logical(true))
}

pub fn map_and_error_to_ws(e: &AndEvalError) -> WorksheetErrorCode {
    match e {
        AndEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        AndEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        AndEvalError::Coercion(_) => WorksheetErrorCode::Value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::{RefResolutionError, ResolverCapabilities};
    use crate::value::{ArrayCellValue, EvalArray, ExcelText, ReferenceKind, ReferenceLike};

    struct MockResolver {
        resolved: Option<EvalValue>,
    }

    impl ReferenceResolver for MockResolver {
        fn capabilities(&self) -> ResolverCapabilities {
            ResolverCapabilities::permissive_local()
        }

        fn resolve_reference(
            &self,
            reference: &ReferenceLike,
        ) -> Result<EvalValue, RefResolutionError> {
            self.resolved
                .clone()
                .ok_or(RefResolutionError::UnresolvedReference {
                    target: reference.target.clone(),
                })
        }
    }

    #[test]
    fn eval_and_returns_false_when_any_arg_is_zero() {
        let got = eval_and_surface(
            &[
                CallArgValue::Eval(EvalValue::Logical(true)),
                CallArgValue::Eval(EvalValue::Number(0.0)),
            ],
            &MockResolver { resolved: None },
        );
        assert_eq!(got, Ok(EvalValue::Logical(false)));
    }

    #[test]
    fn eval_and_ignores_reference_text_and_empty_cells() {
        let got = eval_and_surface(
            &[CallArgValue::Reference(ReferenceLike {
                kind: ReferenceKind::Area,
                target: "A1:A3".to_string(),
            })],
            &MockResolver {
                resolved: Some(EvalValue::Array(
                    EvalArray::from_rows(vec![vec![
                        ArrayCellValue::Text(ExcelText::from_utf16_code_units(
                            "x".encode_utf16().collect(),
                        )),
                        ArrayCellValue::EmptyCell,
                        ArrayCellValue::Logical(true),
                    ]])
                    .unwrap(),
                )),
            },
        );
        assert_eq!(got, Ok(EvalValue::Logical(true)));
    }

    #[test]
    fn eval_and_direct_text_is_value_error() {
        let got = eval_and_surface(
            &[CallArgValue::Eval(EvalValue::Text(
                ExcelText::from_utf16_code_units("1".encode_utf16().collect()),
            ))],
            &MockResolver { resolved: None },
        );
        assert!(matches!(
            got,
            Err(AndEvalError::Coercion(CoercionError::NonNumericText(_)))
        ));
    }

    #[test]
    fn eval_and_returns_value_when_all_inputs_are_ignored() {
        let got = eval_and_surface(
            &[CallArgValue::Reference(ReferenceLike {
                kind: ReferenceKind::Area,
                target: "A1:A2".to_string(),
            })],
            &MockResolver {
                resolved: Some(EvalValue::Array(
                    EvalArray::from_rows(vec![vec![
                        ArrayCellValue::Text(ExcelText::from_utf16_code_units(
                            "x".encode_utf16().collect(),
                        )),
                        ArrayCellValue::EmptyCell,
                    ]])
                    .unwrap(),
                )),
            },
        );
        assert_eq!(got, Ok(EvalValue::Error(WorksheetErrorCode::Value)));
    }

    #[test]
    fn eval_and_direct_arrays_lift_elementwise() {
        let got = eval_and_surface(
            &[
                CallArgValue::Eval(EvalValue::Array(
                    EvalArray::from_rows(vec![vec![
                        ArrayCellValue::Logical(true),
                        ArrayCellValue::Logical(true),
                        ArrayCellValue::Logical(false),
                    ]])
                    .unwrap(),
                )),
                CallArgValue::Eval(EvalValue::Array(
                    EvalArray::from_rows(vec![vec![
                        ArrayCellValue::Logical(true),
                        ArrayCellValue::Logical(false),
                        ArrayCellValue::Logical(true),
                    ]])
                    .unwrap(),
                )),
            ],
            &MockResolver { resolved: None },
        );
        assert_eq!(
            got,
            Ok(EvalValue::Array(
                EvalArray::from_rows(vec![vec![
                    ArrayCellValue::Logical(true),
                    ArrayCellValue::Logical(false),
                    ArrayCellValue::Logical(false),
                ]])
                .unwrap()
            ))
        );
    }
}
