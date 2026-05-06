use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{PreparedArgValue, run_values_only_prepared};
use crate::resolver::ReferenceResolver;
use crate::value::{CallArgValue, EvalValue, WorksheetErrorCode};

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
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, ErrorTypeEvalError> {
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
                PreparedArgValue::Eval(EvalValue::Error(code)) => match error_type_number(*code) {
                    Some(n) => Ok(EvalValue::Number(n)),
                    None => Ok(EvalValue::Error(WorksheetErrorCode::NA)),
                },
                _ => Ok(EvalValue::Error(WorksheetErrorCode::NA)),
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
    fn error_type_maps_excel_error_numbers() {
        assert_eq!(
            eval_error_type_surface(
                &[CallArgValue::Eval(EvalValue::Error(WorksheetErrorCode::NA))],
                &MockResolver { resolved: None },
            ),
            Ok(EvalValue::Number(7.0))
        );
        assert_eq!(
            eval_error_type_surface(
                &[CallArgValue::Eval(EvalValue::Error(
                    WorksheetErrorCode::Div0
                ))],
                &MockResolver { resolved: None },
            ),
            Ok(EvalValue::Number(2.0))
        );
    }

    #[test]
    fn error_type_returns_na_for_non_error_and_blank_reference() {
        assert_eq!(
            eval_error_type_surface(
                &[CallArgValue::Eval(EvalValue::Number(1.0))],
                &MockResolver { resolved: None },
            ),
            Ok(EvalValue::Error(WorksheetErrorCode::NA))
        );
        assert_eq!(
            eval_error_type_surface(
                &[CallArgValue::Reference(ReferenceLike {
                    kind: ReferenceKind::A1,
                    target: "D1".to_string(),
                })],
                &MockResolver {
                    resolved: Some(EvalValue::Array(
                        EvalArray::from_rows(vec![vec![ArrayCellValue::EmptyCell]]).unwrap(),
                    )),
                },
            ),
            Ok(EvalValue::Error(WorksheetErrorCode::NA))
        );
        assert_eq!(
            eval_error_type_surface(
                &[CallArgValue::Eval(EvalValue::Text(
                    ExcelText::from_utf16_code_units(Vec::new(),)
                ))],
                &MockResolver { resolved: None },
            ),
            Ok(EvalValue::Error(WorksheetErrorCode::NA))
        );
    }
}
