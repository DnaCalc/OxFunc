//! Exact replay of live typed radix observations, including blank/reference/array origins.
use oxfunc_core::functions::surface_dispatch::eval_surface_value_call;
use oxfunc_core::resolver::{ReferenceDereferenceRequest, ReferenceResolutionError,
    ReferenceSystemCapabilities, ReferenceSystemProvider};
use oxfunc_core::value::{CalcArray, CalcValue, CoreValue, ExcelText, ReferenceKind,
    ReferenceLike, WorksheetErrorCode};
use serde_json::Value;
use std::collections::BTreeMap;

fn typed(value: &Value) -> CalcValue {
    match value["kind"].as_str().unwrap() {
        "number" => CalcValue::number(value["value"].as_f64().unwrap()),
        "text" => CalcValue::text(ExcelText::from_interop_assignment(value["value"].as_str().unwrap())),
        "logical" => CalcValue::logical(value["value"].as_bool().unwrap()),
        "empty_cell" => CalcValue::empty(),
        "missing_arg" => CalcValue::missing(),
        "error" => CalcValue::error(match value["code"].as_str().unwrap() {
            "Div0" => WorksheetErrorCode::Div0, "Value" => WorksheetErrorCode::Value,
            "Ref" => WorksheetErrorCode::Ref, "NA" => WorksheetErrorCode::NA,
            "Num" => WorksheetErrorCode::Num, "Name" => WorksheetErrorCode::Name,
            "Null" => WorksheetErrorCode::Null, other => panic!("unexpected error {other}"),
        }),
        "reference" => CalcValue::reference(ReferenceLike::new(
            if value["reference_kind"] == "Area" { ReferenceKind::Area } else { ReferenceKind::A1 },
            value["target"].as_str().unwrap().to_string())),
        "array" => CalcValue::array(CalcArray::from_rows(value["rows"].as_array().unwrap().iter()
            .map(|row| row.as_array().unwrap().iter().map(typed).collect()).collect()).unwrap()),
        other => panic!("unexpected value kind {other}"),
    }
}

struct Fixtures(BTreeMap<String, CalcValue>);
impl ReferenceSystemProvider for Fixtures {
    fn capabilities(&self) -> ReferenceSystemCapabilities {
        ReferenceSystemCapabilities::permissive_local()
    }
    fn dereference(&self, request: &ReferenceDereferenceRequest) -> Result<CalcValue, ReferenceResolutionError> {
        self.0.get(request.reference.target()).cloned().ok_or_else(||
            ReferenceResolutionError::UnresolvedReference { target: request.reference.target().to_string() })
    }
}

fn digest(value: &CalcValue) -> String {
    match value.core() {
        CoreValue::Number(n) => format!("number:0x{:016x}", n.to_bits()),
        CoreValue::Text(t) => format!("text:{}", t.to_string_lossy()),
        CoreValue::Error(e) => format!("error:{e:?}"),
        CoreValue::Array(a) => format!("array:{}x{}:[{}]", a.shape().rows, a.shape().cols,
            a.iter_row_major().map(digest).collect::<Vec<_>>().join("|")),
        other => panic!("unexpected radix output {other:?}"),
    }
}

fn replay(source: &str, expected_admitted: usize, expected_withheld: usize) {
    let bank: Value = serde_json::from_str(source).unwrap();
    let mut admitted=0; let mut withheld=0;
    for row in bank["witnesses"].as_array().unwrap() {
        let case=&row["case"]; let oracle=&row["excel"];
        if oracle["outcome"]["kind"] == "harness_error" {
            assert_eq!(case["args"].as_array().unwrap().len(), 1);
            assert_eq!(case["args"][0]["kind"], "missing_arg");
            assert_eq!(oracle["outcome"]["message"], "0x800A03EC");
            withheld+=1; continue;
        }
        assert_eq!(oracle["execution_status"], "ok");
        let resolver=Fixtures(case["cell_fixture"].as_array().unwrap().iter().map(|f|
            (f["target"].as_str().unwrap().to_owned(),typed(&f["value"]))).collect());
        let args:Vec<_>=case["args"].as_array().unwrap().iter().map(typed).collect();
        let actual=eval_surface_value_call(case["function_id"].as_str().unwrap(),&args,
            &resolver,None,None,None,None).unwrap_or_else(CalcValue::error);
        assert_eq!(digest(&actual),oracle["outcome"]["digest_payload"].as_str().unwrap(),
            "{} {} args={}",case["case_id"],case["formula_text"],case["args"]);
        admitted+=1;
    }
    assert_eq!((admitted,withheld),(expected_admitted,expected_withheld));
}

#[test]
fn radix_live_typed_discovery() {
    replay(include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/radix/typed-discovery.json"),3249,12);
}

#[test]
fn radix_live_typed_independent_heldout() {
    replay(include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/radix/typed-heldout.json"),1950,0);
}
