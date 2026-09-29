//! Prepared distribution coercion/lifting; numerical residuals remain explicitly pinned.
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

fn error(code: &str) -> WorksheetErrorCode {
    match code {
        "NA" => WorksheetErrorCode::NA,
        "Value" => WorksheetErrorCode::Value,
        "Div0" => WorksheetErrorCode::Div0,
        "Num" => WorksheetErrorCode::Num,
        "Ref" => WorksheetErrorCode::Ref,
        "Name" => WorksheetErrorCode::Name,
        "Null" => WorksheetErrorCode::Null,
        other => panic!("unknown error {other}"),
    }
}
fn value(raw: &Value) -> CalcValue {
    match raw["kind"].as_str().unwrap() {
        "text" => CalcValue::text(ExcelText::from_utf16_code_units(
            raw["value"].as_str().unwrap().encode_utf16().collect(),
        )),
        "number" => CalcValue::number(raw["value"].as_f64().unwrap()),
        "logical" => CalcValue::logical(raw["value"].as_bool().unwrap()),
        "empty_cell" => CalcValue::empty(),
        "missing_arg" => CalcValue::missing(),
        "error" => CalcValue::error(error(raw["code"].as_str().unwrap())),
        "reference" => CalcValue::reference(ReferenceLike::new(
            if raw["reference_kind"] == "Area" {
                ReferenceKind::Area
            } else {
                ReferenceKind::A1
            },
            raw["target"].as_str().unwrap().to_owned(),
        )),
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
        other => panic!("unsupported {other}"),
    }
}
struct Fixtures(BTreeMap<String, CalcValue>);
impl Fixtures {
    fn value_at(&self, target: &str) -> Option<CalcValue> {
        use oxfunc_core::functions::a1_refs::parse_a1_reference;
        if let Some(value) = self.0.get(target) {
            return Some(value.clone());
        }
        let requested = parse_a1_reference(target)?;
        let mut rows = Vec::new();
        for r in requested.start_row..=requested.end_row {
            let mut row = Vec::new();
            for c in requested.start_col..=requested.end_col {
                let mut found = None;
                for (target, value) in &self.0 {
                    let Some(area) = parse_a1_reference(target) else {
                        continue;
                    };
                    if area.prefix != requested.prefix
                        || r < area.start_row
                        || r > area.end_row
                        || c < area.start_col
                        || c > area.end_col
                    {
                        continue;
                    }
                    let item = match value.core() {
                        CoreValue::Array(array) => {
                            array.get(r - area.start_row, c - area.start_col)?.clone()
                        }
                        _ if area.height() == 1 && area.width() == 1 => value.clone(),
                        _ => return None,
                    };
                    if let Some(previous) = &found {
                        assert_eq!(previous, &item, "conflicting fixture");
                    }
                    found = Some(item);
                }
                row.push(found?);
            }
            rows.push(row);
        }
        CalcArray::from_rows(rows).map(CalcValue::array)
    }
}
impl ReferenceSystemProvider for Fixtures {
    fn capabilities(&self) -> ReferenceSystemCapabilities {
        ReferenceSystemCapabilities::permissive_local()
    }
    fn dereference(
        &self,
        request: &ReferenceDereferenceRequest,
    ) -> Result<CalcValue, ReferenceResolutionError> {
        self.value_at(request.reference.target()).ok_or_else(|| {
            ReferenceResolutionError::UnresolvedReference {
                target: request.reference.target().to_owned(),
            }
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
        other => panic!("unsupported result {other:?}"),
    }
}
fn replay(
    raw_cases: &str,
    raw_oracle: &str,
    raw_open: &str,
    expected_counts: (usize, usize, usize),
) {
    let cases: Value = serde_json::from_str(raw_cases).unwrap();
    let oracle: BTreeMap<String, Value> = raw_oracle
        .lines()
        .map(|line| {
            let r: Value = serde_json::from_str(line).unwrap();
            (r["case_id"].as_str().unwrap().to_owned(), r)
        })
        .collect();
    let open: Value = serde_json::from_str(raw_open).unwrap();
    let known: BTreeMap<String, &Value> = open["misses"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| (r["case_id"].as_str().unwrap().to_owned(), r))
        .collect();
    let (mut exact, mut kernel_open, mut input_limited) = (0, 0, 0);
    for case in cases["cases"].as_array().unwrap() {
        let id = case["case_id"].as_str().unwrap();
        let expected = &oracle[id];
        assert_eq!(case["function_id"], expected["function_id"]);
        assert_eq!(case["formula_text"], expected["formula_text"]);
        if expected["execution_status"] != "ok" {
            assert_eq!(expected["execution_status"], "excel_case_harness_error");
            let invalid_stored_missing = case["cell_fixture"]
                .as_array()
                .unwrap()
                .iter()
                .any(|f| f["value"]["kind"] == "missing_arg");
            let unsupported_empty_unary_call = case["args"].as_array().unwrap().len() == 1
                && case["args"][0]["kind"] == "missing_arg"
                && case["formula_text"].as_str().unwrap().ends_with("()");
            assert!(invalid_stored_missing || unsupported_empty_unary_call);
            input_limited += 1;
            continue;
        }
        let args: Vec<_> = case["args"].as_array().unwrap().iter().map(value).collect();
        let fixtures = Fixtures(
            case["cell_fixture"]
                .as_array()
                .unwrap()
                .iter()
                .map(|f| (f["target"].as_str().unwrap().to_owned(), value(&f["value"])))
                .collect(),
        );
        let actual = eval_surface_value_call(
            case["function_id"].as_str().unwrap(),
            &args,
            &fixtures,
            None,
            None,
            None,
            None,
        )
        .unwrap_or_else(CalcValue::error);
        let actual = digest(&actual);
        let expected = expected["outcome"]["digest_payload"].as_str().unwrap();
        if let Some(residual) = known.get(id) {
            // This row is an explicit open arithmetic/domain discrepancy, not an Excel match.
            assert_eq!(expected, residual["expected"]);
            assert_eq!(actual, residual["actual"], "changed residual {id}");
            assert_ne!(actual, expected);
            kernel_open += 1;
        } else {
            assert_eq!(actual, expected, "{id}: {}", case["formula_text"]);
            exact += 1;
        }
    }
    assert_eq!((exact, kernel_open, input_limited), expected_counts);
}
#[test]
fn distribution_prepared_discovery_matches_or_keeps_known_numeric_residuals() {
    replay(include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/distribution-surface/discovery-cases.json"),
        include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/distribution-surface/discovery-excel.jsonl"),
        include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/distribution-surface/discovery-current-judge.json"),(129,3,0));
}

#[test]
fn distribution_logical_grammar_origins_and_shape_controls_match() {
    replay(include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/distribution-surface/surface-cases.json"),
        include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/distribution-surface/surface-excel.jsonl"),
        include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/distribution-surface/surface-current-judge.json"),(1064,3,16));
}

#[test]
fn distribution_fresh_flags_and_padding_error_order_match() {
    replay(include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/distribution-surface/heldout-cases.json"),
        include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/distribution-surface/heldout-excel.jsonl"),
        include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/distribution-surface/heldout-refined-judge.json"),(1296,0,0));
}

#[test]
fn distribution_independent_padding_positions_preserve_coercion_order() {
    replay(include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/distribution-surface/padding-heldout-cases.json"),
        include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/distribution-surface/padding-heldout-excel.jsonl"),
        include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/distribution-surface/padding-heldout-judge.json"),(912,0,0));
}

#[test]
fn adjacent_distribution_preparation_preserves_known_kernel_discrepancies() {
    replay(include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/distribution-surface/adjacent-cases.json"),
        include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/distribution-surface/adjacent-excel.jsonl"),
        include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/distribution-surface/adjacent-domain-refined-judge.json"),(1493,121,6));
}

#[test]
fn distribution_prepared_holdout_preserves_open_numeric_differences() {
    replay(include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/distribution-surface/prepared-heldout-cases.json"),
        include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/distribution-surface/prepared-heldout-excel.jsonl"),
        include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/distribution-surface/prepared-heldout-current-judge.json"),(2261,280,0));
}

#[test]
fn distribution_unit_array_native_metadata_preserves_padding_order() {
    replay(include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/distribution-surface/unit-padding-cases.json"),
        include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/distribution-surface/unit-padding-excel.jsonl"),
        include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/distribution-surface/unit-padding-current-judge.json"),(108,24,0));
}

#[test]
fn distribution_independent_unit_arrays_preserve_order_and_open_numeric_results() {
    replay(include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/distribution-surface/unit-heldout-cases.json"),
        include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/distribution-surface/unit-heldout-excel.jsonl"),
        include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/distribution-surface/unit-heldout-judge.json"),(552,216,0));
}
