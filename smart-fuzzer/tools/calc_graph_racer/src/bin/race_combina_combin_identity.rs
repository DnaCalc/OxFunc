//! W109 COMBINA graph audit.
//!
//! Scores the current production route and the independently identified
//! stored-x87 `COMBIN(n+k-1,k)` graph against either broad-scalar JSONL
//! comparisons or W109 answered WitnessSet JSON.  This is deliberately an
//! offline scorer: it never invokes Excel and never selects a held-out row.

use oxfunc_core::functions::combin::combin_kernel;
use oxfunc_core::functions::combina::combina_kernel;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

#[derive(Clone, Copy)]
struct Row {
    n: f64,
    k: f64,
    want: u64,
}

#[derive(Default)]
struct Score {
    exact: usize,
    histogram: BTreeMap<i64, usize>,
    first_misses: Vec<String>,
}

fn parse_bits(text: &str) -> Result<u64, String> {
    let hex = text
        .strip_prefix("0x")
        .or_else(|| text.strip_prefix("0X"))
        .ok_or_else(|| format!("bit string lacks 0x prefix: {text}"))?;
    if hex.len() != 16 {
        return Err(format!("bit string is not 16 hex digits: {text}"));
    }
    u64::from_str_radix(hex, 16).map_err(|error| format!("invalid bit string {text}: {error}"))
}

fn as_numeric_bits(value: &Value, context: &str) -> Result<u64, String> {
    if value["kind"].as_str() != Some("number") {
        return Err(format!("{context}: expected a numeric Excel outcome"));
    }
    parse_bits(
        value["bits_hex"]
            .as_str()
            .ok_or_else(|| format!("{context}: numeric outcome lacks bits_hex"))?,
    )
}

fn formula_args(value: &Value, context: &str) -> Result<(f64, f64), String> {
    let formula = value["formula_text"]
        .as_str()
        .ok_or_else(|| format!("{context}: comparison row lacks args and formula_text"))?;
    let body = formula
        .strip_prefix("=COMBINA(")
        .and_then(|text| text.strip_suffix(')'))
        .ok_or_else(|| format!("{context}: unsupported formula {formula}"))?;
    let (n, k) = body
        .split_once(',')
        .ok_or_else(|| format!("{context}: formula does not have two args: {formula}"))?;
    let n = n
        .trim()
        .parse::<f64>()
        .map_err(|error| format!("{context}: invalid n in formula: {error}"))?;
    let k = k
        .trim()
        .parse::<f64>()
        .map_err(|error| format!("{context}: invalid k in formula: {error}"))?;
    Ok((n, k))
}

fn load_jsonl(path: &Path) -> Result<Vec<(String, Row)>, String> {
    let text = fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))?;
    let mut rows = Vec::new();
    for (line_index, line) in text.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let value: Value = serde_json::from_str(line)
            .map_err(|error| format!("{}:{}: {error}", path.display(), line_index + 1))?;
        if value["function_id"].as_str() != Some("FUNC.COMBINA") {
            continue;
        }
        if value["excel_outcome"]["kind"].as_str() != Some("number") {
            continue;
        }
        let id = value["case_id"]
            .as_str()
            .ok_or_else(|| format!("{}:{}: missing case_id", path.display(), line_index + 1))?;
        let (n, k) = if let Some(args) = value["args"].as_array() {
            if args.len() != 2 {
                return Err(format!("{id}: expected two args"));
            }
            let n = args[0]
                .as_f64()
                .ok_or_else(|| format!("{id}: n is not numeric"))?;
            let k = args[1]
                .as_f64()
                .ok_or_else(|| format!("{id}: k is not numeric"))?;
            (n, k)
        } else {
            formula_args(&value, id)?
        };
        let want = as_numeric_bits(&value["excel_outcome"], id)?;
        rows.push((id.to_owned(), Row { n, k, want }));
    }
    Ok(rows)
}

fn load_witness_set(path: &Path, document: &Value) -> Result<Vec<(String, Row)>, String> {
    if document["function"].as_str() != Some("COMBINA") {
        return Err(format!("{} is not a COMBINA witness set", path.display()));
    }
    let witnesses = document["witnesses"]
        .as_array()
        .ok_or_else(|| format!("{} has no witnesses array", path.display()))?;
    let mut rows = Vec::with_capacity(witnesses.len());
    for witness in witnesses {
        let id = witness["id"]
            .as_str()
            .filter(|id| !id.is_empty())
            .ok_or_else(|| format!("{} contains a missing witness id", path.display()))?;
        if witness.get("expected_error").is_some() {
            continue;
        }
        let args = witness["args"]
            .as_array()
            .ok_or_else(|| format!("{id}: missing args"))?;
        if args.len() != 2 {
            return Err(format!("{id}: expected two args"));
        }
        let n = f64::from_bits(parse_bits(
            args[0]
                .as_str()
                .ok_or_else(|| format!("{id}: n arg is not a bit string"))?,
        )?);
        let k = f64::from_bits(parse_bits(
            args[1]
                .as_str()
                .ok_or_else(|| format!("{id}: k arg is not a bit string"))?,
        )?);
        let expected = witness["expected_bits"]
            .as_str()
            .ok_or_else(|| format!("{id}: missing expected_bits"))?;
        if expected.starts_with("error:") {
            continue;
        }
        let want = parse_bits(expected)?;
        rows.push((id.to_owned(), Row { n, k, want }));
    }
    Ok(rows)
}

