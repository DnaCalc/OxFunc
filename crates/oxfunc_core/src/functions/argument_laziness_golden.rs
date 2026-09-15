//! W110-4 (oxf-xvt5.13, OxFml `HANDOFF-OXFUNC-007`) — golden assertions that the declared
//! `argument_laziness_profile` axis on each catalog `FunctionMeta` matches the documented Excel
//! behaviour of that function, so the set of lazy functions (and each one's shape) is declared
//! ONCE here in OxFunc and an evaluator can retire its name-keyed list.
//!
//! What the axis claims, and the provenance of each claim (clean-room rule: public documentation
//! plus reproducible black-box Excel observation; each page below was fetched and its wording
//! checked on 2026-09-15):
//!
//! * `IF` — `BranchOnCondition`. Syntax `IF(logical_test, value_if_true, [value_if_false])`;
//!   the page describes the result as one value if the test is TRUE and another if FALSE
//!   (<https://support.microsoft.com/en-us/office/if-function-69aed7c9-4e8a-4755-a9bc-aa8bbff73be2>).
//!   That an error in the untaken branch does not surface is Microsoft's own `#DIV/0!`
//!   remedy, `=IF(A3,A2/A3,0)` ("How to correct a #DIV/0! error",
//!   <https://support.microsoft.com/en-us/office/how-to-correct-a-div-0-error-3a5a18a9-8d80-4ebb-a908-39e759a009a5>).
//! * `IFS` — `ConditionValuePairs`. "The IFS function checks whether one or more conditions are
//!   met, and returns a value that corresponds to the first TRUE condition."
//!   (<https://support.microsoft.com/en-us/office/ifs-function-36329a26-37b2-467c-972b-4a39bd951d45>).
//!   Retained native-Excel replay (evidence id `W68-LOOKUP-LOGICAL-RESIDUALS-BL-20260401`,
//!   `.tmp/w68-lookup-logical-results.csv`, Excel 16.0.20026): `W68-IFS-004`
//!   `=IFS(TRUE,1,TRUE,1/0)` -> `1` (a later pair is not forced) and `W68-IFS-005`
//!   `=IFS(1/0,1,TRUE,2)` -> `#DIV/0!` (conditions are evaluated in order until one is TRUE).
//! * `CHOOSE` — `IndexedChoice`. "Uses index_num to return a value from the list of value
//!   arguments." / "If index_num is 1, CHOOSE returns value1; if it is 2, CHOOSE returns
//!   value2; and so on."
//!   (<https://support.microsoft.com/en-us/office/choose-function-fc5c184f-cb62-4ec7-a46e-38653b98f5bc>).
//! * `SWITCH` — `MatchedCase`. "The SWITCH function evaluates one value (called the
//!   expression) against a list of values, and returns the result corresponding to the first
//!   matching value." The trailing default "is identified by having no corresponding resultN
//!   expression" and "must be the final argument" — a per-CALL arity fact, which is why the
//!   variant carries no `has_trailing_default` payload
//!   (<https://support.microsoft.com/en-us/excel/functions/switch-function>). Retained
//!   native-Excel replay (`.tmp/w24-batch01-switch-results.csv`): `W24SW-008`
//!   `=SWITCH(2,1,1/0,2,3,4)` -> `3` (an unselected result's error does not surface) and
//!   `W24SW-007` `=SWITCH(1,1,1/0,2,3,4)` -> `#DIV/0!` (the selected result's does).
//! * `IFERROR` — `FallbackOnError`. "IFERROR returns a value you specify if a formula evaluates
//!   to an error; otherwise, it returns the result of the formula." Syntax
//!   `IFERROR(value, value_if_error)`
//!   (<https://support.microsoft.com/en-us/office/iferror-function-c526fd07-caeb-47b8-8bb6-63f3e417f611>).
//!   Retained native-Excel baseline (`.tmp/w12-results-excel.csv`, Excel 16.0 build 19725):
//!   `W12S2-001` `=IFERROR(1/0,5)` -> `5`.
//! * `IFNA` — `FallbackOnError`. "The IFNA function returns the value you specify if a formula
//!   returns the #N/A error value; otherwise it returns the result of the formula." Syntax
//!   `IFNA(value, value_if_na)`
//!   (<https://support.microsoft.com/en-us/office/ifna-function-6626c961-a569-42fc-a49d-79b4951fd461>).
//!   Which errors are caught (all seven for `IFERROR`, only `#N/A` for `IFNA`) is the function's
//!   own semantics; the axis only says that argument 1 is held back until argument 0 is known.
//! * `AND` / `OR` / `XOR` — `Eager` (the default). "The AND function returns TRUE if all its
//!   arguments evaluate to TRUE, and returns FALSE if one or more arguments evaluate to FALSE."
//!   (<https://support.microsoft.com/en-us/office/and-function-5f19b2e8-e1df-4408-897a-ce285a19e9d9>);
//!   "The OR function returns TRUE if any of its arguments evaluate to TRUE, and returns FALSE if
//!   all of its arguments evaluate to FALSE."
//!   (<https://support.microsoft.com/en-us/office/or-function-7d17ad14-8700-4281-b308-00b131e22af0>);
//!   "The XOR function returns a logical Exclusive Or of all arguments."
//!   (<https://support.microsoft.com/en-us/office/xor-function-1548d4c2-5e47-4f77-9a92-0533bba14f37>).
//!   Every argument participates; Excel does not short-circuit these, so an error in a LATER
//!   argument surfaces even when an earlier argument already decides the result. Live Excel
//!   16.0 build 20326, COM probe 2026-09-15 (retained locally as
//!   `.tmp/w110-argument-laziness-probe-results.csv`): `=OR(TRUE,1/0)` -> `#DIV/0!`,
//!   `=OR(TRUE,NA())` -> `#N/A`, `=AND(FALSE,1/0)` -> `#DIV/0!`, `=XOR(TRUE,1/0)` -> `#DIV/0!`
//!   (and the error-first mirrors likewise). The same probe re-confirmed the six selectors'
//!   untaken-branch rows: `=IF(TRUE,1,1/0)` -> `1`, `=IFS(TRUE,1,1/0,2)` -> `1` (a later
//!   CONDITION is not evaluated), `=CHOOSE(1,"a",1/0)` -> `a`, `=SWITCH(1,1,7,1/0,3)` -> `7`
//!   (a later CANDIDATE is not evaluated), `=IFERROR(1,1/0)` -> `1`, `=IFNA(1,1/0)` -> `1`.
//!   Until bead `oxf-xvt5.14` (catalog row G1-01) the `OR` and `AND` kernels returned on the
//!   first deciding value before scanning the remaining arguments, publishing `TRUE` /
//!   `FALSE` for the rows above; both now scan every argument and the seven rows are pinned
//!   through real dispatch below. Which error wins when SEVERAL arguments are errors was
//!   settled by a second live probe on the same build (2026-09-15, retained locally as
//!   `.tmp/w110-and-or-error-precedence-probe-results.csv`, script
//!   `tools/w110-probe/run-w110-and-or-error-precedence-probe.ps1`): it is the FIRST error in
//!   argument order, not an error-code ranking — `=OR(1/0,NA())` -> `#DIV/0!` but
//!   `=OR(NA(),1/0)` -> `#N/A`, `=OR(TRUE,NA(),1/0)` -> `#N/A`, `=AND(FALSE,1/0,NA())` ->
//!   `#DIV/0!`, `=OR({TRUE,#N/A,#DIV/0!})` -> `#N/A`, `=AND(D1,D3,D2)` (FALSE, `#N/A`,
//!   `#DIV/0!` cells) -> `#N/A`, and every `#NUM!`/`#VALUE!`/`#REF!` pairing likewise in both
//!   orders (28 two-error rows, 3 three-error rows, 0 exceptions). The same probe found a
//!   SEPARATE pre-existing gap not fixed here: Excel ignores a direct text argument other
//!   than `"TRUE"`/`"FALSE"` in these folds (`=OR(FALSE,"x")` -> `FALSE`, `=OR("x",TRUE)` ->
//!   `TRUE`, `=OR(FALSE,"1")` -> `FALSE`, `=OR("x")` -> `#VALUE!` only because no logical was
//!   seen) and coerces `"TRUE"`/`"FALSE"` case-insensitively (`=OR(FALSE,"true")` -> `TRUE`),
//!   where OxFunc's `and_argument_truth` raises `#VALUE!` for any direct text — catalog row
//!   G1-02.
//! * `LET`, `LAMBDA`, `_XLFN.SINGLE` — NOT on this axis. They are formula-language forms
//!   (binding scopes, implicit intersection) owned by OxFml's evaluator, carry no `FunctionMeta`
//!   in OxFunc's catalog, and are out of scope by the handoff's own terms. The test below pins
//!   that they stay out of the catalog so no laziness declaration can silently appear for them.
//!
//! The registry-wide cross-check (every `ErrorCollapseProfile::SelectorBranch` meta is lazy and
//! every lazy meta is a `SelectorBranch`) lives in `registry.rs`
//! (`argument_laziness_axis_agrees_with_selector_branch_over_the_catalog`).

