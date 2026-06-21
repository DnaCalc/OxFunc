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

pub const XOR_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.XOR",
    arity: Arity { min: 1, max: 255 },
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: FunctionMeta::DEFAULT_ARG_PREPARATION_PROFILE,
    coercion_lift_profile: CoercionLiftProfile::Custom,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
    real_result_policy: FunctionMeta::DEFAULT_REAL_RESULT_POLICY,
};

#[derive(Debug, Clone, PartialEq)]
pub enum XorEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
}

pub fn eval_xor_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, XorEvalError> {
    let argc = args.len();
    if !XOR_META.arity.accepts(argc) {
        return Err(XorEvalError::ArityMismatch {
            expected_min: XOR_META.arity.min,
            expected_max: XOR_META.arity.max,
            actual: argc,
        });
    }

    let mut saw_value = false;
    let mut parity = false;
    for arg in args {
        for item in expand_aggregate_arg(arg, resolver).map_err(XorEvalError::Coercion)? {
            match and_argument_truth(&item).map_err(XorEvalError::Coercion)? {
                Some(truth) => {
                    saw_value = true;
                    if truth {
                        parity = !parity;
                    }
                }
                None => {}
            }
        }
    }

    if !saw_value {
        return Ok(CalcValue::error(WorksheetErrorCode::Value));
    }

    Ok(CalcValue::logical(parity))
}

pub fn map_xor_error_to_ws(e: &XorEvalError) -> WorksheetErrorCode {
    match e {
        XorEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        XorEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        XorEvalError::Coercion(_) => WorksheetErrorCode::Value,
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
    fn eval_xor_returns_parity_of_true_values() {
        let got = eval_xor_surface(
            &[
                (CalcValue::logical(true)),
                (CalcValue::logical(false)),
                (CalcValue::logical(true)),
            ],
            &MockResolver { resolved: None },
        );
        assert_eq!(got, Ok(CalcValue::logical(false)));
    }

    #[test]
    fn eval_xor_ignores_reference_text_and_empty_cells() {
        let got = eval_xor_surface(
            &[CalcValue::reference(ReferenceLike::new(
                ReferenceKind::Area,
                "A1:A4".to_string(),
            ))],
            &MockResolver {
                resolved: Some(CalcValue::array(
                    CalcArray::from_rows(vec![vec![
                        CalcValue::text(ExcelText::from_utf16_code_units(
                            "x".encode_utf16().collect(),
                        )),
                        CalcValue::empty(),
                        CalcValue::logical(true),
                        CalcValue::number(0.0),
                    ]])
                    .unwrap(),
                )),
            },
        );
        assert_eq!(got, Ok(CalcValue::logical(true)));
    }

    #[test]
    fn eval_xor_direct_text_is_value_error() {
        let got = eval_xor_surface(
            &[(CalcValue::text(ExcelText::from_utf16_code_units(
                "x".encode_utf16().collect(),
            )))],
            &MockResolver { resolved: None },
        );
        assert!(matches!(
            got,
            Err(XorEvalError::Coercion(CoercionError::NonNumericText(_)))
        ));
    }

    #[test]
    fn eval_xor_returns_value_when_all_inputs_are_ignored() {
        let got = eval_xor_surface(
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
}
