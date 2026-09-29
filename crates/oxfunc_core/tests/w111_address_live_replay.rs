//! Retained Excel 16.0 build 20430 / CV2 observations, captured through Value2.
//! Coordinate, coercion-selector, numeric formatting and raw text rendering evidence.
//! Numeric-text parsing remains open and has explicitly ignored reproductions.
use oxfunc_core::functions::surface_dispatch::eval_surface_value_call;
use oxfunc_core::resolver::NULL_REFERENCE_SYSTEM_PROVIDER;
use oxfunc_core::resolver::{
    ReferenceDereferenceRequest, ReferenceResolutionError, ReferenceSystemCapabilities,
    ReferenceSystemProvider,
};
use oxfunc_core::value::{
    CalcArray, CalcValue, CoreValue, ExcelText, ReferenceKind, ReferenceLike, WorksheetErrorCode,
};
use serde_json::Value;
use std::collections::BTreeMap;

fn text_value(text: &str) -> CalcValue {
    CalcValue::text(ExcelText::from_utf16_code_units(
        text.encode_utf16().collect(),
    ))
}

fn typed_value(value: &Value) -> CalcValue {
    match value["kind"].as_str().unwrap() {
        "number" => CalcValue::number(value["value"].as_f64().unwrap()),
        "text" => text_value(value["value"].as_str().unwrap()),
        "logical" => CalcValue::logical(value["value"].as_bool().unwrap()),
        "empty_cell" => CalcValue::empty(),
        "missing_arg" => CalcValue::missing(),
        "error" => CalcValue::error(match value["code"].as_str().unwrap() {
            "Div0" => WorksheetErrorCode::Div0,
            "Value" => WorksheetErrorCode::Value,
            "Ref" => WorksheetErrorCode::Ref,
            "NA" => WorksheetErrorCode::NA,
            "Num" => WorksheetErrorCode::Num,
            "Name" => WorksheetErrorCode::Name,
            other => panic!("unexpected error {other}"),
        }),
        "reference" => CalcValue::reference(ReferenceLike::new(
            if value["reference_kind"] == "Area" {
                ReferenceKind::Area
            } else {
                ReferenceKind::A1
            },
            value["target"].as_str().unwrap().to_string(),
        )),
        "array" => CalcValue::array(
            CalcArray::from_rows(
                value["rows"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|row| row.as_array().unwrap().iter().map(typed_value).collect())
                    .collect(),
            )
            .unwrap(),
        ),
        other => panic!("unexpected value kind {other}"),
    }
}

