use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use oxfml_core::interface::TypedContextQueryBundle;
use oxfml_core::seam::Locus;
use oxfml_core::semantics::{
    LibraryAvailabilityState, LibraryContextSnapshot, LibraryContextSnapshotEntry,
    RegistrationSourceKind,
};
use oxfml_core::test_support::oxfunc_adapter::{
    OxFuncAdapterRequest, run_oxfunc_preparation_adapter,
};
use oxfunc_core::value::{EvalValue, ExcelText, WorksheetErrorCode};
use serde::Deserialize;

// ---------------------------------------------------------------------------
// Fixture types — identical to OxFml's W050 schema for portability
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct FixtureCase {
    scenario_id: String,
    seam_family: String,
    formula: String,
    caller_row: u32,
    caller_col: u32,
    snapshot_surface_name: String,
    cell_fixture: BTreeMap<String, String>,
    expected_value_summary: String,
    expected_returned_value_surface_kind: String,
    expected_commit_decision_kind: Option<String>,
    expected_reject_code: Option<String>,
    expected_prepared_argument_structures: Option<Vec<String>>,
    expected_prepared_argument_sources: Option<Vec<String>>,
    now_serial: Option<f64>,
    random_value: Option<f64>,
}

// ---------------------------------------------------------------------------
// Test runner core — shared by all fixture-driven tests
// ---------------------------------------------------------------------------

