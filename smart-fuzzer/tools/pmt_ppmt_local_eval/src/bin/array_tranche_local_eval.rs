use oxfunc_core::functions::a1_refs::parse_a1_reference;
use oxfunc_core::functions::rand_fn::RandomProvider;
use oxfunc_core::functions::surface_dispatch::eval_surface_value_call;
use oxfunc_core::locale_format::LocaleFormatContext;
use oxfunc_core::resolver::{
    CallerContext, ReferenceDereferenceRequest, ReferenceEnumerationRequest,
    ReferenceResolutionError, ReferenceSystemCapabilities, ReferenceSystemProvider,
    ResolvedReferenceCell, ResolvedReferenceExtent, ResolvedReferenceValues,
};
use oxfunc_core::value::{
    CalcArray, CalcValue, CoreValue, ExcelText, ReferenceKind, ReferenceLike, WorksheetErrorCode,
};
use serde::Serialize;
use serde_json::Value as JsonValue;
use std::collections::BTreeMap;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::PathBuf;

#[cfg(feature = "oxfml-locale")]
#[path = "array_tranche_local_eval/locale.rs"]
mod locale;

#[derive(Debug)]
struct CaseRecord {
    case_id: String,
    function_id: String,
    formula_text: String,
    args: Vec<JsonValue>,
    cell_fixture: Vec<FixtureRecord>,
    formula_cell: Option<String>,
    now_serial: Option<f64>,
    random_provider: Option<String>,
}

struct FixedRandomProvider {
    value: f64,
}

impl RandomProvider for FixedRandomProvider {
    fn random_unit(&self) -> f64 {
        self.value
    }
}

static FIXED_RANDOM_PROVIDER_05: FixedRandomProvider = FixedRandomProvider { value: 0.5 };

fn random_provider_for_case(
    provider: Option<&str>,
) -> Result<Option<&'static dyn RandomProvider>, String> {
    match provider {
        Some("fixed_0_5") => Ok(Some(&FIXED_RANDOM_PROVIDER_05)),
        None => Ok(None),
        Some(other) => Err(format!("unsupported random_provider '{other}'")),
    }
}

