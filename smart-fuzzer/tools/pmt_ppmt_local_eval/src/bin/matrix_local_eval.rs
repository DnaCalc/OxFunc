// matrix_local_eval
//
// Reads a JSONL fixture file of matrix-bearing or scalar OxFunc cases
// and writes per-case local outcomes. Each case has an `args_typed`
// array of tagged arguments:
//   { "kind": "number",  "value": 1.5 }
//   { "kind": "logical", "value": true }
//   { "kind": "matrix",  "rows": 2, "cols": 2, "values": [1,2,3,4] }
//
// For functions that return a matrix (e.g. MINVERSE), the outcome
// contains a per-cell list. For functions that return a scalar, the
// outcome contains a single cell. Used by W097 R-F MINVERSE re-sweep.

use oxfunc_core::functions::surface_dispatch::eval_surface_value_call;
use oxfunc_core::resolver::{RefResolutionError, ReferenceResolver, ResolverCapabilities};
use oxfunc_core::value::{ArrayCellValue, CallArgValue, EvalArray, EvalValue, ReferenceLike};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum TypedArgIn {
    Number {
        value: f64,
    },
    Logical {
        value: bool,
    },
    Matrix {
        rows: usize,
        cols: usize,
        values: Vec<f64>,
    },
}

#[derive(Debug, Deserialize)]
struct CaseRecord {
    case_id: String,
    function_id: String,
    args_typed: Vec<TypedArgIn>,
}

#[derive(Debug, Serialize)]
struct CellOutcome {
    row: usize,
    col: usize,
    kind: String,
    value: Option<f64>,
    bits_hex: Option<String>,
    error_code: Option<String>,
    digest_payload: String,
}