fn run_fixture_corpus(fixtures: &[FixtureCase]) -> Vec<String> {
    let mut failures = Vec::new();

    for fixture in fixtures {
        let provider = oxfml_core::interface::InMemoryLibraryContextProvider::new(test_snapshot(
            &fixture.snapshot_surface_name,
        ));

        let mut request = OxFuncAdapterRequest::new(
            fixture.scenario_id.clone(),
            format!("formula:{}", fixture.scenario_id),
            fixture.formula.clone(),
            locus(fixture.caller_row, fixture.caller_col),
            TypedContextQueryBundle::new(
                None,
                None,
                None,
                fixture.now_serial,
                fixture.random_value,
            ),
        );
        request.library_context_provider = Some(&provider);

        for (target, summary) in &fixture.cell_fixture {
            request
                .cell_fixture
                .insert(target.clone(), parse_eval_value_summary(summary));
        }

        let run = match run_oxfunc_preparation_adapter(request) {
            Ok(run) => run,
            Err(err) => {
                failures.push(format!(
                    "{} [{}] adapter failed: {err}",
                    fixture.scenario_id, fixture.seam_family
                ));
                continue;
            }
        };

        // Check worksheet value
        let actual_value_summary = canonicalize_value_summary(&eval_value_summary(
            &run.evaluation_artifact.worksheet_value,
        ));
        let expected_value_summary = canonicalize_value_summary(&fixture.expected_value_summary);
        if actual_value_summary != expected_value_summary {
            failures.push(format!(
                "{} [{}] worksheet-value mismatch: expected {}, got {}",
                fixture.scenario_id,
                fixture.seam_family,
                fixture.expected_value_summary,
                actual_value_summary
            ));
        }

        // Check return surface kind
        let actual_surface_kind =
            format!("{:?}", run.evaluation_artifact.returned_value_surface.kind);
        if actual_surface_kind != fixture.expected_returned_value_surface_kind {
            failures.push(format!(
                "{} [{}] return surface mismatch: expected {}, got {}",
                fixture.scenario_id,
                fixture.seam_family,
                fixture.expected_returned_value_surface_kind,
                actual_surface_kind
            ));
        }

        // Check prepared argument structures
        if let Some(expected_structures) = &fixture.expected_prepared_argument_structures {
            let Some(prepared_call) = run.preparation_artifact.prepared_calls.first() else {
                failures.push(format!(
                    "{} [{}] expected prepared structures {:?}, but no prepared calls emitted",
                    fixture.scenario_id, fixture.seam_family, expected_structures
                ));
                continue;
            };
            let actual: Vec<String> = prepared_call
                .prepared_arguments
                .iter()
                .map(|arg| format!("{:?}", arg.structure_class))
                .collect();
            if &actual != expected_structures {
                failures.push(format!(
                    "{} [{}] prepared structure mismatch: expected {:?}, got {:?}",
                    fixture.scenario_id, fixture.seam_family, expected_structures, actual
                ));
            }
        }

        // Check prepared argument sources
        if let Some(expected_sources) = &fixture.expected_prepared_argument_sources {
            let Some(prepared_call) = run.preparation_artifact.prepared_calls.first() else {
                failures.push(format!(
                    "{} [{}] expected prepared sources {:?}, but no prepared calls emitted",
                    fixture.scenario_id, fixture.seam_family, expected_sources
                ));
                continue;
            };
            let actual: Vec<String> = prepared_call
                .prepared_arguments
                .iter()
                .map(|arg| format!("{:?}", arg.source_class))
                .collect();
            if &actual != expected_sources {
                failures.push(format!(
                    "{} [{}] prepared source mismatch: expected {:?}, got {:?}",
                    fixture.scenario_id, fixture.seam_family, expected_sources, actual
                ));
            }
        }

        // Check commit decision
        let expected_commit = fixture
            .expected_commit_decision_kind
            .as_deref()
            .unwrap_or("accepted");
        if run.evaluation_artifact.commit_decision_kind != expected_commit {
            failures.push(format!(
                "{} [{}] commit decision mismatch: expected {}, got {}",
                fixture.scenario_id,
                fixture.seam_family,
                expected_commit,
                run.evaluation_artifact.commit_decision_kind
            ));
        }

        // Check reject code
        let actual_reject_code = run
            .evaluation_artifact
            .reject_code
            .map(|code| format!("{code:?}"));
        if actual_reject_code.as_deref() != fixture.expected_reject_code.as_deref() {
            failures.push(format!(
                "{} [{}] reject code mismatch: expected {:?}, got {:?}",
                fixture.scenario_id,
                fixture.seam_family,
                fixture.expected_reject_code,
                actual_reject_code
            ));
        }
    }

    failures
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[test]
fn w050_seam_scenarios_pass_from_oxfunc_side() {
    let fixtures = load_json_fixtures("w050_oxfunc_admitted_fixture_cases.json");
    let failures = run_fixture_corpus(&fixtures);
    assert!(
        failures.is_empty(),
        "W050 seam scenarios diverged when run from OxFunc side:\n{}",
        failures.join("\n")
    );
}

#[test]
fn oxfunc_function_corpus_passes_through_adapter() {
    let fixtures = load_json_fixtures("oxfunc_adapter_function_corpus.json");
    let failures = run_fixture_corpus(&fixtures);
    assert!(
        failures.is_empty(),
        "OxFunc function corpus diverged:\n{}",
        failures.join("\n")
    );
}

#[test]
fn randarray_columns_formula_preserves_generated_width_through_adapter() {
    let run = run_oxfunc_preparation_adapter(OxFuncAdapterRequest::new(
        "randarray-columns-width",
        "formula:randarray-columns-width",
        "=COLUMNS(RANDARRAY(5,3))".to_string(),
        locus(1, 1),
        TypedContextQueryBundle::new(None, None, None, None, Some(0.5)),
    ))
    .expect("randarray columns adapter run");

    assert_eq!(
        run.evaluation_artifact.worksheet_value,
        EvalValue::Number(3.0)
    );
}

#[test]
fn ftc_0601_exact_formula_is_calc_locally_through_adapter() {
    let run = run_oxfunc_preparation_adapter(OxFuncAdapterRequest::new(
        "ftc-0601-exact",
        "formula:ftc-0601-exact",
        "=LET(n,10,seq,SEQUENCE(n),is_prime,MAP(seq,LAMBDA(x,IF(x<2,FALSE,AND(MOD(x,SEQUENCE(MAX(1,INT(SQRT(x)))-1,,2))<>0)))),SUM(--is_prime))".to_string(),
        locus(1, 1),
        TypedContextQueryBundle::default(),
    ))
    .expect("ftc-0601 exact adapter run");

    assert_eq!(
        run.evaluation_artifact.worksheet_value,
        EvalValue::Error(WorksheetErrorCode::Calc)
    );
}

#[test]
fn ftc_0601_map_non_scalar_helper_probe_is_calc_locally_through_adapter() {
    let run = run_oxfunc_preparation_adapter(OxFuncAdapterRequest::new(
        "ftc-0601-map-nonscalar",
        "formula:ftc-0601-map-nonscalar",
        "=MAP(SEQUENCE(10),LAMBDA(x,IF(x<2,FALSE,AND(MOD(x,SEQUENCE(MAX(1,INT(SQRT(x)))-1,,2))<>0))))".to_string(),
        locus(1, 1),
        TypedContextQueryBundle::default(),
    ))
    .expect("ftc-0601 map probe adapter run");

    assert_eq!(
        run.evaluation_artifact.worksheet_value,
        EvalValue::Error(WorksheetErrorCode::Calc)
    );
}

#[test]
fn ftc_0374_skew_exactness_witness_pins_current_local_value_and_excel_gap() {
    let run = run_oxfunc_preparation_adapter(OxFuncAdapterRequest::new(
        "ftc-0374-skew-exactness",
        "formula:ftc-0374-skew-exactness",
        "=SKEW({1,2,2,3,5})".to_string(),
        locus(1, 1),
        TypedContextQueryBundle::default(),
    ))
    .expect("ftc-0374 adapter run");

    let actual = expect_number(&run.evaluation_artifact.worksheet_value);
    let current_local = 1.1180799331493771_f64;
    let excel_target = 1.1180799331493774_f64;

    assert_eq!(actual.to_bits(), current_local.to_bits());
    assert_ne!(actual.to_bits(), excel_target.to_bits());
}

#[test]
fn ftc_0375_kurt_exactness_witness_pins_current_local_value_and_excel_gap() {
    let run = run_oxfunc_preparation_adapter(OxFuncAdapterRequest::new(
        "ftc-0375-kurt-exactness",
        "formula:ftc-0375-kurt-exactness",
        "=KURT({1,2,3,4,5})".to_string(),
        locus(1, 1),
        TypedContextQueryBundle::default(),
    ))
    .expect("ftc-0375 adapter run");

    let actual = expect_number(&run.evaluation_artifact.worksheet_value);
    let current_local = -1.200000000000002_f64;
    let excel_target = -1.1999999999999984_f64;

    assert_eq!(actual.to_bits(), current_local.to_bits());
    assert_ne!(actual.to_bits(), excel_target.to_bits());
}

#[test]
fn ftc_0635_exact_formula_returns_negative_two_locally_through_adapter() {
    let run = run_oxfunc_preparation_adapter(OxFuncAdapterRequest::new(
        "ftc-0635-exact",
        "formula:ftc-0635-exact",
        "=POWER(-8,1/3)".to_string(),
        locus(1, 1),
        TypedContextQueryBundle::default(),
    ))
    .expect("ftc-0635 adapter run");

    assert_eq!(
        run.evaluation_artifact.worksheet_value,
        EvalValue::Number(-2.0)
    );
}

#[test]
fn ftc_0667_upper_sharp_s_matches_local_excel_like_probe_through_adapter() {
    let run = run_oxfunc_preparation_adapter(OxFuncAdapterRequest::new(
        "ftc-0667-upper-sharp-s",
        "formula:ftc-0667-upper-sharp-s",
        "=UPPER(\"straße\")".to_string(),
        locus(1, 1),
        TypedContextQueryBundle::default(),
    ))
    .expect("ftc-0667 adapter run");

    assert_eq!(
        run.evaluation_artifact.worksheet_value,
        EvalValue::Text(ExcelText::from_interop_assignment("STRAßE"))
    );
}

#[test]
fn ftc_1006_exact_formula_returns_201_locally_through_adapter() {
    let run = run_oxfunc_preparation_adapter(OxFuncAdapterRequest::new(
        "ftc-1006-exact",
        "formula:ftc-1006-exact",
        "=LET(Wrap,LAMBDA(z,p,WRAPCOLS(TOCOL(z,,TRUE),p)),IoS,LAMBDA(x,p,LET(w,Wrap(x,2),x0,TAKE(w,1),x1,TAKE(w,-1),y0,Wrap(x0,p),y1,Wrap(x1,p),VSTACK(y0,y1))),data,{1;2;3;4;5;6;7;8},result,IoS(data,2),INDEX(TOCOL(result),1)+INDEX(TOCOL(result),5)*100)".to_string(),
        locus(1, 1),
        TypedContextQueryBundle::default(),
    ))
    .expect("ftc-1006 adapter run");

    assert_eq!(
        run.evaluation_artifact.worksheet_value,
        EvalValue::Number(201.0)
    );
}

#[test]
fn ftc_1007_exact_formula_returns_6_locally_through_adapter() {
    let run = run_oxfunc_preparation_adapter(OxFuncAdapterRequest::new(
        "ftc-1007-exact",
        "formula:ftc-1007-exact",
        "=LET(x,HSTACK(3,0,1,0),Re,LAMBDA(z,TAKE(z,,COLUMNS(z)/2)),Im,LAMBDA(z,TAKE(z,,-COLUMNS(z)/2)),x0,TAKE(x,1),x1,TAKE(x,-1),y0Re,INDEX(Re(x0),1,1)+INDEX(Re(x1),1,1),y1Re,INDEX(Re(x0),1,1)-INDEX(Re(x1),1,1),y0Re+y1Re*100)".to_string(),
        locus(1, 1),
        TypedContextQueryBundle::default(),
    ))
    .expect("ftc-1007 adapter run");

    assert_eq!(
        run.evaluation_artifact.worksheet_value,
        EvalValue::Number(6.0)
    );
}

#[test]
fn worksheet_text_casing_family_matches_excel_observed_matrix_through_adapter() {
    let cases = [
        ("upper-strasse", "=UPPER(\"straße\")", "STRAßE"),
        ("lower-strasse-cap-sharp-s", "=LOWER(\"STRAẞE\")", "straẞe"),
        ("proper-strasse", "=PROPER(\"straße\")", "Straße"),
        ("upper-weiss", "=UPPER(\"weiß\")", "WEIß"),
        ("upper-istanbul-dotted", "=UPPER(\"İstanbul\")", "İSTANBUL"),
        ("lower-istanbul-dotted", "=LOWER(\"İSTANBUL\")", "istanbul"),
        ("upper-istanbul-ascii", "=UPPER(\"istanbul\")", "ISTANBUL"),
        ("lower-i-ascii", "=LOWER(\"I\")", "i"),
        ("lower-i-dotted", "=LOWER(\"İ\")", "i"),
        ("upper-kosmos", "=UPPER(\"κόσμος\")", "ΚΟΣΜΟΣ"),
        ("lower-greek-final-sigma", "=LOWER(\"ΟΣ\")", "ος"),
        ("upper-cafe", "=UPPER(\"café\")", "CAFÉ"),
    ];

    for (name, formula, expected) in cases {
        let run = run_oxfunc_preparation_adapter(OxFuncAdapterRequest::new(
            format!("worksheet-text-casing-{name}"),
            format!("formula:worksheet-text-casing-{name}"),
            formula.to_string(),
            locus(1, 1),
            TypedContextQueryBundle::default(),
        ))
        .unwrap_or_else(|_| panic!("worksheet text casing adapter run should succeed: {name}"));

        assert_eq!(
            run.evaluation_artifact.worksheet_value,
            EvalValue::Text(ExcelText::from_interop_assignment(expected)),
            "{name}"
        );
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn load_json_fixtures(filename: &str) -> Vec<FixtureCase> {
    let mut path = fixture_dir();
    path.push(filename);
    let content = fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("fixture file should exist: {}", path.display()));
    serde_json::from_str(&content)
        .unwrap_or_else(|e| panic!("fixture file should deserialize: {} — {e}", path.display()))
}

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
}

fn parse_eval_value_summary(summary: &str) -> EvalValue {
    if let Some(number) = summary
        .strip_prefix("Number(")
        .and_then(|rest| rest.strip_suffix(')'))
    {
        return EvalValue::Number(number.parse().expect("numeric summary should parse"));
    }
    if let Some(text) = summary
        .strip_prefix("Text(")
        .and_then(|rest| rest.strip_suffix(')'))
    {
        return EvalValue::Text(ExcelText::from_utf16_code_units(
            text.encode_utf16().collect(),
        ));
    }
    if let Some(logical) = summary
        .strip_prefix("Logical(")
        .and_then(|rest| rest.strip_suffix(')'))
    {
        return EvalValue::Logical(matches!(logical, "TRUE" | "True" | "true"));
    }
    if let Some(code) = summary
        .strip_prefix("Error(")
        .and_then(|rest| rest.strip_suffix(')'))
    {
        let code = match code {
            "#NULL!" => WorksheetErrorCode::Null,
            "#DIV/0!" => WorksheetErrorCode::Div0,
            "#VALUE!" => WorksheetErrorCode::Value,
            "#REF!" => WorksheetErrorCode::Ref,
            "#NAME?" => WorksheetErrorCode::Name,
            "#NUM!" => WorksheetErrorCode::Num,
            "#N/A" => WorksheetErrorCode::NA,
            "#CALC!" => WorksheetErrorCode::Calc,
            "#SPILL!" => WorksheetErrorCode::Spill,
            other => panic!("unsupported error summary: {other}"),
        };
        return EvalValue::Error(code);
    }

    panic!("unsupported cell summary: {summary}");
}

fn eval_value_summary(value: &EvalValue) -> String {
    match value {
        EvalValue::Number(n) => format!("Number({n})"),
        EvalValue::Text(t) => format!("Text({})", t.to_string_lossy()),
        EvalValue::Logical(b) => format!("Logical({b})"),
        EvalValue::Error(code) => format!("Error({})", worksheet_error_summary(*code)),
        EvalValue::Array(array) => {
            let shape = array.shape();
            format!("Array({}x{})", shape.rows, shape.cols)
        }
        EvalValue::Reference(reference) => format!("Reference({})", reference.target),
        EvalValue::Lambda(lambda) => format!("Lambda({})", lambda.callable_token),
    }
}

fn worksheet_error_summary(code: WorksheetErrorCode) -> &'static str {
    match code {
        WorksheetErrorCode::Null => "#NULL!",
        WorksheetErrorCode::Div0 => "#DIV/0!",
        WorksheetErrorCode::Value => "#VALUE!",
        WorksheetErrorCode::Ref => "#REF!",
        WorksheetErrorCode::Name => "#NAME?",
        WorksheetErrorCode::Num => "#NUM!",
        WorksheetErrorCode::NA => "#N/A",
        WorksheetErrorCode::Busy => "#BUSY!",
        WorksheetErrorCode::GettingData => "#GETTING_DATA",
        WorksheetErrorCode::Spill => "#SPILL!",
        WorksheetErrorCode::Calc => "#CALC!",
        WorksheetErrorCode::Field => "#FIELD!",
        WorksheetErrorCode::Blocked => "#BLOCKED!",
        WorksheetErrorCode::Connect => "#CONNECT!",
    }
}

