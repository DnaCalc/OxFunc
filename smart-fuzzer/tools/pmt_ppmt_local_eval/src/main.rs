use oxfunc_core::functions::surface_dispatch::{
    eval_surface_value_call, FUNC_ID_PMT, FUNC_ID_PPMT,
};
use oxfunc_core::resolver::{RefResolutionError, ReferenceResolver, ResolverCapabilities};
use oxfunc_core::value::{CallArgValue, EvalValue, ReferenceLike, WorksheetErrorCode};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
struct CaseRecord {
    case_id: String,
    function_id: String,
    args: Vec<f64>,
}

#[derive(Debug, Serialize)]
struct OutcomeRecord {
    schema_version: &'static str,
    case_id: String,
    function_id: String,
    evaluator_id: &'static str,
    execution_status: &'static str,
    outcome: Outcome,
}

#[derive(Debug, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum Outcome {
    Number {
        value: f64,
        bits_hex: String,
        digest_payload: String,
    },
    Error {
        code: String,
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

fn number_arg(value: f64) -> CallArgValue {
    CallArgValue::Eval(EvalValue::Number(value))
}

fn worksheet_error_code(code: WorksheetErrorCode) -> String {
    format!("{code:?}")
}

fn value_to_outcome(value: EvalValue) -> Outcome {
    match value {
        EvalValue::Number(value) => {
            let bits_hex = format!("0x{:016x}", value.to_bits());
            Outcome::Number {
                value,
                bits_hex: bits_hex.clone(),
                digest_payload: format!("number:{bits_hex}"),
            }
        }
        EvalValue::Error(code) => {
            let code = worksheet_error_code(code);
            Outcome::Error {
                digest_payload: format!("error:{code}"),
                code,
            }
        }
        EvalValue::Text(text) => {
            let value = text.to_string_lossy();
            Outcome::Text {
                digest_payload: format!("text:{value}"),
                value,
            }
        }
        EvalValue::Logical(value) => Outcome::Logical {
            digest_payload: format!("logical:{value}"),
            value,
        },
        EvalValue::Array(_) => Outcome::Error {
            code: "ArrayOutcomeUnexpected".to_string(),
            digest_payload: "error:ArrayOutcomeUnexpected".to_string(),
        },
        EvalValue::Reference(_) | EvalValue::Lambda(_) => Outcome::Error {
            code: "NonScalarOutcomeUnexpected".to_string(),
            digest_payload: "error:NonScalarOutcomeUnexpected".to_string(),
        },
    }
}

fn evaluate_case(case: CaseRecord) -> OutcomeRecord {
    let function_id = match case.function_id.as_str() {
        "FUNC.PMT" => FUNC_ID_PMT,
        "FUNC.PPMT" => FUNC_ID_PPMT,
        other => other,
    };
    let args: Vec<CallArgValue> = case.args.iter().copied().map(number_arg).collect();
    let resolver = NoResolver;
    let result = eval_surface_value_call(function_id, &args, &resolver, None, None, None, None)
        .map(value_to_outcome)
        .unwrap_or_else(|code| {
            let code = worksheet_error_code(code);
            Outcome::Error {
                digest_payload: format!("error:{code}"),
                code,
            }
        });

    OutcomeRecord {
        schema_version: "oxfunc.smart_fuzzer.local_outcome.v0",
        case_id: case.case_id,
        function_id: case.function_id,
        evaluator_id: "oxfunc_core.surface_dispatch.pmt_ppmt_local_eval/0.1.0",
        execution_status: "ok",
        outcome: result,
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (cases_path, out_path) = parse_args().map_err(|message| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, message)
    })?;

    let input = BufReader::new(File::open(cases_path)?);
    let mut output = BufWriter::new(File::create(out_path)?);
    for line in input.lines() {
        let line = line?;
        let line = line.trim_start_matches('\u{feff}');
        if line.trim().is_empty() {
            continue;
        }
        let case: CaseRecord = serde_json::from_str(&line)?;
        let outcome = evaluate_case(case);
        serde_json::to_writer(&mut output, &outcome)?;
        output.write_all(b"\n")?;
    }
    output.flush()?;
    Ok(())
}
