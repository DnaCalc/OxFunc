use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::expand_aggregate_arg;
use crate::functions::aggregate_common::and_argument_truth;
use crate::resolver::ReferenceSystemProvider;
use crate::value::{FunctionArg, FunctionValue, WorksheetErrorCode};

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

pub fn eval_and_surface(
    args: &[FunctionArg],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<FunctionValue, AndEvalError> {
    let argc = args.len();
    if !AND_META.arity.accepts(argc) {
        return Err(AndEvalError::ArityMismatch {
            expected_min: AND_META.arity.min,
            expected_max: AND_META.arity.max,
            actual: argc,
        });
    }

    let mut saw_value = false;
    for arg in args {
        for item in expand_aggregate_arg(arg, resolver).map_err(AndEvalError::Coercion)? {
            match and_argument_truth(&item).map_err(AndEvalError::Coercion)? {
                Some(false) => return Ok(FunctionValue::Logical(false)),
                Some(true) => saw_value = true,
                None => {}
            }
        }
    }

    if !saw_value {
        return Ok(FunctionValue::Error(WorksheetErrorCode::Value));
    }

    Ok(FunctionValue::Logical(true))
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
    use crate::resolver::ReferenceSystemCapabilities;
    use crate::value::{ExcelText, FunctionArray, FunctionArrayCell, ReferenceKind, ReferenceLike};

    struct MockResolver {
        resolved: Option<FunctionValue>,
    }

    impl ReferenceSystemProvider for MockResolver {
        fn capabilities(&self) -> ReferenceSystemCapabilities {
            ReferenceSystemCapabilities::permissive_local()
        }

        fn dereference(
            &self,
            request: &crate::resolver::ReferenceDereferenceRequest,
        ) -> Result<FunctionValue, crate::resolver::ReferenceResolutionError> {
            let reference = &request.reference;
            self.resolved.clone().ok_or(
                crate::resolver::ReferenceResolutionError::UnresolvedReference {
                    target: reference.target().to_string(),
                },
            )
        }
    }

    #[test]
    fn eval_and_returns_false_when_any_arg_is_zero() {
        let got = eval_and_surface(
            &[
                FunctionArg::Eval(FunctionValue::Logical(true)),
                FunctionArg::Eval(FunctionValue::Number(0.0)),
            ],
            &MockResolver { resolved: None },
        );
        assert_eq!(got, Ok(FunctionValue::Logical(false)));
    }

    #[test]
    fn eval_and_ignores_reference_text_and_empty_cells() {
        let got = eval_and_surface(
            &[FunctionArg::Reference(ReferenceLike::new(
                ReferenceKind::Area,
                "A1:A3".to_string(),
            ))],
            &MockResolver {
                resolved: Some(FunctionValue::Array(
                    FunctionArray::from_rows(vec![vec![
                        FunctionArrayCell::Text(ExcelText::from_utf16_code_units(
                            "x".encode_utf16().collect(),
                        )),
                        FunctionArrayCell::EmptyCell,
                        FunctionArrayCell::Logical(true),
                    ]])
                    .unwrap(),
                )),
            },
        );
        assert_eq!(got, Ok(FunctionValue::Logical(true)));
    }

    #[test]
    fn eval_and_direct_text_is_value_error() {
        let got = eval_and_surface(
            &[FunctionArg::Eval(FunctionValue::Text(
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
            &[FunctionArg::Reference(ReferenceLike::new(
                ReferenceKind::Area,
                "A1:A2".to_string(),
            ))],
            &MockResolver {
                resolved: Some(FunctionValue::Array(
                    FunctionArray::from_rows(vec![vec![
                        FunctionArrayCell::Text(ExcelText::from_utf16_code_units(
                            "x".encode_utf16().collect(),
                        )),
                        FunctionArrayCell::EmptyCell,
                    ]])
                    .unwrap(),
                )),
            },
        );
        assert_eq!(got, Ok(FunctionValue::Error(WorksheetErrorCode::Value)));
    }

    #[test]
    fn ftc_0907_single_direct_true_array_scalarizes_to_true() {
        let got = eval_and_surface(
            &[FunctionArg::Eval(FunctionValue::Array(
                FunctionArray::from_rows(vec![vec![
                    FunctionArrayCell::Logical(true),
                    FunctionArrayCell::Logical(true),
                    FunctionArrayCell::Logical(true),
                ]])
                .unwrap(),
            ))],
            &MockResolver { resolved: None },
        );
        assert_eq!(got, Ok(FunctionValue::Logical(true)));
    }

    #[test]
    fn ftc_0907_single_direct_mixed_array_scalarizes_to_false() {
        let got = eval_and_surface(
            &[FunctionArg::Eval(FunctionValue::Array(
                FunctionArray::from_rows(vec![vec![
                    FunctionArrayCell::Logical(true),
                    FunctionArrayCell::Logical(false),
                    FunctionArrayCell::Logical(true),
                ]])
                .unwrap(),
            ))],
            &MockResolver { resolved: None },
        );
        assert_eq!(got, Ok(FunctionValue::Logical(false)));
    }

    #[test]
    fn ftc_1032_multi_arg_direct_arrays_scalarize_to_false() {
        let got = eval_and_surface(
            &[
                FunctionArg::Eval(FunctionValue::Array(
                    FunctionArray::from_rows(vec![vec![
                        FunctionArrayCell::Logical(false),
                        FunctionArrayCell::Logical(true),
                        FunctionArrayCell::Logical(true),
                    ]])
                    .unwrap(),
                )),
                FunctionArg::Eval(FunctionValue::Array(
                    FunctionArray::from_rows(vec![vec![
                        FunctionArrayCell::Logical(true),
                        FunctionArrayCell::Logical(true),
                        FunctionArrayCell::Logical(true),
                    ]])
                    .unwrap(),
                )),
            ],
            &MockResolver { resolved: None },
        );
        assert_eq!(got, Ok(FunctionValue::Logical(false)));
    }
}
