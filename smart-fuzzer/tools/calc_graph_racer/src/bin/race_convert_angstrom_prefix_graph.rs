//! Race the public angstrom-based CONVERT length table with prefix staging.
//!
//! The six direct-unit discovery ladders are explained exactly by one uniform
//! binary64 graph using integer angstroms-per-unit constants:
//! `(number * from_angstroms) / to_angstroms`.  This scorer keeps that table
//! frozen and races only the placement of decimal-prefix operations.

#[path = "convert_research/common.rs"]
mod common;

use common::{MetaDocument, ordered_bits};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::PathBuf;

const MODELS: [&str; 7] = [
    "core_then_pow10_delta",
    "core_then_mul_from_div_to_prefix",
    "mul_from_prefix_then_core_div_to_prefix",
    "mul_from_angstrom_then_prefix_div_to_angstrom_then_prefix",
    "effective_factor_mul_div",
    "core_then_div_inverse_pow10_delta",
    "pow10_delta_then_core",
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

#[derive(Clone)]
struct Resolved {
    direct: String,
    prefix_exponent: i32,
}

#[derive(Default, Serialize)]
struct Score {
    exact: usize,
    total: usize,
    max_abs_ulp: String,
    sum_abs_ulp: String,
    class_exact: BTreeMap<String, usize>,
    class_total: BTreeMap<String, usize>,
    pair_exact: BTreeMap<String, usize>,
    pair_total: BTreeMap<String, usize>,
    first_misses: Vec<String>,
}

#[derive(Serialize)]
struct Report {
    schema_version: &'static str,
    function: &'static str,
    table: BTreeMap<String, String>,
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

fn prefix_exponent(prefix: &str) -> i32 {
    match prefix {
        "Y" => 24,
        "Z" => 21,
        "E" => 18,
        "P" => 15,
        "T" => 12,
        "G" => 9,
        "M" => 6,
        "k" => 3,
        "h" => 2,
        "da" => 1,
        "d" => -1,
        "c" => -2,
        "m" => -3,
        "u" => -6,
        "n" => -9,
        "p" => -12,
        "f" => -15,
        _ => panic!("unknown prefix {prefix}"),
    }
}

fn resolve(name: &str) -> Resolved {
    if common::direct_unit(name).is_some() {
        return Resolved {
            direct: name.to_string(),
            prefix_exponent: 0,
        };
    }
    for (prefix, _) in common::PREFIXES {
        if name == format!("{prefix}m") {
            return Resolved {
                direct: "m".to_string(),
                prefix_exponent: prefix_exponent(prefix),
            };
        }
    }
    panic!("cannot resolve length unit {name}");
}

fn angstroms(unit: &str) -> f64 {
    match unit {
        "m" => 10_000_000_000.0,
        "in" => 254_000_000.0,
        "ft" => 3_048_000_000.0,
        "yd" => 9_144_000_000.0,
        "mi" => 16_093_440_000_000.0,
        "Nmi" => 18_520_000_000_000.0,
        _ => panic!("no angstrom factor for {unit}"),
    }
}

fn pow10(exponent: i32) -> f64 {
    format!("1e{exponent}").parse().unwrap()
}

fn predictions(number: f64, from: &Resolved, to: &Resolved) -> [f64; MODELS.len()] {
    let from_angstroms = angstroms(&from.direct);
    let to_angstroms = angstroms(&to.direct);
    let from_prefix = pow10(from.prefix_exponent);
    let to_prefix = pow10(to.prefix_exponent);
    let delta = pow10(from.prefix_exponent - to.prefix_exponent);
    let core = (number * from_angstroms) / to_angstroms;
    [
        core * delta,
        (core * from_prefix) / to_prefix,
        (((number * from_prefix) * from_angstroms) / to_angstroms) / to_prefix,
        (((number * from_angstroms) * from_prefix) / to_angstroms) / to_prefix,
        (number * (from_angstroms * from_prefix)) / (to_angstroms * to_prefix),
        core / pow10(to.prefix_exponent - from.prefix_exponent),
        ((number * delta) * from_angstroms) / to_angstroms,
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
        if row.category != "length" {
            continue;
        }
        let witness = by_id[row.id.as_str()];
        let Some(actual) = bits(&witness.expected_bits) else {
            continue;
        };
        let from = resolve(&row.from_unit);
        let to = resolve(&row.to_unit);
        let number = common::f64_from_hex(&row.number_bits).unwrap();
        let values = predictions(number, &from, &to);
        for (name, predicted) in MODELS.iter().zip(values) {
            let score = scores.get_mut(*name).unwrap();
            score.total += 1;
            *score.class_total.entry(row.class.clone()).or_default() += 1;
            let pair = format!("{}->{}", row.from_unit, row.to_unit);
            *score.pair_total.entry(pair.clone()).or_default() += 1;
            let residual = ordered_bits(actual) - ordered_bits(predicted.to_bits());
            if residual == 0 {
                score.exact += 1;
                *score.class_exact.entry(row.class.clone()).or_default() += 1;
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
                if score.first_misses.len() < 1_000 {
                    score.first_misses.push(format!(
                        "{} {}({},{},{}) residual={:+} predicted=0x{:016x} oracle=0x{:016x}",
                        row.id,
                        row.class,
                        row.number_bits,
                        row.from_unit,
                        row.to_unit,
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
    let table = ["m", "in", "ft", "yd", "mi", "Nmi"]
        .into_iter()
        .map(|unit| (unit.to_string(), format!("0x{:016x}", angstroms(unit).to_bits())))
        .collect();
    let report = Report {
        schema_version: "w109.convert.angstrom_prefix_graph_race.v1",
        function: "CONVERT",
        table,
        scores,
        capture_provenance: answer_set.capture_provenance,
    };
    if let Some(path) = args.out {
        std::fs::write(&path, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
        println!("wrote angstrom-prefix race -> {}", path.display());
    }
}
