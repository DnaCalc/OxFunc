//! Exact NOT logical-text observations, including prepared references and array cells.
use oxfunc_core::functions::surface_dispatch::eval_surface_value_call;
use oxfunc_core::resolver::{ReferenceDereferenceRequest, ReferenceResolutionError,
    ReferenceSystemCapabilities, ReferenceSystemProvider};
use oxfunc_core::value::{CalcArray, CalcValue, CoreValue, ExcelText, ReferenceKind,
    ReferenceLike, WorksheetErrorCode};
use serde_json::Value;
use std::collections::BTreeMap;

fn error(code: &str) -> WorksheetErrorCode {
    match code { "NA"=>WorksheetErrorCode::NA, "Value"=>WorksheetErrorCode::Value,
        "Div0"=>WorksheetErrorCode::Div0, "Num"=>WorksheetErrorCode::Num,
        "Ref"=>WorksheetErrorCode::Ref, "Name"=>WorksheetErrorCode::Name,
        "Null"=>WorksheetErrorCode::Null, other=>panic!("unknown error {other}") }
}
fn value(raw: &Value) -> CalcValue {
    match raw["kind"].as_str().unwrap() {
        "text"=>CalcValue::text(ExcelText::from_utf16_code_units(raw["value"].as_str().unwrap().encode_utf16().collect())),
        "number"=>CalcValue::number(raw["value"].as_f64().unwrap()),
        "logical"=>CalcValue::logical(raw["value"].as_bool().unwrap()),
        "empty_cell"=>CalcValue::empty(),
        "error"=>CalcValue::error(error(raw["code"].as_str().unwrap())),
        "reference"=>CalcValue::reference(ReferenceLike::new(
            if raw["reference_kind"]=="Area" {ReferenceKind::Area} else {ReferenceKind::A1},
            raw["target"].as_str().unwrap().to_owned())),
        "array"=>CalcValue::array(CalcArray::from_rows(raw["rows"].as_array().unwrap().iter()
            .map(|row|row.as_array().unwrap().iter().map(value).collect()).collect()).unwrap()),
        other=>panic!("unsupported {other}"),
    }
}
struct Fixtures(BTreeMap<String,CalcValue>);
impl ReferenceSystemProvider for Fixtures {
    fn capabilities(&self)->ReferenceSystemCapabilities {ReferenceSystemCapabilities::permissive_local()}
    fn dereference(&self,request:&ReferenceDereferenceRequest)->Result<CalcValue,ReferenceResolutionError> {
        self.0.get(request.reference.target()).cloned().ok_or_else(||ReferenceResolutionError::UnresolvedReference{
            target:request.reference.target().to_owned()})
    }
}
fn digest(value:&CalcValue)->String {
    match value.core() {
        CoreValue::Logical(b)=>format!("logical:{b}"),
        CoreValue::Error(e)=>format!("error:{e:?}"),
        CoreValue::Array(a)=>format!("array:{}x{}:[{}]",a.shape().rows,a.shape().cols,
            a.iter_row_major().map(digest).collect::<Vec<_>>().join("|")),
        other=>panic!("unsupported result {other:?}"),
    }
}
fn replay(raw_cases:&str,raw_oracle:&str,expected_counts:(usize,usize)) {
    let cases:Value=serde_json::from_str(raw_cases).unwrap();
    let mut oracle=BTreeMap::new();
    for line in raw_oracle.lines() {
        let row:Value=serde_json::from_str(line).unwrap();
        assert!(oracle.insert(row["case_id"].as_str().unwrap().to_owned(),row).is_none());
    }
    let mut tested=0;let mut transport_limited=0;let mut misses=Vec::new();
    for case in cases["cases"].as_array().unwrap() {
        let observed=&oracle[case["case_id"].as_str().unwrap()];
        assert_eq!(case["function_id"],observed["function_id"]);
        assert_eq!(case["formula_text"],observed["formula_text"]);
        if case["case_tag"]=="missing" {
            // Formula entry rejects NOT(); it does not observe a prepared Missing value.
            assert_eq!(observed["execution_status"],"excel_case_harness_error");
            transport_limited+=1;continue;
        }
        assert_eq!(observed["execution_status"],"ok");
        let args:Vec<_>=case["args"].as_array().unwrap().iter().map(value).collect();
        let fixtures=Fixtures(case["cell_fixture"].as_array().unwrap().iter().map(|f|
            (f["target"].as_str().unwrap().to_owned(),value(&f["value"]))).collect());
        let actual=eval_surface_value_call("FUNC.NOT",&args,&fixtures,None,None,None,None)
            .unwrap_or_else(CalcValue::error);
        let actual=digest(&actual);
        let expected=observed["outcome"]["digest_payload"].as_str().unwrap();
        if actual!=expected {misses.push(format!("{} actual={actual} expected={expected}",case["case_id"]));}
        tested+=1;
    }
    assert_eq!((tested,transport_limited),expected_counts);
    assert!(misses.is_empty(),"{} differences:\n{}",misses.len(),misses.join("\n"));
}

#[test]
fn not_text_truth_is_shared_by_direct_reference_and_array_origins() {
    replay(include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/not/cases.json"),
        include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/not/excel.jsonl"),(542,1));
}

#[test]
fn independent_mixed_shape_not_holdout_matches_exactly() {
    replay(include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/not/heldout-cases.json"),
        include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/not/heldout-excel.jsonl"),(120,0));
}
