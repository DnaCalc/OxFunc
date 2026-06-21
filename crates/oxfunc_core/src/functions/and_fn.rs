use crate::coercion::CoercionError;
use crate::function::{
    Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile, FunctionMeta,
    HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::expand_aggregate_arg;
use crate::functions::aggregate_common::and_argument_truth;
use crate::resolver::ReferenceSystemProvider;
use crate::value::CalcValue;
use crate::value::WorksheetErrorCode;

pub const AND_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.AND",
    arity: Arity { min: 1, max: 255 },
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
pub enum AndEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
}

pub fn eval_and_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, AndEvalError> {
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
                Some(false) => return Ok(CalcValue::logical(false)),
                Some(true) => saw_value = true,
                None => {}
            }
        }
    }

    if !saw_value {
        return Ok(CalcValue::error(WorksheetErrorCode::Value));
    }

    Ok(CalcValue::logical(true))
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
    fn eval_and_returns_false_when_any_arg_is_zero() {
        let got = eval_and_surface(
            &[(CalcValue::logical(true)), (CalcValue::number(0.0))],
            &MockResolver { resolved: None },
        );
        assert_eq!(got, Ok(CalcValue::logical(false)));
    }

    #[test]
    fn eval_and_ignores_reference_text_and_empty_cells() {
        let got = eval_and_surface(
            &[CalcValue::reference(ReferenceLike::new(
                ReferenceKind::Area,
                "A1:A3".to_string(),
            ))],
            &MockResolver {
                resolved: Some(CalcValue::array(
                    CalcArray::from_rows(vec![vec![
                        CalcValue::text(ExcelText::from_utf16_code_units(
                            "x".encode_utf16().collect(),
                        )),
                        CalcValue::empty(),
                        CalcValue::logical(true),
                    ]])
                    .unwrap(),
                )),
            },
        );
        assert_eq!(got, Ok(CalcValue::logical(true)));
    }

    #[test]
    fn eval_and_direct_text_is_value_error() {
        let got = eval_and_surface(
            &[(CalcValue::text(ExcelText::from_utf16_code_units(
                "1".encode_utf16().collect(),
            )))],
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
            &[CalcValue::reference(ReferenceLike::new(
                ReferenceKind::Area,
                "A1:A2".to_string(),
            ))],
            &MockResolver {
                resolved: Some(CalcValue::array(
                    CalcArray::from_rows(vec![vec![
                        CalcValue::text(ExcelText::from_utf16_code_units(
                            "x".encode_utf16().collect(),
                        )),
                        CalcValue::empty(),
                    ]])
                    .unwrap(),
                )),
            },
        );
        assert_eq!(got, Ok(CalcValue::error(WorksheetErrorCode::Value)));
    }

    #[test]
    fn ftc_0907_single_direct_true_array_scalarizes_to_true() {
        let got = eval_and_surface(
            &[(CalcValue::array(
                CalcArray::from_rows(vec![vec![
                    CalcValue::logical(true),
                    CalcValue::logical(true),
                    CalcValue::logical(true),
                ]])
                .unwrap(),
            ))],
            &MockResolver { resolved: None },
        );
        assert_eq!(got, Ok(CalcValue::logical(true)));
    }

    #[test]
    fn ftc_0907_single_direct_mixed_array_scalarizes_to_false() {
        let got = eval_and_surface(
            &[(CalcValue::array(
                CalcArray::from_rows(vec![vec![
                    CalcValue::logical(true),
                    CalcValue::logical(false),
                    CalcValue::logical(true),
                ]])
                .unwrap(),
            ))],
            &MockResolver { resolved: None },
        );
        assert_eq!(got, Ok(CalcValue::logical(false)));
    }

    #[test]
    fn ftc_1032_multi_arg_direct_arrays_scalarize_to_false() {
        let got = eval_and_surface(
            &[
                (CalcValue::array(
                    CalcArray::from_rows(vec![vec![
                        CalcValue::logical(false),
                        CalcValue::logical(true),
                        CalcValue::logical(true),
                    ]])
                    .unwrap(),
                )),
                (CalcValue::array(
                    CalcArray::from_rows(vec![vec![
                        CalcValue::logical(true),
                        CalcValue::logical(true),
                        CalcValue::logical(true),
                    ]])
                    .unwrap(),
                )),
            ],
            &MockResolver { resolved: None },
        );
        assert_eq!(got, Ok(CalcValue::logical(false)));
    }
}
