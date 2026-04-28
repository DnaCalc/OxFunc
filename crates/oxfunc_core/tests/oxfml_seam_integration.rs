use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use oxfml_core::format::current_excel_host_context;
use oxfml_core::interface::TypedContextQueryBundle;
use oxfml_core::seam::Locus;
use oxfml_core::semantics::{
    LibraryAvailabilityState, LibraryContextSnapshot, LibraryContextSnapshotEntry,
    RegistrationSourceKind,
};
use oxfml_core::test_support::oxfunc_adapter::{
    OxFuncAdapterRequest, run_oxfunc_preparation_adapter,
};
use oxfunc_core::value::{ArrayCellValue, EvalArray, EvalValue, ExcelText, WorksheetErrorCode};
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
fn ftc_0601_map_non_scalar_helper_probe_matches_current_local_mask_through_adapter() {
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
        EvalValue::Array(
            EvalArray::from_rows(vec![
                vec![ArrayCellValue::Logical(false)],
                vec![ArrayCellValue::Error(WorksheetErrorCode::Calc)],
                vec![ArrayCellValue::Error(WorksheetErrorCode::Calc)],
                vec![ArrayCellValue::Logical(false)],
                vec![ArrayCellValue::Logical(true)],
                vec![ArrayCellValue::Logical(false)],
                vec![ArrayCellValue::Logical(true)],
                vec![ArrayCellValue::Logical(false)],
                vec![ArrayCellValue::Logical(false)],
                vec![ArrayCellValue::Logical(false)],
            ])
            .unwrap(),
        )
    );
}

#[test]
fn ftc_0910_exact_formula_matches_scalar_330_through_adapter() {
    let run = run_oxfunc_preparation_adapter(OxFuncAdapterRequest::new(
        "ftc-0910-exact",
        "formula:ftc-0910-exact",
        "=LET(d,{10,20,30,40,50,60,70,80,90,100},w,5,n,COLUMNS(d),avgs,MAP(SEQUENCE(n-w+1,,w),LAMBDA(i,AVERAGE(INDEX(d,,SEQUENCE(w,,i-w+1))))),SUM(avgs))".to_string(),
        locus(1, 1),
        TypedContextQueryBundle::default(),
    ))
    .expect("ftc-0910 exact adapter run");

    assert_eq!(
        run.evaluation_artifact.worksheet_value,
        EvalValue::Number(330.0)
    );
}

#[test]
fn ftc_0702_day_of_date_1900_march_zero_matches_29_through_adapter() {
    let run = run_oxfunc_preparation_adapter(OxFuncAdapterRequest::new(
        "ftc-0702-day-date-1900-march-zero",
        "formula:ftc-0702-day-date-1900-march-zero",
        "=DAY(DATE(1900,3,0))".to_string(),
        locus(1, 1),
        TypedContextQueryBundle::default(),
    ))
    .expect("ftc-0702 adapter run");

    assert_eq!(
        run.evaluation_artifact.worksheet_value,
        EvalValue::Number(29.0)
    );
}

#[test]
fn ftc_0640_len_emoji_matches_one_through_adapter() {
    let run = run_oxfunc_preparation_adapter(OxFuncAdapterRequest::new(
        "ftc-0640-len-emoji",
        "formula:ftc-0640-len-emoji",
        "=LEN(\"😀\")".to_string(),
        locus(1, 1),
        TypedContextQueryBundle::default(),
    ))
    .expect("ftc-0640 adapter run");

    assert_eq!(
        run.evaluation_artifact.worksheet_value,
        EvalValue::Number(1.0)
    );
}