#[derive(Debug, Serialize)]
struct OutcomeRecord {
    schema_version: &'static str,
    case_id: String,
    function_id: String,
    evaluator_id: &'static str,
    execution_status: &'static str,
    rows: usize,
    cols: usize,
    cells: Vec<CellOutcome>,
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

fn typed_to_call_arg(t: &TypedArgIn) -> CallArgValue {
    match t {
        TypedArgIn::Number { value } => CallArgValue::Eval(EvalValue::Number(*value)),
        TypedArgIn::Logical { value } => CallArgValue::Eval(EvalValue::Logical(*value)),
        TypedArgIn::Matrix { rows, cols, values } => {
            let row_vecs: Vec<Vec<ArrayCellValue>> = (0..*rows)
                .map(|r| {
                    (0..*cols)
                        .map(|c| ArrayCellValue::Number(values[r * cols + c]))
                        .collect()
                })
                .collect();
            let array = EvalArray::from_rows(row_vecs).expect("non-empty rectangular matrix");
            CallArgValue::Eval(EvalValue::Array(array))
        }
    }
}

fn cell_outcome_from_array_cell(row: usize, col: usize, c: &ArrayCellValue) -> CellOutcome {
    match c {
        ArrayCellValue::Number(v) => {
            let bits = format!("0x{:016x}", v.to_bits());
            CellOutcome {
                row,
                col,
                kind: "number".into(),
                value: Some(*v),
                bits_hex: Some(bits.clone()),
                error_code: None,
                digest_payload: format!("number:{bits}"),
            }
        }
        ArrayCellValue::Logical(b) => CellOutcome {
            row,
            col,
            kind: "logical".into(),
            value: None,
            bits_hex: None,
            error_code: None,
            digest_payload: format!("logical:{b}"),
        },
        ArrayCellValue::Error(code) => CellOutcome {
            row,
            col,
            kind: "error".into(),
            value: None,
            bits_hex: None,
            error_code: Some(format!("{code:?}")),
            digest_payload: format!("error:{code:?}"),
        },
        other => CellOutcome {
            row,
            col,
            kind: "other".into(),
            value: None,
            bits_hex: None,
            error_code: None,
            digest_payload: format!("other:{other:?}"),
        },
    }
}

fn cell_outcome_from_scalar(value: EvalValue) -> CellOutcome {
    match value {
        EvalValue::Number(v) => {
            let bits = format!("0x{:016x}", v.to_bits());
            CellOutcome {
                row: 0,
                col: 0,
                kind: "number".into(),
                value: Some(v),
                bits_hex: Some(bits.clone()),
                error_code: None,
                digest_payload: format!("number:{bits}"),
            }
        }
        EvalValue::Error(code) => CellOutcome {
            row: 0,
            col: 0,
            kind: "error".into(),
            value: None,
            bits_hex: None,
            error_code: Some(format!("{code:?}")),
            digest_payload: format!("error:{code:?}"),
        },
        EvalValue::Logical(b) => CellOutcome {
            row: 0,
            col: 0,
            kind: "logical".into(),
            value: None,
            bits_hex: None,
            error_code: None,
            digest_payload: format!("logical:{b}"),
        },
        EvalValue::Text(t) => CellOutcome {
            row: 0,
            col: 0,
            kind: "text".into(),
            value: None,
            bits_hex: None,
            error_code: None,
            digest_payload: format!("text:{}", t.to_string_lossy()),
        },
        other => CellOutcome {
            row: 0,
            col: 0,
            kind: "other".into(),
            value: None,
            bits_hex: None,
            error_code: None,
            digest_payload: format!("other:{other:?}"),
        },
    }
}

fn evaluate_case(case: CaseRecord) -> OutcomeRecord {
    let args: Vec<CallArgValue> = case.args_typed.iter().map(typed_to_call_arg).collect();
    let resolver = NoResolver;
    let res = eval_surface_value_call(&case.function_id, &args, &resolver, None, None, None, None);
    match res {
        Ok(EvalValue::Array(arr)) => {
            let shape = arr.shape();
            let mut cells = Vec::with_capacity(shape.rows * shape.cols);
            for r in 0..shape.rows {
                for c in 0..shape.cols {
                    if let Some(cell) = arr.get(r, c) {
                        cells.push(cell_outcome_from_array_cell(r, c, cell));
                    }
                }
            }
            OutcomeRecord {
                schema_version: "oxfunc.smart_fuzzer.matrix_local_outcome.v0",
                case_id: case.case_id,
                function_id: case.function_id,
                evaluator_id: "oxfunc_core.surface_dispatch.matrix_local_eval/0.1.0",
                execution_status: "ok",
                rows: shape.rows,
                cols: shape.cols,
                cells,
            }
        }
        Ok(scalar) => {
            let cell = cell_outcome_from_scalar(scalar);
            OutcomeRecord {
                schema_version: "oxfunc.smart_fuzzer.matrix_local_outcome.v0",
                case_id: case.case_id,
                function_id: case.function_id,
                evaluator_id: "oxfunc_core.surface_dispatch.matrix_local_eval/0.1.0",
                execution_status: "ok",
                rows: 1,
                cols: 1,
                cells: vec![cell],
            }
        }
        Err(code) => {
            let code_str = format!("{code:?}");
            let cell = CellOutcome {
                row: 0,
                col: 0,
                kind: "error".into(),
                value: None,
                bits_hex: None,
                error_code: Some(code_str.clone()),
                digest_payload: format!("error:{code_str}"),
            };
            OutcomeRecord {
                schema_version: "oxfunc.smart_fuzzer.matrix_local_outcome.v0",
                case_id: case.case_id,
                function_id: case.function_id,
                evaluator_id: "oxfunc_core.surface_dispatch.matrix_local_eval/0.1.0",
                execution_status: "ok",
                rows: 1,
                cols: 1,
                cells: vec![cell],
            }
        }
    }
}

fn parse_args() -> Result<(PathBuf, PathBuf), String> {
    let args: Vec<String> = env::args().collect();
    let mut cases = None;
    let mut out = None;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--cases" => {
                i += 1;
                cases = args.get(i).map(PathBuf::from);
            }
            "--out" => {
                i += 1;
                out = args.get(i).map(PathBuf::from);
            }
            _ => return Err(format!("unknown arg: {}", args[i])),
        }
        i += 1;
    }
    match (cases, out) {
        (Some(c), Some(o)) => Ok((c, o)),
        _ => Err("usage: --cases <jsonl> --out <jsonl>".into()),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (cases_path, out_path) =
        parse_args().map_err(|m| std::io::Error::new(std::io::ErrorKind::InvalidInput, m))?;
    let input = BufReader::new(File::open(cases_path)?);
    let mut output = BufWriter::new(File::create(out_path)?);
    for line in input.lines() {
        let line = line?;
        let line = line.trim_start_matches('\u{feff}');
        if line.trim().is_empty() {
            continue;
        }
        let case: CaseRecord = serde_json::from_str(line)?;
        let outcome = evaluate_case(case);
        serde_json::to_writer(&mut output, &outcome)?;
        output.write_all(b"\n")?;
    }
    output.flush()?;
    Ok(())
}
