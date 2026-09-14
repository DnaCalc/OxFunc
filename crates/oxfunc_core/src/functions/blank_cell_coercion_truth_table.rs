//! W110-1 (oxf-xvt5.1) — blank-cell coercion truth table, exercised through the REAL
//! function/operator dispatch (`eval_surface_value_call`) with a `ReferenceSystemProvider`
//! that resolves `A1` to `CoreValue::Empty`. These tests deliberately do NOT call the
//! coercion helpers directly: the claim under test is what a worksheet formula referencing a
//! blank cell publishes, end to end through the by-index dispatch table, the argument
//! preparation layer, and each function's own adapter.
//!
//! Provenance of every expected value (clean-room rule: public documentation plus reproducible
//! black-box Excel observation; each page below was fetched and its wording checked on
//! 2026-09-14):
//!
//! * SCALAR contexts treat a referenced blank cell as `0` (numeric), `""` (text) or `FALSE`
//!   (logical). Microsoft's operator page documents the conversion algebra and the `#VALUE!`
//!   for text that cannot be converted to a number — "Calculation operators and precedence in
//!   Excel", section "How Excel converts values in formulas"
//!   (<https://support.microsoft.com/en-us/office/48be406d-4975-4d31-b2b8-7af9e0e2878a>) —
//!   but does NOT itself spell out the blank-cell case. The blank-as-zero values here are the
//!   Excel truth table the bead (oxf-xvt5.1) supplies, and they agree with this repo's own
//!   native-Excel COM probe records that pinned "blank-reference coercion to zero" for
//!   `DOLLARDE`/`DOLLARFR` and `DAY`/`MONTH`/`YEAR`/`DAYS` (rows `W16-BATCH37-DOLLAR-
//!   FRACTION-20260316` and `W16-BATCH38-DATE-PARTS-20260316` in
//!   `docs/function-lane/FUNCTION_LANE_EVIDENCE_ID_REGISTRY.md`). The `=A1+""` row pins the
//!   boundary the operator page does document: a *text* empty string is not a blank cell and
//!   stays `#VALUE!` in arithmetic.
//! * AGGREGATE contexts IGNORE a referenced blank cell rather than counting it as zero:
//!   - SUM: "SUM will ignore text values and give you the sum of just the numeric values."
//!     (<https://support.microsoft.com/en-us/excel/functions/sum-function>); the page no
//!     longer states the empty-cell rule in so many words — `=SUM(A1)` -> `0` and
//!     `=SUM(A1,1)` -> `1` below pin it, and the sibling aggregate pages state it.
//!   - AVERAGE: "If a range or cell reference argument contains text, logical values, or empty
//!     cells, those values are ignored; however, cells with the value zero are included."
//!     (<https://support.microsoft.com/en-us/excel/functions/average-function>)
//!   - COUNT: "Empty cells, logical values, text, or error values in the array or reference
//!     are not counted." (<https://support.microsoft.com/en-us/excel/functions/count-function>)
//!   - COUNTA: "The COUNTA function does not count empty cells."
//!     (<https://support.microsoft.com/en-us/excel/functions/counta-function>)
//!   - MAX: "Empty cells, logical values, or text in the array or reference are ignored." and
//!     "If the arguments contain no numbers, MAX returns 0 (zero)."
//!     (<https://support.microsoft.com/en-us/excel/functions/max-function>)
//!   - PRODUCT: "Empty cells, logical values, and text in the array or reference are ignored."
//!     (<https://support.microsoft.com/en-us/excel/functions/product-function>)
//! * PREDICATE contexts see the blank as a blank: ISBLANK "Value refers to an empty cell";
//!   ISNUMBER "Value refers to a number." Microsoft, "IS functions"
//!   (<https://support.microsoft.com/en-us/office/0f2d7971-6019-40a0-a171-f2d869135665>).
//!
//! The grid this resolver models: `A1` blank, `A2 = 2`, `A3 = 3`. Single-cell references go
//! through `dereference`; the `A1:A3` area is served both DENSE (a 3x1 array with an `Empty`
//! cell, the path a materialising provider takes) and SPARSE (`enumerate_values` listing only
//! the defined cells, the path an OxCalc-style sparse provider takes), because blank-skipping
//! in aggregates is exactly the behaviour a scalar-layer fix must not disturb.

