use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::run_values_only_prepared;
use crate::resolver::ReferenceSystemProvider;
use crate::value::{CalcValue, CoreValue, WorksheetErrorCode};

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
    real_result_policy: FunctionMeta::DEFAULT_REAL_RESULT_POLICY,
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
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, ErrorTypeEvalError> {
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
            match prepared[0].core() {
                CoreValue::Error(code) => match error_type_number(*code) {
                    Some(n) => Ok(CalcValue::number(n)),
                    None => Ok(CalcValue::error(WorksheetErrorCode::NA)),
                },
                _ => Ok(CalcValue::error(WorksheetErrorCode::NA)),
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
    use crate::value::{CalcArray, ExcelText, ReferenceKind, ReferenceLike};

    struct MockResolver {
        resolved: Option<CalcValue>,
    }

    impl ReferenceSystemProvider for MockResolver {
        fn capabilities(&self) -> ReferenceSystemCapabilities {
            ReferenceSystemCapabilities::permissive_local()
        }

        fn dereference(
            &self,
            request: &crate::resolver::ReferenceDereferenceRequest,
        ) -> Result<CalcValue, crate::resolver::ReferenceResolutionError> {
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
                &[(CalcValue::error(WorksheetErrorCode::NA))],
                &MockResolver { resolved: None },
            ),
            Ok(CalcValue::number(7.0))
        );
        assert_eq!(
            eval_error_type_surface(
                &[(CalcValue::error(WorksheetErrorCode::Div0))],
                &MockResolver { resolved: None },
            ),
            Ok(CalcValue::number(2.0))
        );
    }

    #[test]
    fn error_type_returns_na_for_non_error_and_blank_reference() {
        assert_eq!(
            eval_error_type_surface(
                &[(CalcValue::number(1.0))],
                &MockResolver { resolved: None },
            ),
            Ok(CalcValue::error(WorksheetErrorCode::NA))
        );
        assert_eq!(
            eval_error_type_surface(
                &[CalcValue::reference(ReferenceLike::new(
                    ReferenceKind::A1,
                    "D1".to_string()
                ))],
                &MockResolver {
                    resolved: Some(CalcValue::array(
                        CalcArray::from_rows(vec![vec![CalcValue::empty()]]).unwrap(),
                    )),
                },
            ),
            Ok(CalcValue::error(WorksheetErrorCode::NA))
        );
        assert_eq!(
            eval_error_type_surface(
                &[(CalcValue::text(ExcelText::from_utf16_code_units(Vec::new(),)))],
                &MockResolver { resolved: None },
            ),
            Ok(CalcValue::error(WorksheetErrorCode::NA))
        );
    }
}
