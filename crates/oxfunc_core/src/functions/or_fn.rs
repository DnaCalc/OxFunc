use crate::coercion::CoercionError;
use crate::function::{
    Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile, FunctionMeta,
    HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::expand_aggregate_arg;
use crate::functions::aggregate_common::and_argument_truth;
use crate::resolver::ReferenceSystemProvider;
use crate::value::CalcValue;
use crate::value::CoreValue;
use crate::value::WorksheetErrorCode;

pub const OR_META: FunctionMeta = function_spec! {
    function_id: "FUNC.OR",
    arity: Arity { min: 1, max: 255 },
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
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
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, OrEvalError> {
    let argc = args.len();
    if !OR_META.arity.accepts(argc) {
        return Err(OrEvalError::ArityMismatch {
            expected_min: OR_META.arity.min,
            expected_max: OR_META.arity.max,
            actual: argc,
        });
    }

    // Excel evaluates every argument of OR; it does not short-circuit on the first TRUE. An
    // error value anywhere among the arguments surfaces, and when several arguments are errors
    // the FIRST one in argument order wins — a positional rule, not an error-code ranking
    // (live Excel 16.0 build 20326, COM probe 2026-09-15, bead `oxf-xvt5.14`: `=OR(TRUE,1/0)`
    // -> `#DIV/0!`, `=OR(TRUE,NA())` -> `#N/A`, `=OR(1/0,NA())` -> `#DIV/0!`, `=OR(NA(),1/0)`
    // -> `#N/A`, `=OR(TRUE,NA(),1/0)` -> `#N/A`, `=OR({TRUE,#N/A,#DIV/0!})` -> `#N/A`, and the
    // same for `#NUM!`/`#VALUE!`/`#REF!` pairs in both orders). So once an item has decided
    // the result the scan continues, looking only for error VALUES in the remaining items — not
    // for direct-text coercion failures: Excel ignores non-`"TRUE"`/`"FALSE"` direct text in
    // these folds altogether (`=OR(TRUE,"x")` -> `TRUE`, `=OR(FALSE,"x")` -> `FALSE`), a
    // pre-existing gap `and_argument_truth` still carries for the items before the decision
    // (catalog G1-02, bead `oxf-xvt5.15`); this loop neither widens nor narrows it.
    let mut saw_value = false;
    let mut decided = false;
    for arg in args {
        for item in expand_aggregate_arg(arg, resolver).map_err(OrEvalError::Coercion)? {
            if decided {
                if let CoreValue::Error(code) = item.0.core() {
                    return Err(OrEvalError::Coercion(CoercionError::WorksheetError(*code)));
                }
                continue;
            }
            match and_argument_truth(&item).map_err(OrEvalError::Coercion)? {
                Some(true) => decided = true,
                Some(false) => saw_value = true,
                None => {}
            }
        }
    }

    if decided {
        return Ok(CalcValue::logical(true));
    }

    if !saw_value {
        return Ok(CalcValue::error(WorksheetErrorCode::Value));
    }

    Ok(CalcValue::logical(false))
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
    fn eval_or_returns_true_when_any_arg_is_true() {
        let got = eval_or_surface(
            &[(CalcValue::logical(false)), (CalcValue::number(1.0))],
            &MockResolver { resolved: None },
        );
        assert_eq!(got, Ok(CalcValue::logical(true)));
    }

    #[test]
    fn eval_or_ignores_reference_text_and_empty_cells() {
        let got = eval_or_surface(
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
                        CalcValue::number(0.0),
                    ]])
                    .unwrap(),
                )),
            },
        );
        assert_eq!(got, Ok(CalcValue::logical(false)));
    }

    #[test]
    fn eval_or_direct_text_is_value_error() {
        let got = eval_or_surface(
            &[(CalcValue::text(ExcelText::from_utf16_code_units(
                "x".encode_utf16().collect(),
            )))],
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

    /// `=OR(TRUE,1/0)` -> `#DIV/0!` (live Excel 16.0 build 20326, `oxf-xvt5.14`): OR evaluates every
    /// argument, so an error AFTER the deciding TRUE still surfaces.
    #[test]
    fn eval_or_surfaces_an_error_after_the_deciding_true() {
        let got = eval_or_surface(
            &[
                CalcValue::logical(true),
                CalcValue::error(WorksheetErrorCode::Div0),
            ],
            &MockResolver { resolved: None },
        );
        assert_eq!(
            got,
            Err(OrEvalError::Coercion(CoercionError::WorksheetError(
                WorksheetErrorCode::Div0
            )))
        );
    }

    /// `=OR(TRUE,NA(),1/0)` -> `#N/A` and `=OR(TRUE,1/0,NA())` -> `#DIV/0!`: with several error
    /// arguments the first in argument order wins, even after the deciding value.
    #[test]
    fn eval_or_first_error_in_argument_order_wins_after_the_deciding_true() {
        for (second, third, expected) in [
            (
                WorksheetErrorCode::NA,
                WorksheetErrorCode::Div0,
                WorksheetErrorCode::NA,
            ),
            (
                WorksheetErrorCode::Div0,
                WorksheetErrorCode::NA,
                WorksheetErrorCode::Div0,
            ),
        ] {
            let got = eval_or_surface(
                &[
                    CalcValue::logical(true),
                    CalcValue::error(second),
                    CalcValue::error(third),
                ],
                &MockResolver { resolved: None },
            );
            assert_eq!(
                got,
                Err(OrEvalError::Coercion(CoercionError::WorksheetError(
                    expected
                )))
            );
        }
    }

    /// `=OR({TRUE,#N/A,#DIV/0!})` -> `#N/A`: the same in-order rule inside a single array argument.
    #[test]
    fn eval_or_scans_an_array_argument_past_the_deciding_true_for_errors() {
        let got = eval_or_surface(
            &[CalcValue::array(
                CalcArray::from_rows(vec![vec![
                    CalcValue::logical(true),
                    CalcValue::error(WorksheetErrorCode::NA),
                    CalcValue::error(WorksheetErrorCode::Div0),
                ]])
                .unwrap(),
            )],
            &MockResolver { resolved: None },
        );
        assert_eq!(
            got,
            Err(OrEvalError::Coercion(CoercionError::WorksheetError(
                WorksheetErrorCode::NA
            )))
        );
    }

    /// `=OR(TRUE,FALSE,0)` is still TRUE: continuing the scan past the deciding value changes
    /// nothing when no later argument is an error.
    #[test]
    fn eval_or_still_returns_true_when_later_arguments_are_not_errors() {
        let got = eval_or_surface(
            &[
                CalcValue::logical(true),
                CalcValue::logical(false),
                CalcValue::number(0.0),
            ],
            &MockResolver { resolved: None },
        );
        assert_eq!(got, Ok(CalcValue::logical(true)));
    }
}