use crate::functions::surface_dispatch::{self as sd, eval_surface_value_call};
use crate::locale_format::test_current_excel_host_context;
use crate::resolver::{
    ReferenceDereferenceRequest, ReferenceEnumerationRequest, ReferenceResolutionError,
    ReferenceSystemCapabilities, ReferenceSystemProvider, ResolvedReferenceCell,
    ResolvedReferenceExtent, ResolvedReferenceValues,
};
use crate::value::{
    CalcArray, CalcValue, CoreValue, ExcelText, ReferenceKind, ReferenceLike, WorksheetErrorCode,
};

/// `A1` blank, `A2 = 2`, `A3 = 3`. `sparse_area` selects how `A1:A3` is served.
struct BlankGridResolver {
    sparse_area: bool,
}

impl BlankGridResolver {
    const DENSE: Self = Self { sparse_area: false };
    const SPARSE: Self = Self { sparse_area: true };
}

impl ReferenceSystemProvider for BlankGridResolver {
    fn capabilities(&self) -> ReferenceSystemCapabilities {
        ReferenceSystemCapabilities::permissive_local()
    }

    fn dereference(
        &self,
        request: &ReferenceDereferenceRequest,
    ) -> Result<CalcValue, ReferenceResolutionError> {
        match request.reference.target() {
            "A1" => Ok(CalcValue::empty()),
            "A2" => Ok(CalcValue::number(2.0)),
            "A3" => Ok(CalcValue::number(3.0)),
            "A1:A3" => Ok(CalcValue::array(
                CalcArray::from_rows(vec![
                    vec![CalcValue::empty()],
                    vec![CalcValue::number(2.0)],
                    vec![CalcValue::number(3.0)],
                ])
                .expect("3x1 area"),
            )),
            other => Err(ReferenceResolutionError::UnresolvedReference {
                target: other.to_string(),
            }),
        }
    }

    fn enumerate_values(
        &self,
        request: &ReferenceEnumerationRequest,
    ) -> Result<Option<ResolvedReferenceValues>, ReferenceResolutionError> {
        if !self.sparse_area || request.reference.target() != "A1:A3" {
            return Ok(None);
        }
        // A1 is undefined (blank): only rows 2 and 3 are listed.
        Ok(Some(ResolvedReferenceValues::new(
            ResolvedReferenceExtent::new(3, 1),
            vec![
                ResolvedReferenceCell::new(2, 1, CalcValue::number(2.0)),
                ResolvedReferenceCell::new(3, 1, CalcValue::number(3.0)),
            ],
            Some("reader:blank-grid:3x1".to_string()),
        )))
    }
}

fn a1() -> CalcValue {
    CalcValue::reference(ReferenceLike::new(ReferenceKind::A1, "A1"))
}

fn a1_a3() -> CalcValue {
    CalcValue::reference(ReferenceLike::new(ReferenceKind::Area, "A1:A3"))
}

fn num(n: f64) -> CalcValue {
    CalcValue::number(n)
}

fn txt(s: &str) -> CalcValue {
    CalcValue::text(ExcelText::from_utf16_code_units(s.encode_utf16().collect()))
}

fn logical(b: bool) -> CalcValue {
    CalcValue::logical(b)
}

fn err(code: WorksheetErrorCode) -> CalcValue {
    CalcValue::error(code)
}

/// Evaluate `function_id(args)` through the real dispatch. A top-level `Err(code)` is folded
/// into the worksheet error value so a row can pin either publication shape as one expected
/// `CalcValue` (a host publishes both the same way).
fn eval(function_id: &str, args: &[CalcValue], resolver: &BlankGridResolver) -> CalcValue {
    let locale = test_current_excel_host_context();
    match eval_surface_value_call(function_id, args, resolver, None, None, Some(&locale), None) {
        Ok(value) => value,
        Err(code) => CalcValue::error(code),
    }
}

