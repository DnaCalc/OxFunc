//! Score the frozen CONVERT v3 refinement discriminator against its eight
//! precomputed, oracle-blind arithmetic variants.

#[path = "convert_research/common.rs"]
mod common;

use common::{MetaDocument, ordered_bits};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Deserialize)]
struct WitnessSet {
    function: String,
    witnesses: Vec<Witness>,
    #[serde(default)]
    capture_provenance: Value,
}

#[derive(Deserialize)]
struct Witness {
    id: String,
    args: Value,
    expected_bits: String,
}

#[derive(Default, Serialize)]
struct Score {
    exact: usize,
    total: usize,
    sum_abs_ulp: String,
    max_abs_ulp: String,
    class_exact: BTreeMap<String, usize>,
    class_total: BTreeMap<String, usize>,
    pair_exact: BTreeMap<String, usize>,
    pair_total: BTreeMap<String, usize>,
    first_misses: Vec<String>,
    #[serde(skip)]
    sum_value: u128,
    #[serde(skip)]
    max_value: u128,
}

#[derive(Serialize)]
struct Report {
    schema_version: &'static str,
    function: &'static str,
    split: String,
    rows: usize,
    metadata_model_names: Vec<String>,
    scores: BTreeMap<String, Score>,
    capture_provenance: Value,
}

struct Args { meta: PathBuf, answers: PathBuf, out: PathBuf }

fn args() -> Args {
    let mut meta = None;
    let mut answers = None;
    let mut out = None;
    let mut values = std::env::args().skip(1);
    while let Some(arg) = values.next() {
        match arg.as_str() {
            "--meta" => meta = values.next().map(PathBuf::from),
            "--answers" => answers = values.next().map(PathBuf::from),
            "--out" => out = values.next().map(PathBuf::from),
            other => panic!("unknown argument {other}"),
        }
    }
    Args { meta: meta.expect("--meta"), answers: answers.expect("--answers"), out: out.expect("--out") }
}

fn bits(raw: &str) -> Option<u64> {
    let digits = raw.strip_prefix("0x")?;
    (digits.len() == 16).then(|| u64::from_str_radix(digits, 16).ok()).flatten()
}

fn expected_args(row: &common::MetaRow) -> Value {
    json!([row.number_bits, row.from_unit, row.to_unit])
}

fn main() {
    let args = args();
    let metadata: MetaDocument = serde_json::from_slice(&std::fs::read(&args.meta).unwrap()).unwrap();
    let answers: WitnessSet = serde_json::from_slice(&std::fs::read(&args.answers).unwrap()).unwrap();
    assert_eq!(metadata.function, "CONVERT");
    assert_eq!(answers.function, "CONVERT");
    let by_id: BTreeMap<_, _> = answers.witnesses.iter().map(|w| (w.id.as_str(), w)).collect();
    assert_eq!(metadata.rows.len(), by_id.len(), "row count drift");
    let mut scores: BTreeMap<_, _> = metadata.model_names.iter()
        .map(|name| (name.clone(), Score::default())).collect();
    for row in &metadata.rows {
        let witness = by_id[row.id.as_str()];
        assert_eq!(witness.args, expected_args(row), "argument drift at {}", row.id);
        let actual = bits(&witness.expected_bits).expect("length result must be numeric");
        let pair = format!("{}->{}", row.from_unit, row.to_unit);
        for name in &metadata.model_names {
            let predicted_raw = &row.predictions[name];
            let predicted = bits(predicted_raw).expect("prediction must be numeric");
            let score = scores.get_mut(name).unwrap();
            score.total += 1;
            *score.class_total.entry(row.class.clone()).or_default() += 1;
            *score.pair_total.entry(pair.clone()).or_default() += 1;
            let residual = ordered_bits(actual) - ordered_bits(predicted);
            if residual == 0 {
                score.exact += 1;
                *score.class_exact.entry(row.class.clone()).or_default() += 1;
                *score.pair_exact.entry(pair.clone()).or_default() += 1;
            } else {
                let abs = residual.unsigned_abs();
                score.sum_value = score.sum_value.saturating_add(abs);
                score.max_value = score.max_value.max(abs);
                if score.first_misses.len() < 64 {
                    score.first_misses.push(format!(
                        "{} {} {} x={} residual={residual:+} predicted=0x{predicted:016x} oracle=0x{actual:016x}",
                        row.id, row.class, pair, row.number_bits
                    ));
                }
            }
        }
    }
    for score in scores.values_mut() {
        score.sum_abs_ulp = score.sum_value.to_string();
        score.max_abs_ulp = score.max_value.to_string();
    }
    for (name, score) in &scores {
        println!("{name}: {}/{} sum={} max={}", score.exact, score.total, score.sum_abs_ulp, score.max_abs_ulp);
    }
    let report = Report {
        schema_version: "w109.convert.v3_length_discriminator_score.v1",
        function: "CONVERT",
        split: metadata.rows.first().map(|row| row.split.clone()).unwrap_or_default(),
        rows: metadata.rows.len(),
        metadata_model_names: metadata.model_names,
        scores,
        capture_provenance: answers.capture_provenance,
    };
    if let Some(parent) = args.out.parent() { std::fs::create_dir_all(parent).unwrap(); }
    std::fs::write(&args.out, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
    println!("wrote -> {}", args.out.display());
}