#[derive(Debug)]
struct FixtureRecord {
    target: String,
    value: JsonValue,
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

struct CaseResolver {
    by_target: BTreeMap<String, CalcValue>,
    caller: Option<CallerContext>,
}

impl CaseResolver {
    fn fixture_value(&self, target: &str) -> Option<CalcValue> {
        if let Some(value) = self.by_target.get(target) {
            return Some(value.clone());
        }

        // INDEX and similar adapters can request a subrange of a fixture. Keep
        // its worksheet prefix and copy the original typed cells unchanged.
        let requested = parse_a1_reference(target)?;
        for (fixture_target, value) in &self.by_target {
            let Some(fixture) = parse_a1_reference(fixture_target) else {
                continue;
            };
            if requested.prefix != fixture.prefix
                || requested.start_row < fixture.start_row
                || requested.end_row > fixture.end_row
                || requested.start_col < fixture.start_col
                || requested.end_col > fixture.end_col
            {
                continue;
            }
            let CoreValue::Array(array) = value.core() else {
                if requested.height() == 1 && requested.width() == 1 {
                    return Some(value.clone());
                }
                continue;
            };
            if array.shape().rows != fixture.height() || array.shape().cols != fixture.width() {
                continue;
            }
            let row_offset = requested.start_row - fixture.start_row;
            let col_offset = requested.start_col - fixture.start_col;
            if requested.height() == 1 && requested.width() == 1 {
                return array.get(row_offset, col_offset).cloned();
            }
            let rows = (0..requested.height())
                .map(|row| {
                    (0..requested.width())
                        .map(|col| array.get(row_offset + row, col_offset + col).cloned())
                        .collect::<Option<Vec<_>>>()
                })
                .collect::<Option<Vec<_>>>()?;
            return CalcArray::from_rows(rows).map(CalcValue::array);
        }
        // A packet may declare each worksheet cell separately while passing a
        // rectangular reference to the function. Assemble only declared cells:
        // absent fixture coordinates remain unresolved, never invented blanks.
        let mut cells = BTreeMap::new();
        for (fixture_target, value) in &self.by_target {
            let Some(fixture) = parse_a1_reference(fixture_target) else {
                continue;
            };
            if requested.prefix != fixture.prefix {
                continue;
            }
            let row_start = requested.start_row.max(fixture.start_row);
            let row_end = requested.end_row.min(fixture.end_row);
            let col_start = requested.start_col.max(fixture.start_col);
            let col_end = requested.end_col.min(fixture.end_col);
            if row_start > row_end || col_start > col_end {
                continue;
            }
            let array = match value.core() {
                CoreValue::Array(array)
                    if array.shape().rows == fixture.height()
                        && array.shape().cols == fixture.width() =>
                {
                    Some(array)
                }
                CoreValue::Array(_) => continue,
                _ if fixture.height() == 1 && fixture.width() == 1 => None,
                _ => continue,
            };
            for row in row_start..=row_end {
                for col in col_start..=col_end {
                    let cell = match array {
                        Some(array) => array
                            .get(row - fixture.start_row, col - fixture.start_col)?
                            .clone(),
                        None => value.clone(),
                    };
                    if let Some(previous) = cells.insert((row, col), cell.clone()) {
                        if previous != cell {
                            // Fixture order is not retained in this resolver;
                            // conflicting overlapping declarations are ambiguous.
                            return None;
                        }
                    }
                }
            }
        }
        if cells.len() != requested.height().checked_mul(requested.width())? {
            return None;
        }
        if requested.height() == 1 && requested.width() == 1 {
            return cells.remove(&(requested.start_row, requested.start_col));
        }
        let rows = (requested.start_row..=requested.end_row)
            .map(|row| {
                (requested.start_col..=requested.end_col)
                    .map(|col| cells.remove(&(row, col)))
                    .collect::<Option<Vec<_>>>()
            })
            .collect::<Option<Vec<_>>>()?;
        CalcArray::from_rows(rows).map(CalcValue::array)
    }
}

impl ReferenceSystemProvider for CaseResolver {
    fn capabilities(&self) -> ReferenceSystemCapabilities {
        ReferenceSystemCapabilities::permissive_local()
    }

    fn dereference(
        &self,
        request: &ReferenceDereferenceRequest,
    ) -> Result<CalcValue, ReferenceResolutionError> {
        self.fixture_value(request.reference.target())
            .ok_or_else(|| ReferenceResolutionError::UnresolvedReference {
                target: request.reference.target().to_string(),
            })
    }

    fn enumerate_values(
        &self,
        request: &ReferenceEnumerationRequest,
    ) -> Result<Option<ResolvedReferenceValues>, ReferenceResolutionError> {
        Ok(self.fixture_value(request.reference.target()).map(|value| {
            let (extent, cells) = match value.core() {
                CoreValue::Array(array) => {
                    let shape = array.shape();
                    let mut cells = Vec::with_capacity(shape.cell_count());
                    for row in 0..shape.rows {
                        for col in 0..shape.cols {
                            cells.push(ResolvedReferenceCell::new(
                                row + 1,
                                col + 1,
                                array
                                    .get(row, col)
                                    .expect("fixture shape validated")
                                    .clone(),
                            ));
                        }
                    }
                    (ResolvedReferenceExtent::new(shape.rows, shape.cols), cells)
                }
                _ => (
                    ResolvedReferenceExtent::new(1, 1),
                    vec![ResolvedReferenceCell::new(1, 1, value)],
                ),
            };
            ResolvedReferenceValues::new(
                extent,
                cells,
                Some(format!(
                    "array_tranche_local_eval_fixture:{}",
                    request.reference.target()
                )),
            )
        }))
    }

