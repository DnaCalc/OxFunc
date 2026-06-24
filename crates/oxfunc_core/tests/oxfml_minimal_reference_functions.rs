//! Reference-taking functions exercised through OxFml's minimal (non-grid) test
//! reference profile.
//!
//! These cases were migrated out of the bare-adapter function corpus
//! (`oxfunc_adapter_function_corpus.json`): OxFml core is now grid-agnostic and
//! the adapter has no reference-bind-profile seam, so A1 references no longer
//! bind there. The *real* grid-reference behavior of these functions (R1C1,
//! sheet-qualified, whole-axis, spill, `$` fidelity, ...) is owned and tested by
//! OxCalc against the strict-excel-grid provider. Here we only assert that the
//! minimal same-sheet A1 scaffolding (an opaque profile-symbolic reference whose
//! cells come from `with_cell_values`) resolves each function correctly from the
//! OxFunc side.

use oxfml_core::test_support::host::SingleFormulaHost;
use oxfml_core::test_support::minimal::MINIMAL_REFERENCE_PROFILE;
use oxfunc_core::value::{CalcValue, ExcelText, WorksheetErrorCode};

fn worksheet_value(formula: &str, cells: &[(&str, CalcValue)]) -> CalcValue {
    let mut host = SingleFormulaHost::new("minimal-ref-fn", formula);
    for (key, value) in cells {
        host.set_cell_value(*key, value.clone());
    }
    host.recalc_with_reference_bind_profile(None, None, Some(&MINIMAL_REFERENCE_PROFILE))
        .expect("recalc under the minimal reference profile")
        .published_worksheet_value
}

fn num(n: f64) -> CalcValue {
    CalcValue::number(n)
}

fn text(s: &str) -> CalcValue {
    CalcValue::text(ExcelText::from_interop_assignment(s))
}

#[test]
fn columns_reports_range_width() {
    assert_eq!(worksheet_value("=COLUMNS(A1:C1)", &[]), num(3.0));
}

#[test]
fn index_selects_cell_from_area() {
    let cells = [
        ("A1", num(10.0)),
        ("B1", num(20.0)),
        ("A2", num(30.0)),
        ("B2", num(40.0)),
        ("A3", num(50.0)),
        ("B3", num(60.0)),
    ];
    assert_eq!(worksheet_value("=INDEX(A1:B3,2,2)", &cells), num(40.0));
}

#[test]
fn ifna_substitutes_for_na_cell() {
    let cells = [("A1", CalcValue::error(WorksheetErrorCode::NA))];
    assert_eq!(worksheet_value("=IFNA(A1,0)", &cells), num(0.0));
}

#[test]
fn isblank_is_true_for_absent_cell() {
    assert_eq!(worksheet_value("=ISBLANK(A1)", &[]), CalcValue::logical(true));
}

#[test]
fn iserror_is_true_for_error_cell() {
    let cells = [("A1", CalcValue::error(WorksheetErrorCode::Value))];
    assert_eq!(worksheet_value("=ISERROR(A1)", &cells), CalcValue::logical(true));
}

#[test]
fn countblank_counts_absent_cells_in_range() {
    let cells = [("A1", num(1.0)), ("A3", num(3.0))];
    assert_eq!(worksheet_value("=COUNTBLANK(A1:A3)", &cells), num(1.0));
}

#[test]
fn countif_over_range() {
    let cells = [
        ("A1", num(1.0)),
        ("A2", num(2.0)),
        ("A3", num(4.0)),
        ("A4", num(5.0)),
        ("A5", num(6.0)),
    ];
    assert_eq!(worksheet_value("=COUNTIF(A1:A5,\">3\")", &cells), num(3.0));
}

#[test]
fn sumif_over_range() {
    let cells = [("A1", num(1.0)), ("A2", num(2.0)), ("A3", num(3.0))];
    assert_eq!(worksheet_value("=SUMIF(A1:A3,\">1\")", &cells), num(5.0));
}

#[test]
fn averageif_over_range() {
    let cells = [
        ("A1", num(1.0)),
        ("A2", num(2.0)),
        ("A3", num(3.0)),
        ("A4", num(4.0)),
    ];
    assert_eq!(worksheet_value("=AVERAGEIF(A1:A4,\">2\")", &cells), num(3.5));
}

#[test]
fn hlookup_over_area() {
    let cells = [
        ("A1", num(1.0)),
        ("B1", num(2.0)),
        ("C1", num(3.0)),
        ("A2", num(10.0)),
        ("B2", num(20.0)),
        ("C2", num(30.0)),
    ];
    assert_eq!(worksheet_value("=HLOOKUP(2,A1:C2,2,FALSE)", &cells), num(20.0));
}

#[test]
fn xlookup_over_columns() {
    let cells = [
        ("A1", num(1.0)),
        ("A2", num(2.0)),
        ("A3", num(3.0)),
        ("B1", text("one")),
        ("B2", text("two")),
        ("B3", text("three")),
    ];
    assert_eq!(worksheet_value("=XLOOKUP(2,A1:A3,B1:B3)", &cells), text("two"));
}