/// One truth-table row: the formula as a person would write it, the dispatch id and args that
/// formula lowers to, and the value Excel publishes.
struct Row {
    formula: &'static str,
    function_id: &'static str,
    args: Vec<CalcValue>,
    expected: CalcValue,
}

fn row(
    formula: &'static str,
    function_id: &'static str,
    args: Vec<CalcValue>,
    expected: CalcValue,
) -> Row {
    Row {
        formula,
        function_id,
        args,
        expected,
    }
}

/// Run every row against `resolver`; return the rows that do not match Excel, formatted.
fn mismatches(rows: &[Row], resolver: &BlankGridResolver) -> Vec<String> {
    rows.iter()
        .filter_map(|row| {
            let got = eval(row.function_id, &row.args, resolver);
            (got != row.expected).then(|| {
                format!(
                    "{:<18} expected {:?}, got {:?}",
                    row.formula,
                    row.expected.core(),
                    got.core()
                )
            })
        })
        .collect()
}

fn assert_rows_match_excel(rows: &[Row], resolver: &BlankGridResolver) {
    let failed = mismatches(rows, resolver);
    assert!(
        failed.is_empty(),
        "{} of {} blank-cell rows diverge from Excel:\n{}",
        failed.len(),
        rows.len(),
        failed.join("\n")
    );
}

// -------------------------------------------------------------------------------------------
// SCALAR NUMERIC contexts: a referenced blank is 0 (provenance in the module header).
// Function pages: ABS
// (<https://support.microsoft.com/en-us/office/abs-function-3420200f-5628-4e8c-99da-c99d7c87713c>);
// ROUND (<https://support.microsoft.com/en-us/excel/functions/round-function>). The `=A1+""`
// boundary row is the operator page's documented rule: text that cannot be converted to a
// number is `#VALUE!` ("Calculation operators and precedence in Excel",
// <https://support.microsoft.com/en-us/office/48be406d-4975-4d31-b2b8-7af9e0e2878a>).
// -------------------------------------------------------------------------------------------

fn scalar_numeric_rows() -> Vec<Row> {
    vec![
        row("=A1+1", sd::FUNC_ID_OP_ADD, vec![a1(), num(1.0)], num(1.0)),
        row(
            "=A1*2",
            sd::FUNC_ID_OP_MULTIPLY,
            vec![a1(), num(2.0)],
            num(0.0),
        ),
        row("=-A1", sd::FUNC_ID_OP_NEGATE, vec![a1()], num(0.0)),
        row("=ABS(A1)", sd::FUNC_ID_ABS, vec![a1()], num(0.0)),
        row(
            "=ROUND(A1,2)",
            sd::FUNC_ID_ROUND,
            vec![a1(), num(2.0)],
            num(0.0),
        ),
        // Same rule family, the remaining arithmetic operators that share the binary / unary
        // numeric executors with the rows above.
        row(
            "=A1-1",
            sd::FUNC_ID_OP_SUBTRACT,
            vec![a1(), num(1.0)],
            num(-1.0),
        ),
        row(
            "=A1/2",
            sd::FUNC_ID_OP_DIVIDE,
            vec![a1(), num(2.0)],
            num(0.0),
        ),
        row(
            "=2/A1",
            sd::FUNC_ID_OP_DIVIDE,
            vec![num(2.0), a1()],
            err(WorksheetErrorCode::Div0),
        ),
        row(
            "=A1^2",
            sd::FUNC_ID_OP_POWER,
            vec![a1(), num(2.0)],
            num(0.0),
        ),
        row("=A1%", sd::FUNC_ID_OP_PERCENT, vec![a1()], num(0.0)),
        row("=+A1", sd::FUNC_ID_OP_UNARY_PLUS, vec![a1()], num(0.0)),
        // A TEXT empty string is not a blank cell: arithmetic on it is #VALUE!.
        row(
            "=A1+\"\"",
            sd::FUNC_ID_OP_ADD,
            vec![a1(), txt("")],
            err(WorksheetErrorCode::Value),
        ),
    ]
}

