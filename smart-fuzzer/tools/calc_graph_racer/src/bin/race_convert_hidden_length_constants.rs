//! Race clean-room hidden-base constructions for Excel CONVERT length units.
//!
//! Discovery identity probes require a non-trivial common scale (for example,
//! units per kilometre rather than units per metre).  This scorer constructs
//! that algebraically-cancelling table from exact public physical decimals or
//! from the public ATP-compatible units-per-metre spellings, with both f64 and
//! x87-extended construction variants, then races uniform operation schedules.

#[path = "convert_research/common.rs"]
mod common;

use common::{MetaDocument, ordered_bits};
use oxfunc_core::excel_numeric::research as rx;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::PathBuf;

const CONSTRUCTIONS: [&str; 8] = [
    "f64_hidden_over_factor_product",
    "f64_hidden_over_factor_then_prefix",
    "x87_decimal_hidden_over_product_store_f64",
    "x87_decimal_hidden_over_factor_over_prefix_store_f64",
    "public_direct_f64_div_scale_div_prefix",
    "public_direct_x87_decimal_div_scale_div_prefix_store_f64",
    "f64_factor_prefix_scale",
    "x87_decimal_factor_prefix_scale_store_f64",
];

const SCHEDULES: [&str; 12] = [
    "f64_div_mul",
    "f64_mul_div",
    "f64_ratio_mul",
    "x87_f64_div_mul_pc64",
    "x87_f64_mul_div_pc64",
    "x87_f64_ratio_mul_pc64",
    "factor_f64_mul_div",
    "factor_f64_div_mul",
    "factor_f64_ratio_mul",
    "factor_x87_f64_mul_div_pc64",
    "factor_x87_f64_div_mul_pc64",
    "factor_x87_f64_ratio_mul_pc64",
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

#[derive(Default, Clone, Serialize)]
struct Score {
    scale_exponent: i32,
    construction: String,
    schedule: String,
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
    candidates: Vec<Score>,
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

fn public_direct(unit: &str) -> &'static str {
    match unit {
        "m" => "1",
        "mi" => "6.2137119223733397E-04",
        "Nmi" => "5.3995680345572354E-04",
        "in" => "3.9370078740157480E01",
        "ft" => "3.2808398950131234E00",
        "yd" => "1.0936132983377078E00",
        _ => panic!("no public length direct constant for {unit}"),
    }
}

fn resolve(name: &str) -> (&'static str, &'static str, &'static str) {
    if let Some(unit) = common::direct_unit(name) {
        assert_eq!(unit.category.name(), "length");
        let physical = common::DIRECT_UNITS
            .iter()
            .find(|candidate| candidate.name == name)
            .unwrap()
            .factor_decimal;
        return (name_for_static(name), physical, "1");
    }
    for (prefix, decimal) in common::PREFIXES {
        if name == format!("{prefix}m") {
            return ("m", "1", decimal);
        }
    }
    panic!("cannot resolve length unit {name}");
}

fn name_for_static(name: &str) -> &'static str {
    common::DIRECT_UNITS
        .iter()
        .find(|candidate| candidate.name == name)
        .unwrap()
        .name
}

fn constant(name: &str, scale_exponent: i32, construction: &str) -> f64 {
    let (direct, factor_decimal, prefix_decimal) = resolve(name);
    let scale_decimal = format!("1e{scale_exponent}");
    let hidden_decimal = format!("1e{}", -scale_exponent);
    let factor = factor_decimal.parse::<f64>().unwrap();
    let prefix = prefix_decimal.parse::<f64>().unwrap();
    let hidden = hidden_decimal.parse::<f64>().unwrap();
    let cw = rx::CW_PC64_RN;
    match construction {
        "f64_hidden_over_factor_product" => hidden / (factor * prefix),
        "f64_hidden_over_factor_then_prefix" => (hidden / factor) / prefix,
        "x87_decimal_hidden_over_product_store_f64" => {
            let product = rx::ext_mul(
                &common::ext_from_decimal(factor_decimal),
                &common::ext_from_decimal(prefix_decimal),
                cw,
            );
            let value = rx::ext_div(&common::ext_from_decimal(&hidden_decimal), &product, cw);
            rx::ext_to_f64(&value, cw)
        }
        "x87_decimal_hidden_over_factor_over_prefix_store_f64" => {
            let first = rx::ext_div(
                &common::ext_from_decimal(&hidden_decimal),
                &common::ext_from_decimal(factor_decimal),
                cw,
            );
            let value = rx::ext_div(&first, &common::ext_from_decimal(prefix_decimal), cw);
            rx::ext_to_f64(&value, cw)
        }
        "public_direct_f64_div_scale_div_prefix" => {
            (public_direct(direct).parse::<f64>().unwrap() / scale_decimal.parse::<f64>().unwrap())
                / prefix
        }
        "public_direct_x87_decimal_div_scale_div_prefix_store_f64" => {
            let first = rx::ext_div(
                &common::ext_from_decimal(public_direct(direct)),
                &common::ext_from_decimal(&scale_decimal),
                cw,
            );
            let value = rx::ext_div(&first, &common::ext_from_decimal(prefix_decimal), cw);
            rx::ext_to_f64(&value, cw)
        }
        "f64_factor_prefix_scale" => (factor * prefix) * scale_decimal.parse::<f64>().unwrap(),
        "x87_decimal_factor_prefix_scale_store_f64" => {
            let product = rx::ext_mul(
                &common::ext_from_decimal(factor_decimal),
                &common::ext_from_decimal(prefix_decimal),
                cw,
            );
            let value = rx::ext_mul(&product, &common::ext_from_decimal(&scale_decimal), cw);
            rx::ext_to_f64(&value, cw)
        }
        _ => panic!("unknown construction {construction}"),
    }
}

