use oxfunc_core::functions::surface_dispatch::eval_surface_value_call;
use oxfunc_core::resolver::{RefResolutionError, ReferenceResolver, ResolverCapabilities};
use oxfunc_core::value::{
    ArrayCellValue, CallArgValue, EvalArray, EvalValue, ExcelText, ReferenceLike,
    WorksheetErrorCode,
};
use serde::Serialize;
use serde_json::Value as JsonValue;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;

#[derive(Debug)]
struct CaseRecord {
    case_id: String,
    function_id: String,
    formula_text: String,
    args: Vec<JsonValue>,
}

#[derive(Debug, Serialize)]
struct OutcomeRecord {
    schema_version: &'static str,
    case_id: String,
    function_id: String,
    formula_text: String,
    evaluator_id: &'static str,
    execution_status: &'static str,
    outcome: Outcome,
}

#[derive(Debug, Serialize, Clone)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum Outcome {
    Number {
        value: f64,
        bits_hex: String,
        digest_payload: String,
    },
    Text {
        value: String,
        digest_payload: String,
    },
    Logical {
        value: bool,
        digest_payload: String,
    },
    Error {
        code: String,
        digest_payload: String,
    },
    EmptyCell {
        digest_payload: String,
    },
    Array {
        rows: usize,
        cols: usize,
        cells: Vec<Vec<Outcome>>,
        digest_payload: String,
    },
    HarnessError {
        message: String,
        digest_payload: String,
    },
}

struct NoResolver;

impl ReferenceResolver for NoResolver {
    fn capabilities(&self) -> ResolverCapabilities {
        ResolverCapabilities::permissive_local()
    }

    fn resolve_reference(
        &self,
        reference: &ReferenceLike,
    ) -> Result<EvalValue, RefResolutionError> {
        Err(RefResolutionError::UnresolvedReference {
            target: reference.target.clone(),
        })
    }
}

fn usage(program: &str) -> String {
    format!("usage: {program} --cases <cases.jsonl> --out <local-outcomes.jsonl>")
}

fn parse_args() -> Result<(PathBuf, PathBuf), String> {
    let args: Vec<String> = env::args().collect();
    let mut cases = None;
    let mut out = None;
    let mut index = 1;
    while index < args.len() {
        match args[index].as_str() {
            "--cases" => {
                index += 1;
                cases = args.get(index).map(PathBuf::from);
            }
            "--out" => {
                index += 1;
                out = args.get(index).map(PathBuf::from);
            }
            _ => return Err(usage(&args[0])),
        }
        index += 1;
    }
    match (cases, out) {
        (Some(cases), Some(out)) => Ok((cases, out)),
        _ => Err(usage(&args[0])),
    }
}

fn worksheet_error_code(code: WorksheetErrorCode) -> String {
    format!("{code:?}")
}

fn parse_worksheet_error_code(code: &str) -> Result<WorksheetErrorCode, String> {
    match code {
        "Null" | "#NULL!" => Ok(WorksheetErrorCode::Null),
        "Div0" | "#DIV/0!" => Ok(WorksheetErrorCode::Div0),
        "Value" | "#VALUE!" => Ok(WorksheetErrorCode::Value),
        "Ref" | "#REF!" => Ok(WorksheetErrorCode::Ref),
        "Name" | "#NAME?" => Ok(WorksheetErrorCode::Name),
        "Num" | "#NUM!" => Ok(WorksheetErrorCode::Num),
        "NA" | "#N/A" => Ok(WorksheetErrorCode::NA),
        "Busy" | "#BUSY!" => Ok(WorksheetErrorCode::Busy),
        "GettingData" | "#GETTING_DATA" | "#GETTING_DATA!" => Ok(WorksheetErrorCode::GettingData),
        "Spill" | "#SPILL!" => Ok(WorksheetErrorCode::Spill),
        "Calc" | "#CALC!" => Ok(WorksheetErrorCode::Calc),
        "Field" | "#FIELD!" => Ok(WorksheetErrorCode::Field),
        "Blocked" | "#BLOCKED!" => Ok(WorksheetErrorCode::Blocked),
        "Connect" | "#CONNECT!" => Ok(WorksheetErrorCode::Connect),
        other => Err(format!("unsupported worksheet error code: {other}")),
    }
}

fn input_kind(input: &JsonValue) -> Result<&str, String> {
    input
        .as_object()
        .and_then(|object| object.get("kind"))
        .and_then(JsonValue::as_str)
        .ok_or_else(|| "input value is missing string kind".to_string())
}

fn input_field<'a>(input: &'a JsonValue, field: &str) -> Result<&'a JsonValue, String> {
    input
        .as_object()
        .and_then(|object| object.get(field))
        .ok_or_else(|| format!("input value is missing field: {field}"))
}

fn required_string_field(input: &JsonValue, field: &str) -> Result<String, String> {
    input_field(input, field)?
        .as_str()
        .map(ToOwned::to_owned)
        .ok_or_else(|| format!("case field is not a string: {field}"))
}

