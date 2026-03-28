use oxfml_core::interface::TypedContextQueryBundle;
use oxfml_core::oxfunc_adapter::{OxFuncAdapterRequest, run_oxfunc_preparation_adapter};
use oxfml_core::seam::{Locus, RejectCode};
use oxfunc_core::value::{ArrayCellValue, EvalArray, EvalValue, ExcelText};

fn text_cell(value: &str) -> ArrayCellValue {
    ArrayCellValue::Text(ExcelText::from_interop_assignment(value))
}

fn locus(row: u32, col: u32) -> Locus {
    Locus {
        sheet_id: "sheet:default".to_string(),
        row,
        col,
    }
}

#[test]
fn groupby_builtin_sum_callable_lane_passes_from_oxfunc_side() {
    let run = run_oxfunc_preparation_adapter(OxFuncAdapterRequest::new(
        "groupby-builtin-sum-callable",
        "formula:groupby-builtin-sum-callable",
        "=GROUPBY({\"2024\";\"2024\";\"2025\";\"2025\"},{10;20;30;40},SUM)",
        locus(1, 1),
        TypedContextQueryBundle::default(),
    ))
    .expect("groupby builtin callable adapter run");

    let expected = EvalArray::from_rows(vec![
        vec![text_cell("2024"), ArrayCellValue::Number(30.0)],
        vec![text_cell("2025"), ArrayCellValue::Number(70.0)],
        vec![text_cell("Total"), ArrayCellValue::Number(100.0)],
    ])
    .expect("expected array");

    assert_eq!(
        run.evaluation_artifact.worksheet_value,
        EvalValue::Array(expected)
    );
}

#[test]
fn pivotby_builtin_sum_visible_headers_lane_passes_from_oxfunc_side() {
    let run = run_oxfunc_preparation_adapter(OxFuncAdapterRequest::new(
        "pivotby-builtin-sum-visible-headers",
        "formula:pivotby-builtin-sum-visible-headers",
        "=PIVOTBY({\"Region\";\"East\";\"West\"},{\"Product\";\"A\";\"B\"},{\"Sales\";40;50},SUM,3)",
        locus(1, 1),
        TypedContextQueryBundle::default(),
    ))
    .expect("pivotby builtin visible headers adapter run");

    let expected = EvalArray::from_rows(vec![
        vec![
            ArrayCellValue::EmptyCell,
            text_cell("Product"),
            ArrayCellValue::EmptyCell,
            ArrayCellValue::EmptyCell,
        ],
        vec![
            ArrayCellValue::EmptyCell,
            text_cell("A"),
            text_cell("B"),
            text_cell("Total"),
        ],
        vec![
            text_cell("Region"),
            text_cell("Sales"),
            text_cell("Sales"),
            text_cell("Sales"),
        ],
        vec![
            text_cell("East"),
            ArrayCellValue::Number(40.0),
            ArrayCellValue::Number(0.0),
            ArrayCellValue::Number(40.0),
        ],
        vec![
            text_cell("West"),
            ArrayCellValue::Number(0.0),
            ArrayCellValue::Number(50.0),
            ArrayCellValue::Number(50.0),
        ],
        vec![
            text_cell("Total"),
            ArrayCellValue::Number(40.0),
            ArrayCellValue::Number(50.0),
            ArrayCellValue::Number(90.0),
        ],
    ])
    .expect("expected array");

    assert_eq!(
        run.evaluation_artifact.worksheet_value,
        EvalValue::Array(expected)
    );
}

#[test]
fn duplicate_lambda_parameter_names_reject_as_bind_mismatch_from_oxfunc_side() {
    let run = run_oxfunc_preparation_adapter(OxFuncAdapterRequest::new(
        "duplicate-lambda-parameters",
        "formula:duplicate-lambda-parameters",
        "=LAMBDA(x,x,x)",
        locus(1, 1),
        TypedContextQueryBundle::default(),
    ))
    .expect("duplicate lambda adapter run");

    assert_eq!(run.evaluation_artifact.commit_decision_kind, "rejected");
    assert_eq!(
        run.evaluation_artifact.reject_code,
        Some(RejectCode::BindMismatch)
    );
    assert!(
        run.preparation_artifact
            .bind_diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message == "duplicate LAMBDA parameter name 'x'")
    );
}

#[test]
fn malformed_lambda_parameter_rejects_as_bind_mismatch_from_oxfunc_side() {
    let run = run_oxfunc_preparation_adapter(OxFuncAdapterRequest::new(
        "malformed-lambda-parameter",
        "formula:malformed-lambda-parameter",
        "=LAMBDA(1,1)",
        locus(1, 1),
        TypedContextQueryBundle::default(),
    ))
    .expect("malformed lambda adapter run");

    assert_eq!(run.evaluation_artifact.commit_decision_kind, "rejected");
    assert_eq!(
        run.evaluation_artifact.reject_code,
        Some(RejectCode::BindMismatch)
    );
    assert!(run.preparation_artifact.bind_diagnostics.iter().any(
        |diagnostic| diagnostic.message == "LAMBDA parameter did not bind as helper parameter"
    ));
}
