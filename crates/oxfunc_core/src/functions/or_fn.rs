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
    // the result the scan continues, looking only for error VALUES in the remaining items. A
    // direct text item is a value only when it spells `TRUE`/`FALSE` and is otherwise ignored
    // (`and_argument_truth`, bead `oxf-xvt5.15`: `=OR(FALSE,"TRUE")` -> `TRUE`, `=OR(FALSE,"x")`
    // -> `FALSE`, `=OR("x")` -> `#VALUE!` by the no-value rule below), so nothing text-shaped
    // can surface from the post-decision scan either.
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

    fn direct_text(text: &str) -> CalcValue {
        CalcValue::text(ExcelText::from_utf16_code_units(
            text.encode_utf16().collect(),
        ))
    }

    /// `=OR("x")` -> `#VALUE!` (live Excel 16.0 build 20326, `oxf-xvt5.15`). The worksheet value
    /// is unchanged from before that bead; the ROUTE is not: a direct text other than a
    /// `TRUE`/`FALSE` spelling is now IGNORED (`=OR(FALSE,"x")` -> `FALSE` on the same build) and
    /// the `#VALUE!` comes from the no-value rule, no longer from an `Err(NonNumericText)`
    /// coercion failure. Re-pinned to the observed route, a correction rather than a weakening.
    #[test]
    fn eval_or_direct_text_is_value_error() {
        let got = eval_or_surface(&[direct_text("x")], &MockResolver { resolved: None });
        assert_eq!(got, Ok(CalcValue::error(WorksheetErrorCode::Value)));
    }

    /// `=OR("TRUE")` -> `TRUE`, `=OR("FALSE")` -> `FALSE`, `=OR(FALSE,"TRUE")` / `"true"` /
    /// `"TrUe"` / `"tRUE"` -> `TRUE`, `=OR("FALSE","FALSE")` -> `FALSE`, `=OR("TRUE",0)` ->
    /// `TRUE`: a direct text spelling `TRUE`/`FALSE` coerces, ASCII-case-insensitively, and
    /// counts as a seen value.
    #[test]
    fn eval_or_coerces_direct_logical_spellings() {
        let resolver = MockResolver { resolved: None };
        for (args, expected) in [
            (vec![direct_text("TRUE")], true),
            (vec![direct_text("FALSE")], false),
            (vec![CalcValue::logical(false), direct_text("TRUE")], true),
            (vec![CalcValue::logical(false), direct_text("true")], true),
            (vec![CalcValue::logical(false), direct_text("TrUe")], true),
            (vec![CalcValue::logical(false), direct_text("tRUE")], true),
            (vec![direct_text("FALSE"), direct_text("FALSE")], false),
            (vec![direct_text("TRUE"), CalcValue::number(0.0)], true),
            (vec![direct_text("x"), direct_text("FALSE")], false),
        ] {
            let got = eval_or_surface(&args, &resolver);
            assert_eq!(got, Ok(CalcValue::logical(expected)), "{args:?}");
        }
    }

    /// `=OR(FALSE,"x")` / `"1"` / `"2"` / `"1.5"` / `" 1 "` / `"1e0"` / `"$1"` / `"1/2"` /
    /// `"12/31/2020"` / `""` / `" TRUE"` / `"TRUE "` / `" TRUE "` / `CHAR(9)&"TRUE"` /
    /// `"TRUE"&CHAR(10)` / `CHAR(160)&"TRUE"` / `"TRUE."` / `"Yes"` -> `FALSE`, `=OR(0,"x")` ->
    /// `FALSE`, `=OR("x",TRUE)` -> `TRUE`: any other direct text is ignored, never `#VALUE!`, and
    /// whitespace around a spelling is not trimmed.
    #[test]
    fn eval_or_ignores_direct_text_that_is_not_a_logical_spelling() {
        let resolver = MockResolver { resolved: None };
        for ignored in [
            "x",
            "1",
            "2",
            "1.5",
            " 1 ",
            "1e0",
            "$1",
            "1/2",
            "12/31/2020",
            "",
            " TRUE",
            "TRUE ",
            " TRUE ",
            "\tTRUE",
            "TRUE\n",
            "\u{a0}TRUE",
            "TRUE.",
            "Yes",
        ] {
            let got = eval_or_surface(
                &[CalcValue::logical(false), direct_text(ignored)],
                &resolver,
            );
            assert_eq!(got, Ok(CalcValue::logical(false)), "{ignored:?}");
        }
        let got = eval_or_surface(&[CalcValue::number(0.0), direct_text("x")], &resolver);
        assert_eq!(got, Ok(CalcValue::logical(false)));
        let got = eval_or_surface(&[direct_text("x"), CalcValue::logical(true)], &resolver);
        assert_eq!(got, Ok(CalcValue::logical(true)));
    }

    /// `=OR("x","y")`, `=OR("1")`, `=OR("0")`, `=OR("")`, `=OR(" TRUE")`, `=OR("x","1")` ->
    /// `#VALUE!`: ignored direct text alone leaves nothing seen.
    #[test]
    fn eval_or_returns_value_when_only_ignored_direct_text_is_given() {
        let resolver = MockResolver { resolved: None };
        for args in [
            vec![direct_text("x"), direct_text("y")],
            vec![direct_text("1")],
            vec![direct_text("0")],
            vec![direct_text("")],
            vec![direct_text(" TRUE")],
            vec![direct_text("x"), direct_text("1")],
        ] {
            let got = eval_or_surface(&args, &resolver);
            assert_eq!(
                got,
                Ok(CalcValue::error(WorksheetErrorCode::Value)),
                "{args:?}"
            );
        }
    }

    /// `=OR({"TRUE"})` -> `#VALUE!`, `=OR(FALSE,{"TRUE"})` -> `FALSE`, `=OR(E1)` with the TEXT
    /// `TRUE` in `E1` -> `#VALUE!`, `=OR(FALSE,E1)` -> `FALSE`: the spelling coerces only as a
    /// DIRECT text; inside an array constant or a cell it is reference-like text and stays
    /// ignored.
    #[test]
    fn eval_or_still_ignores_a_logical_spelling_inside_an_array_or_reference() {
        let resolver = MockResolver { resolved: None };
        let array_true =
            || CalcValue::array(CalcArray::from_rows(vec![vec![direct_text("TRUE")]]).unwrap());
        assert_eq!(
            eval_or_surface(&[array_true()], &resolver),
            Ok(CalcValue::error(WorksheetErrorCode::Value))
        );
        assert_eq!(
            eval_or_surface(&[CalcValue::logical(false), array_true()], &resolver),
            Ok(CalcValue::logical(false))
        );

        let resolver = MockResolver {
            resolved: Some(direct_text("TRUE")),
        };
        let e1 = || CalcValue::reference(ReferenceLike::new(ReferenceKind::A1, "E1".to_string()));
        assert_eq!(
            eval_or_surface(&[e1()], &resolver),
            Ok(CalcValue::error(WorksheetErrorCode::Value))
        );
        assert_eq!(
            eval_or_surface(&[CalcValue::logical(false), e1()], &resolver),
            Ok(CalcValue::logical(false))
        );
    }

    /// `=OR("x",1/0)` -> `#DIV/0!`, `=OR("TRUE",1/0)` -> `#DIV/0!`, `=OR("x",TRUE,1/0)` ->
    /// `#DIV/0!`: neither an ignored nor a coerced direct text masks an error.
    #[test]
    fn eval_or_direct_text_never_masks_an_error() {
        let resolver = MockResolver { resolved: None };
        for args in [
            vec![direct_text("x"), CalcValue::error(WorksheetErrorCode::Div0)],
            vec![direct_text("TRUE"), CalcValue::error(WorksheetErrorCode::Div0)],
            vec![
                direct_text("x"),
                CalcValue::logical(true),
                CalcValue::error(WorksheetErrorCode::Div0),
            ],
        ] {
            let got = eval_or_surface(&args, &resolver);
            assert_eq!(
                got,
                Err(OrEvalError::Coercion(CoercionError::WorksheetError(
                    WorksheetErrorCode::Div0
                ))),
                "{args:?}"
            );
        }
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