fn predict(schedule: &str, number: f64, from: f64, to: f64) -> f64 {
    let cw = rx::CW_PC64_RN;
    match schedule {
        "f64_div_mul" => (number / from) * to,
        "f64_mul_div" => (number * to) / from,
        "f64_ratio_mul" => number * (to / from),
        "x87_f64_div_mul_pc64" => {
            let divided = rx::ext_div(&rx::ext_from_f64(number), &rx::ext_from_f64(from), cw);
            rx::ext_to_f64(&rx::ext_mul(&divided, &rx::ext_from_f64(to), cw), cw)
        }
        "x87_f64_mul_div_pc64" => {
            let multiplied = rx::ext_mul(&rx::ext_from_f64(number), &rx::ext_from_f64(to), cw);
            rx::ext_to_f64(&rx::ext_div(&multiplied, &rx::ext_from_f64(from), cw), cw)
        }
        "x87_f64_ratio_mul_pc64" => {
            let ratio = rx::ext_div(&rx::ext_from_f64(to), &rx::ext_from_f64(from), cw);
            rx::ext_to_f64(&rx::ext_mul(&rx::ext_from_f64(number), &ratio, cw), cw)
        }
        "factor_f64_mul_div" => (number * from) / to,
        "factor_f64_div_mul" => (number / to) * from,
        "factor_f64_ratio_mul" => number * (from / to),
        "factor_x87_f64_mul_div_pc64" => {
            let multiplied = rx::ext_mul(&rx::ext_from_f64(number), &rx::ext_from_f64(from), cw);
            rx::ext_to_f64(&rx::ext_div(&multiplied, &rx::ext_from_f64(to), cw), cw)
        }
        "factor_x87_f64_div_mul_pc64" => {
            let divided = rx::ext_div(&rx::ext_from_f64(number), &rx::ext_from_f64(to), cw);
            rx::ext_to_f64(&rx::ext_mul(&divided, &rx::ext_from_f64(from), cw), cw)
        }
        "factor_x87_f64_ratio_mul_pc64" => {
            let ratio = rx::ext_div(&rx::ext_from_f64(from), &rx::ext_from_f64(to), cw);
            rx::ext_to_f64(&rx::ext_mul(&rx::ext_from_f64(number), &ratio, cw), cw)
        }
        _ => panic!("unknown schedule {schedule}"),
    }
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
    let mut scores = Vec::new();

    for scale_exponent in -24..=24 {
        for construction in CONSTRUCTIONS {
            for schedule in SCHEDULES {
                let mut score = Score {
                    scale_exponent,
                    construction: construction.to_string(),
                    schedule: schedule.to_string(),
                    ..Score::default()
                };
                for row in &metadata.rows {
                    if row.category != "length" {
                        continue;
                    }
                    let witness = by_id[row.id.as_str()];
                    let Some(actual) = bits(&witness.expected_bits) else {
                        continue;
                    };
                    let from = constant(&row.from_unit, scale_exponent, construction);
                    let to = constant(&row.to_unit, scale_exponent, construction);
                    let number = common::f64_from_hex(&row.number_bits).unwrap();
                    let predicted = predict(schedule, number, from, to);
                    let residual = ordered_bits(actual) - ordered_bits(predicted.to_bits());
                    score.total += 1;
                    *score.class_total.entry(row.class.clone()).or_default() += 1;
                    let pair = format!("{}->{}", row.from_unit, row.to_unit);
                    *score.pair_total.entry(pair.clone()).or_default() += 1;
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
                if score.max_abs_ulp.is_empty() {
                    score.max_abs_ulp = "0".to_string();
                }
                if score.sum_abs_ulp.is_empty() {
                    score.sum_abs_ulp = "0".to_string();
                }
                scores.push(score);
            }
        }
    }
    scores.sort_by(|left, right| {
        right.exact.cmp(&left.exact).then(
            left.sum_abs_ulp
                .parse::<u128>()
                .unwrap()
                .cmp(&right.sum_abs_ulp.parse::<u128>().unwrap()),
        )
    });
    for score in scores.iter().take(20) {
        println!(
            "scale=1e{:+} {:55} {:28} {}/{} sum_ulp={}",
            score.scale_exponent,
            score.construction,
            score.schedule,
            score.exact,
            score.total,
            score.sum_abs_ulp
        );
    }
    let report = Report {
        schema_version: "w109.convert.hidden_length_constant_race.v1",
        function: "CONVERT",
        candidates: scores,
        capture_provenance: answer_set.capture_provenance,
    };
    if let Some(path) = args.out {
        std::fs::write(&path, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
        println!("wrote hidden-length race -> {}", path.display());
    }
}
