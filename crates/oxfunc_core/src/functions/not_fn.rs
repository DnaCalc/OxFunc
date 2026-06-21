use crate::coercion::CoercionError;
use crate::function::{
    Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile, FunctionMeta,
    HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{coerce_prepared_to_number, run_values_only_prepared};
use crate::resolver::ReferenceSystemProvider;
use crate::value::{CalcArray, CalcValue, CoreValue, WorksheetErrorCode};

pub const NOT_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.NOT",
    arity: Arity::exact(1),
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: FunctionMeta::DEFAULT_ARG_PREPARATION_PROFILE,
    coercion_lift_profile: CoercionLiftProfile::Custom,
    lift_broadcast_profile: FunctionMeta::DEFAULT_LIFT_BROADCAST_PROFILE,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
    real_result_policy: FunctionMeta::DEFAULT_REAL_RESULT_POLICY,
    error_collapse_profile: FunctionMeta::DEFAULT_ERROR_COLLAPSE_PROFILE,
    precision_rounding_profile: FunctionMeta::DEFAULT_PRECISION_ROUNDING_PROFILE,
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

fn eval_not_prepared(args: &[CalcValue]) -> Result<CalcValue, NotEvalError> {
    if !NOT_META.arity.accepts(args.len()) {
        return Err(NotEvalError::ArityMismatch {
            expected_min: NOT_META.arity.min,
            expected_max: NOT_META.arity.max,
            actual: args.len(),
        });
    }
    match args[0].core() {
        CoreValue::Array(array) => {
            let cells = array.iter_row_major().map(not_cell).collect();
            Ok(CalcValue::array(
                CalcArray::new(array.shape(), cells).expect("input array shape is valid"),
            ))
        }
        _ => {
            let value = coerce_prepared_to_number(&args[0]).map_err(NotEvalError::Coercion)?;
            Ok(CalcValue::logical(value == 0.0))
        }
    }
}

fn not_cell(cell: &CalcValue) -> CalcValue {
    match cell.core() {
        CoreValue::Number(n) => CalcValue::logical(*n == 0.0),
        CoreValue::Logical(b) => CalcValue::logical(!b),
        CoreValue::Error(_) => cell.clone(),
        CoreValue::Empty => CalcValue::logical(true),
        CoreValue::Text(_) | CoreValue::Missing | CoreValue::Array(_) | CoreValue::Reference(_) => {
            CalcValue::error(WorksheetErrorCode::Value)
        }
    }
}

pub fn eval_not_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, NotEvalError> {
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
        resolved_value: Option<CalcValue>,
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
                &[(CalcValue::logical(true))],
                &MockResolver {
                    resolved_value: None
                }
            ),
            Ok(CalcValue::logical(false))
        );
        assert_eq!(
            eval_not_surface(
                &[(CalcValue::number(0.0))],
                &MockResolver {
                    resolved_value: None
                }
            ),
            Ok(CalcValue::logical(true))
        );
        assert_eq!(
            eval_not_surface(
                &[(CalcValue::number(2.0))],
                &MockResolver {
                    resolved_value: None
                }
            ),
            Ok(CalcValue::logical(false))
        );
    }

    #[test]
    fn eval_not_direct_text_is_value_error() {
        let got = eval_not_surface(
            &[(CalcValue::text(ExcelText::from_utf16_code_units(
                "x".encode_utf16().collect(),
            )))],
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
            &[CalcValue::reference(ReferenceLike::new(
                ReferenceKind::A1,
                "A1".to_string(),
            ))],
            &MockResolver {
                resolved_value: Some(CalcValue::logical(true)),
            },
        );
        assert_eq!(got, Ok(CalcValue::logical(false)));
    }

    #[test]
    fn eval_not_array_lifts_elementwise() {
        let got = eval_not_surface(
            &[(CalcValue::array(
                CalcArray::from_rows(vec![vec![
                    CalcValue::logical(true),
                    CalcValue::logical(false),
                    CalcValue::number(0.0),
                    CalcValue::number(2.0),
                    CalcValue::error(WorksheetErrorCode::NA),
                ]])
                .unwrap(),
            ))],
            &MockResolver {
                resolved_value: None,
            },
        );
        assert_eq!(
            got,
            Ok(CalcValue::array(
                CalcArray::from_rows(vec![vec![
                    CalcValue::logical(false),
                    CalcValue::logical(true),
                    CalcValue::logical(true),
                    CalcValue::logical(false),
                    CalcValue::error(WorksheetErrorCode::NA),
                ]])
                .unwrap()
            ))
        );
    }
}