fn case_from_json(input: JsonValue) -> Result<CaseRecord, String> {
    let args_value = input_field(&input, "args")?;
    let args = match args_value {
        JsonValue::Array(args) => args.clone(),
        JsonValue::Object(_) => vec![args_value.clone()],
        _ => {
            return Err(format!(
                "case args is not an array or singleton object: {args_value}"
            ));
        }
    };
    Ok(CaseRecord {
        case_id: required_string_field(&input, "case_id")?,
        function_id: required_string_field(&input, "function_id")?,
        formula_text: required_string_field(&input, "formula_text")?,
        args,
    })
}

fn input_to_call_arg(input: &JsonValue) -> Result<CallArgValue, String> {
    match input_kind(input)? {
        "number" => Ok(CallArgValue::Eval(EvalValue::Number(
            input_field(input, "value")?
                .as_f64()
                .ok_or_else(|| "number input has non-numeric value".to_string())?,
        ))),
        "text" => {
            let value = input_field(input, "value")?
                .as_str()
                .ok_or_else(|| "text input has non-string value".to_string())?;
            Ok(CallArgValue::Eval(EvalValue::Text(
                ExcelText::from_interop_assignment(value),
            )))
        }
        "logical" => Ok(CallArgValue::Eval(EvalValue::Logical(
            input_field(input, "value")?
                .as_bool()
                .ok_or_else(|| "logical input has non-boolean value".to_string())?,
        ))),
        "error" => {
            let code = input_field(input, "code")?
                .as_str()
                .ok_or_else(|| "error input has non-string code".to_string())?;
            Ok(CallArgValue::Eval(EvalValue::Error(
                parse_worksheet_error_code(code)?,
            )))
        }
        "empty_cell" => Ok(CallArgValue::EmptyCell),
        "missing_arg" => Ok(CallArgValue::MissingArg),
        "array" => Ok(CallArgValue::Eval(EvalValue::Array(input_to_array(
            input_field(input, "rows")?,
        )?))),
        other => Err(format!("unsupported input kind: {other}")),
    }
}

fn input_to_array(rows: &JsonValue) -> Result<EvalArray, String> {
    let rows = rows
        .as_array()
        .ok_or_else(|| "array input rows is not an array".to_string())?;
    if rows.is_empty() {
        return Err("array input has no rows".to_string());
    }
    let first_row = rows[0]
        .as_array()
        .ok_or_else(|| "array input row is not an array".to_string())?;
    let expected_cols = first_row.len();
    if expected_cols == 0 {
        return Err("array input has no columns".to_string());
    }

    let mut converted_rows = Vec::with_capacity(rows.len());
    for row_value in rows {
        let row = row_value
            .as_array()
            .ok_or_else(|| "array input row is not an array".to_string())?;
        if row.len() != expected_cols {
            return Err("array input has ragged rows".to_string());
        }
        let mut converted = Vec::with_capacity(row.len());
        for cell_value in row {
            converted.push(input_to_array_cell(cell_value)?);
        }
        converted_rows.push(converted);
    }

    EvalArray::from_rows(converted_rows).ok_or_else(|| "invalid array input shape".to_string())
}

fn input_to_array_cell(input: &JsonValue) -> Result<ArrayCellValue, String> {
    match input_kind(input)? {
        "number" => Ok(ArrayCellValue::Number(
            input_field(input, "value")?
                .as_f64()
                .ok_or_else(|| "number array cell has non-numeric value".to_string())?,
        )),
        "text" => {
            let value = input_field(input, "value")?
                .as_str()
                .ok_or_else(|| "text array cell has non-string value".to_string())?;
            Ok(ArrayCellValue::Text(ExcelText::from_interop_assignment(
                value,
            )))
        }
        "logical" => Ok(ArrayCellValue::Logical(
            input_field(input, "value")?
                .as_bool()
                .ok_or_else(|| "logical array cell has non-boolean value".to_string())?,
        )),
        "error" => {
            let code = input_field(input, "code")?
                .as_str()
                .ok_or_else(|| "error array cell has non-string code".to_string())?;
            Ok(ArrayCellValue::Error(parse_worksheet_error_code(code)?))
        }
        "empty_cell" => Ok(ArrayCellValue::EmptyCell),
        "missing_arg" => Err("missing_arg is not valid inside array literals".to_string()),
        "array" => Err("nested array literals are not supported".to_string()),
        other => Err(format!("unsupported array cell kind: {other}")),
    }
}

fn number_outcome(value: f64) -> Outcome {
    let bits_hex = format!("0x{:016x}", value.to_bits());
    Outcome::Number {
        value,
        bits_hex: bits_hex.clone(),
        digest_payload: format!("number:{bits_hex}"),
    }
}

fn text_outcome(value: String) -> Outcome {
    Outcome::Text {
        digest_payload: format!("text:{value}"),
        value,
    }
}

fn logical_outcome(value: bool) -> Outcome {
    Outcome::Logical {
        digest_payload: format!("logical:{value}"),
        value,
    }
}