#[test]
fn scalar_numeric_contexts_coerce_a_referenced_blank_to_zero() {
    assert_rows_match_excel(&scalar_numeric_rows(), &BlankGridResolver::DENSE);
}

// -------------------------------------------------------------------------------------------
// SCALAR TEXT contexts: a referenced blank is "" (provenance in the module header).
// Function pages: LEN
// (<https://support.microsoft.com/en-us/office/len-lenb-functions-29236f94-cedc-429d-affd-b5e33d2c67cb>);
// TEXT (<https://support.microsoft.com/en-us/office/text-function-20d5ac4d-7b94-49fd-bb38-93d29371225c>)
// formats the blank as the number 0.
// -------------------------------------------------------------------------------------------

fn scalar_text_rows() -> Vec<Row> {
    vec![
        row(
            "=A1&\"x\"",
            sd::FUNC_ID_OP_CONCAT,
            vec![a1(), txt("x")],
            txt("x"),
        ),
        row("=LEN(A1)", sd::FUNC_ID_LEN, vec![a1()], num(0.0)),
        row(
            "=TEXT(A1,\"0\")",
            sd::FUNC_ID_TEXT,
            vec![a1(), txt("0")],
            txt("0"),
        ),
    ]
}

#[test]
fn scalar_text_contexts_coerce_a_referenced_blank_to_empty_text() {
    assert_rows_match_excel(&scalar_text_rows(), &BlankGridResolver::DENSE);
}

// -------------------------------------------------------------------------------------------
// SCALAR LOGICAL / COMPARISON contexts: a referenced blank is FALSE, and compares equal to
// both 0 and "" (provenance in the module header).
// Function pages: NOT
// (<https://support.microsoft.com/en-us/office/not-function-9cfc6011-a054-40c7-a140-cd4ba2d87d77>);
// IF (<https://support.microsoft.com/en-us/office/if-function-69aed7c9-4e8a-4755-a9bc-aa8bbff73be2>).
// -------------------------------------------------------------------------------------------

fn scalar_logical_rows() -> Vec<Row> {
    vec![
        row(
            "=A1=0",
            sd::FUNC_ID_OP_EQUAL,
            vec![a1(), num(0.0)],
            logical(true),
        ),
        row(
            "=A1=\"\"",
            sd::FUNC_ID_OP_EQUAL,
            vec![a1(), txt("")],
            logical(true),
        ),
        row("=NOT(A1)", sd::FUNC_ID_NOT, vec![a1()], logical(true)),
        row(
            "=IF(A1,1,2)",
            sd::FUNC_ID_IF,
            vec![a1(), num(1.0), num(2.0)],
            num(2.0),
        ),
    ]
}

#[test]
fn scalar_logical_contexts_coerce_a_referenced_blank_to_false() {
    assert_rows_match_excel(&scalar_logical_rows(), &BlankGridResolver::DENSE);
}

// -------------------------------------------------------------------------------------------
// AGGREGATE contexts: a referenced blank is IGNORED, never counted as zero (doc citations in
// the module header).
// -------------------------------------------------------------------------------------------

fn aggregate_rows() -> Vec<Row> {
    vec![
        row("=SUM(A1)", sd::FUNC_ID_SUM, vec![a1()], num(0.0)),
        row(
            "=SUM(A1,1)",
            sd::FUNC_ID_SUM,
            vec![a1(), num(1.0)],
            num(1.0),
        ),
        row("=COUNT(A1)", sd::FUNC_ID_COUNT, vec![a1()], num(0.0)),
        row("=COUNTA(A1)", sd::FUNC_ID_COUNTA, vec![a1()], num(0.0)),
        row(
            "=AVERAGE(A1)",
            sd::FUNC_ID_AVERAGE,
            vec![a1()],
            err(WorksheetErrorCode::Div0),
        ),
        row(
            "=AVERAGE(A1,4)",
            sd::FUNC_ID_AVERAGE,
            vec![a1(), num(4.0)],
            num(4.0),
        ),
        row("=MAX(A1)", sd::FUNC_ID_MAX, vec![a1()], num(0.0)),
        // Blank ignored, not zero: the maximum of {-3} is -3, not 0.
        row(
            "=MAX(A1,-3)",
            sd::FUNC_ID_MAX,
            vec![a1(), num(-3.0)],
            num(-3.0),
        ),
        row(
            "=PRODUCT(A1,5)",
            sd::FUNC_ID_PRODUCT,
            vec![a1(), num(5.0)],
            num(5.0),
        ),
    ]
}