fn expect_number(value: &EvalValue) -> f64 {
    match value {
        EvalValue::Number(n) => *n,
        other => panic!("expected numeric worksheet value, got {other:?}"),
    }
}

fn canonicalize_value_summary(summary: &str) -> String {
    let Some(code) = summary
        .strip_prefix("Error(")
        .and_then(|rest| rest.strip_suffix(')'))
    else {
        return summary.to_string();
    };

    let canonical = match code {
        "#NULL!" | "Null" => "#NULL!",
        "#DIV/0!" | "Div0" => "#DIV/0!",
        "#VALUE!" | "Value" => "#VALUE!",
        "#REF!" | "Ref" => "#REF!",
        "#NAME?" | "Name" => "#NAME?",
        "#NUM!" | "Num" => "#NUM!",
        "#N/A" | "NA" => "#N/A",
        "#CALC!" | "Calc" => "#CALC!",
        "#SPILL!" | "Spill" => "#SPILL!",
        "#BUSY!" | "Busy" => "#BUSY!",
        "#GETTING_DATA" | "GettingData" => "#GETTING_DATA",
        "#FIELD!" | "Field" => "#FIELD!",
        "#BLOCKED!" | "Blocked" => "#BLOCKED!",
        "#CONNECT!" | "Connect" => "#CONNECT!",
        other => other,
    };

    format!("Error({canonical})")
}