    fn caller_context(&self) -> Option<CallerContext> {
        self.caller.clone()
    }
}

fn usage(program: &str) -> String {
    format!(
        "usage: {program} --cases <cases.jsonl> --out <local-outcomes.jsonl> [--locale-profile <canonical-id> --locale-profile-record <record.json> [--use-recorded-locale-settings]] (locale options require --features oxfml-locale)"
    )
}

struct RunOptions {
    cases: PathBuf,
    out: PathBuf,
    locale_profile: Option<String>,
    locale_profile_record: Option<PathBuf>,
    use_recorded_locale_settings: bool,
}

fn parse_args() -> Result<RunOptions, String> {
    let args: Vec<String> = env::args().collect();
    let mut cases = None;
    let mut out = None;
    let mut locale_profile = None;
    let mut locale_profile_record = None;
    let mut use_recorded_locale_settings = false;
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
            "--locale-profile" => {
                index += 1;
                locale_profile = args.get(index).cloned();
            }
            "--locale-profile-record" => {
                index += 1;
                locale_profile_record = args.get(index).map(PathBuf::from);
            }
            "--use-recorded-locale-settings" => use_recorded_locale_settings = true,
            _ => return Err(usage(&args[0])),
        }
        index += 1;
    }
    if locale_profile.is_some() != locale_profile_record.is_some() {
        return Err(
            "--locale-profile and --locale-profile-record must be supplied together".to_string(),
        );
    }
    if use_recorded_locale_settings && locale_profile.is_none() {
        return Err(
            "--use-recorded-locale-settings requires explicit profile and record".to_string(),
        );
    }
    match (cases, out) {
        (Some(cases), Some(out)) => Ok(RunOptions {
            cases,
            out,
            locale_profile,
            locale_profile_record,
            use_recorded_locale_settings,
        }),
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

fn optional_input_field<'a>(input: &'a JsonValue, field: &str) -> Option<&'a JsonValue> {
    input.as_object().and_then(|object| object.get(field))
}

fn required_string_field(input: &JsonValue, field: &str) -> Result<String, String> {
    input_field(input, field)?
        .as_str()
        .map(ToOwned::to_owned)
        .ok_or_else(|| format!("case field is not a string: {field}"))
}

fn optional_string_field(input: &JsonValue, field: &str) -> Result<Option<String>, String> {
    match optional_input_field(input, field) {
        None | Some(JsonValue::Null) => Ok(None),
        Some(value) => value
            .as_str()
            .map(|text| Some(text.to_owned()))
            .ok_or_else(|| format!("case field is not a string: {field}")),
    }
}

fn optional_f64_field(input: &JsonValue, field: &str) -> Result<Option<f64>, String> {
    match optional_input_field(input, field) {
        None | Some(JsonValue::Null) => Ok(None),
        Some(value) => value
            .as_f64()
            .map(Some)
            .ok_or_else(|| format!("case field is not a number: {field}")),
    }
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
    let cell_fixture = match optional_input_field(&input, "cell_fixture") {
        None | Some(JsonValue::Null) => Vec::new(),
        Some(JsonValue::Array(items)) => items
            .iter()
            .map(|item| {
                Ok(FixtureRecord {
                    target: required_string_field(item, "target")?,
                    value: input_field(item, "value")?.clone(),
                })
            })
            .collect::<Result<Vec<_>, String>>()?,
        Some(JsonValue::Object(object)) if object.is_empty() => Vec::new(),
        Some(JsonValue::Object(_)) => vec![FixtureRecord {
            target: required_string_field(input_field(&input, "cell_fixture")?, "target")?,
            value: input_field(input_field(&input, "cell_fixture")?, "value")?.clone(),
        }],
        Some(other) => return Err(format!("cell_fixture is not an array: {other}")),
    };

    Ok(CaseRecord {
        case_id: required_string_field(&input, "case_id")?,
        function_id: required_string_field(&input, "function_id")?,
        formula_text: required_string_field(&input, "formula_text")?,
        args,
        cell_fixture,
        formula_cell: optional_string_field(&input, "formula_cell")?,
        now_serial: optional_f64_field(&input, "now_serial")?,
        random_provider: optional_string_field(&input, "random_provider")?,
    })
}