#[test]
fn ftc_0696_text_serial_zero_date_format_matches_excel_compat_string_through_adapter() {
    let locale = current_excel_host_context();
    let run = run_oxfunc_preparation_adapter(OxFuncAdapterRequest::new(
        "ftc-0696-text-serial-zero-date-format",
        "formula:ftc-0696-text-serial-zero-date-format",
        "=TEXT(0,\"yyyy-mm-dd\")".to_string(),
        locus(1, 1),
        TypedContextQueryBundle::new(None, None, Some(&locale), None, None),
    ))
    .expect("ftc-0696 adapter run");

    assert_eq!(
        run.evaluation_artifact.worksheet_value,
        EvalValue::Text(ExcelText::from_interop_assignment("1900-01-00"))
    );
}

#[test]
fn ftc_0930_exact_formula_matches_value_error_through_adapter() {
    let run = run_oxfunc_preparation_adapter(OxFuncAdapterRequest::new(
        "ftc-0930-exact",
        "formula:ftc-0930-exact",
        "=LET(m,{3,7,1;8,2,9;4,6,5},flat,TOCOL(m),sorted,SORT(flat,-1),INDEX(sorted,3))"
            .to_string(),
        locus(1, 1),
        TypedContextQueryBundle::default(),
    ))
    .expect("ftc-0930 exact adapter run");

    assert_eq!(
        run.evaluation_artifact.worksheet_value,
        EvalValue::Error(WorksheetErrorCode::Value)
    );
}

#[test]
fn ftc_1032_exact_formula_matches_scalar_zero_through_adapter() {
    let run = run_oxfunc_preparation_adapter(OxFuncAdapterRequest::new(
        "ftc-1032-exact",
        "formula:ftc-1032-exact",
        "=LET(yr,2024,m,1,firstDay,DATE(yr,m,1),lastDay,EOMONTH(firstDay,0),daysInMonth,DAY(lastDay),offset,WEEKDAY(firstDay,1)-1,grid,SEQUENCE(42),dayVals,IF(AND(grid>offset,grid<=offset+daysInMonth),grid-offset,0),weekly,WRAPROWS(dayVals,7),SUM(INDEX(weekly,1,0)))".to_string(),
        locus(1, 1),
        TypedContextQueryBundle::default(),
    ))
    .expect("ftc-1032 exact adapter run");

    assert_eq!(
        run.evaluation_artifact.worksheet_value,
        EvalValue::Number(0.0)
    );
}

#[test]
fn ftc_0353_countblank_let_array_matches_retained_excel_shaped_value_errors_through_adapter() {
    let run = run_oxfunc_preparation_adapter(OxFuncAdapterRequest::new(
        "ftc-0353-countblank-let-array-shape",
        "formula:ftc-0353-countblank-let-array-shape",
        "=LET(d,{\"a\",1,\"\",2,\"b\"},COUNTBLANK(d))".to_string(),
        locus(1, 1),
        TypedContextQueryBundle::default(),
    ))
    .expect("ftc-0353 adapter run");

    assert_eq!(
        run.evaluation_artifact.worksheet_value,
        EvalValue::Array(
            EvalArray::from_rows(vec![vec![
                ArrayCellValue::Error(WorksheetErrorCode::Value),
                ArrayCellValue::Error(WorksheetErrorCode::Value),
                ArrayCellValue::Error(WorksheetErrorCode::Value),
                ArrayCellValue::Error(WorksheetErrorCode::Value),
                ArrayCellValue::Error(WorksheetErrorCode::Value),
            ]])
            .unwrap(),
        )
    );
}

