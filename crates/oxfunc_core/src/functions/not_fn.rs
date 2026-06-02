use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{
    PreparedValue, coerce_prepared_to_number, run_values_only_prepared,
};
use crate::resolver::ReferenceSystemProvider;
use crate::value::{
    FunctionArg, FunctionArray, FunctionArrayCell, FunctionValue, WorksheetErrorCode,
};

pub const NOT_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.NOT",
    arity: Arity::exact(1),
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
pub enum NotEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
}

fn eval_not_prepared(args: &[PreparedValue]) -> Result<FunctionValue, NotEvalError> {
    if !NOT_META.arity.accepts(args.len()) {
        return Err(NotEvalError::ArityMismatch {
            expected_min: NOT_META.arity.min,
            expected_max: NOT_META.arity.max,
            actual: args.len(),
        });
    }
    match &args[0] {
        PreparedValue::Eval(FunctionValue::Array(array)) => {
            let cells = array.iter_row_major().map(not_cell).collect();
            Ok(FunctionValue::Array(
                FunctionArray::new(array.shape(), cells).expect("input array shape is valid"),
            ))
        }
        _ => {
            let value = coerce_prepared_to_number(&args[0]).map_err(NotEvalError::Coercion)?;
            Ok(FunctionValue::Logical(value == 0.0))
        }
    }
}

fn not_cell(cell: &FunctionArrayCell) -> FunctionArrayCell {
    match cell {
        FunctionArrayCell::Number(n) => FunctionArrayCell::Logical(*n == 0.0),
        FunctionArrayCell::Logical(b) => FunctionArrayCell::Logical(!b),
        FunctionArrayCell::Error(code) => FunctionArrayCell::Error(*code),
        FunctionArrayCell::EmptyCell => FunctionArrayCell::Logical(true),
        FunctionArrayCell::Text(_) => FunctionArrayCell::Error(WorksheetErrorCode::Value),
    }
}

pub fn eval_not_surface(
    args: &[FunctionArg],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<FunctionValue, NotEvalError> {
    run_values_only_prepared(args, resolver, eval_not_prepared, NotEvalError::Coercion)
}

pub fn map_not_error_to_ws(e: &NotEvalError) -> WorksheetErrorCode {
    match e {
        NotEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        NotEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        NotEvalError::Coercion(_) => WorksheetErrorCode::Value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::ReferenceSystemCapabilities;
    use crate::value::{ExcelText, ReferenceKind, ReferenceLike};

    struct MockResolver {
        resolved_value: Option<FunctionValue>,
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
            self.resolved_value.clone().ok_or(
                crate::resolver::ReferenceResolutionError::UnresolvedReference {
                    target: reference.target().to_string(),
                },
            )
        }
    }

    #[test]
    fn eval_not_basic_lanes() {
        assert_eq!(
            eval_not_surface(
                &[FunctionArg::Eval(FunctionValue::Logical(true))],
                &MockResolver {
                    resolved_value: None
                }
            ),
            Ok(FunctionValue::Logical(false))
        );
        assert_eq!(
            eval_not_surface(
                &[FunctionArg::Eval(FunctionValue::Number(0.0))],
                &MockResolver {
                    resolved_value: None
                }
            ),
            Ok(FunctionValue::Logical(true))
        );
        assert_eq!(
            eval_not_surface(
                &[FunctionArg::Eval(FunctionValue::Number(2.0))],
                &MockResolver {
                    resolved_value: None
                }
            ),
            Ok(FunctionValue::Logical(false))
        );
    }

    #[test]
    fn eval_not_direct_text_is_value_error() {
        let got = eval_not_surface(
            &[FunctionArg::Eval(FunctionValue::Text(
                ExcelText::from_utf16_code_units("x".encode_utf16().collect()),
            ))],
            &MockResolver {
                resolved_value: None,
            },
        );
        assert!(matches!(
            got,
            Err(NotEvalError::Coercion(CoercionError::NonNumericText(_)))
        ));
    }

    #[test]
    fn eval_not_reference_uses_resolved_scalar() {
        let got = eval_not_surface(
            &[FunctionArg::Reference(ReferenceLike::new(
                ReferenceKind::A1,
                "A1".to_string(),
            ))],
            &MockResolver {
                resolved_value: Some(FunctionValue::Logical(true)),
            },
        );
        assert_eq!(got, Ok(FunctionValue::Logical(false)));
    }

    #[test]
    fn eval_not_array_lifts_elementwise() {
        let got = eval_not_surface(
            &[FunctionArg::Eval(FunctionValue::Array(
                FunctionArray::from_rows(vec![vec![
                    FunctionArrayCell::Logical(true),
                    FunctionArrayCell::Logical(false),
                    FunctionArrayCell::Number(0.0),
                    FunctionArrayCell::Number(2.0),
                    FunctionArrayCell::Error(WorksheetErrorCode::NA),
                ]])
                .unwrap(),
            ))],
            &MockResolver {
                resolved_value: None,
            },
        );
        assert_eq!(
            got,
            Ok(FunctionValue::Array(
                FunctionArray::from_rows(vec![vec![
                    FunctionArrayCell::Logical(false),
                    FunctionArrayCell::Logical(true),
                    FunctionArrayCell::Logical(true),
                    FunctionArrayCell::Logical(false),
                    FunctionArrayCell::Error(WorksheetErrorCode::NA),
                ]])
                .unwrap()
            ))
        );
    }
}
