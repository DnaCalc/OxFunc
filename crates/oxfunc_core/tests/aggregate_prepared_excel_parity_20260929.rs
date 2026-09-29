//! Prepared-origin/error-order observations from the W111 public Excel capture.
use oxfunc_core::functions::surface_dispatch::eval_surface_value_call;
use oxfunc_core::resolver::NULL_REFERENCE_SYSTEM_PROVIDER;
use oxfunc_core::value::{CalcArray, CalcValue, CoreValue, ExcelText, WorksheetErrorCode};

fn text(s: &str) -> CalcValue { CalcValue::text(ExcelText::from_interop_assignment(s)) }
fn evaluate(name: &str, args: &[CalcValue]) -> CalcValue {
    match eval_surface_value_call(name, args, &NULL_REFERENCE_SYSTEM_PROVIDER, None, None, None, None) {
        Ok(value) => value,
        Err(code) => CalcValue::error(code),
    }
}
fn error(name: &str, args: &[CalcValue], code: WorksheetErrorCode) {
    assert_eq!(evaluate(name,args), CalcValue::error(code));
}

#[test]
fn blank_cells_are_ignored_but_explicit_missing_contributes_zero() {
    let empty = CalcValue::empty();
    let missing = CalcValue::missing();
    let eight = CalcValue::number(8.0);
    assert_eq!(evaluate("FUNC.HARMEAN", &[empty.clone(), eight.clone()]), eight);
    error("FUNC.HARMEAN", &[missing.clone(), eight.clone()], WorksheetErrorCode::Num);
    assert_eq!(evaluate("FUNC.DEVSQ", &[empty.clone(), eight.clone()]), CalcValue::number(0.0));
    assert_eq!(evaluate("FUNC.DEVSQ", &[missing, eight]), CalcValue::number(32.0));
    error("FUNC.HARMEAN", &[empty.clone()], WorksheetErrorCode::NA);
    error("FUNC.DEVSQ", &[empty], WorksheetErrorCode::Num);
}

#[test]
fn harmean_coercion_and_explicit_errors_precede_numeric_domain_checks() {
    for first in [CalcValue::number(0.0), CalcValue::number(-1.0), CalcValue::missing()] {
        error("FUNC.HARMEAN", &[first.clone(), text("x")], WorksheetErrorCode::Value);
        error("FUNC.HARMEAN", &[first, CalcValue::error(WorksheetErrorCode::Div0)], WorksheetErrorCode::Div0);
    }
    for fn_id in ["FUNC.HARMEAN", "FUNC.DEVSQ"] {
        error(fn_id, &[CalcValue::error(WorksheetErrorCode::Ref), CalcValue::error(WorksheetErrorCode::Div0)], WorksheetErrorCode::Ref);
        error(fn_id, &[CalcValue::error(WorksheetErrorCode::Div0), text("x")], WorksheetErrorCode::Div0);
    }
}

#[test]
fn array_collection_keeps_origin_policy_and_error_order() {
    let values = CalcValue::array(CalcArray::from_rows(vec![vec![text("2"), CalcValue::logical(true), CalcValue::number(8.0)]]).unwrap());
    for (fn_id,bits) in [("FUNC.HARMEAN",8.0f64.to_bits()),("FUNC.DEVSQ",0)] {
        let got=evaluate(fn_id,&[values.clone()]);
        let CoreValue::Number(value)=got.core() else {panic!("{got:?}")};
        assert_eq!(value.to_bits(),bits);
    }
    let errors = CalcValue::array(CalcArray::from_rows(vec![vec![CalcValue::number(-1.0), CalcValue::error(WorksheetErrorCode::Div0), CalcValue::number(8.0)]]).unwrap());
    error("FUNC.HARMEAN", &[errors], WorksheetErrorCode::Div0);
}

#[test]
fn direct_scalar_errors_precede_collection_errors_without_flattening_origins() {
    for fn_id in ["FUNC.HARMEAN", "FUNC.DEVSQ"] {
        let unit = CalcValue::array(CalcArray::from_rows(vec![vec![CalcValue::error(WorksheetErrorCode::Div0)]]).unwrap());
        let area = CalcValue::array(CalcArray::from_rows(vec![vec![CalcValue::error(WorksheetErrorCode::Num), CalcValue::number(4.0)]]).unwrap());
        error(fn_id, &[unit.clone(), CalcValue::error(WorksheetErrorCode::Num)], WorksheetErrorCode::Num);
        error(fn_id, &[unit.clone(), text("x")], WorksheetErrorCode::Value);
        error(fn_id, &[unit.clone(), area.clone()], WorksheetErrorCode::Div0);
        error(fn_id, &[area.clone(), unit], WorksheetErrorCode::Num);
        error(fn_id, &[area, CalcValue::error(WorksheetErrorCode::Div0), text("x")], WorksheetErrorCode::Div0);
    }
}