#[test]
fn multinomial_widened_empirical_machine_witnesses_match_excel_targets_through_adapter() {
    for (id, formula, expected) in [
        (
            "ftc-0254-multinomial-exactness",
            "=MULTINOMIAL(2,3,4)",
            1259.9999999999991_f64,
        ),
        (
            "multinomial-anchor-permutation",
            "=MULTINOMIAL(4,2,3)",
            1259.9999999999991_f64,
        ),
        (
            "multinomial-anchor-zero-padded",
            "=MULTINOMIAL(0,2,3,4)",
            1259.9999999999991_f64,
        ),
        (
            "multinomial-two-positive-with-zero",
            "=MULTINOMIAL(0,2,3)",
            9.999999999999998_f64,
        ),
        (
            "multinomial-two-positive-spread",
            "=MULTINOMIAL(2,7)",
            35.99999999999998_f64,
        ),
        (
            "multinomial-four-arg-consecutive",
            "=MULTINOMIAL(1,2,3,4)",
            12599.999999999995_f64,
        ),
        (
            "multinomial-five-sum-consecutive",
            "=MULTINOMIAL(2,3,4,5)",
            2522520.0000000005_f64,
        ),
    ] {
        let run = run_oxfunc_preparation_adapter(OxFuncAdapterRequest::new(
            id,
            &format!("formula:{id}"),
            formula.to_string(),
            locus(1, 1),
            TypedContextQueryBundle::default(),
        ))
        .expect("multinomial widened empirical adapter run");

        let actual = expect_number(&run.evaluation_artifact.worksheet_value);
        assert_eq!(actual.to_bits(), expected.to_bits(), "{formula}");
    }
}

#[test]
fn combinatorial_adjacent_exact_controls_remain_exact_through_adapter() {
    for (id, formula, expected) in [
        (
            "multinomial-factor-expression-exact-control",
            "=FACT(9)/(FACT(2)*FACT(3)*FACT(4))",
            1260.0_f64,
        ),
        ("fact-exact-control", "=FACT(9)", 362880.0_f64),
        ("factdouble-exact-control", "=FACTDOUBLE(9)", 945.0_f64),
        ("combin-exact-control", "=COMBIN(10,3)", 120.0_f64),
        ("combina-exact-control", "=COMBINA(4,3)", 20.0_f64),
        ("permut-exact-control", "=PERMUT(10,3)", 720.0_f64),
        ("permutationa-exact-control", "=PERMUTATIONA(3,2)", 9.0_f64),
    ] {
        let run = run_oxfunc_preparation_adapter(OxFuncAdapterRequest::new(
            id,
            &format!("formula:{id}"),
            formula.to_string(),
            locus(1, 1),
            TypedContextQueryBundle::default(),
        ))
        .expect("combinatorial exact control adapter run");

        let actual = expect_number(&run.evaluation_artifact.worksheet_value);
        assert_eq!(actual.to_bits(), expected.to_bits(), "{formula}");
    }
}

#[test]
fn ftc_0833_index_row_vector_selector_array_direct_call_matches_witness_through_adapter() {
    let run = run_oxfunc_preparation_adapter(OxFuncAdapterRequest::new(
        "ftc-0833-index-vector-selector-direct",
        "formula:ftc-0833-index-vector-selector-direct",
        "=INDEX({10,20,30,40,50},SEQUENCE(3))".to_string(),
        locus(1, 1),
        TypedContextQueryBundle::default(),
    ))
    .expect("ftc-0833 index adapter run");

    assert_eq!(
        run.evaluation_artifact.worksheet_value,
        EvalValue::Array(
            EvalArray::from_rows(vec![
                vec![ArrayCellValue::Number(10.0)],
                vec![ArrayCellValue::Number(20.0)],
                vec![ArrayCellValue::Number(30.0)],
            ])
            .unwrap(),
        )
    );
}

#[test]
fn ftc_0836_sortby_row_vector_multi_key_direct_call_matches_witness_through_adapter() {
    let run = run_oxfunc_preparation_adapter(OxFuncAdapterRequest::new(
        "ftc-0836-sortby-row-vector-direct",
        "formula:ftc-0836-sortby-row-vector-direct",
        "=SORTBY({\"a\",\"b\",\"c\",\"d\"},{2,1,2,1},{1,2,1,2})".to_string(),
        locus(1, 1),
        TypedContextQueryBundle::default(),
    ))
    .expect("ftc-0836 sortby adapter run");

    assert_eq!(
        run.evaluation_artifact.worksheet_value,
        EvalValue::Array(
            EvalArray::from_rows(vec![vec![
                ArrayCellValue::Text(ExcelText::from_interop_assignment("b")),
                ArrayCellValue::Text(ExcelText::from_interop_assignment("d")),
                ArrayCellValue::Text(ExcelText::from_interop_assignment("a")),
                ArrayCellValue::Text(ExcelText::from_interop_assignment("c")),
            ]])
            .unwrap(),
        )
    );
}