use crate::function::{ArgumentLazinessProfile, ErrorCollapseProfile};
use crate::function_call::FunctionCallTarget;
use crate::functions::surface_dispatch::{self as sd, eval_surface_value_call};
use crate::locale_format::test_current_excel_host_context;
use crate::resolver::NULL_REFERENCE_SYSTEM_PROVIDER;
use crate::value::{CalcValue, CoreValue, WorksheetErrorCode};
use crate::xll_export_specs::{function_catalog, lookup_function_meta_by_id};

/// The documented Excel evaluation shape of every function that has lazy argument semantics,
/// plus the three logical folds the bead asked to review (all eager). Provenance per row in the
/// module header.
const GOLDEN: &[(&str, ArgumentLazinessProfile)] = &[
    ("FUNC.IF", ArgumentLazinessProfile::BranchOnCondition),
    ("FUNC.IFS", ArgumentLazinessProfile::ConditionValuePairs),
    ("FUNC.CHOOSE", ArgumentLazinessProfile::IndexedChoice),
    ("FUNC.SWITCH", ArgumentLazinessProfile::MatchedCase),
    ("FUNC.IFERROR", ArgumentLazinessProfile::FallbackOnError),
    ("FUNC.IFNA", ArgumentLazinessProfile::FallbackOnError),
    ("FUNC.AND", ArgumentLazinessProfile::Eager),
    ("FUNC.OR", ArgumentLazinessProfile::Eager),
    ("FUNC.XOR", ArgumentLazinessProfile::Eager),
];

