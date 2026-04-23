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
fn ftc_1032_wraprows_scalar_input_returns_scalar_zero_through_adapter() {
    let run = run_oxfunc_preparation_adapter(OxFuncAdapterRequest::new(
        "ftc-1032-wraprows-scalar-direct",
        "formula:ftc-1032-wraprows-scalar-direct",
        "=WRAPROWS(0,7)".to_string(),
        locus(1, 1),
        TypedContextQueryBundle::default(),
    ))
    .expect("ftc-1032 wraprows scalar adapter run");

    assert_eq!(
        run.evaluation_artifact.worksheet_value,
        EvalValue::Number(0.0)
    );
}

#[test]
fn ftc_1032_index_of_wraprows_scalar_input_returns_scalar_zero_through_adapter() {
    let run = run_oxfunc_preparation_adapter(OxFuncAdapterRequest::new(
        "ftc-1032-index-wraprows-scalar-direct",
        "formula:ftc-1032-index-wraprows-scalar-direct",
        "=INDEX(WRAPROWS(0,7),1,0)".to_string(),
        locus(1, 1),
        TypedContextQueryBundle::default(),
    ))
    .expect("ftc-1032 index wraprows scalar adapter run");

    assert_eq!(
        run.evaluation_artifact.worksheet_value,
        EvalValue::Number(0.0)
    );
}