struct FixtureResolver(BTreeMap<String, CalcValue>);
impl ReferenceSystemProvider for FixtureResolver {
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

fn observed(value: &CalcValue) -> Value {
    match value.core() {
        CoreValue::Text(text) => serde_json::json!({"kind":"text","value":text.to_string_lossy()}),
        CoreValue::Error(code) => serde_json::json!({"kind":"error","code":format!("{code:?}")}),
        CoreValue::Array(array) => {
            let rows: Vec<Vec<Value>> = (0..array.shape().rows)
                .map(|r| {
                    (0..array.shape().cols)
                        .map(|c| observed(array.get(r, c).unwrap()))
                        .collect()
                })
                .collect();
            serde_json::json!({"kind":"array","rows":array.shape().rows,"cols":array.shape().cols,"cells":rows})
        }
        other => panic!("unexpected ADDRESS output: {other:?}"),
    }
}

fn without_digest(mut value: Value) -> Value {
    value.as_object_mut().unwrap().remove("digest_payload");
    if value["kind"] == "array" {
        if value["rows"] == 1 && value["cells"][0].is_object() {
            value["cells"] = serde_json::json!([value["cells"].clone()]);
        }
        for row in value["cells"].as_array_mut().unwrap() {
            for cell in row.as_array_mut().unwrap() {
                *cell = without_digest(cell.clone());
            }
        }
    }
    value
}

fn replay_typed(source: &str) {
    let evidence: Value = serde_json::from_str(source).unwrap();
    let mut misses = Vec::new();
    for row in evidence["witnesses"].as_array().unwrap() {
        let case = &row["case"];
        let expected = &row["excel"]["outcome"];
        // Ingress rejections are retained as seam evidence, never function answers.
        if expected["kind"] == "harness_error" {
            continue;
        }
        let resolver = FixtureResolver(
            case["cell_fixture"]
                .as_array()
                .unwrap()
                .iter()
                .map(|fixture| {
                    (
                        fixture["target"].as_str().unwrap().to_string(),
                        typed_value(&fixture["value"]),
                    )
                })
                .collect(),
        );
        let args: Vec<_> = case["args"]
            .as_array()
            .unwrap()
            .iter()
            .map(typed_value)
            .collect();
        let actual =
            eval_surface_value_call("FUNC.ADDRESS", &args, &resolver, None, None, None, None)
                .unwrap_or_else(CalcValue::error);
        let actual = observed(&actual);
        if actual != without_digest(expected.clone()) {
            misses.push(format!(
                "{} expected={} actual={actual}",
                case["case_id"], expected
            ));
        }
    }
    assert!(
        misses.is_empty(),
        "{} differing typed rows:\n{}",
        misses.len(),
        misses.join("\n")
    );
}

fn replay(source: &str) {
    let evidence: serde_json::Value = serde_json::from_str(source).unwrap();
    let mut misses = Vec::new();
    for witness in evidence["witnesses"].as_array().unwrap() {
        if evidence["ingress_excluded_ids"]
            .as_array()
            .is_some_and(|ids| ids.contains(&witness["id"]))
        {
            continue;
        }
        let args = witness["args"]
            .as_array()
            .unwrap()
            .iter()
            .map(|arg| {
                let text = arg.as_str().unwrap();
                if text.len() == 18 && text.starts_with("0x") {
                    let bits = u64::from_str_radix(&text[2..], 16).unwrap();
                    CalcValue::number(f64::from_bits(bits))
                } else {
                    text_value(text)
                }
            })
            .collect::<Vec<_>>();
        let actual = match eval_surface_value_call(
            "FUNC.ADDRESS",
            &args,
            &NULL_REFERENCE_SYSTEM_PROVIDER,
            None,
            None,
            None,
            None,
        ) {
            Err(code) => format!("error:{code:?}"),
            Ok(value) => match value.core {
                CoreValue::Text(text) => format!("text:{}", text.to_string_lossy()),
                CoreValue::Error(code) => format!("error:{code:?}"),
                other => panic!("unexpected ADDRESS result: {other:?}"),
            },
        };
        if actual != witness["expected_bits"].as_str().unwrap() {
            misses.push(format!(
                "{} args={} expected={} actual={actual}",
                witness["id"], witness["args"], witness["expected_bits"]
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
fn address_replays_independent_composed_names() {
    replay(include_str!(
        "../../../docs/function-lane/evidence/w111-broad-20260929/address/grammar-validation.json"
    ));
}

#[test]
fn address_replays_live_coordinate_boundaries() {
    replay(include_str!(
        "../../../docs/function-lane/evidence/w111-broad-20260929/address/boundaries.json"
    ));
}

#[test]
fn address_replays_all_fresh_broad_excel_witnesses() {
    replay(include_str!(
        "../../../docs/function-lane/evidence/w111-broad-20260929/address/discovery.json"
    ));
}

#[test]
fn address_replays_integer_conversion_discriminators() {
    for source in [
        include_str!(
            "../../../docs/function-lane/evidence/w111-broad-20260929/address/rounding.json"
        ),
        include_str!(
            "../../../docs/function-lane/evidence/w111-broad-20260929/address/floor-threshold.json"
        ),
        include_str!(
            "../../../docs/function-lane/evidence/w111-broad-20260929/address/floor-bits.json"
        ),
        include_str!(
            "../../../docs/function-lane/evidence/w111-broad-20260929/address/floor-zoom.json"
        ),
    ] {
        replay(source);
    }
}

#[test]
fn address_replays_typed_errors_sheet_names_and_broadcasting() {
    for source in [
        include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/address/typed.json"),
        include_str!(
            "../../../docs/function-lane/evidence/w111-broad-20260929/address/errors.json"
        ),
        include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/address/sheet.json"),
        include_str!(
            "../../../docs/function-lane/evidence/w111-broad-20260929/address/refinement.json"
        ),
        include_str!(
            "../../../docs/function-lane/evidence/w111-broad-20260929/address/blank-omission.json"
        ),
        include_str!(
            "../../../docs/function-lane/evidence/w111-broad-20260929/address/tokens.json"
        ),
    ] {
        replay_typed(source);
    }
}

#[test]
fn address_replays_numeric_sheet_formatting_and_reference_tokens() {
    for source in [
        include_str!(
            "../../../docs/function-lane/evidence/w111-broad-20260929/address/numeric-sheet.json"
        ),
        include_str!(
            "../../../docs/function-lane/evidence/w111-broad-20260929/address/numeric-format-discriminators.json"
        ),
        include_str!(
            "../../../docs/function-lane/evidence/w111-broad-20260929/address/numeric-ties.json"
        ),
        include_str!(
            "../../../docs/function-lane/evidence/w111-broad-20260929/address/reference-prefix.json"
        ),
    ] {
        replay(source);
    }
}

#[test]
fn address_preserves_admitted_raw_utf16_units() {
    let evidence: Value = serde_json::from_str(include_str!(
        "../../../docs/function-lane/evidence/w111-broad-20260929/address/utf16.json"
    ))
    .unwrap();
    let units = |text: &str| {
        (0..text.len())
            .step_by(4)
            .map(|i| u16::from_str_radix(&text[i..i + 4], 16).unwrap())
            .collect::<Vec<_>>()
    };
    let mut count = 0;
    for row in evidence["witnesses"].as_array().unwrap() {
        if row["ingress_exact"] != true {
            continue;
        }
        let sheet =
            ExcelText::from_utf16_code_units(units(row["input_utf16_hex"].as_str().unwrap()));
        let args = [
            CalcValue::number(1.0),
            CalcValue::number(1.0),
            CalcValue::number(1.0),
            CalcValue::number(1.0),
            CalcValue::text(sheet),
        ];
        let result = eval_surface_value_call(
            "FUNC.ADDRESS",
            &args,
            &NULL_REFERENCE_SYSTEM_PROVIDER,
            None,
            None,
            None,
            None,
        )
        .unwrap();
        let CoreValue::Text(text) = result.core() else {
            panic!("unexpected {result:?}")
        };
        assert_eq!(
            text.utf16_code_units(),
            units(row["result_utf16_hex"].as_str().unwrap()),
            "{}",
            row["id"]
        );
        count += 1;
    }
    assert_eq!(count, 4100);
}

#[test]
fn address_replays_frozen_candidate_heldout() {
    replay(include_str!(
        "../../../docs/function-lane/evidence/w111-broad-20260929/address/refined-heldout.json"
    ));
}

// These are retained failing observations, not accepted equivalence checks.
// They depend on the shared locale/date numeric-text coercion lane, which is
// outside this ADDRESS rendering repair. Run explicitly while repairing it.
#[test]
#[ignore = "BUG-FUNC-052: shared locale/date numeric-text coercion remains open"]
fn address_open_numeric_text_coercion_reproduction() {
    replay_typed(include_str!(
        "../../../docs/function-lane/evidence/w111-broad-20260929/address/numeric-text-syntax.json"
    ));
}

#[test]
#[ignore = "BUG-FUNC-052: shared locale/date numeric-text coercion remains open"]
fn address_open_initial_numeric_text_reproduction() {
    replay_typed(include_str!(
        "../../../docs/function-lane/evidence/w111-broad-20260929/address/blank-numeric-text.json"
    ));
}