/// The six lazy functions, by catalog id — the set an evaluator would otherwise keep by name.
const LAZY_FUNCTION_IDS: [&str; 6] = [
    "FUNC.IF",
    "FUNC.IFS",
    "FUNC.CHOOSE",
    "FUNC.SWITCH",
    "FUNC.IFERROR",
    "FUNC.IFNA",
];

#[test]
fn declared_axis_matches_documented_excel_behaviour_for_each_named_function() {
    for (function_id, expected) in GOLDEN {
        let meta = lookup_function_meta_by_id(function_id)
            .unwrap_or_else(|| panic!("{function_id} must be in the catalog"));
        assert_eq!(
            meta.argument_laziness_profile, *expected,
            "{function_id}: declared argument_laziness_profile must match the documented Excel shape"
        );
    }
}

#[test]
fn the_lazy_set_is_exactly_the_six_branch_selectors() {
    let declared_lazy: Vec<&str> = function_catalog()
        .iter()
        .filter(|meta| meta.argument_laziness_profile.is_lazy())
        .map(|meta| meta.function_id)
        .collect();
    let mut expected: Vec<&str> = LAZY_FUNCTION_IDS.to_vec();
    expected.sort_unstable();
    let mut actual = declared_lazy.clone();
    actual.sort_unstable();
    assert_eq!(
        actual, expected,
        "the catalog's lazy set must be exactly IF/IFS/CHOOSE/SWITCH/IFERROR/IFNA"
    );
    // The two selector-shaped axes agree on membership (the handoff's cross-check).
    for meta in function_catalog() {
        assert_eq!(
            meta.argument_laziness_profile.is_lazy(),
            matches!(
                meta.error_collapse_profile,
                ErrorCollapseProfile::SelectorBranch
            ),
            "{}: SelectorBranch and a non-Eager laziness profile must coincide",
            meta.function_id
        );
    }
}

#[test]
fn every_lazy_shape_discriminates_on_argument_zero_and_eager_has_no_discriminator() {
    for meta in function_catalog() {
        let profile = meta.argument_laziness_profile;
        assert_eq!(
            profile.discriminator_index(),
            profile.is_lazy().then_some(0),
            "{}: every Excel lazy shape decides on argument 0",
            meta.function_id
        );
    }
}

