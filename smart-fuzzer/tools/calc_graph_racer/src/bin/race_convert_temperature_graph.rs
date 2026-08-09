//! Race affine CONVERT temperature graphs against typed discovery controls.

#[path = "convert_research/common.rs"]
mod common;

use common::{MetaDocument, ordered_bits};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::PathBuf;

const MODELS: [&str; 5] = [
    "kelvin_staged_1p8_identity",
    "direct_pairs_1p8_identity",
    "kelvin_staged_9_over_5_identity",
    "direct_pairs_9_over_5_identity",
    "kelvin_staged_1p8_no_identity",
];

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
    expected_bits: String,
}

#[derive(Default, Serialize)]
struct Score {
    exact: usize,
    total: usize,
    max_abs_ulp: String,
    sum_abs_ulp: String,
    pair_exact: BTreeMap<String, usize>,
    pair_total: BTreeMap<String, usize>,
    first_misses: Vec<String>,
}

#[derive(Serialize)]
struct Report {
    schema_version: &'static str,
    function: &'static str,
    scores: BTreeMap<String, Score>,
    capture_provenance: Value,
}

struct Args {
    meta: PathBuf,
    answers: PathBuf,
    out: Option<PathBuf>,
}

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
    Args {
        meta: meta.expect("--meta"),
        answers: answers.expect("--answers"),
        out,
    }
}

fn bits(raw: &str) -> Option<u64> {
    let digits = raw.strip_prefix("0x")?;
    (digits.len() == 16)
        .then(|| u64::from_str_radix(digits, 16).ok())
        .flatten()
}

fn to_kelvin_1p8(value: f64, unit: &str) -> f64 {
    match unit {
        "K" => value,
        "C" => value + 273.15,
        "F" => (value - 32.0) / 1.8 + 273.15,
        _ => panic!("unknown temperature unit {unit}"),
    }
}

fn from_kelvin_1p8(value: f64, unit: &str) -> f64 {
    match unit {
        "K" => value,
        "C" => value - 273.15,
        "F" => (value - 273.15) * 1.8 + 32.0,
        _ => panic!("unknown temperature unit {unit}"),
    }
}

fn to_kelvin_9_5(value: f64, unit: &str) -> f64 {
    match unit {
        "K" => value,
        "C" => value + 273.15,
        "F" => ((value - 32.0) * 5.0) / 9.0 + 273.15,
        _ => panic!("unknown temperature unit {unit}"),
    }
}

fn from_kelvin_9_5(value: f64, unit: &str) -> f64 {
    match unit {
        "K" => value,
        "C" => value - 273.15,
        "F" => (((value - 273.15) * 9.0) / 5.0) + 32.0,
        _ => panic!("unknown temperature unit {unit}"),
    }
}

fn direct_1p8(value: f64, from: &str, to: &str) -> f64 {
    match (from, to) {
        (a, b) if a == b => value,
        ("K", "C") => value - 273.15,
        ("C", "K") => value + 273.15,
        ("K", "F") => (value - 273.15) * 1.8 + 32.0,
        ("F", "K") => (value - 32.0) / 1.8 + 273.15,
        ("C", "F") => value * 1.8 + 32.0,
        ("F", "C") => (value - 32.0) / 1.8,
        _ => panic!("unknown temperature pair {from}->{to}"),
    }
}

fn direct_9_5(value: f64, from: &str, to: &str) -> f64 {
    match (from, to) {
        (a, b) if a == b => value,
        ("K", "C") => value - 273.15,
        ("C", "K") => value + 273.15,
        ("K", "F") => (((value - 273.15) * 9.0) / 5.0) + 32.0,
        ("F", "K") => (((value - 32.0) * 5.0) / 9.0) + 273.15,
        ("C", "F") => ((value * 9.0) / 5.0) + 32.0,
        ("F", "C") => ((value - 32.0) * 5.0) / 9.0,
        _ => panic!("unknown temperature pair {from}->{to}"),
    }
}

fn predictions(value: f64, from: &str, to: &str) -> [f64; MODELS.len()] {
    [
        if from == to {
            value
        } else {
            from_kelvin_1p8(to_kelvin_1p8(value, from), to)
        },
        direct_1p8(value, from, to),
        if from == to {
            value
        } else {
            from_kelvin_9_5(to_kelvin_9_5(value, from), to)
        },
        direct_9_5(value, from, to),
        from_kelvin_1p8(to_kelvin_1p8(value, from), to),
    ]
}

fn main() {
    let args = args();
    let metadata: MetaDocument =
        serde_json::from_slice(&std::fs::read(&args.meta).unwrap()).unwrap();
    let answer_set: WitnessSet =
        serde_json::from_slice(&std::fs::read(&args.answers).unwrap()).unwrap();
    assert_eq!(answer_set.function, "CONVERT");
    let by_id: BTreeMap<_, _> = answer_set
        .witnesses
        .iter()
        .map(|witness| (witness.id.as_str(), witness))
        .collect();
    let mut scores: BTreeMap<String, Score> = MODELS
        .iter()
        .map(|name| ((*name).to_string(), Score::default()))
        .collect();
    for row in &metadata.rows {
        if row.category != "temperature" {
            continue;
        }
        let witness = by_id[row.id.as_str()];
        let actual = bits(&witness.expected_bits).expect("temperature answer must be numeric");
        let number = common::f64_from_hex(&row.number_bits).unwrap();
        for (name, predicted) in MODELS
            .iter()
            .zip(predictions(number, &row.from_unit, &row.to_unit))
        {
            let score = scores.get_mut(*name).unwrap();
            score.total += 1;
            let pair = format!("{}->{}", row.from_unit, row.to_unit);
            *score.pair_total.entry(pair.clone()).or_default() += 1;
            let residual = ordered_bits(actual) - ordered_bits(predicted.to_bits());
            if residual == 0 {
                score.exact += 1;
                *score.pair_exact.entry(pair).or_default() += 1;
            } else {
                let abs = residual.unsigned_abs();
                let max = score.max_abs_ulp.parse::<u128>().unwrap_or(0).max(abs);
                let sum = score
                    .sum_abs_ulp
                    .parse::<u128>()
                    .unwrap_or(0)
                    .saturating_add(abs);
                score.max_abs_ulp = max.to_string();
                score.sum_abs_ulp = sum.to_string();
                if score.first_misses.len() < 100 {
                    score.first_misses.push(format!(
                        "{} {}->{} x={} residual={:+} predicted=0x{:016x} oracle=0x{:016x}",
                        row.id,
                        row.from_unit,
                        row.to_unit,
                        row.number_bits,
                        residual,
                        predicted.to_bits(),
                        actual
                    ));
                }
            }
        }
    }
    for (name, score) in &mut scores {
        if score.max_abs_ulp.is_empty() {
            score.max_abs_ulp = "0".to_string();
        }
        if score.sum_abs_ulp.is_empty() {
            score.sum_abs_ulp = "0".to_string();
        }
        println!(
            "{name}: {}/{} exact, max_ulp={}, sum_ulp={}",
            score.exact, score.total, score.max_abs_ulp, score.sum_abs_ulp
        );
    }
    let report = Report {
        schema_version: "w109.convert.temperature_graph_race.v1",
        function: "CONVERT",
        scores,
        capture_provenance: answer_set.capture_provenance,
    };
    if let Some(path) = args.out {
        std::fs::write(&path, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
        println!("wrote temperature race -> {}", path.display());
    }
}