fn error_outcome(code: WorksheetErrorCode) -> Outcome {
    let code = worksheet_error_code(code);
    Outcome::Error {
        digest_payload: format!("error:{code}"),
        code,
    }
}

fn empty_cell_outcome() -> Outcome {
    Outcome::EmptyCell {
        digest_payload: "empty_cell".to_string(),
    }
}

fn harness_error_outcome(message: impl Into<String>) -> Outcome {
    let message = message.into();
    Outcome::HarnessError {
        digest_payload: format!("harness_error:{message}"),
        message,
    }
}

fn outcome_digest(outcome: &Outcome) -> &str {
    match outcome {
        Outcome::Number { digest_payload, .. }
        | Outcome::Text { digest_payload, .. }
        | Outcome::Logical { digest_payload, .. }
        | Outcome::Error { digest_payload, .. }
        | Outcome::EmptyCell { digest_payload }
        | Outcome::Array { digest_payload, .. }
        | Outcome::HarnessError { digest_payload, .. } => digest_payload,
    }
}

fn array_cell_to_outcome(cell: &ArrayCellValue) -> Outcome {
    match cell {
        ArrayCellValue::Number(value) => number_outcome(*value),
        ArrayCellValue::Text(value) => text_outcome(value.to_string_lossy()),
        ArrayCellValue::Logical(value) => logical_outcome(*value),
        ArrayCellValue::Error(code) => error_outcome(*code),
        ArrayCellValue::EmptyCell => empty_cell_outcome(),
    }
}

fn array_to_outcome(array: EvalArray) -> Outcome {
    let shape = array.shape();
    let mut rows = Vec::with_capacity(shape.rows);
    let mut cell_digests = Vec::with_capacity(shape.cell_count());

    for row in 0..shape.rows {
        let mut row_outcomes = Vec::with_capacity(shape.cols);
        for col in 0..shape.cols {
            let outcome = array
                .get(row, col)
                .map(array_cell_to_outcome)
                .unwrap_or_else(|| harness_error_outcome("array_get_out_of_bounds"));
            cell_digests.push(outcome_digest(&outcome).to_string());
            row_outcomes.push(outcome);
        }
        rows.push(row_outcomes);
    }

    Outcome::Array {
        rows: shape.rows,
        cols: shape.cols,
        cells: rows,
        digest_payload: format!(
            "array:{}x{}:[{}]",
            shape.rows,
            shape.cols,
            cell_digests.join("|")
        ),
    }
}

fn value_to_outcome(value: EvalValue) -> Outcome {
    match value {
        EvalValue::Number(value) => number_outcome(value),
        EvalValue::Text(text) => text_outcome(text.to_string_lossy()),
        EvalValue::Logical(value) => logical_outcome(value),
        EvalValue::Error(code) => error_outcome(code),
        EvalValue::Array(array) => array_to_outcome(array),
        EvalValue::Reference(_) | EvalValue::Lambda(_) => {
            harness_error_outcome("non_materialized_reference_or_lambda")
        }
    }
}

fn evaluate_case(case: CaseRecord) -> OutcomeRecord {
    let args = match case
        .args
        .iter()
        .map(input_to_call_arg)
        .collect::<Result<Vec<_>, _>>()
    {
        Ok(args) => args,
        Err(message) => {
            return OutcomeRecord {
                schema_version: "oxfunc.smart_fuzzer.array_outcome.v0",
                case_id: case.case_id,
                function_id: case.function_id,
                formula_text: case.formula_text,
                evaluator_id: "oxfunc_core.surface_dispatch.array_tranche_local_eval/0.1.0",
                execution_status: "local_case_materialization_error",
                outcome: harness_error_outcome(message),
            };
        }
    };

    let resolver = NoResolver;
    let outcome =
        eval_surface_value_call(&case.function_id, &args, &resolver, None, None, None, None)
            .map(value_to_outcome)
            .unwrap_or_else(error_outcome);

    OutcomeRecord {
        schema_version: "oxfunc.smart_fuzzer.array_outcome.v0",
        case_id: case.case_id,
        function_id: case.function_id,
        formula_text: case.formula_text,
        evaluator_id: "oxfunc_core.surface_dispatch.array_tranche_local_eval/0.1.0",
        execution_status: "ok",
        outcome,
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (cases_path, out_path) = parse_args()
        .map_err(|message| std::io::Error::new(std::io::ErrorKind::InvalidInput, message))?;

    let input = BufReader::new(File::open(cases_path)?);
    let mut output = BufWriter::new(File::create(out_path)?);
    for line in input.lines() {
        let line = line?;
        let line = line.trim_start_matches('\u{feff}');
        if line.trim().is_empty() {
            continue;
        }
        let case_json: JsonValue = serde_json::from_str(line)?;
        let case = case_from_json(case_json)
            .map_err(|message| std::io::Error::new(std::io::ErrorKind::InvalidData, message))?;
        let outcome = evaluate_case(case);
        serde_json::to_writer(&mut output, &outcome)?;
        output.write_all(b"\n")?;
    }
    output.flush()?;
    Ok(())
}