/// The dispatch-target query OxFml will make instead of keying on the surface name: resolve
/// the call target by surface name (as `CompiledFunctionCallTarget::from_surface_name` does)
/// and read the declared axis off it. The same fact is reachable through the registry entry's
/// projected `function_spec_axes_metadata.argument_laziness_profile` key.
#[test]
fn dispatch_target_exposes_the_declared_axis_by_surface_name() {
    for (surface_name, expected) in [
        ("IF", ArgumentLazinessProfile::BranchOnCondition),
        ("IFS", ArgumentLazinessProfile::ConditionValuePairs),
        ("CHOOSE", ArgumentLazinessProfile::IndexedChoice),
        ("SWITCH", ArgumentLazinessProfile::MatchedCase),
        ("IFERROR", ArgumentLazinessProfile::FallbackOnError),
        ("IFNA", ArgumentLazinessProfile::FallbackOnError),
        ("AND", ArgumentLazinessProfile::Eager),
        ("OR", ArgumentLazinessProfile::Eager),
        ("XOR", ArgumentLazinessProfile::Eager),
        ("SUM", ArgumentLazinessProfile::Eager),
    ] {
        let target = FunctionCallTarget::from_surface_name(surface_name)
            .unwrap_or_else(|err| panic!("{surface_name} must resolve to a call target: {err:?}"));
        assert_eq!(
            target.argument_laziness_profile(),
            expected,
            "{surface_name}: FunctionCallTarget::argument_laziness_profile must expose the declared axis"
        );
        assert_eq!(
            target.function_meta().argument_laziness_profile,
            expected,
            "{surface_name}: the accessor must read the same field the meta carries"
        );
    }
}

/// `LET` / `LAMBDA` / `_XLFN.SINGLE` are OxFml special forms, not catalog functions: the
/// catalog carries no meta for them, so no laziness declaration exists — or can appear
/// unnoticed — for a language form.
#[test]
fn language_forms_are_not_catalog_functions_and_carry_no_laziness_declaration() {
    for function_id in [
        "FUNC.LET",
        "FUNC.LAMBDA",
        "FUNC.SINGLE",
        "FUNC._XLFN.SINGLE",
    ] {
        assert!(
            lookup_function_meta_by_id(function_id).is_none(),
            "{function_id} is an OxFml language form and must not carry a catalog FunctionMeta"
        );
    }
    for surface_name in ["LET", "LAMBDA", "_XLFN.SINGLE", "SINGLE"] {
        assert!(
            FunctionCallTarget::from_surface_name(surface_name).is_err(),
            "{surface_name} must not resolve to an OxFunc dispatch target"
        );
    }
}

fn eval(function_id: &str, args: &[CalcValue]) -> CalcValue {
    let locale = test_current_excel_host_context();
    match eval_surface_value_call(
        function_id,
        args,
        &NULL_REFERENCE_SYSTEM_PROVIDER,
        None,
        None,
        Some(&locale),
        None,
    ) {
        Ok(value) => value,
        Err(code) => CalcValue::error(code),
    }
}

/// The executable consequence of `Eager` for a logical fold: an error in a LATER argument
/// surfaces even though the FIRST argument already decides the result (`XOR(TRUE, …)` would be
/// TRUE if the rest were FALSE). A short-circuiting evaluator would publish a logical here;
/// Excel (16.0 build 20326, probed 2026-09-15) and OxFunc publish the error. `XOR` always
/// folded every argument; `OR` and `AND` joined it in `oxf-xvt5.14` — all seven live rows are
/// pinned in `logical_folds_surface_a_later_arguments_error_because_they_are_eager`.
#[test]
fn xor_surfaces_an_error_in_a_later_argument_because_it_is_eager() {
    let got = eval(
        sd::FUNC_ID_XOR,
        &[
            CalcValue::logical(true),
            CalcValue::error(WorksheetErrorCode::Div0),
        ],
    );
    assert_eq!(
        got.core(),
        &CoreValue::Error(WorksheetErrorCode::Div0),
        "=XOR(TRUE,1/0): an eager logical fold must surface the later argument's error"
    );
    assert_eq!(
        lookup_function_meta_by_id(sd::FUNC_ID_XOR)
            .expect("catalog meta")
            .argument_laziness_profile,
        ArgumentLazinessProfile::Eager,
        "the behaviour above is what the declared Eager axis names"
    );
}

