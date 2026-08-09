//! Race the final prefix multiply after a frozen f64 physical mul/div core.
//!
//! The retired v1 set killed the all-f64 mass graph at one extreme-prefix
//! row.  This scorer isolates only the final prefix operation, including the
//! mixed graph where the f64 core is reloaded into x87 extended precision for
//! the final multiplication and rounded once to binary64.

#[path = "convert_research/common.rs"]
mod common;

use common::{MetaDocument, ordered_bits};
use oxfunc_core::excel_numeric::research as rx;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::PathBuf;

const MODELS: [&str; 6] = [
    "f64_core_f64_delta",
    "f64_core_x87_delta_final",
    "f64_core_f64_from_div_to",
    "f64_core_f64_div_to_mul_from",
    "f64_core_x87_from_div_to_final",
    "f64_core_x87_div_to_mul_from_final",
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
    sum_abs_ulp: String,
    max_abs_ulp: String,
    first_misses: Vec<String>,
}

#[derive(Serialize)]
struct Report {
    schema_version: &'static str,
    function: &'static str,
    scores: BTreeMap<String, BTreeMap<String, Score>>,
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

fn base(category: &str) -> &'static str {
    match category {
        "length" => "m",
        "mass" => "g",
        "time" => "sec",
        "volume" => "l",
        other => panic!("unknown category {other}"),
    }
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
        other => panic!("unknown prefix {other}"),
    }
}

fn resolve(unit: &str, category: &str) -> (String, i32) {
    if let Some(direct) = common::direct_unit(unit) {
        assert_eq!(direct.category.name(), category);
        return (unit.to_string(), 0);
    }
    for (prefix, _) in common::PREFIXES {
        if unit == format!("{prefix}{}", base(category)) {
            return (base(category).to_string(), prefix_exponent(prefix));
        }
    }
    panic!("cannot resolve {category} unit {unit}");
}

fn factor(unit: &str) -> f64 {
    common::DIRECT_UNITS
        .iter()
        .find(|candidate| candidate.name == unit)
        .unwrap()
        .factor_decimal
        .parse()
        .unwrap()
}

fn angstroms(unit: &str) -> f64 {
    match unit {
        "m" => 10_000_000_000.0,
        "in" => 254_000_000.0,
        "ft" => 3_048_000_000.0,
        "yd" => 9_144_000_000.0,
        "mi" => 16_093_440_000_000.0,
        "Nmi" => 18_520_000_000_000.0,
        other => panic!("no angstrom factor for {other}"),
    }
}

fn pow10(exponent: i32) -> f64 {
    format!("1e{exponent}").parse().unwrap()
}

fn predictions(
    category: &str,
    number: f64,
    from_direct: &str,
    to_direct: &str,
    from_exponent: i32,
    to_exponent: i32,
) -> [f64; MODELS.len()] {
    let core = if category == "length" {
        (number * angstroms(from_direct)) / angstroms(to_direct)
    } else {
        (number * factor(from_direct)) / factor(to_direct)
    };
    let delta = pow10(from_exponent - to_exponent);
    let from = pow10(from_exponent);
    let to = pow10(to_exponent);
    let cw = rx::CW_PC64_RN;
    let ext_core = rx::ext_from_f64(core);
    let ext_delta = rx::ext_from_f64(delta);
    let ext_from = rx::ext_from_f64(from);
    let ext_to = rx::ext_from_f64(to);
    let ext_delta_product = rx::ext_mul(&ext_core, &ext_delta, cw);
    let ext_from_product = rx::ext_mul(&ext_core, &ext_from, cw);
    let ext_from_div_to = rx::ext_div(&ext_from_product, &ext_to, cw);
    let ext_div_to = rx::ext_div(&ext_core, &ext_to, cw);
    let ext_div_to_mul_from = rx::ext_mul(&ext_div_to, &ext_from, cw);
    [
        core * delta,
        rx::ext_to_f64(&ext_delta_product, cw),
        (core * from) / to,
        (core / to) * from,
        rx::ext_to_f64(&ext_from_div_to, cw),
        rx::ext_to_f64(&ext_div_to_mul_from, cw),
    ]
}

fn main() {
    let args = args();
    let metadata: MetaDocument =
        serde_json::from_slice(&std::fs::read(&args.meta).unwrap()).unwrap();
    let answers: WitnessSet =
        serde_json::from_slice(&std::fs::read(&args.answers).unwrap()).unwrap();
    assert_eq!(answers.function, "CONVERT");
    let by_id: BTreeMap<_, _> = answers
        .witnesses
        .iter()
        .map(|witness| (witness.id.as_str(), witness))
        .collect();
    let mut scores = BTreeMap::new();
    for category in ["length", "mass", "time", "volume"] {
        scores.insert(
            category.to_string(),
            MODELS
                .iter()
                .map(|model| ((*model).to_string(), Score::default()))
                .collect::<BTreeMap<_, _>>(),
        );
    }
    for row in &metadata.rows {
        if !matches!(row.category.as_str(), "length" | "mass" | "time" | "volume") {
            continue;
        }
        let Some(actual) = bits(&by_id[row.id.as_str()].expected_bits) else {
            continue;
        };
        let (from_direct, from_exponent) = resolve(&row.from_unit, &row.category);
        let (to_direct, to_exponent) = resolve(&row.to_unit, &row.category);
        let values = predictions(
            &row.category,
            common::f64_from_hex(&row.number_bits).unwrap(),
            &from_direct,
            &to_direct,
            from_exponent,
            to_exponent,
        );
        for (model, predicted) in MODELS.iter().zip(values) {
            let score = scores.get_mut(&row.category).unwrap().get_mut(*model).unwrap();
            score.total += 1;
            let residual = ordered_bits(actual) - ordered_bits(predicted.to_bits());
            if residual == 0 {
                score.exact += 1;
            } else {
                let abs = residual.unsigned_abs();
                score.sum_abs_ulp = score
                    .sum_abs_ulp
                    .parse::<u128>()
                    .unwrap_or(0)
                    .saturating_add(abs)
                    .to_string();
                score.max_abs_ulp = score
                    .max_abs_ulp
                    .parse::<u128>()
                    .unwrap_or(0)
                    .max(abs)
                    .to_string();
                if score.first_misses.len() < 32 {
                    score.first_misses.push(format!(
                        "{} {}->{} x={} residual={:+} predicted=0x{:016x} oracle=0x{:016x}",
                        row.id,
                        row.from_unit,
                        row.to_unit,
                        row.number_bits,
                        residual,
                        predicted.to_bits(),
                        actual,
                    ));
                }
            }
        }
    }
    for (category, category_scores) in &mut scores {
        for (model, score) in category_scores {
            if score.sum_abs_ulp.is_empty() {
                score.sum_abs_ulp = "0".to_string();
            }
            if score.max_abs_ulp.is_empty() {
                score.max_abs_ulp = "0".to_string();
            }
            println!(
                "{category} {model}: {}/{} exact sum={} max={}",
                score.exact, score.total, score.sum_abs_ulp, score.max_abs_ulp
            );
        }
    }
    let report = Report {
        schema_version: "w109.convert.linear_prefix_final_race.v1",
        function: "CONVERT",
        scores,
        capture_provenance: answers.capture_provenance,
    };
    if let Some(path) = args.out {
        std::fs::write(&path, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
        println!("wrote prefix-final race -> {}", path.display());
    }
}
