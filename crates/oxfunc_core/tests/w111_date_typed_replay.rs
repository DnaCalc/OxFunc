//! Exact replay of live typed date observations, including blank/reference/array origins.
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
        other => panic!("unexpected date output {other:?}"),
    }
}

fn unsupported_weeknum_selector(value: &Value, fixtures: &BTreeMap<String, CalcValue>) -> bool {
    let value = typed(value);
    fn inspect(value: &CalcValue, fixtures: &BTreeMap<String, CalcValue>) -> bool {
        match value.core() {
            CoreValue::Reference(r) => inspect(fixtures.get(r.target()).unwrap(), fixtures),
            CoreValue::Array(a) => a.iter_row_major().any(|v| inspect(v, fixtures)),
            CoreValue::Empty => true,
            CoreValue::Number(n) => unsupported(*n),
            CoreValue::Text(t) => t.to_string_lossy().parse::<f64>().ok().is_some_and(unsupported),
            _ => false,
        }
    }
    fn unsupported(n: f64) -> bool {
        let upper = n.ceil();
        let threshold = if upper > 0.0 { 2.0_f64.powi(-22) + 2.0_f64.powi(-33) }
            else { 2.0_f64.powi(-23) + 2.0_f64.powi(-34) };
        let selector = if upper - n <= threshold { upper } else { n.floor() } as i64;
        !matches!(selector, 1 | 2 | 11..=17 | 21)
    }
    inspect(&value, fixtures)
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
        // Repeated live answers contradict each other for unsupported WEEKNUM
        // selectors. Preserve these observations without treating one sample as
        // a deterministic parity target, including blank/reference/array slots.
        if case["function_id"] == "FUNC.WEEKNUM" && case["args"].as_array().unwrap().len() > 1
            && unsupported_weeknum_selector(&case["args"][1], &resolver.0) {
            withheld += 1;
            continue;
        }
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
fn date_live_typed_discovery() {
    replay(include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/dates/typed-first-bank.json"),165,8);
}

#[test]
fn date_live_typed_refinement() {
    replay(include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/dates/typed-refinement-bank.json"),1110,27);
}

#[test]
fn date_live_typed_second_heldout() {
    replay(include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/dates/typed-heldout2-bank.json"),780,0);
}

#[test]
fn date_live_typed_array_independent_heldout() {
    replay(include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/dates/typed-array-heldout-bank.json"),720,0);
}
