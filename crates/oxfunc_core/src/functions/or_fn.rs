use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::expand_aggregate_arg;
use crate::functions::aggregate_common::and_argument_truth;
use crate::resolver::ReferenceResolver;
use crate::value::{CallArgValue, EvalValue, WorksheetErrorCode};

pub const OR_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.OR",
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
pub enum OrEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
}

pub fn eval_or_surface(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, OrEvalError> {
    let argc = args.len();
    if !OR_META.arity.accepts(argc) {
        return Err(OrEvalError::ArityMismatch {
            expected_min: OR_META.arity.min,
            expected_max: OR_META.arity.max,
            actual: argc,
        });
    }

    let mut saw_value = false;
    for arg in args {
        for item in expand_aggregate_arg(arg, resolver).map_err(OrEvalError::Coercion)? {
            match and_argument_truth(&item).map_err(OrEvalError::Coercion)? {
                Some(true) => return Ok(EvalValue::Logical(true)),
                Some(false) => saw_value = true,
                None => {}
            }
        }
    }

    if !saw_value {
        return Ok(EvalValue::Error(WorksheetErrorCode::Value));
    }

    Ok(EvalValue::Logical(false))
}

pub fn map_or_error_to_ws(e: &OrEvalError) -> WorksheetErrorCode {
    match e {
        OrEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        OrEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        OrEvalError::Coercion(_) => WorksheetErrorCode::Value,
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
    fn eval_or_returns_true_when_any_arg_is_true() {
        let got = eval_or_surface(
            &[
                CallArgValue::Eval(EvalValue::Logical(false)),
                CallArgValue::Eval(EvalValue::Number(1.0)),
            ],
            &MockResolver { resolved: None },
        );
        assert_eq!(got, Ok(EvalValue::Logical(true)));
    }

    #[test]
    fn eval_or_ignores_reference_text_and_empty_cells() {
        let got = eval_or_surface(
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
                        ArrayCellValue::Number(0.0),
                    ]])
                    .unwrap(),
                )),
            },
        );
        assert_eq!(got, Ok(EvalValue::Logical(false)));
    }

    #[test]
    fn eval_or_direct_text_is_value_error() {
        let got = eval_or_surface(
            &[CallArgValue::Eval(EvalValue::Text(
                ExcelText::from_utf16_code_units("x".encode_utf16().collect()),
            ))],
            &MockResolver { resolved: None },
        );
        assert!(matches!(
            got,
            Err(OrEvalError::Coercion(CoercionError::NonNumericText(_)))
        ));
    }

    #[test]
    fn eval_or_returns_value_when_all_inputs_are_ignored() {
        let got = eval_or_surface(
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
}
