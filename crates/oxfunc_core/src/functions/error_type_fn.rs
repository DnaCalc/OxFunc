use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{PreparedValue, run_values_only_prepared};
use crate::resolver::ReferenceSystemProvider;
use crate::value::{FunctionArg, FunctionValue, WorksheetErrorCode};

pub const ERROR_TYPE_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.ERROR.TYPE",
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
pub enum ErrorTypeEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Preparation(CoercionError),
}

fn error_type_number(code: WorksheetErrorCode) -> Option<f64> {
    match code {
        WorksheetErrorCode::Null => Some(1.0),
        WorksheetErrorCode::Div0 => Some(2.0),
        WorksheetErrorCode::Value => Some(3.0),
        WorksheetErrorCode::Ref => Some(4.0),
        WorksheetErrorCode::Name => Some(5.0),
        WorksheetErrorCode::Num => Some(6.0),
        WorksheetErrorCode::NA => Some(7.0),
        WorksheetErrorCode::GettingData => Some(8.0),
        _ => None,
    }
}

pub fn eval_error_type_surface(
    args: &[FunctionArg],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<FunctionValue, ErrorTypeEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        |prepared| {
            if !ERROR_TYPE_META.arity.accepts(prepared.len()) {
                return Err(ErrorTypeEvalError::ArityMismatch {
                    expected_min: ERROR_TYPE_META.arity.min,
                    expected_max: ERROR_TYPE_META.arity.max,
                    actual: prepared.len(),
                });
            }
            match &prepared[0] {
                PreparedValue::Eval(FunctionValue::Error(code)) => match error_type_number(*code) {
                    Some(n) => Ok(FunctionValue::Number(n)),
                    None => Ok(FunctionValue::Error(WorksheetErrorCode::NA)),
                },
                _ => Ok(FunctionValue::Error(WorksheetErrorCode::NA)),
            }
        },
        ErrorTypeEvalError::Preparation,
    )
}

pub fn map_error_type_error_to_ws(e: &ErrorTypeEvalError) -> WorksheetErrorCode {
    match e {
        ErrorTypeEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        ErrorTypeEvalError::Preparation(CoercionError::WorksheetError(code)) => *code,
        ErrorTypeEvalError::Preparation(_) => WorksheetErrorCode::Value,
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
    fn error_type_maps_excel_error_numbers() {
        assert_eq!(
            eval_error_type_surface(
                &[FunctionArg::Eval(FunctionValue::Error(
                    WorksheetErrorCode::NA
                ))],
                &MockResolver { resolved: None },
            ),
            Ok(FunctionValue::Number(7.0))
        );
        assert_eq!(
            eval_error_type_surface(
                &[FunctionArg::Eval(FunctionValue::Error(
                    WorksheetErrorCode::Div0
                ))],
                &MockResolver { resolved: None },
            ),
            Ok(FunctionValue::Number(2.0))
        );
    }

    #[test]
    fn error_type_returns_na_for_non_error_and_blank_reference() {
        assert_eq!(
            eval_error_type_surface(
                &[FunctionArg::Eval(FunctionValue::Number(1.0))],
                &MockResolver { resolved: None },
            ),
            Ok(FunctionValue::Error(WorksheetErrorCode::NA))
        );
        assert_eq!(
            eval_error_type_surface(
                &[FunctionArg::Reference(ReferenceLike::new(
                    ReferenceKind::A1,
                    "D1".to_string()
                ))],
                &MockResolver {
                    resolved: Some(FunctionValue::Array(
                        FunctionArray::from_rows(vec![vec![FunctionArrayCell::EmptyCell]]).unwrap(),
                    )),
                },
            ),
            Ok(FunctionValue::Error(WorksheetErrorCode::NA))
        );
        assert_eq!(
            eval_error_type_surface(
                &[FunctionArg::Eval(FunctionValue::Text(
                    ExcelText::from_utf16_code_units(Vec::new(),)
                ))],
                &MockResolver { resolved: None },
            ),
            Ok(FunctionValue::Error(WorksheetErrorCode::NA))
        );
    }
}
