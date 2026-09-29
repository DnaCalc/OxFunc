//! QUOTIENT prepared-value controls; numerical bits and output shapes remain exact.
use oxfunc_core::functions::quotient_fn::{eval_quotient_surface, map_quotient_error_to_ws};
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
        "number" => CalcValue::number(raw["value"].as_f64().unwrap()),
        "text" => CalcValue::text(ExcelText::from_utf16_code_units(
            raw["value"].as_str().unwrap().encode_utf16().collect(),
        )),
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
            other => panic!("unknown {other}"),
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
            if raw["reference_kind"] == "Area" {
                ReferenceKind::Area
            } else {
                ReferenceKind::A1
            },
            raw["target"].as_str().unwrap().to_owned(),
        )),
        other => panic!("unknown input {other}"),
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
                target: request.reference.target().to_owned(),
            })
    }
}
fn digest(value: &CalcValue) -> String {
    match value.core() {
        CoreValue::Number(n) => format!("number:0x{:016x}", n.to_bits()),
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
fn replay(raw: &str, count: usize) {
    let bank: Value = serde_json::from_str(raw).unwrap();
    let rows = bank["witnesses"].as_array().unwrap();
    assert_eq!(rows.len(), count);
    let mut misses = Vec::new();
    for row in rows {
        let case = &row["case"];
        assert_eq!(row["excel"]["execution_status"], "ok");
        let args: Vec<_> = case["args"].as_array().unwrap().iter().map(value).collect();
        let fixtures = Fixtures(
            case["cell_fixture"]
                .as_array()
                .unwrap()
                .iter()
                .map(|f| (f["target"].as_str().unwrap().to_owned(), value(&f["value"])))
                .collect(),
        );
        let actual =
            eval_surface_value_call("FUNC.QUOTIENT", &args, &fixtures, None, None, None, None)
                .unwrap_or_else(CalcValue::error);
        let direct = eval_quotient_surface(&args, &fixtures)
            .unwrap_or_else(|e| CalcValue::error(map_quotient_error_to_ws(&e)));
        let expected = row["excel"]["outcome"]["digest_payload"].as_str().unwrap();
        for (path, result) in [("dispatch", actual), ("direct", direct)] {
            if digest(&result) != expected {
                misses.push(format!(
                    "{} {path}: expected {expected}; actual {}",
                    case["case_id"],
                    digest(&result)
                ));
            }
        }
    }
    assert!(
        misses.is_empty(),
        "{} differences:\n{}",
        misses.len(),
        misses.join("\n")
    );
}
#[test]
fn quotient_reference_discovery_matches_public_and_direct_surfaces() {
    replay(
        include_str!(
            "../../../docs/function-lane/evidence/w111-broad-20260929/quotient/reference-discovery.json"
        ),
        80,
    );
}

#[test]
fn quotient_reference_first_heldout_matches_positional_refinement() {
    replay(
        include_str!(
            "../../../docs/function-lane/evidence/w111-broad-20260929/quotient/reference-heldout.json"
        ),
        512,
    );
}

#[test]
fn quotient_positional_refinement_matches_fresh_independent_shapes() {
    replay(
        include_str!(
            "../../../docs/function-lane/evidence/w111-broad-20260929/quotient/padding-heldout.json"
        ),
        624,
    );
}

#[test]
fn quotient_prior_typed_discovery_matches_public_and_direct_surfaces() {
    replay(
        include_str!(
            "../../../docs/function-lane/evidence/w111-broad-20260929/quotient/typed.json"
        ),
        216,
    );
}

#[test]
fn quotient_prior_typed_validation_matches_public_and_direct_surfaces() {
    replay(
        include_str!(
            "../../../docs/function-lane/evidence/w111-broad-20260929/quotient/typed-validation.json"
        ),
        288,
    );
}