fn load(path: &Path) -> Result<Vec<(String, Row)>, String> {
    let text = fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))?;
    if path.extension().and_then(|part| part.to_str()) == Some("jsonl") {
        return load_jsonl(path);
    }
    let document: Value = serde_json::from_str(&text).map_err(|error| error.to_string())?;
    load_witness_set(path, &document)
}

fn combin_identity(n: f64, k: f64) -> Result<f64, String> {
    let n = n.trunc();
    let k = k.trunc();
    if k == 0.0 {
        return Ok(1.0);
    }
    combin_kernel(n + k - 1.0, k).map_err(|error| format!("{error:?}"))
}

fn former_product_control(n: f64, k: f64) -> Result<f64, String> {
    let n = n.trunc() as i64;
    let k = k.trunc() as i64;
    if k == 0 {
        return Ok(1.0);
    }
    if n == 0 && k > 0 {
        return Err("Num".to_owned());
    }
    let total = n
        .checked_add(k)
        .and_then(|value| value.checked_sub(1))
        .ok_or_else(|| "Num".to_owned())?;
    let reduced_k = k.min(total - k);
    let mut acc = 1.0;
    for i in 1..=reduced_k {
        acc *= (total - reduced_k + i) as f64;
        acc /= i as f64;
    }
    Ok(acc)
}

fn score(
    name: &str,
    rows: &[(String, Row)],
    candidate: impl Fn(f64, f64) -> Result<f64, String>,
) -> Result<Score, String> {
    let mut score = Score::default();
    for (id, row) in rows {
        let got = candidate(row.n, row.k)
            .map_err(|error| format!("{id}: candidate {name} returned {error}"))?
            .to_bits();
        let signed_delta = (got as i128) - (row.want as i128);
        let delta = i64::try_from(signed_delta)
            .map_err(|_| format!("{id}: candidate {name} ULP delta does not fit i64"))?;
        *score.histogram.entry(delta).or_default() += 1;
        if delta == 0 {
            score.exact += 1;
        } else if score.first_misses.len() < 20 {
            score.first_misses.push(format!(
                "{id}: n={} k={} got=0x{got:016x} want=0x{:016x} delta={delta:+}",
                row.n, row.k, row.want
            ));
        }
    }
    Ok(score)
}

fn main() -> Result<(), String> {
    let paths: Vec<_> = std::env::args_os().skip(1).collect();
    if paths.is_empty() {
        return Err(
            "pass one or more broad-scalar comparison JSONL or COMBINA WitnessSet JSON paths"
                .to_owned(),
        );
    }

    let mut rows = Vec::new();
    let mut keys = BTreeSet::new();
    for raw_path in paths {
        let path = Path::new(&raw_path);
        for (id, row) in load(path)? {
            let key = (row.n.to_bits(), row.k.to_bits(), row.want);
            if keys.insert(key) {
                rows.push((format!("{}:{id}", path.display()), row));
            }
        }
    }
    if rows.is_empty() {
        return Err("no numeric COMBINA rows found".to_owned());
    }
    println!("unique numeric COMBINA rows={}", rows.len());

    let candidates = [
        (
            "production-combina",
            score("production-combina", &rows, |n, k| {
                combina_kernel(n, k).map_err(|error| format!("{error:?}"))
            })?,
        ),
        (
            "landed-combin(trunc(n)+trunc(k)-1,trunc(k))",
            score(
                "landed-combin(trunc(n)+trunc(k)-1,trunc(k))",
                &rows,
                combin_identity,
            )?,
        ),
        (
            "former-multiply-first-product-control",
            score(
                "former-multiply-first-product-control",
                &rows,
                former_product_control,
            )?,
        ),
    ];
    for (name, score) in candidates {
        println!(
            "{name}: exact={}/{} histogram={:?}",
            score.exact,
            rows.len(),
            score.histogram
        );
        for mismatch in score.first_misses {
            println!("  {mismatch}");
        }
    }
    Ok(())
}