fn parse_reference_kind(kind: &str) -> Result<ReferenceKind, String> {
    match kind {
        "A1" | "a1" | "single_cell" => Ok(ReferenceKind::A1),
        "Area" | "area" | "rectangular_area" => Ok(ReferenceKind::Area),
        "MultiArea" | "multi_area" | "same_sheet_multi_area" => Ok(ReferenceKind::MultiArea),
        "ThreeD" | "three_d" | "cross_sheet_reference" => Ok(ReferenceKind::ThreeD),
        "Structured" | "structured" | "structured_reference" => Ok(ReferenceKind::Structured),
        "SpillAnchor" | "spill_anchor" => Ok(ReferenceKind::SpillAnchor),
        other => Err(format!("unsupported reference kind: {other}")),
    }
}

fn input_to_reference(input: &JsonValue) -> Result<ReferenceLike, String> {
    let kind = input_field(input, "reference_kind")?
        .as_str()
        .ok_or_else(|| "reference input has non-string reference_kind".to_string())?;
    let target = input_field(input, "target")?
        .as_str()
        .ok_or_else(|| "reference input has non-string target".to_string())?;
    Ok(ReferenceLike::new(parse_reference_kind(kind)?, target))
}

fn input_to_calc_value(input: &JsonValue) -> Result<CalcValue, String> {
    match input_kind(input)? {
        "number" => Ok(CalcValue::number(
            input_field(input, "value")?
                .as_f64()
                .ok_or_else(|| "number input has non-numeric value".to_string())?,
        )),
        "text" => {
            let value = input_field(input, "value")?
                .as_str()
                .ok_or_else(|| "text input has non-string value".to_string())?;
            Ok(CalcValue::text(ExcelText::from_interop_assignment(value)))
        }
        "logical" => Ok(CalcValue::logical(
            input_field(input, "value")?
                .as_bool()
                .ok_or_else(|| "logical input has non-boolean value".to_string())?,
        )),
        "error" => {
            let code = input_field(input, "code")?
                .as_str()
                .ok_or_else(|| "error input has non-string code".to_string())?;
            Ok(CalcValue::error(parse_worksheet_error_code(code)?))
        }
        "empty_cell" => Ok(CalcValue::empty()),
        "missing_arg" => Ok(CalcValue::missing()),
        "array" => Ok(CalcValue::array(input_to_array(input_field(
            input, "rows",
        )?)?)),
        "reference" => Ok(CalcValue::reference(input_to_reference(input)?)),
        other => Err(format!("unsupported input kind: {other}")),
    }
}

fn input_to_fixture_value(input: &JsonValue) -> Result<CalcValue, String> {
    match input_kind(input)? {
        "number" => Ok(CalcValue::number(
            input_field(input, "value")?
                .as_f64()
                .ok_or_else(|| "number input has non-numeric value".to_string())?,
        )),
        "text" => {
            let value = input_field(input, "value")?
                .as_str()
                .ok_or_else(|| "text input has non-string value".to_string())?;
            Ok(CalcValue::text(ExcelText::from_interop_assignment(value)))
        }
        "logical" => Ok(CalcValue::logical(
            input_field(input, "value")?
                .as_bool()
                .ok_or_else(|| "logical input has non-boolean value".to_string())?,
        )),
        "error" => {
            let code = input_field(input, "code")?
                .as_str()
                .ok_or_else(|| "error input has non-string code".to_string())?;
            Ok(CalcValue::error(parse_worksheet_error_code(code)?))
        }
        "array" => Ok(CalcValue::array(input_to_array(input_field(
            input, "rows",
        )?)?)),
        "empty_cell" => Ok(CalcValue::empty()),
        "reference" => Ok(CalcValue::reference(input_to_reference(input)?)),
        "missing_arg" => Err("missing_arg is not a fixture value".to_string()),
        other => Err(format!("unsupported fixture value kind: {other}")),
    }
}

fn input_to_array(rows: &JsonValue) -> Result<CalcArray, String> {
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

    CalcArray::from_rows(converted_rows).ok_or_else(|| "invalid array input shape".to_string())
}

