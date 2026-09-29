//! GCD/LCM retained typed observations through production dispatch.
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
fn replay_typed(source: &str) {
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

fn replay_numeric(source: &str) {
    let evidence: Value = serde_json::from_str(source).unwrap();
    let mut misses = Vec::new();
    for row in evidence["witnesses"].as_array().unwrap() {
        if evidence["ingress_excluded_ids"]
            .as_array()
            .is_some_and(|ids| ids.contains(&row["id"]))
        {
            continue;
        }
        let args: Vec<_> = row["args"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| {
                CalcValue::number(f64::from_bits(
                    u64::from_str_radix(&v.as_str().unwrap()[2..], 16).unwrap(),
                ))
            })
            .collect();
        let got = eval_surface_value_call(
            &format!("FUNC.{}", evidence["function"].as_str().unwrap()),
            &args,
            &oxfunc_core::resolver::NULL_REFERENCE_SYSTEM_PROVIDER,
            None,
            None,
            None,
            None,
        )
        .unwrap_or_else(CalcValue::error);
        let actual = match got.core() {
            CoreValue::Number(n) => format!("0x{:016x}", n.to_bits()),
            CoreValue::Error(e) => format!("error:{e:?}"),
            other => panic!("unexpected {other:?}"),
        };
        if actual != row["expected_bits"].as_str().unwrap() {
            misses.push(format!(
                "{} expected={} actual={actual}",
                row["id"], row["expected_bits"]
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
fn gcd_lcm_typed_initial() { replay_typed(include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/gcd-lcm/typed-initial.json")); }

#[test]
fn gcd_lcm_typed_discovery2() { replay_typed(include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/gcd-lcm/typed-discovery2.json")); }

#[test]
fn gcd_lcm_typed_refinement() { replay_typed(include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/gcd-lcm/typed-refinement.json")); }

#[test]
fn gcd_lcm_typed_array_order() { replay_typed(include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/gcd-lcm/typed-array-order.json")); }

#[test]
fn gcd_lcm_typed_heldout() { replay_typed(include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/gcd-lcm/typed-heldout.json")); }

#[test]
fn gcd_numeric_initial() { replay_numeric(include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/gcd-lcm/numeric-gcd.json")); }

#[test]
fn lcm_numeric_initial() { replay_numeric(include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/gcd-lcm/numeric-lcm.json")); }

#[test]
fn gcd_numeric_discovery2_() { replay_numeric(include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/gcd-lcm/numeric-discovery2-gcd.json")); }

#[test]
fn lcm_numeric_discovery2_() { replay_numeric(include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/gcd-lcm/numeric-discovery2-lcm.json")); }

#[test]
fn gcd_numeric_refinement_() { replay_numeric(include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/gcd-lcm/numeric-refinement-gcd.json")); }

#[test]
fn lcm_numeric_refinement_() { replay_numeric(include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/gcd-lcm/numeric-refinement-lcm.json")); }

#[test]
fn gcd_numeric_heldout_() { replay_numeric(include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/gcd-lcm/numeric-heldout-gcd.json")); }

#[test]
fn lcm_numeric_heldout_() { replay_numeric(include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/gcd-lcm/numeric-heldout-lcm.json")); }
