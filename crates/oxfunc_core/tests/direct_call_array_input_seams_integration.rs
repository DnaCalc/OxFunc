use oxfml_core::interface::TypedContextQueryBundle;
use oxfml_core::seam::Locus;
use oxfml_core::test_support::oxfunc_adapter::{
    run_oxfunc_preparation_adapter, OxFuncAdapterRequest,
};
use oxfunc_core::value::{ArrayCellValue, EvalArray, EvalValue};

fn locus(row: u32, col: u32) -> Locus {
    Locus {
        sheet_id: "sheet:default".to_string(),
        row,
        col,
    }
}

#[test]
fn ftc_0959_gcd_array_literal_direct_call_matches_scalar_one_through_adapter() {
    let run = run_oxfunc_preparation_adapter(OxFuncAdapterRequest::new(
        "ftc-0959-gcd-array-literal-direct",
        "formula:ftc-0959-gcd-array-literal-direct",
        "=GCD({1,2,3,4,5,6,7,8,9,10,11,12},12)".to_string(),
        locus(1, 1),
        TypedContextQueryBundle::default(),
    ))
    .expect("ftc-0959 gcd array literal adapter run");

    assert_eq!(
        run.evaluation_artifact.worksheet_value,
        EvalValue::Number(1.0)
    );
}

#[test]
fn ftc_0959_gcd_sequence_direct_call_matches_scalar_one_through_adapter() {
    let run = run_oxfunc_preparation_adapter(OxFuncAdapterRequest::new(
        "ftc-0959-gcd-sequence-direct",
        "formula:ftc-0959-gcd-sequence-direct",
        "=GCD(SEQUENCE(12),12)".to_string(),
        locus(1, 1),
        TypedContextQueryBundle::default(),
    ))
    .expect("ftc-0959 gcd sequence adapter run");

    assert_eq!(
        run.evaluation_artifact.worksheet_value,
        EvalValue::Number(1.0)
    );
}

#[test]
fn ftc_1032_and_array_literal_direct_call_scalarizes_to_false_through_adapter() {
    let run = run_oxfunc_preparation_adapter(OxFuncAdapterRequest::new(
        "ftc-1032-and-array-literal-direct",
        "formula:ftc-1032-and-array-literal-direct",
        "=AND({FALSE;TRUE;TRUE},{TRUE;TRUE;TRUE})".to_string(),
        locus(1, 1),
        TypedContextQueryBundle::default(),
    ))
    .expect("ftc-1032 and array literal adapter run");

    assert_eq!(
        run.evaluation_artifact.worksheet_value,
        EvalValue::Logical(false)
    );
}

#[test]
fn ftc_1032_and_sequence_bounds_direct_call_scalarizes_to_false_through_adapter() {
    let run = run_oxfunc_preparation_adapter(OxFuncAdapterRequest::new(
        "ftc-1032-and-sequence-bounds-direct",
        "formula:ftc-1032-and-sequence-bounds-direct",
        "=LET(grid,SEQUENCE(7),AND(grid>1,grid<=3))".to_string(),
        locus(1, 1),
        TypedContextQueryBundle::default(),
    ))
    .expect("ftc-1032 and sequence bounds adapter run");

    assert_eq!(
        run.evaluation_artifact.worksheet_value,
        EvalValue::Logical(false)
    );
}

#[test]
fn ftc_1032_if_scalar_false_continuation_returns_scalar_zero_through_adapter() {
    let run = run_oxfunc_preparation_adapter(OxFuncAdapterRequest::new(
        "ftc-1032-if-scalar-false-direct",
        "formula:ftc-1032-if-scalar-false-direct",
        "=IF(FALSE,SEQUENCE(7),0)".to_string(),
        locus(1, 1),
        TypedContextQueryBundle::default(),
    ))
    .expect("ftc-1032 if scalar false adapter run");

    assert_eq!(
        run.evaluation_artifact.worksheet_value,
        EvalValue::Number(0.0)
    );
}

#[test]
fn ftc_0966_log_array_direct_call_matches_elementwise_row_through_adapter() {
    let run = run_oxfunc_preparation_adapter(OxFuncAdapterRequest::new(
        "ftc-0966-log-array-direct",
        "formula:ftc-0966-log-array-direct",
        "=LOG({0.5,0.25,0.125,0.125},2)".to_string(),
        locus(1, 1),
        TypedContextQueryBundle::default(),
    ))
    .expect("ftc-0966 log array adapter run");

    assert_eq!(
        run.evaluation_artifact.worksheet_value,
        EvalValue::Array(
            EvalArray::from_rows(vec![vec![
                ArrayCellValue::Number(-1.0),
                ArrayCellValue::Number(-2.0),
                ArrayCellValue::Number(-3.0),
                ArrayCellValue::Number(-3.0),
            ]])
            .unwrap(),
        )
    );
}