/// An error in the FIRST argument surfaces for all three logical folds (Excel 16.0 build 20326:
/// `=OR(1/0,TRUE)`, `=AND(1/0,FALSE)`, `=XOR(1/0,TRUE)` all `#DIV/0!`). This half of the eager
/// rule was right before `oxf-xvt5.14`; the later-argument half is pinned just below.
#[test]
fn logical_folds_surface_an_error_in_the_first_argument() {
    let div0 = CalcValue::error(WorksheetErrorCode::Div0);
    for (formula, function_id, second) in [
        ("=OR(1/0,TRUE)", sd::FUNC_ID_OR, CalcValue::logical(true)),
        (
            "=AND(1/0,FALSE)",
            sd::FUNC_ID_AND,
            CalcValue::logical(false),
        ),
        ("=XOR(1/0,TRUE)", sd::FUNC_ID_XOR, CalcValue::logical(true)),
    ] {
        let got = eval(function_id, &[div0.clone(), second]);
        assert_eq!(
            got.core(),
            &CoreValue::Error(WorksheetErrorCode::Div0),
            "{formula}: the first argument's error must surface, got {:?}",
            got.core()
        );
    }
}

/// The seven live rows recorded on `oxf-xvt5.13` and re-confirmed on `oxf-xvt5.14` (Excel 16.0
/// build 20326, COM probe 2026-09-15), through real dispatch with the error passed as a
/// prepared argument value — the shape OxFml hands OxFunc after evaluating `1/0` / `NA()`.
/// `AND`/`OR` published `TRUE`/`FALSE` for rows 1, 3 and 4 before the fix (catalog G1-01).
#[test]
fn logical_folds_surface_a_later_arguments_error_because_they_are_eager() {
    let div0 = || CalcValue::error(WorksheetErrorCode::Div0);
    let na = || CalcValue::error(WorksheetErrorCode::NA);
    let rows: [(&str, &str, Vec<CalcValue>, WorksheetErrorCode); 7] = [
        (
            "=OR(TRUE,1/0)",
            sd::FUNC_ID_OR,
            vec![CalcValue::logical(true), div0()],
            WorksheetErrorCode::Div0,
        ),
        (
            "=OR(1/0,TRUE)",
            sd::FUNC_ID_OR,
            vec![div0(), CalcValue::logical(true)],
            WorksheetErrorCode::Div0,
        ),
        (
            "=OR(TRUE,NA())",
            sd::FUNC_ID_OR,
            vec![CalcValue::logical(true), na()],
            WorksheetErrorCode::NA,
        ),
        (
            "=AND(FALSE,1/0)",
            sd::FUNC_ID_AND,
            vec![CalcValue::logical(false), div0()],
            WorksheetErrorCode::Div0,
        ),
        (
            "=AND(1/0,FALSE)",
            sd::FUNC_ID_AND,
            vec![div0(), CalcValue::logical(false)],
            WorksheetErrorCode::Div0,
        ),
        (
            "=XOR(TRUE,1/0)",
            sd::FUNC_ID_XOR,
            vec![CalcValue::logical(true), div0()],
            WorksheetErrorCode::Div0,
        ),
        (
            "=XOR(1/0,TRUE)",
            sd::FUNC_ID_XOR,
            vec![div0(), CalcValue::logical(true)],
            WorksheetErrorCode::Div0,
        ),
    ];
    for (formula, function_id, args, expected) in rows {
        let got = eval(function_id, &args);
        assert_eq!(
            got.core(),
            &CoreValue::Error(expected),
            "{formula}: Excel evaluates every argument, so this error must surface; got {:?}",
            got.core()
        );
        assert_eq!(
            lookup_function_meta_by_id(function_id)
                .expect("catalog meta")
                .argument_laziness_profile,
            ArgumentLazinessProfile::Eager,
            "{formula}: the behaviour is what the declared Eager axis names"
        );
    }
}