fn input_to_array_cell(input: &JsonValue) -> Result<CalcValue, String> {
    match input_kind(input)? {
        "number" => Ok(CalcValue::number(
            input_field(input, "value")?
                .as_f64()
                .ok_or_else(|| "number array cell has non-numeric value".to_string())?,
        )),
        "text" => {
            let value = input_field(input, "value")?
                .as_str()
                .ok_or_else(|| "text array cell has non-string value".to_string())?;
            Ok(CalcValue::text(ExcelText::from_interop_assignment(value)))
        }
        "logical" => Ok(CalcValue::logical(
            input_field(input, "value")?
                .as_bool()
                .ok_or_else(|| "logical array cell has non-boolean value".to_string())?,
        )),
        "error" => {
            let code = input_field(input, "code")?
                .as_str()
                .ok_or_else(|| "error array cell has non-string code".to_string())?;
            Ok(CalcValue::error(parse_worksheet_error_code(code)?))
        }
        "empty_cell" => Ok(CalcValue::empty()),
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

fn panic_payload_to_string(payload: &(dyn std::any::Any + Send)) -> String {
    if let Some(message) = payload.downcast_ref::<String>() {
        message.clone()
    } else if let Some(message) = payload.downcast_ref::<&'static str>() {
        (*message).to_string()
    } else {
        "unknown panic payload".to_string()
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

fn array_to_outcome(array: CalcArray) -> Outcome {
    let shape = array.shape();
    let mut rows = Vec::with_capacity(shape.rows);
    let mut cell_digests = Vec::with_capacity(shape.cell_count());

    for row in 0..shape.rows {
        let mut row_outcomes = Vec::with_capacity(shape.cols);
        for col in 0..shape.cols {
            let outcome = array
                .get(row, col)
                .map(value_to_outcome)
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

fn value_to_outcome(value: &CalcValue) -> Outcome {
    if value.callable_value().is_some() {
        return harness_error_outcome("non_materialized_callable");
    }

    match value.core() {
        CoreValue::Number(value) => number_outcome(*value),
        CoreValue::Text(text) => text_outcome(text.to_string_lossy()),
        CoreValue::Logical(value) => logical_outcome(*value),
        CoreValue::Error(code) => error_outcome(*code),
        CoreValue::Empty | CoreValue::Missing => empty_cell_outcome(),
        CoreValue::Array(array) => array_to_outcome(array.clone()),
        CoreValue::Reference(_) => harness_error_outcome("non_materialized_reference"),
    }
}

fn parse_caller_context(cell: Option<&str>) -> Option<CallerContext> {
    let cell = cell?;
    let mut col = 0usize;
    let mut row_text = String::new();
    for ch in cell.chars() {
        if ch.is_ascii_alphabetic() {
            let upper = ch.to_ascii_uppercase() as u8;
            col = col * 26 + usize::from(upper - b'A' + 1);
        } else if ch.is_ascii_digit() {
            row_text.push(ch);
        }
    }
    let row = row_text.parse::<usize>().ok()?;
    if row == 0 || col == 0 {
        return None;
    }
    Some(CallerContext {
        prefix: None,
        row,
        col,
    })
}

fn evaluate_case(case: CaseRecord, locale_ctx: Option<&LocaleFormatContext<'_>>) -> OutcomeRecord {
    let args = match case
        .args
        .iter()
        .map(input_to_calc_value)
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

    let fixture_result = case
        .cell_fixture
        .iter()
        .map(|fixture| {
            Ok((
                fixture.target.clone(),
                input_to_fixture_value(&fixture.value)?,
            ))
        })
        .collect::<Result<BTreeMap<_, _>, String>>();
    let resolver = match fixture_result {
        Ok(by_target) => CaseResolver {
            by_target,
            caller: parse_caller_context(case.formula_cell.as_deref()),
        },
        Err(message) => {
            return OutcomeRecord {
                schema_version: "oxfunc.smart_fuzzer.array_outcome.v0",
                case_id: case.case_id,
                function_id: case.function_id,
                formula_text: case.formula_text,
                evaluator_id: "oxfunc_core.surface_dispatch.array_tranche_local_eval/0.1.0",
                execution_status: "local_fixture_materialization_error",
                outcome: harness_error_outcome(message),
            };
        }
    };
    let random_provider = match random_provider_for_case(case.random_provider.as_deref()) {
        Ok(provider) => provider,
        Err(message) => {
            return OutcomeRecord {
                schema_version: "oxfunc.smart_fuzzer.local_eval_outcome.v0",
                case_id: case.case_id,
                function_id: case.function_id,
                formula_text: case.formula_text,
                evaluator_id: "oxfunc_core.surface_dispatch.array_tranche_local_eval/0.1.0",
                execution_status: "local_fixture_materialization_error",
                outcome: harness_error_outcome(message),
            };
        }
    };
    let eval_result = catch_unwind(AssertUnwindSafe(|| {
        eval_surface_value_call(
            &case.function_id,
            &args,
            &resolver,
            case.now_serial,
            random_provider,
            locale_ctx,
            None,
        )
    }));

    let (execution_status, outcome) = match eval_result {
        Ok(result) => (
            "ok",
            result
                .as_ref()
                .map(value_to_outcome)
                .unwrap_or_else(|code| error_outcome(*code)),
        ),
        Err(payload) => (
            "local_eval_panic",
            harness_error_outcome(panic_payload_to_string(payload.as_ref())),
        ),
    };

    OutcomeRecord {
        schema_version: "oxfunc.smart_fuzzer.array_outcome.v0",
        case_id: case.case_id,
        function_id: case.function_id,
        formula_text: case.formula_text,
        evaluator_id: "oxfunc_core.surface_dispatch.array_tranche_local_eval/0.1.0",
        execution_status,
        outcome,
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = parse_args()
        .map_err(|message| std::io::Error::new(std::io::ErrorKind::InvalidInput, message))?;

    #[cfg(feature = "oxfml-locale")]
    let locale_binding = match (&options.locale_profile, &options.locale_profile_record) {
        (Some(profile), Some(record)) => Some(
            locale::bind(profile, record, options.use_recorded_locale_settings).map_err(
                |message| std::io::Error::new(std::io::ErrorKind::InvalidInput, message),
            )?,
        ),
        _ => None,
    };
    #[cfg(not(feature = "oxfml-locale"))]
    if options.locale_profile.is_some()
        || options.locale_profile_record.is_some()
        || options.use_recorded_locale_settings
    {
        return Err(
            "locale binding requires Cargo feature oxfml-locale; no fallback provider was selected"
                .into(),
        );
    }
    #[cfg(feature = "oxfml-locale")]
    let locale_ctx = locale_binding.as_ref().map(|binding| &binding.context);
    #[cfg(not(feature = "oxfml-locale"))]
    let locale_ctx = None;

    let input = BufReader::new(File::open(options.cases)?);
    let mut output = BufWriter::new(File::create(&options.out)?);
    for line in input.lines() {
        let line = line?;
        let line = line.trim_start_matches('\u{feff}');
        if line.trim().is_empty() {
            continue;
        }
        let case_json: JsonValue = serde_json::from_str(line)?;
        let case = case_from_json(case_json)
            .map_err(|message| std::io::Error::new(std::io::ErrorKind::InvalidData, message))?;
        let outcome = evaluate_case(case, locale_ctx);
        #[cfg(feature = "oxfml-locale")]
        if let Some(binding) = &locale_binding {
            let mut value = serde_json::to_value(&outcome)?;
            value["locale_context"] = serde_json::json!({
                "provider":binding.provenance["provider"],
                "profile_id":binding.provenance["profile_id"],
                "date_system":binding.provenance["date_system"],
                "profile_validation":binding.provenance["profile_validation"],
                "profile_record_sha256":binding.provenance["profile_record_sha256"],
            });
            serde_json::to_writer(&mut output, &value)?;
        } else {
            serde_json::to_writer(&mut output, &outcome)?;
        }
        #[cfg(not(feature = "oxfml-locale"))]
        serde_json::to_writer(&mut output, &outcome)?;
        output.write_all(b"\n")?;
    }
    output.flush()?;
    #[cfg(feature = "oxfml-locale")]
    if let Some(binding) = &locale_binding {
        let mut sidecar =
            BufWriter::new(File::create(options.out.with_extension("provenance.json"))?);
        serde_json::to_writer_pretty(&mut sidecar, &binding.provenance)?;
        sidecar.flush()?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxfunc_core::resolver::materialize_resolved_reference_values;
    use serde_json::json;

    #[test]
    fn typed_json_decimal_ingress_preserves_original_binary64_bits() {
        // Original source bits from the independent 10,000-row ABS ingress
        // probe, not values inferred from the JSON decoder under test.
        for (decimal, expected) in [
            ("8.76062371711073e-303", 0x013807eb4fb3ba95_u64),
            ("1.9737081258726104e+244", 0x72a71fed759c9875),
            ("6.585554626534559e+304", 0x7f38020efc84768a),
            ("2.1381717900254593e+263", 0x769b2939ce1c779f),
            ("3.231130561040196e-231", 0x101410c82ffe240e),
        ] {
            let source = format!(
                r#"{{"case_id":"ingress","function_id":"FUNC.ABS","formula_text":"=ABS(A1)","args":[{{"kind":"number","value":{decimal}}}]}}"#
            );
            let case = case_from_json(serde_json::from_str(&source).unwrap()).unwrap();
            let result = evaluate_case(case, None);
            let Outcome::Number { bits_hex, .. } = result.outcome else {
                panic!("{result:?}");
            };
            assert_eq!(bits_hex, format!("0x{expected:016x}"), "{decimal}");
        }
    }

    fn matrix_resolver() -> CaseResolver {
        CaseResolver {
            by_target: BTreeMap::from([(
                "A1:C2".to_string(),
                CalcValue::array(
                    CalcArray::from_rows(vec![
                        vec![
                            CalcValue::number(7.0),
                            CalcValue::empty(),
                            CalcValue::logical(true),
                        ],
                        vec![
                            CalcValue::text(ExcelText::from_interop_assignment("02")),
                            CalcValue::error(WorksheetErrorCode::NA),
                            CalcValue::number(-0.0),
                        ],
                    ])
                    .unwrap(),
                ),
            )]),
            caller: None,
        }
    }

    #[test]
    fn fixture_enumeration_preserves_rectangular_shape_and_typed_cells() {
        let resolver = matrix_resolver();
        let values = resolver
            .enumerate_values(&ReferenceEnumerationRequest {
                reference: ReferenceLike::new(ReferenceKind::Area, "A1:C2"),
            })
            .unwrap()
            .unwrap();
        assert_eq!(values.declared_extent, ResolvedReferenceExtent::new(2, 3));
        assert_eq!(values.defined_cardinality, 6);
        assert_eq!(
            (values.defined_cells[0].row, values.defined_cells[0].col),
            (1, 1)
        );
        assert_eq!(
            (values.defined_cells[5].row, values.defined_cells[5].col),
            (2, 3)
        );
        let materialized = materialize_resolved_reference_values(&values).unwrap();
        assert_eq!(
            value_to_outcome(&CalcValue::array(materialized)).digest(),
            value_to_outcome(resolver.by_target.get("A1:C2").unwrap()).digest()
        );
    }

    #[test]
    fn fixture_projection_keeps_sheet_identity_and_original_cell_types() {
        let resolver = matrix_resolver();
        assert!(matches!(
            resolver.fixture_value("$B$2").unwrap().core(),
            CoreValue::Error(WorksheetErrorCode::NA)
        ));
        assert!(resolver.fixture_value("Other!B2").is_none());
        assert!(resolver.fixture_value("D2").is_none());
        let row = resolver.fixture_value("A2:C2").unwrap();
        assert_eq!(
            value_to_outcome(&row).digest(),
            "array:1x3:[text:02|error:NA|number:0x8000000000000000]"
        );
        let scalar = resolver
            .enumerate_values(&ReferenceEnumerationRequest {
                reference: ReferenceLike::new(ReferenceKind::A1, "A1"),
            })
            .unwrap()
            .unwrap();
        assert_eq!(scalar.declared_extent, ResolvedReferenceExtent::new(1, 1));
        assert_eq!(
            (scalar.defined_cells[0].row, scalar.defined_cells[0].col),
            (1, 1)
        );
        materialize_resolved_reference_values(&scalar).unwrap();
    }

    impl Outcome {
        fn digest(&self) -> &str {
            outcome_digest(self)
        }
    }

    #[test]
    fn fixture_assembles_explicit_cells_and_areas_without_inventing_blanks() {
        let mut resolver = CaseResolver {
            by_target: BTreeMap::from([
                ("Sheet1!B1".to_string(), CalcValue::number(7.0)),
                ("Sheet1!C1".to_string(), CalcValue::empty()),
                ("Sheet1!D1".to_string(), CalcValue::logical(true)),
                (
                    "Sheet1!B2:D2".to_string(),
                    CalcValue::array(
                        CalcArray::from_rows(vec![vec![
                            CalcValue::text(ExcelText::from_interop_assignment("02")),
                            CalcValue::error(WorksheetErrorCode::NA),
                            CalcValue::number(-0.0),
                        ]])
                        .unwrap(),
                    ),
                ),
            ]),
            caller: None,
        };
        let request = ReferenceEnumerationRequest {
            reference: ReferenceLike::new(ReferenceKind::Area, "Sheet1!$B$1:$D$2"),
        };
        let values = resolver.enumerate_values(&request).unwrap().unwrap();
        assert_eq!(values.declared_extent, ResolvedReferenceExtent::new(2, 3));
        assert_eq!(values.defined_cardinality, 6);
        let value = CalcValue::array(materialize_resolved_reference_values(&values).unwrap());
        assert_eq!(value_to_outcome(&value).digest(),
            "array:2x3:[number:0x401c000000000000|empty_cell|logical:true|text:02|error:NA|number:0x8000000000000000]");
        assert!(resolver.fixture_value("Other!B1:D2").is_none());
        resolver.by_target.remove("Sheet1!C1");
        assert!(resolver.fixture_value("Sheet1!B1:D2").is_none());
    }

    #[test]
    fn fixture_surface_replays_criteria_lookup_and_index() {
        let number = |value| json!({"kind":"number", "value":value});
        let reference =
            |target| json!({"kind":"reference", "reference_kind":"Area", "target":target});
        let fixture = json!([
            {"target":"A1:A3", "value":{"kind":"array", "rows":[[number(1)],[number(2)],[number(3)]]}},
            {"target":"B1:B3", "value":{"kind":"array", "rows":[[number(10)],[number(20)],[number(30)]]}},
            {"target":"D1:E2", "value":{"kind":"array", "rows":[[number(1),number(2)],[number(3),number(4)]]}}
        ]);
        for (function_id, args, expected) in [
            (
                "FUNC.COUNTIF",
                vec![reference("A1:A3"), json!({"kind":"text","value":">1"})],
                2.0,
            ),
            (
                "FUNC.SUMIF",
                vec![
                    reference("A1:A3"),
                    json!({"kind":"text","value":">1"}),
                    reference("B1:B3"),
                ],
                50.0,
            ),
            (
                "FUNC.MATCH",
                vec![number(2), reference("A1:A3"), number(0)],
                2.0,
            ),
            (
                "FUNC.XMATCH",
                vec![number(2), reference("A1:A3"), number(0)],
                2.0,
            ),
            (
                "FUNC.XLOOKUP",
                vec![number(2), reference("A1:A3"), reference("B1:B3")],
                20.0,
            ),
            (
                "FUNC.INDEX",
                vec![reference("D1:E2"), number(2), number(2)],
                4.0,
            ),
            ("FUNC.MDETERM", vec![reference("D1:E2")], -2.0),
        ] {
            let record =
                case_from_json(json!({"case_id":"fixture-smoke", "function_id":function_id,
                "formula_text":"fixture smoke", "args":args, "cell_fixture":fixture}))
                .unwrap();
            let result = evaluate_case(record, None);
            assert_eq!(result.execution_status, "ok");
            assert_eq!(
                result.outcome.digest(),
                number_outcome(expected).digest(),
                "{function_id}"
            );
        }
    }
}