fn locus(row: u32, col: u32) -> Locus {
    Locus {
        sheet_id: "sheet:default".to_string(),
        row,
        col,
    }
}

fn test_snapshot(surface_name: &str) -> LibraryContextSnapshot {
    LibraryContextSnapshot {
        snapshot_id: "oxfunc-libctx-v1".to_string(),
        snapshot_version: "2026-03-22".to_string(),
        entries: vec![LibraryContextSnapshotEntry {
            surface_name: surface_name.to_string(),
            canonical_id: Some(format!("FUNC.{surface_name}")),
            surface_stable_id: Some(format!("surface:{surface_name}")),
            name_resolution_table_ref: Some("name-table:v1".to_string()),
            semantic_trait_profile_ref: Some("traits:v1".to_string()),
            gating_profile_ref: Some("gating:v1".to_string()),
            metadata_status: Some("runtime".to_string()),
            special_interface_kind: None,
            admission_interface_kind: Some("ordinary".to_string()),
            preparation_owner: Some("oxfunc".to_string()),
            runtime_boundary_kind: Some("ordinary_eval".to_string()),
            arity_shape_note: None,
            interface_contract_ref: Some("iface:v1".to_string()),
            registration_source_kind: RegistrationSourceKind::BuiltIn,
            parse_bind_state: LibraryAvailabilityState::CatalogKnown,
            semantic_plan_state: LibraryAvailabilityState::CatalogKnown,
            runtime_capability_state: Some(LibraryAvailabilityState::CatalogKnown),
            post_dispatch_state: Some(LibraryAvailabilityState::CatalogKnown),
        }],
    }
}
