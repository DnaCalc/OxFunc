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

pub const AND_META: FunctionMeta = function_spec! {
    function_id: "FUNC.AND",
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

    // Excel evaluates every argument of AND; it does not short-circuit on the first FALSE. An
    // error value anywhere among the arguments surfaces, and when several arguments are errors
    // the FIRST one in argument order wins — a positional rule, not an error-code ranking
    // (live Excel 16.0 build 20326, COM probe 2026-09-15, bead `oxf-xvt5.14`: `=AND(FALSE,1/0)`
    // -> `#DIV/0!`, `=AND(1/0,NA())` -> `#DIV/0!`, `=AND(NA(),1/0)` -> `#N/A`,
    // `=AND(FALSE,NA(),1/0)` -> `#N/A`, `=AND({FALSE,#N/A,#DIV/0!})` -> `#N/A`, and the same
    // for `#NUM!`/`#VALUE!`/`#REF!` pairs in both orders). So once an item has decided the
    // result the scan continues, looking only for error VALUES in the remaining items. A direct
    // text item is a value only when it spells `TRUE`/`FALSE` and is otherwise ignored
    // (`and_argument_truth`, bead `oxf-xvt5.15`: `=AND(TRUE,"FALSE")` -> `FALSE`,
    // `=AND(TRUE,"x")` -> `TRUE`, `=AND("x")` -> `#VALUE!` by the no-value rule below), so
    // nothing text-shaped can surface from the post-decision scan either.
    let mut saw_value = false;
    let mut decided = false;
    for arg in args {
        for item in expand_aggregate_arg(arg, resolver).map_err(AndEvalError::Coercion)? {
            if decided {
                if let CoreValue::Error(code) = item.0.core() {
                    return Err(AndEvalError::Coercion(CoercionError::WorksheetError(*code)));
                }
                continue;
            }
            match and_argument_truth(&item).map_err(AndEvalError::Coercion)? {
                Some(false) => decided = true,
                Some(true) => saw_value = true,
                None => {}
            }
        }
    }

    if decided {
        return Ok(CalcValue::logical(false));
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

    fn direct_text(text: &str) -> CalcValue {
        CalcValue::text(ExcelText::from_utf16_code_units(
            text.encode_utf16().collect(),
        ))
    }

    /// `=AND("1")` -> `#VALUE!` (live Excel 16.0 build 20326, `oxf-xvt5.15`). The worksheet value
    /// is unchanged from before that bead; the ROUTE is not: numeric direct text is now IGNORED
    /// (`=AND(TRUE,"0")` -> `TRUE` on the same build) and the `#VALUE!` comes from the no-value
    /// rule, no longer from an `Err(NonNumericText)` coercion failure. Re-pinned to the observed
    /// route, a correction rather than a weakening.
    #[test]
    fn eval_and_direct_text_is_value_error() {
        let got = eval_and_surface(&[direct_text("1")], &MockResolver { resolved: None });
        assert_eq!(got, Ok(CalcValue::error(WorksheetErrorCode::Value)));
    }

    /// `=AND("TRUE")` -> `TRUE`, `=AND("FALSE")` -> `FALSE`, `=AND(TRUE,"FALSE")` -> `FALSE`,
    /// `=AND(TRUE,"fAlSe")` -> `FALSE`, `=AND("TRUE","TRUE")` -> `TRUE`: a direct text spelling
    /// `TRUE`/`FALSE` coerces, ASCII-case-insensitively, and counts as a seen value.
    #[test]
    fn eval_and_coerces_direct_logical_spellings() {
        let resolver = MockResolver { resolved: None };
        for (args, expected) in [
            (vec![direct_text("TRUE")], true),
            (vec![direct_text("FALSE")], false),
            (vec![CalcValue::logical(true), direct_text("FALSE")], false),
            (vec![CalcValue::logical(true), direct_text("fAlSe")], false),
            (vec![direct_text("TRUE"), direct_text("TRUE")], true),
            (vec![direct_text("x"), direct_text("TRUE")], true),
            (vec![direct_text("FALSE"), CalcValue::number(1.0)], false),
        ] {
            let got = eval_and_surface(&args, &resolver);
            assert_eq!(got, Ok(CalcValue::logical(expected)), "{args:?}");
        }
    }

    /// `=AND(TRUE,"x")` -> `TRUE`, `=AND(1,"x")` -> `TRUE`, `=AND(TRUE,"0")` -> `TRUE`,
    /// `=AND(TRUE,"")` -> `TRUE`, `=AND(TRUE," FALSE ")` -> `TRUE` (whitespace is not trimmed),
    /// `=AND("x",FALSE)` -> `FALSE`: any other direct text is ignored, never `#VALUE!`.
    #[test]
    fn eval_and_ignores_direct_text_that_is_not_a_logical_spelling() {
        let resolver = MockResolver { resolved: None };
        for (args, expected) in [
            (vec![CalcValue::logical(true), direct_text("x")], true),
            (vec![CalcValue::number(1.0), direct_text("x")], true),
            (vec![CalcValue::logical(true), direct_text("0")], true),
            (vec![CalcValue::logical(true), direct_text("")], true),
            (vec![CalcValue::logical(true), direct_text(" FALSE ")], true),
            (vec![CalcValue::logical(true), direct_text("FALSE ")], true),
            (vec![direct_text("x"), CalcValue::logical(false)], false),
        ] {
            let got = eval_and_surface(&args, &resolver);
            assert_eq!(got, Ok(CalcValue::logical(expected)), "{args:?}");
        }
    }

    /// `=AND("x")`, `=AND("x","y")`, `=AND("0")`, `=AND(" FALSE ")` -> `#VALUE!`: ignored direct
    /// text alone leaves nothing seen, and the no-value rule publishes `#VALUE!`.
    #[test]
    fn eval_and_returns_value_when_only_ignored_direct_text_is_given() {
        let resolver = MockResolver { resolved: None };
        for args in [
            vec![direct_text("x")],
            vec![direct_text("x"), direct_text("y")],
            vec![direct_text("0")],
            vec![direct_text(" FALSE ")],
        ] {
            let got = eval_and_surface(&args, &resolver);
            assert_eq!(
                got,
                Ok(CalcValue::error(WorksheetErrorCode::Value)),
                "{args:?}"
            );
        }
    }

    /// `=AND(TRUE,{"FALSE"})` -> `TRUE` and `=AND(TRUE,E2)` with the TEXT `FALSE` in `E2` ->
    /// `TRUE`: the spelling coerces only as a DIRECT text; inside an array constant or a cell it
    /// is reference-like text and stays ignored.
    #[test]
    fn eval_and_still_ignores_a_logical_spelling_inside_an_array_or_reference() {
        let got = eval_and_surface(
            &[
                CalcValue::logical(true),
                CalcValue::array(CalcArray::from_rows(vec![vec![direct_text("FALSE")]]).unwrap()),
            ],
            &MockResolver { resolved: None },
        );
        assert_eq!(got, Ok(CalcValue::logical(true)));

        let got = eval_and_surface(
            &[
                CalcValue::logical(true),
                CalcValue::reference(ReferenceLike::new(ReferenceKind::A1, "E2".to_string())),
            ],
            &MockResolver {
                resolved: Some(direct_text("FALSE")),
            },
        );
        assert_eq!(got, Ok(CalcValue::logical(true)));
    }

    /// `=AND("x",NA())` -> `#N/A`, `=AND("FALSE",NA())` -> `#N/A`: neither an ignored nor a
    /// coerced direct text masks an error.
    #[test]
    fn eval_and_direct_text_never_masks_an_error() {
        let resolver = MockResolver { resolved: None };
        for first in [direct_text("x"), direct_text("FALSE")] {
            let got = eval_and_surface(
                &[first.clone(), CalcValue::error(WorksheetErrorCode::NA)],
                &resolver,
            );
            assert_eq!(
                got,
                Err(AndEvalError::Coercion(CoercionError::WorksheetError(
                    WorksheetErrorCode::NA
                ))),
                "{first:?}"
            );
        }
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

    /// `=AND(FALSE,1/0)` -> `#DIV/0!` (live Excel 16.0 build 20326, `oxf-xvt5.14`): AND evaluates
    /// every argument, so an error AFTER the deciding FALSE still surfaces.
    #[test]
    fn eval_and_surfaces_an_error_after_the_deciding_false() {
        let got = eval_and_surface(
            &[
                CalcValue::logical(false),
                CalcValue::error(WorksheetErrorCode::Div0),
            ],
            &MockResolver { resolved: None },
        );
        assert_eq!(
            got,
            Err(AndEvalError::Coercion(CoercionError::WorksheetError(
                WorksheetErrorCode::Div0
            )))
        );
    }

    /// `=AND(FALSE,NA(),1/0)` -> `#N/A` and `=AND(FALSE,1/0,NA())` -> `#DIV/0!`: with several error
    /// arguments the first in argument order wins, even after the deciding value.
    #[test]
    fn eval_and_first_error_in_argument_order_wins_after_the_deciding_false() {
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
            let got = eval_and_surface(
                &[
                    CalcValue::logical(false),
                    CalcValue::error(second),
                    CalcValue::error(third),
                ],
                &MockResolver { resolved: None },
            );
            assert_eq!(
                got,
                Err(AndEvalError::Coercion(CoercionError::WorksheetError(
                    expected
                )))
            );
        }
    }

    /// `=AND({FALSE,#N/A,#DIV/0!})` -> `#N/A`: the same in-order rule inside a single array
    /// argument.
    #[test]
    fn eval_and_scans_an_array_argument_past_the_deciding_false_for_errors() {
        let got = eval_and_surface(
            &[CalcValue::array(
                CalcArray::from_rows(vec![vec![
                    CalcValue::logical(false),
                    CalcValue::error(WorksheetErrorCode::NA),
                    CalcValue::error(WorksheetErrorCode::Div0),
                ]])
                .unwrap(),
            )],
            &MockResolver { resolved: None },
        );
        assert_eq!(
            got,
            Err(AndEvalError::Coercion(CoercionError::WorksheetError(
                WorksheetErrorCode::NA
            )))
        );
    }

    /// `=AND(FALSE,TRUE,1)` is still FALSE: continuing the scan past the deciding value changes
    /// nothing when no later argument is an error.
    #[test]
    fn eval_and_still_returns_false_when_later_arguments_are_not_errors() {
        let got = eval_and_surface(
            &[
                CalcValue::logical(false),
                CalcValue::logical(true),
                CalcValue::number(1.0),
            ],
            &MockResolver { resolved: None },
        );
        assert_eq!(got, Ok(CalcValue::logical(false)));
    }
}