#[test]
fn ftc_1032_sum_of_indexed_wraprows_scalar_input_returns_scalar_zero_through_adapter() {
    let run = run_oxfunc_preparation_adapter(OxFuncAdapterRequest::new(
        "ftc-1032-sum-index-wraprows-scalar-direct",
        "formula:ftc-1032-sum-index-wraprows-scalar-direct",
        "=SUM(INDEX(WRAPROWS(0,7),1,0))".to_string(),
        locus(1, 1),
        TypedContextQueryBundle::default(),
    ))
    .expect("ftc-1032 sum index wraprows scalar adapter run");

    assert_eq!(
        run.evaluation_artifact.worksheet_value,
        EvalValue::Number(0.0)
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
fn ftc_0907_and_single_true_array_scalarizes_to_true_through_adapter() {
    let run = run_oxfunc_preparation_adapter(OxFuncAdapterRequest::new(
        "ftc-0907-and-single-true-array-direct",
        "formula:ftc-0907-and-single-true-array-direct",
        "=AND({TRUE;TRUE;TRUE})".to_string(),
        locus(1, 1),
        TypedContextQueryBundle::default(),
    ))
    .expect("ftc-0907 and single true array adapter run");

    assert_eq!(
        run.evaluation_artifact.worksheet_value,
        EvalValue::Logical(true)
    );
}

#[test]
fn ftc_0907_and_single_mixed_array_scalarizes_to_false_through_adapter() {
    let run = run_oxfunc_preparation_adapter(OxFuncAdapterRequest::new(
        "ftc-0907-and-single-mixed-array-direct",
        "formula:ftc-0907-and-single-mixed-array-direct",
        "=AND({TRUE;FALSE;TRUE})".to_string(),
        locus(1, 1),
        TypedContextQueryBundle::default(),
    ))
    .expect("ftc-0907 and single mixed array adapter run");

    assert_eq!(
        run.evaluation_artifact.worksheet_value,
        EvalValue::Logical(false)
    );
}

#[test]
fn ftc_0907_and_map_true_array_scalarizes_to_true_through_adapter() {
    let run = run_oxfunc_preparation_adapter(OxFuncAdapterRequest::new(
        "ftc-0907-and-map-true-array-direct",
        "formula:ftc-0907-and-map-true-array-direct",
        "=AND(MAP(SEQUENCE(3),LAMBDA(x,TRUE)))".to_string(),
        locus(1, 1),
        TypedContextQueryBundle::default(),
    ))
    .expect("ftc-0907 and map true array adapter run");

    assert_eq!(
        run.evaluation_artifact.worksheet_value,
        EvalValue::Logical(true)
    );
}

#[test]
fn ftc_0910_index_row_vector_omitted_row_selector_array_returns_first_five_values_through_adapter()
{
    let run = run_oxfunc_preparation_adapter(OxFuncAdapterRequest::new(
        "ftc-0910-index-row-vector-first-five-direct",
        "formula:ftc-0910-index-row-vector-first-five-direct",
        "=INDEX({10,20,30,40,50,60,70,80,90,100},,SEQUENCE(5,,1))".to_string(),
        locus(1, 1),
        TypedContextQueryBundle::default(),
    ))
    .expect("ftc-0910 first five adapter run");

    assert_eq!(
        run.evaluation_artifact.worksheet_value,
        EvalValue::Array(
            EvalArray::from_rows(vec![
                vec![ArrayCellValue::Number(10.0)],
                vec![ArrayCellValue::Number(20.0)],
                vec![ArrayCellValue::Number(30.0)],
                vec![ArrayCellValue::Number(40.0)],
                vec![ArrayCellValue::Number(50.0)],
            ])
            .unwrap(),
        )
    );
}

#[test]
fn ftc_0910_index_row_vector_omitted_row_selector_array_returns_last_five_values_through_adapter() {
    let run = run_oxfunc_preparation_adapter(OxFuncAdapterRequest::new(
        "ftc-0910-index-row-vector-last-five-direct",
        "formula:ftc-0910-index-row-vector-last-five-direct",
        "=INDEX({10,20,30,40,50,60,70,80,90,100},,SEQUENCE(5,,6))".to_string(),
        locus(1, 1),
        TypedContextQueryBundle::default(),
    ))
    .expect("ftc-0910 last five adapter run");

    assert_eq!(
        run.evaluation_artifact.worksheet_value,
        EvalValue::Array(
            EvalArray::from_rows(vec![
                vec![ArrayCellValue::Number(60.0)],
                vec![ArrayCellValue::Number(70.0)],
                vec![ArrayCellValue::Number(80.0)],
                vec![ArrayCellValue::Number(90.0)],
                vec![ArrayCellValue::Number(100.0)],
            ])
            .unwrap(),
        )
    );
}

#[test]
fn ftc_0930_index_over_value_error_result_propagates_value_through_adapter() {
    let run = run_oxfunc_preparation_adapter(OxFuncAdapterRequest::new(
        "ftc-0930-index-over-error-direct",
        "formula:ftc-0930-index-over-error-direct",
        "=INDEX(SORT(TOCOL({3,7,1;8,2,9;4,6,5}),-1),3)".to_string(),
        locus(1, 1),
        TypedContextQueryBundle::default(),
    ))
    .expect("ftc-0930 index over error adapter run");

    assert_eq!(
        run.evaluation_artifact.worksheet_value,
        EvalValue::Error(oxfunc_core::value::WorksheetErrorCode::Value)
    );
}

#[test]
fn ftc_0670_valuetotext_strict_array_returns_quoted_text_grid_through_adapter() {
    let run = run_oxfunc_preparation_adapter(OxFuncAdapterRequest::new(
        "ftc-0670-valuetotext-strict-array",
        "formula:ftc-0670-valuetotext-strict-array",
        "=VALUETOTEXT({\"a\",\"b\";\"c\",\"d\"},1)".to_string(),
        locus(1, 1),
        TypedContextQueryBundle::default(),
    ))
    .expect("ftc-0670 valuetotext adapter run");

    assert_eq!(
        run.evaluation_artifact.worksheet_value,
        EvalValue::Array(
            EvalArray::from_rows(vec![
                vec![
                    ArrayCellValue::Text(oxfunc_core::value::ExcelText::from_interop_assignment(
                        "\"a\""
                    )),
                    ArrayCellValue::Text(oxfunc_core::value::ExcelText::from_interop_assignment(
                        "\"b\""
                    )),
                ],
                vec![
                    ArrayCellValue::Text(oxfunc_core::value::ExcelText::from_interop_assignment(
                        "\"c\""
                    )),
                    ArrayCellValue::Text(oxfunc_core::value::ExcelText::from_interop_assignment(
                        "\"d\""
                    )),
                ],
            ])
            .unwrap(),
        )
    );
}

#[test]
fn ftc_0692_maxifs_direct_array_ranges_return_shaped_value_error_row_through_adapter() {
    let run = run_oxfunc_preparation_adapter(OxFuncAdapterRequest::new(
        "ftc-0692-maxifs-array-ranges-direct",
        "formula:ftc-0692-maxifs-array-ranges-direct",
        "=LET(v,{10,20,30,40,50},c,{\"a\",\"b\",\"a\",\"b\",\"a\"},MAXIFS(v,c,\"a\"))".to_string(),
        locus(1, 1),
        TypedContextQueryBundle::default(),
    ))
    .expect("ftc-0692 maxifs adapter run");

    assert_eq!(
        run.evaluation_artifact.worksheet_value,
        EvalValue::Array(
            EvalArray::from_rows(vec![vec![
                ArrayCellValue::Error(oxfunc_core::value::WorksheetErrorCode::Value),
                ArrayCellValue::Error(oxfunc_core::value::WorksheetErrorCode::Value),
                ArrayCellValue::Error(oxfunc_core::value::WorksheetErrorCode::Value),
                ArrayCellValue::Error(oxfunc_core::value::WorksheetErrorCode::Value),
                ArrayCellValue::Error(oxfunc_core::value::WorksheetErrorCode::Value),
            ]])
            .unwrap(),
        )
    );
}

#[test]
fn ftc_0693_minifs_direct_array_ranges_return_shaped_value_error_row_through_adapter() {
    let run = run_oxfunc_preparation_adapter(OxFuncAdapterRequest::new(
        "ftc-0693-minifs-array-ranges-direct",
        "formula:ftc-0693-minifs-array-ranges-direct",
        "=LET(v,{10,20,30,40,50},c,{\"a\",\"b\",\"a\",\"b\",\"a\"},MINIFS(v,c,\"b\"))".to_string(),
        locus(1, 1),
        TypedContextQueryBundle::default(),
    ))
    .expect("ftc-0693 minifs adapter run");

    assert_eq!(
        run.evaluation_artifact.worksheet_value,
        EvalValue::Array(
            EvalArray::from_rows(vec![vec![
                ArrayCellValue::Error(oxfunc_core::value::WorksheetErrorCode::Value),
                ArrayCellValue::Error(oxfunc_core::value::WorksheetErrorCode::Value),
                ArrayCellValue::Error(oxfunc_core::value::WorksheetErrorCode::Value),
                ArrayCellValue::Error(oxfunc_core::value::WorksheetErrorCode::Value),
                ArrayCellValue::Error(oxfunc_core::value::WorksheetErrorCode::Value),
            ]])
            .unwrap(),
        )
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
