//! Exact replay of live typed ATAN observations, including blank/reference/array origins.
use oxfunc_core::functions::surface_dispatch::eval_surface_value_call;
use oxfunc_core::functions::a1_refs::parse_a1_reference;
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
impl Fixtures {
    fn resolve(&self, target: &str) -> Option<CalcValue> {
        if let Some(value) = self.0.get(target) { return Some(value.clone()); }
        let requested = parse_a1_reference(target)?;
        let cells: BTreeMap<_, _> = self.0.iter().filter_map(|(address, value)| {
            let cell = parse_a1_reference(address)?;
            (cell.prefix == requested.prefix && cell.height() == 1 && cell.width() == 1)
                .then(|| ((cell.start_row, cell.start_col), value.clone()))
        }).collect();
        let rows = (requested.start_row..=requested.end_row).map(|row|
            (requested.start_col..=requested.end_col).map(|col| cells.get(&(row, col)).cloned())
                .collect::<Option<Vec<_>>>()).collect::<Option<Vec<_>>>()?;
        CalcArray::from_rows(rows).map(CalcValue::array)
    }
}
impl ReferenceSystemProvider for Fixtures {
    fn capabilities(&self) -> ReferenceSystemCapabilities {
        ReferenceSystemCapabilities::permissive_local()
    }
    fn dereference(&self, request: &ReferenceDereferenceRequest) -> Result<CalcValue, ReferenceResolutionError> {
        self.resolve(request.reference.target()).ok_or_else(||
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
        other => panic!("unexpected ATAN output {other:?}"),
    }
}

fn replay(source: &str, expected: usize) {
    let bank: Value = serde_json::from_str(source).unwrap();
    let rows=bank["witnesses"].as_array().unwrap();
    assert_eq!(rows.len(),expected);
    for row in rows {
        let case=&row["case"]; let oracle=&row["excel"];
        assert_eq!(case["function_id"],"FUNC.ATAN");
        assert_eq!(oracle["execution_status"],"ok");
        let resolver=Fixtures(case["cell_fixture"].as_array().unwrap().iter().map(|f|
            (f["target"].as_str().unwrap().to_owned(),typed(&f["value"]))).collect());
        let args:Vec<_>=case["args"].as_array().unwrap().iter().map(typed).collect();
        let actual=eval_surface_value_call("FUNC.ATAN",&args,&resolver,None,None,None,None).unwrap_or_else(CalcValue::error);
        assert_eq!(digest(&actual),oracle["outcome"]["digest_payload"].as_str().unwrap(),"{}",case["case_id"]);
    }
}
#[test]
fn atan_typed_independent_heldout() {
    replay(include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/atan/typed-heldout-bank.json"),297);
}
#[test]
fn atan_typed_repeat_controls() {
    replay(include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/atan/typed-repeat-controls-bank.json"),544);
}