#[test]
fn ftc_0917_sort_row_vector_default_axis_direct_call_matches_witness_through_adapter() {
    let run = run_oxfunc_preparation_adapter(OxFuncAdapterRequest::new(
        "ftc-0917-sort-row-vector-direct",
        "formula:ftc-0917-sort-row-vector-direct",
        "=SORT({3,1,4,1,5,9,2,6})".to_string(),
        locus(1, 1),
        TypedContextQueryBundle::default(),
    ))
    .expect("ftc-0917 sort adapter run");

    assert_eq!(
        run.evaluation_artifact.worksheet_value,
        EvalValue::Array(
            EvalArray::from_rows(vec![vec![
                ArrayCellValue::Number(3.0),
                ArrayCellValue::Number(1.0),
                ArrayCellValue::Number(4.0),
                ArrayCellValue::Number(1.0),
                ArrayCellValue::Number(5.0),
                ArrayCellValue::Number(9.0),
                ArrayCellValue::Number(2.0),
                ArrayCellValue::Number(6.0),
            ]])
            .unwrap(),
        )
    );
}

#[test]
fn ftc_0941_and_ftc_0995_isna_xmatch_direct_call_matches_logical_mask_through_adapter() {
    let run = run_oxfunc_preparation_adapter(OxFuncAdapterRequest::new(
        "ftc-0941-0995-isna-xmatch-direct",
        "formula:ftc-0941-0995-isna-xmatch-direct",
        "=ISNA(XMATCH({1,2,3,4,5},{2,4,6,8}))".to_string(),
        locus(1, 1),
        TypedContextQueryBundle::default(),
    ))
    .expect("ftc-0941-0995 isna adapter run");

    assert_eq!(
        run.evaluation_artifact.worksheet_value,
        EvalValue::Array(
            EvalArray::from_rows(vec![vec![
                ArrayCellValue::Logical(true),
                ArrayCellValue::Logical(false),
                ArrayCellValue::Logical(true),
                ArrayCellValue::Logical(false),
                ArrayCellValue::Logical(true),
            ]])
            .unwrap(),
        )
    );
}

#[test]
fn ftc_0399_disc_exactness_witness_matches_excel_target_through_adapter() {
    let run = run_oxfunc_preparation_adapter(OxFuncAdapterRequest::new(
        "ftc-0399-disc-exactness",
        "formula:ftc-0399-disc-exactness",
        "=DISC(44927,45292,97,100)".to_string(),
        locus(1, 1),
        TypedContextQueryBundle::default(),
    ))
    .expect("disc adapter run");

    let actual = expect_number(&run.evaluation_artifact.worksheet_value);
    let prior_local = f64::from_bits(0x3f9eb851eb851eb8);
    let excel_target = f64::from_bits(0x3f9eb851eb851ec0);

    assert_eq!(actual.to_bits(), excel_target.to_bits());
    assert_ne!(actual.to_bits(), prior_local.to_bits());
}

#[test]
fn ftc_0391_ppmt_exactness_witness_pins_current_local_bits_and_excel_gap_through_adapter() {
    let run = run_oxfunc_preparation_adapter(OxFuncAdapterRequest::new(
        "ftc-0391-ppmt-exactness",
        "formula:ftc-0391-ppmt-exactness",
        "=PPMT(0.05/12,1,360,200000)".to_string(),
        locus(1, 1),
        TypedContextQueryBundle::default(),
    ))
    .expect("ppmt adapter run");

    let actual = expect_number(&run.evaluation_artifact.worksheet_value);
    let current_local = f64::from_bits(0xc06e09eace0506e4);
    let excel_target = f64::from_bits(0xc06e09eace050723);

    assert_eq!(actual.to_bits(), current_local.to_bits());
    assert_ne!(actual.to_bits(), excel_target.to_bits());
}

