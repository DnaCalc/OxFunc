//! Retained typed preparation observations for the four decimal rounding functions.
use oxfunc_core::functions::surface_dispatch::eval_surface_value_call;
use oxfunc_core::resolver::{
    ReferenceDereferenceRequest, ReferenceResolutionError, ReferenceSystemCapabilities,
    ReferenceSystemProvider,
};
use oxfunc_core::value::{
    CalcArray, CalcValue, CoreValue, ExcelText, ReferenceKind, ReferenceLike, WorksheetErrorCode,
};
use serde_json::Value;
use std::collections::BTreeMap;

fn value(raw: &Value) -> CalcValue {
    match raw["kind"].as_str().unwrap() {
        "text" => CalcValue::text(ExcelText::from_utf16_code_units(
            raw["value"].as_str().unwrap().encode_utf16().collect(),
        )),
        "number" => CalcValue::number(raw["value"].as_f64().unwrap()),
        "logical" => CalcValue::logical(raw["value"].as_bool().unwrap()),
        "empty_cell" => CalcValue::empty(),
        "missing_arg" => CalcValue::missing(),
        "error" => CalcValue::error(match raw["code"].as_str().unwrap() {
            "NA" => WorksheetErrorCode::NA,
            "Value" => WorksheetErrorCode::Value,
            "Div0" => WorksheetErrorCode::Div0,
            "Num" => WorksheetErrorCode::Num,
            "Ref" => WorksheetErrorCode::Ref,
            "Name" => WorksheetErrorCode::Name,
            "Null" => WorksheetErrorCode::Null,
            other => panic!("error {other}"),
        }),
        "array" => CalcValue::array(
            CalcArray::from_rows(
                raw["rows"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|row| row.as_array().unwrap().iter().map(value).collect())
                    .collect(),
            )
            .unwrap(),
        ),
        "reference" => CalcValue::reference(ReferenceLike::new(
            ReferenceKind::A1,
            raw["target"].as_str().unwrap().to_string(),
        )),
        other => panic!("unexpected input {other}"),
    }
}
struct Fixtures(BTreeMap<String, CalcValue>);
impl ReferenceSystemProvider for Fixtures {
    fn capabilities(&self) -> ReferenceSystemCapabilities {
        ReferenceSystemCapabilities::permissive_local()
    }
    fn dereference(
        &self,
        request: &ReferenceDereferenceRequest,
    ) -> Result<CalcValue, ReferenceResolutionError> {
        self.0
            .get(request.reference.target())
            .cloned()
            .ok_or_else(|| ReferenceResolutionError::UnresolvedReference {
                target: request.reference.target().to_string(),
            })
    }
}
fn digest(value: &CalcValue) -> String {
    match value.core() {
        CoreValue::Number(n) => format!("number:0x{:016x}", n.to_bits()),
        CoreValue::Text(t) => format!("text:{}", t.to_string_lossy()),
        CoreValue::Logical(b) => format!("logical:{b}"),
        CoreValue::Error(e) => format!("error:{e:?}"),
        CoreValue::Array(a) => format!(
            "array:{}x{}:[{}]",
            a.shape().rows,
            a.shape().cols,
            a.iter_row_major().map(digest).collect::<Vec<_>>().join("|")
        ),
        other => panic!("unexpected output {other:?}"),
    }
}
fn replay(source: &str) {
    let evidence: Value = serde_json::from_str(source).unwrap();
    let mut misses = Vec::new();
    for row in evidence["witnesses"].as_array().unwrap() {
        let case = &row["case"];
        let args: Vec<_> = case["args"].as_array().unwrap().iter().map(value).collect();
        let fixtures = Fixtures(
            case["cell_fixture"]
                .as_array()
                .unwrap()
                .iter()
                .map(|f| {
                    (
                        f["target"].as_str().unwrap().to_string(),
                        value(&f["value"]),
                    )
                })
                .collect(),
        );
        let got = eval_surface_value_call(
            case["function_id"].as_str().unwrap(),
            &args,
            &fixtures,
            None,
            None,
            None,
            None,
        )
        .unwrap_or_else(CalcValue::error);
        let digest = digest(&got);
        let expected = row["excel"]["outcome"]["digest_payload"].as_str().unwrap();
        if digest != expected {
            misses.push(format!(
                "{} expected={expected} actual={digest}",
                case["case_id"]
            ));
        }
    }
    assert!(
        misses.is_empty(),
        "{} differing rows:\n{}",
        misses.len(),
        misses.join("\n")
    );
}

#[test]
fn rounding_preparation_matches_live() {
    replay(include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/rounding-boundaries/prepared-discovery.json"));
}

#[test]
fn rounding_preparation_matches_frozen_independent_heldout() {
    replay(include_str!(
        "../../../docs/function-lane/evidence/w111-broad-20260929/rounding-boundaries/prepared-heldout.json"
    ));
}
