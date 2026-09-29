//! Retained elementary prepared-value and optional LOG observations.
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
fn coordinate(target: &str) -> Option<(usize, usize)> {
    let split = target.find(|c: char| c.is_ascii_digit())?;
    let column = target[..split].bytes().try_fold(0usize, |acc, c| {
        c.is_ascii_uppercase()
            .then(|| acc * 26 + usize::from(c - b'A' + 1))
    })?;
    Some((target[split..].parse().ok()?, column))
}
impl ReferenceSystemProvider for Fixtures {
    fn capabilities(&self) -> ReferenceSystemCapabilities {
        ReferenceSystemCapabilities::permissive_local()
    }
    fn dereference(
        &self,
        request: &ReferenceDereferenceRequest,
    ) -> Result<CalcValue, ReferenceResolutionError> {
        let target = request.reference.target();
        if let Some(value) = self.0.get(target) {
            return Ok(value.clone());
        }
        // Fresh aggregate controls also declare every cell separately. Assemble
        // only a fully supplied rectangle; absent fixtures remain unresolved.
        let assembled = (|| {
            let (first, last) = target.split_once(':')?;
            let (r0, c0) = coordinate(first)?;
            let (r1, c1) = coordinate(last)?;
            if r1 < r0 || c1 < c0 {
                return None;
            }
            let cells: BTreeMap<_, _> = self
                .0
                .iter()
                .filter_map(|(key, value)| coordinate(key).map(|c| (c, value)))
                .collect();
            let rows: Option<Vec<Vec<CalcValue>>> = (r0..=r1)
                .map(|r| {
                    (c0..=c1)
                        .map(|c| cells.get(&(r, c)).map(|v| (*v).clone()))
                        .collect()
                })
                .collect();
            Some(CalcValue::array(CalcArray::from_rows(rows?)?))
        })();
        assembled.ok_or_else(|| ReferenceResolutionError::UnresolvedReference {
            target: target.to_string(),
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
        // The public entry above reaches the generated catalog-index table.
        // Also exercise the function's direct surface so route drift is visible.
        let direct = match case["canonical_surface_name"].as_str().unwrap() {
            "POWER" => oxfunc_core::functions::power_fn::eval_power_surface(&args, &fixtures)
                .map_err(|e| oxfunc_core::functions::power_fn::map_power_error_to_ws(&e)),
            "LOG" => oxfunc_core::functions::log_fn::eval_log_surface(&args, &fixtures)
                .map_err(|e| oxfunc_core::functions::log_fn::map_log_error_to_ws(&e)),
            "MEDIAN" => oxfunc_core::functions::median_fn::eval_median_surface(&args, &fixtures)
                .map_err(|e| oxfunc_core::functions::median_fn::map_median_error_to_ws(&e)),
            "FISHER" => oxfunc_core::functions::fisher_fn::eval_fisher_surface(&args, &fixtures)
                .map_err(|e| oxfunc_core::functions::fisher_fn::map_fisher_error_to_ws(&e)),
            "HARMEAN" => oxfunc_core::functions::harmean_fn::eval_harmean_surface(&args, &fixtures)
                .map_err(|e| oxfunc_core::functions::harmean_fn::map_harmean_error_to_ws(&e)),
            "DEVSQ" => oxfunc_core::functions::devsq_fn::eval_devsq_surface(&args, &fixtures)
                .map_err(|e| oxfunc_core::functions::devsq_fn::map_devsq_error_to_ws(&e)),
            other => panic!("unexpected surface {other}"),
        }
        .unwrap_or_else(CalcValue::error);
        let expected = row["excel"]["outcome"]["digest_payload"].as_str().unwrap();
        if digest(&direct) != expected {
            misses.push(format!(
                "{} direct expected={expected} actual={}",
                case["case_id"],
                digest(&direct)
            ));
        }
        let dispatched = digest(&got);
        if dispatched != expected {
            misses.push(format!(
                "{} generated expected={expected} actual={dispatched}",
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
fn elementary_prepared_discovery_matches_live() {
    replay(include_str!(
        "../../../docs/function-lane/evidence/w111-broad-20260929/elementary-publication/prepared-discovery.json"
    ));
}

#[test]
fn elementary_first_heldout_and_median_origin_refinement_match_live() {
    replay(include_str!(
        "../../../docs/function-lane/evidence/w111-broad-20260929/elementary-publication/prepared-heldout.json"
    ));
    replay(include_str!(
        "../../../docs/function-lane/evidence/w111-broad-20260929/elementary-publication/median-error-origin.json"
    ));
}

#[test]
fn aggregate_origin_refinement_matches_serialized_independent_live_cases() {
    replay(include_str!(
        "../../../docs/function-lane/evidence/w111-broad-20260929/elementary-publication/aggregate-origin-heldout.json"
    ));
}

#[test]
fn log_omitted_base_and_explicit_base_follow_distinct_live_graphs() {
    let sources = [
        include_str!(
            "../../../docs/function-lane/evidence/w111-broad-20260929/elementary-publication/omitted-base/answers-log.json"
        ),
        include_str!(
            "../../../docs/function-lane/evidence/w111-broad-20260929/elementary-publication/omitted-base/answers-log10.json"
        ),
        include_str!(
            "../../../docs/function-lane/evidence/w111-broad-20260929/elementary-publication/prepared-first-heldout/answers-log.json"
        ),
        include_str!(
            "../../../docs/function-lane/evidence/w111-broad-20260929/elementary-publication/prepared-first-heldout/answers-log10.json"
        ),
    ];
    for source in sources {
        let evidence: Value = serde_json::from_str(source).unwrap();
        let function = format!("FUNC.{}", evidence["function"].as_str().unwrap());
        for row in evidence["witnesses"].as_array().unwrap() {
            let args: Vec<_> = row["args"]
                .as_array()
                .unwrap()
                .iter()
                .map(|raw| {
                    let bits =
                        u64::from_str_radix(raw.as_str().unwrap().trim_start_matches("0x"), 16)
                            .unwrap();
                    CalcValue::number(f64::from_bits(bits))
                })
                .collect();
            let got = eval_surface_value_call(
                &function,
                &args,
                &Fixtures(BTreeMap::new()),
                None,
                None,
                None,
                None,
            )
            .unwrap_or_else(CalcValue::error);
            let expected = row["expected_bits"].as_str().unwrap();
            let expected = if expected.starts_with("0x") {
                format!("number:{expected}")
            } else {
                expected.to_string()
            };
            assert_eq!(digest(&got), expected, "{}", row["id"]);
        }
    }
}