#[test]
fn ftc_0395_effect_exactness_witness_matches_excel_target_through_adapter() {
    let run = run_oxfunc_preparation_adapter(OxFuncAdapterRequest::new(
        "ftc-0395-effect-exactness",
        "formula:ftc-0395-effect-exactness",
        "=EFFECT(0.05,12)".to_string(),
        locus(1, 1),
        TypedContextQueryBundle::default(),
    ))
    .expect("effect adapter run");

    let actual = expect_number(&run.evaluation_artifact.worksheet_value);
    let prior_local = f64::from_bits(0x3faa31e46c681b80);
    let excel_target = f64::from_bits(0x3faa31e46c681bc0);

    assert_eq!(actual.to_bits(), excel_target.to_bits());
    assert_ne!(actual.to_bits(), prior_local.to_bits());
}

#[test]
fn effect_widened_publication_family_rows_pin_current_local_bits_through_adapter() {
    for (id, formula, expected) in [
        (
            "effect-widened-family-2",
            "=EFFECT(0.05,2)",
            f64::from_bits(0x3fa9eb851eb851e0),
        ),
        (
            "effect-widened-family-4",
            "=EFFECT(0.05,4)",
            f64::from_bits(0x3faa1581d7dbf480),
        ),
        (
            "effect-widened-family-12",
            "=EFFECT(0.05,12)",
            f64::from_bits(0x3faa31e46c681bc0),
        ),
        (
            "effect-widened-family-365",
            "=EFFECT(0.05,365)",
            f64::from_bits(0x3faa3fbbb959cb00),
        ),
    ] {
        let run = run_oxfunc_preparation_adapter(OxFuncAdapterRequest::new(
            id,
            &format!("formula:{id}"),
            formula.to_string(),
            locus(1, 1),
            TypedContextQueryBundle::default(),
        ))
        .expect("effect widened family adapter run");

        let actual = expect_number(&run.evaluation_artifact.worksheet_value);
        assert_eq!(actual.to_bits(), expected.to_bits(), "{formula}");
    }
}

#[test]
fn nominal_of_effect_stays_just_below_nominal_input_for_widened_rows_through_adapter() {
    for (id, formula) in [
        ("effect-nominal-roundtrip-2", "=NOMINAL(EFFECT(0.05,2),2)"),
        ("effect-nominal-roundtrip-4", "=NOMINAL(EFFECT(0.05,4),4)"),
        (
            "effect-nominal-roundtrip-12",
            "=NOMINAL(EFFECT(0.05,12),12)",
        ),
        (
            "effect-nominal-roundtrip-365",
            "=NOMINAL(EFFECT(0.05,365),365)",
        ),
    ] {
        let run = run_oxfunc_preparation_adapter(OxFuncAdapterRequest::new(
            id,
            &format!("formula:{id}"),
            formula.to_string(),
            locus(1, 1),
            TypedContextQueryBundle::default(),
        ))
        .expect("effect nominal roundtrip adapter run");

        let actual = expect_number(&run.evaluation_artifact.worksheet_value);
        assert!(actual < 0.05, "{formula}: {actual}");
    }
}

#[test]
fn ftc_0393_and_ftc_0394_exactness_witness_rows_match_excel_targets_through_adapter() {
    for (id, formula, excel_target) in [
        (
            "ftc-0393-cumipmt-exactness",
            "=CUMIPMT(0.05/12,360,200000,1,12,0)",
            f64::from_bits(0xc0c3667e7f577146),
        ),
        (
            "ftc-0394-cumprinc-exactness",
            "=CUMPRINC(0.05/12,360,200000,1,12,0)",
            f64::from_bits(0xc0a70d761d260042),
        ),
    ] {
        let run = run_oxfunc_preparation_adapter(OxFuncAdapterRequest::new(
            id,
            &format!("formula:{id}"),
            formula.to_string(),
            locus(1, 1),
            TypedContextQueryBundle::default(),
        ))
        .expect("cumulative_finance exactness adapter run");

        let actual = expect_number(&run.evaluation_artifact.worksheet_value);
        assert_eq!(actual.to_bits(), excel_target.to_bits(), "{formula}");
    }
}