#[test]
fn aggregate_contexts_ignore_a_referenced_blank() {
    assert_rows_match_excel(&aggregate_rows(), &BlankGridResolver::DENSE);
}

// -------------------------------------------------------------------------------------------
// PREDICATE contexts: the blank stays visible as a blank ("IS functions" page, module header).
// -------------------------------------------------------------------------------------------

fn predicate_rows() -> Vec<Row> {
    vec![
        row(
            "=ISBLANK(A1)",
            sd::FUNC_ID_ISBLANK,
            vec![a1()],
            logical(true),
        ),
        row(
            "=ISNUMBER(A1)",
            sd::FUNC_ID_ISNUMBER,
            vec![a1()],
            logical(false),
        ),
    ]
}

#[test]
fn predicate_contexts_see_a_referenced_blank_as_blank() {
    assert_rows_match_excel(&predicate_rows(), &BlankGridResolver::DENSE);
}

// -------------------------------------------------------------------------------------------
// AREA / lifted contexts: SUM over a range skips the blank slot; a lifted scalar operator
// treats the blank slot as 0.
// -------------------------------------------------------------------------------------------

fn area_rows() -> Vec<Row> {
    vec![
        // A1 blank, A2 = 2, A3 = 3: the blank is skipped, so 5 — not 5 plus a counted zero,
        // and not an error.
        row("=SUM(A1:A3)", sd::FUNC_ID_SUM, vec![a1_a3()], num(5.0)),
        // Lifted: the blank slot becomes 0 + 1 = 1.
        row(
            "=A1:A3+1",
            sd::FUNC_ID_OP_ADD,
            vec![a1_a3(), num(1.0)],
            CalcValue::array(
                CalcArray::from_rows(vec![vec![num(1.0)], vec![num(3.0)], vec![num(4.0)]])
                    .expect("3x1 result"),
            ),
        ),
    ]
}

#[test]
fn area_contexts_skip_the_blank_in_aggregates_and_lift_it_as_zero_in_operators() {
    assert_rows_match_excel(&area_rows(), &BlankGridResolver::DENSE);
}

/// The same aggregate rows served by a SPARSE provider (only the defined cells are listed,
/// as an OxCalc-style grid provider does): blank-skipping must hold on that path too.
#[test]
fn sparse_area_provider_still_skips_the_blank_in_sum() {
    let rows = vec![row("=SUM(A1:A3)", sd::FUNC_ID_SUM, vec![a1_a3()], num(5.0))];
    assert_rows_match_excel(&rows, &BlankGridResolver::SPARSE);
}

/// The `A1:A3` array a dense provider hands back really does carry an `Empty` cell (guards
/// the fixture itself: if the resolver ever materialised the blank as 0, every aggregate row
/// above would pass vacuously).
#[test]
fn dense_area_fixture_carries_an_empty_cell() {
    let request = ReferenceDereferenceRequest {
        reference: ReferenceLike::new(ReferenceKind::Area, "A1:A3"),
    };
    let area = BlankGridResolver::DENSE
        .dereference(&request)
        .expect("A1:A3 resolves");
    let CoreValue::Array(array) = area.core() else {
        panic!("A1:A3 must materialise as an array");
    };
    assert_eq!(
        array.get(0, 0).map(CalcValue::core),
        Some(&CoreValue::Empty)
    );
}