/// When SEVERAL arguments are errors Excel publishes the FIRST one in argument order — a
/// positional rule, not an error-code ranking (Excel 16.0 build 20326, COM probe 2026-09-15,
/// `.tmp/w110-and-or-error-precedence-probe-results.csv`): every pairing of `#DIV/0!`, `#N/A`,
/// `#NUM!`, `#VALUE!` and `#REF!` flipped with its order, with or without a deciding value in
/// front, and inside an array constant or a cell range as much as across direct arguments.
/// The rows below are the probed formulas with the error-producing sub-expressions replaced by
/// their error values; the reference rows (`=OR(C1,C3,C2)` etc.) are pinned through their
/// array-constant twins, which take the same in-order aggregate path.
#[test]
fn logical_folds_publish_the_first_error_in_argument_order() {
    use crate::value::CalcArray;
    use WorksheetErrorCode::{Div0, NA, Num, Ref, Value};
    let err = CalcValue::error;
    let arr = |cells: Vec<CalcValue>| {
        CalcValue::array(CalcArray::from_rows(vec![cells]).expect("one-row array constant"))
    };
    let rows: Vec<(&str, &str, Vec<CalcValue>, WorksheetErrorCode)> = vec![
        // two different errors, no deciding value, both orders
        (
            "=OR(1/0,NA())",
            sd::FUNC_ID_OR,
            vec![err(Div0), err(NA)],
            Div0,
        ),
        (
            "=OR(NA(),1/0)",
            sd::FUNC_ID_OR,
            vec![err(NA), err(Div0)],
            NA,
        ),
        (
            "=AND(1/0,NA())",
            sd::FUNC_ID_AND,
            vec![err(Div0), err(NA)],
            Div0,
        ),
        (
            "=AND(NA(),1/0)",
            sd::FUNC_ID_AND,
            vec![err(NA), err(Div0)],
            NA,
        ),
        (
            "=XOR(1/0,NA())",
            sd::FUNC_ID_XOR,
            vec![err(Div0), err(NA)],
            Div0,
        ),
        (
            "=XOR(NA(),1/0)",
            sd::FUNC_ID_XOR,
            vec![err(NA), err(Div0)],
            NA,
        ),
        // two different errors after the deciding value
        (
            "=OR(TRUE,1/0,NA())",
            sd::FUNC_ID_OR,
            vec![CalcValue::logical(true), err(Div0), err(NA)],
            Div0,
        ),
        (
            "=OR(TRUE,NA(),1/0)",
            sd::FUNC_ID_OR,
            vec![CalcValue::logical(true), err(NA), err(Div0)],
            NA,
        ),
        (
            "=AND(FALSE,1/0,NA())",
            sd::FUNC_ID_AND,
            vec![CalcValue::logical(false), err(Div0), err(NA)],
            Div0,
        ),
        (
            "=AND(FALSE,NA(),1/0)",
            sd::FUNC_ID_AND,
            vec![CalcValue::logical(false), err(NA), err(Div0)],
            NA,
        ),
        (
            "=XOR(TRUE,1/0,NA())",
            sd::FUNC_ID_XOR,
            vec![CalcValue::logical(true), err(Div0), err(NA)],
            Div0,
        ),
        (
            "=XOR(TRUE,NA(),1/0)",
            sd::FUNC_ID_XOR,
            vec![CalcValue::logical(true), err(NA), err(Div0)],
            NA,
        ),
        // other error-code pairs: position wins over any code ranking
        (
            "=OR(SQRT(-1),1/0)",
            sd::FUNC_ID_OR,
            vec![err(Num), err(Div0)],
            Num,
        ),
        (
            "=OR(1/0,SQRT(-1))",
            sd::FUNC_ID_OR,
            vec![err(Div0), err(Num)],
            Div0,
        ),
        (
            "=OR(\"x\"+1,1/0)",
            sd::FUNC_ID_OR,
            vec![err(Value), err(Div0)],
            Value,
        ),
        (
            "=OR(1/0,\"x\"+1)",
            sd::FUNC_ID_OR,
            vec![err(Div0), err(Value)],
            Div0,
        ),
        (
            "=OR(NA(),SQRT(-1))",
            sd::FUNC_ID_OR,
            vec![err(NA), err(Num)],
            NA,
        ),
        (
            "=OR(SQRT(-1),NA())",
            sd::FUNC_ID_OR,
            vec![err(Num), err(NA)],
            Num,
        ),
        (
            "=OR(OFFSET(A1,-1,0),1/0)",
            sd::FUNC_ID_OR,
            vec![err(Ref), err(Div0)],
            Ref,
        ),
        (
            "=OR(1/0,OFFSET(A1,-1,0))",
            sd::FUNC_ID_OR,
            vec![err(Div0), err(Ref)],
            Div0,
        ),
        (
            "=OR(NA(),\"x\"+1)",
            sd::FUNC_ID_OR,
            vec![err(NA), err(Value)],
            NA,
        ),
        (
            "=OR(\"x\"+1,NA())",
            sd::FUNC_ID_OR,
            vec![err(Value), err(NA)],
            Value,
        ),
        (
            "=AND(SQRT(-1),1/0)",
            sd::FUNC_ID_AND,
            vec![err(Num), err(Div0)],
            Num,
        ),
        (
            "=AND(1/0,SQRT(-1))",
            sd::FUNC_ID_AND,
            vec![err(Div0), err(Num)],
            Div0,
        ),
        (
            "=AND(\"x\"+1,NA())",
            sd::FUNC_ID_AND,
            vec![err(Value), err(NA)],
            Value,
        ),
        (
            "=AND(NA(),\"x\"+1)",
            sd::FUNC_ID_AND,
            vec![err(NA), err(Value)],
            NA,
        ),
        (
            "=AND(OFFSET(A1,-1,0),NA())",
            sd::FUNC_ID_AND,
            vec![err(Ref), err(NA)],
            Ref,
        ),
        (
            "=AND(NA(),OFFSET(A1,-1,0))",
            sd::FUNC_ID_AND,
            vec![err(NA), err(Ref)],
            NA,
        ),
        // three errors: strictly the first
        (
            "=OR(SQRT(-1),NA(),1/0)",
            sd::FUNC_ID_OR,
            vec![err(Num), err(NA), err(Div0)],
            Num,
        ),
        (
            "=OR(NA(),1/0,SQRT(-1))",
            sd::FUNC_ID_OR,
            vec![err(NA), err(Div0), err(Num)],
            NA,
        ),
        (
            "=AND(1/0,SQRT(-1),NA())",
            sd::FUNC_ID_AND,
            vec![err(Div0), err(Num), err(NA)],
            Div0,
        ),
        // errors inside one array argument (also the shape of `=OR(C1:C3)` / `=AND(D1:D3)`)
        (
            "=OR({TRUE,#DIV/0!})",
            sd::FUNC_ID_OR,
            vec![arr(vec![CalcValue::logical(true), err(Div0)])],
            Div0,
        ),
        (
            "=OR({TRUE,#DIV/0!,#N/A})",
            sd::FUNC_ID_OR,
            vec![arr(vec![CalcValue::logical(true), err(Div0), err(NA)])],
            Div0,
        ),
        (
            "=OR({TRUE,#N/A,#DIV/0!})",
            sd::FUNC_ID_OR,
            vec![arr(vec![CalcValue::logical(true), err(NA), err(Div0)])],
            NA,
        ),
        (
            "=AND({FALSE,#DIV/0!})",
            sd::FUNC_ID_AND,
            vec![arr(vec![CalcValue::logical(false), err(Div0)])],
            Div0,
        ),
        (
            "=AND({FALSE,#N/A,#DIV/0!})",
            sd::FUNC_ID_AND,
            vec![arr(vec![CalcValue::logical(false), err(NA), err(Div0)])],
            NA,
        ),
        // a deciding scalar followed by an error-bearing cell (`=OR(TRUE,C2)`, `=AND(FALSE,D2)`)
        (
            "=OR(TRUE,C2)",
            sd::FUNC_ID_OR,
            vec![CalcValue::logical(true), arr(vec![err(Div0)])],
            Div0,
        ),
        (
            "=AND(FALSE,D2)",
            sd::FUNC_ID_AND,
            vec![CalcValue::logical(false), arr(vec![err(Div0)])],
            Div0,
        ),
    ];
    assert_eq!(rows.len(), 38, "every probed precedence row is pinned");
    for (formula, function_id, args, expected) in rows {
        let got = eval(function_id, &args);
        assert_eq!(
            got.core(),
            &CoreValue::Error(expected),
            "{formula}: the first error in argument order must be published; got {:?}",
            got.core()
        );
    }
}