#[test]
fn ftc_0365_correl_exactness_witness_matches_excel_target_through_adapter() {
    let run = run_oxfunc_preparation_adapter(OxFuncAdapterRequest::new(
        "ftc-0365-correl-exactness",
        "formula:ftc-0365-correl-exactness",
        "=CORREL({1,2,3,4,5},{2,4,6,8,10})".to_string(),
        locus(1, 1),
        TypedContextQueryBundle::default(),
    ))
    .expect("ftc-0365 adapter run");

    let actual = expect_number(&run.evaluation_artifact.worksheet_value);
    let prior_local = 1.0_f64;
    let excel_target = 0.9999999999999998_f64;

    assert_eq!(actual.to_bits(), excel_target.to_bits());
    assert_ne!(actual.to_bits(), prior_local.to_bits());
}

#[test]
fn ftc_0366_correl_negative_exactness_witness_matches_excel_target_through_adapter() {
    let run = run_oxfunc_preparation_adapter(OxFuncAdapterRequest::new(
        "ftc-0366-correl-negative-exactness",
        "formula:ftc-0366-correl-negative-exactness",
        "=CORREL({1,2,3,4,5},{10,8,6,4,2})".to_string(),
        locus(1, 1),
        TypedContextQueryBundle::default(),
    ))
    .expect("ftc-0366 adapter run");

    let actual = expect_number(&run.evaluation_artifact.worksheet_value);
    let prior_local = -1.0_f64;
    let excel_target = -0.9999999999999998_f64;

    assert_eq!(actual.to_bits(), excel_target.to_bits());
    assert_ne!(actual.to_bits(), prior_local.to_bits());
}

#[test]
fn ftc_0369_rsq_exactness_witness_matches_excel_target_through_adapter() {
    let run = run_oxfunc_preparation_adapter(OxFuncAdapterRequest::new(
        "ftc-0369-rsq-exactness",
        "formula:ftc-0369-rsq-exactness",
        "=RSQ({2,4,6,8,10},{1,2,3,4,5})".to_string(),
        locus(1, 1),
        TypedContextQueryBundle::default(),
    ))
    .expect("ftc-0369 adapter run");

    let actual = expect_number(&run.evaluation_artifact.worksheet_value);
    let prior_local = 1.0_f64;
    let excel_target = 0.9999999999999996_f64;

    assert_eq!(actual.to_bits(), excel_target.to_bits());
    assert_ne!(actual.to_bits(), prior_local.to_bits());
}

#[test]
fn ftc_0374_skew_exactness_witness_matches_excel_target_through_adapter() {
    let run = run_oxfunc_preparation_adapter(OxFuncAdapterRequest::new(
        "ftc-0374-skew-exactness",
        "formula:ftc-0374-skew-exactness",
        "=SKEW({1,2,2,3,5})".to_string(),
        locus(1, 1),
        TypedContextQueryBundle::default(),
    ))
    .expect("ftc-0374 adapter run");

    let actual = expect_number(&run.evaluation_artifact.worksheet_value);
    let prior_local = 1.1180799331493771_f64;
    let excel_target = 1.1180799331493774_f64;

    assert_eq!(actual.to_bits(), excel_target.to_bits());
    assert_ne!(actual.to_bits(), prior_local.to_bits());
}

#[test]
fn ftc_0375_kurt_exactness_witness_matches_excel_target_through_adapter() {
    let run = run_oxfunc_preparation_adapter(OxFuncAdapterRequest::new(
        "ftc-0375-kurt-exactness",
        "formula:ftc-0375-kurt-exactness",
        "=KURT({1,2,3,4,5})".to_string(),
        locus(1, 1),
        TypedContextQueryBundle::default(),
    ))
    .expect("ftc-0375 adapter run");

    let actual = expect_number(&run.evaluation_artifact.worksheet_value);
    let prior_local = -1.200000000000002_f64;
    let excel_target = -1.1999999999999984_f64;

    assert_eq!(actual.to_bits(), excel_target.to_bits());
    assert_ne!(actual.to_bits(), prior_local.to_bits());
}

#[test]
fn ftc_0377_pmt_exactness_witness_pins_current_local_value_and_excel_gap() {
    let run = run_oxfunc_preparation_adapter(OxFuncAdapterRequest::new(
        "ftc-0377-pmt-exactness",
        "formula:ftc-0377-pmt-exactness",
        "=PMT(0.05/12,360,200000)".to_string(),
        locus(1, 1),
        TypedContextQueryBundle::default(),
    ))
    .expect("ftc-0377 adapter run");

    let actual = expect_number(&run.evaluation_artifact.worksheet_value);
    let current_local = -1073.6432460242763_f64;
    let excel_target = -1073.643246024278_f64;

    assert_eq!(actual.to_bits(), current_local.to_bits());
    assert_ne!(actual.to_bits(), excel_target.to_bits());
}

#[test]
fn ftc_0381_rate_exactness_witness_pins_current_local_value_and_excel_gap_through_adapter() {
    let run = run_oxfunc_preparation_adapter(OxFuncAdapterRequest::new(
        "ftc-0381-rate-exactness",
        "formula:ftc-0381-rate-exactness",
        "=RATE(360,-1073.64,200000)".to_string(),
        locus(1, 1),
        TypedContextQueryBundle::default(),
    ))
    .expect("ftc-0381 adapter run");

    let actual = expect_number(&run.evaluation_artifact.worksheet_value);
    let current_local = 0.0041666445363460975_f64;
    let excel_target = 0.004166644536345589_f64;

    assert_eq!(actual.to_bits(), current_local.to_bits());
    assert_ne!(actual.to_bits(), excel_target.to_bits());
}

#[test]
fn ftc_0382_npv_exactness_witness_matches_excel_target_through_adapter() {
    let run = run_oxfunc_preparation_adapter(OxFuncAdapterRequest::new(
        "ftc-0382-npv-exactness",
        "formula:ftc-0382-npv-exactness",
        "=NPV(0.1,{-10000,3000,4200,6800})".to_string(),
        locus(1, 1),
        TypedContextQueryBundle::default(),
    ))
    .expect("ftc-0382 adapter run");

    let actual = expect_number(&run.evaluation_artifact.worksheet_value);
    let prior_local = 1188.4434123352216_f64;
    let excel_target = 1188.4434123352207_f64;

    assert_eq!(actual.to_bits(), excel_target.to_bits());
    assert_ne!(actual.to_bits(), prior_local.to_bits());
}

#[test]
fn ftc_0383_irr_exactness_witness_matches_excel_target_through_adapter() {
    let run = run_oxfunc_preparation_adapter(OxFuncAdapterRequest::new(
        "ftc-0383-irr-exactness",
        "formula:ftc-0383-irr-exactness",
        "=IRR({-10000,3000,4200,6800})".to_string(),
        locus(1, 1),
        TypedContextQueryBundle::default(),
    ))
    .expect("ftc-0383 adapter run");

    let actual = expect_number(&run.evaluation_artifact.worksheet_value);
    let prior_local = 0.1634056006889894_f64;
    let excel_target = 0.16340560068898924_f64;

    assert_eq!(actual.to_bits(), excel_target.to_bits());
    assert_ne!(actual.to_bits(), prior_local.to_bits());
}

#[test]
fn ftc_0635_exact_formula_matches_excel_bit_value_through_adapter() {
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
        EvalValue::Number(-1.9999999999999998)
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
