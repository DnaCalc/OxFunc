//! Coordinate-fit a single staged length-unit table against discovery rows.
//!
//! The common category scale is algebraically invisible but affects binary64
//! staging.  For each decimal-power anchor, initialize the six direct-unit
//! constants from the oracle-blind `m -> unit, x=1` discovery subset, then
//! adjust only a small ULP neighborhood and score one uniform arithmetic
//! schedule over every remaining direct-length observation.

#[path = "convert_research/common.rs"]
mod common;

use common::{MetaDocument, ordered_bits};
use oxfunc_core::excel_numeric::research as rx;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::PathBuf;

const UNITS: [&str; 23] = [
    "Em", "Gm", "Mm", "Nmi", "Pm", "Tm", "Ym", "Zm", "cm", "dam", "dm", "fm",
    "ft", "hm", "in", "km", "m", "mi", "mm", "nm", "pm", "um", "yd",
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

#[derive(Clone)]
struct Row {
    id: String,
    number: f64,
    from: usize,
    to: usize,
    actual: u64,
}

#[derive(Clone, Copy, Default, Serialize, PartialEq, Eq)]
struct Fitness {
    exact: usize,
    total: usize,
    sum_abs_ulp: u128,
    max_abs_ulp: u128,
}

#[derive(Serialize)]
struct Candidate {
    scale_exponent: i32,
    schedule: String,
    fitness: Fitness,
    constants_bits: BTreeMap<String, String>,
    misses: Vec<String>,
}

#[derive(Serialize)]
struct Report {
    schema_version: &'static str,
    function: &'static str,
    length_rows: usize,
    neighborhood_radius: i64,
    candidates: Vec<Candidate>,
    capture_provenance: Value,
}

struct Args {
    meta: PathBuf,
    answers: PathBuf,
    out: Option<PathBuf>,
    radius: i64,
    scale_exponent: Option<i32>,
}

fn args() -> Args {
    let mut meta = None;
    let mut answers = None;
    let mut out = None;
    let mut radius = 32_i64;
    let mut scale_exponent = None;
    let mut values = std::env::args().skip(1);
    while let Some(arg) = values.next() {
        match arg.as_str() {
            "--meta" => meta = values.next().map(PathBuf::from),
            "--answers" => answers = values.next().map(PathBuf::from),
            "--out" => out = values.next().map(PathBuf::from),
            "--radius" => radius = values.next().unwrap().parse().unwrap(),
            "--scale-exponent" => scale_exponent = Some(values.next().unwrap().parse().unwrap()),
            other => panic!("unknown argument {other}"),
        }
    }
    assert!(radius >= 0);
    Args {
        meta: meta.expect("--meta"),
        answers: answers.expect("--answers"),
        out,
        radius,
        scale_exponent,
    }
}

fn bits(raw: &str) -> Option<u64> {
    let digits = raw.strip_prefix("0x")?;
    (digits.len() == 16)
        .then(|| u64::from_str_radix(digits, 16).ok())
        .flatten()
}

fn unit_index(name: &str) -> Option<usize> {
    UNITS.iter().position(|unit| *unit == name)
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

fn fitness(rows: &[Row], constants: &[f64; 23], schedule: &str) -> Fitness {
    let mut result = Fitness {
        total: rows.len(),
        ..Fitness::default()
    };
    for row in rows {
        let predicted = predict(schedule, row.number, constants[row.from], constants[row.to]);
        let residual = ordered_bits(row.actual) - ordered_bits(predicted.to_bits());
        if residual == 0 {
            result.exact += 1;
        } else {
            let abs = residual.unsigned_abs();
            result.sum_abs_ulp = result.sum_abs_ulp.saturating_add(abs);
            result.max_abs_ulp = result.max_abs_ulp.max(abs);
        }
    }
    result
}

fn better(left: Fitness, right: Fitness) -> bool {
    left.exact > right.exact
        || (left.exact == right.exact && left.sum_abs_ulp < right.sum_abs_ulp)
        || (left.exact == right.exact
            && left.sum_abs_ulp == right.sum_abs_ulp
            && left.max_abs_ulp < right.max_abs_ulp)
}

fn offset_bits(value: f64, offset: i64) -> f64 {
    assert!(value.is_sign_positive() && value.is_finite());
    let bits = value.to_bits();
    let adjusted = if offset >= 0 {
        bits.checked_add(offset as u64).unwrap()
    } else {
        bits.checked_sub(offset.unsigned_abs()).unwrap()
    };
    f64::from_bits(adjusted)
}

fn optimize(
    rows: &[Row],
    mut constants: [f64; 23],
    schedule: &str,
    radius: i64,
) -> ([f64; 23], Fitness) {
    let mut current = fitness(rows, &constants, schedule);
    for _pass in 0..8 {
        let mut changed = false;
        for index in 0..UNITS.len() {
            let center = constants[index];
            let mut best_value = center;
            let mut best = current;
            for offset in -radius..=radius {
                let candidate = offset_bits(center, offset);
                constants[index] = candidate;
                let candidate_fitness = fitness(rows, &constants, schedule);
                if better(candidate_fitness, best) {
                    best = candidate_fitness;
                    best_value = candidate;
                }
            }
            constants[index] = best_value;
            if best_value.to_bits() != center.to_bits() {
                changed = true;
            }
            current = best;
        }
        if !changed {
            break;
        }
    }
    (constants, current)
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
    let mut rows = Vec::new();
    let mut base_ratios = [f64::NAN; 23];
    for row in &metadata.rows {
        if row.category != "length" {
            continue;
        }
        let (Some(from), Some(to)) = (unit_index(&row.from_unit), unit_index(&row.to_unit)) else {
            continue;
        };
        let witness = by_id[row.id.as_str()];
        let Some(actual) = bits(&witness.expected_bits) else {
            continue;
        };
        if row.number_bits == "0x3ff0000000000000" && row.from_unit == "m" {
            base_ratios[to] = f64::from_bits(actual);
        }
        rows.push(Row {
            id: row.id.clone(),
            number: common::f64_from_hex(&row.number_bits).unwrap(),
            from,
            to,
            actual,
        });
    }
    assert!(base_ratios.iter().all(|value| value.is_finite()));

    let mut candidates = Vec::new();
    let scale_exponents: Vec<i32> = match args.scale_exponent {
        Some(value) => vec![value],
        None => (-24..=24).collect(),
    };
    for scale_exponent in scale_exponents {
        let scale = format!("1e{scale_exponent}").parse::<f64>().unwrap();
        for schedule in SCHEDULES {
            // Use both source-level spellings because K/scale and K*anchor
            // can differ by one ULP even when `anchor == 1/scale`.
            let initializations = if schedule.starts_with("factor_") {
                [
                    std::array::from_fn(|index| scale / base_ratios[index]),
                    std::array::from_fn(|index| scale * (1.0 / base_ratios[index])),
                ]
            } else {
                let anchor = 1.0 / scale;
                [
                    std::array::from_fn(|index| base_ratios[index] / scale),
                    std::array::from_fn(|index| base_ratios[index] * anchor),
                ]
            };
            let mut best_constants = initializations[0];
            let mut best = Fitness::default();
            for initial in initializations {
                let (fitted, result) = optimize(&rows, initial, schedule, args.radius);
                if best.total == 0 || better(result, best) {
                    best = result;
                    best_constants = fitted;
                }
            }
            let mut misses = Vec::new();
            for row in &rows {
                let predicted = predict(
                    schedule,
                    row.number,
                    best_constants[row.from],
                    best_constants[row.to],
                );
                let residual = ordered_bits(row.actual) - ordered_bits(predicted.to_bits());
                if residual != 0 && misses.len() < 1_000 {
                    misses.push(format!(
                        "{} {}->{} x=0x{:016x} residual={:+} predicted=0x{:016x} oracle=0x{:016x}",
                        row.id,
                        UNITS[row.from],
                        UNITS[row.to],
                        row.number.to_bits(),
                        residual,
                        predicted.to_bits(),
                        row.actual
                    ));
                }
            }
            let constants_bits = UNITS
                .iter()
                .enumerate()
                .map(|(index, unit)| {
                    ((*unit).to_string(), format!("0x{:016x}", best_constants[index].to_bits()))
                })
                .collect();
            candidates.push(Candidate {
                scale_exponent,
                schedule: schedule.to_string(),
                fitness: best,
                constants_bits,
                misses,
            });
        }
    }
    candidates.sort_by(|left, right| {
        right
            .fitness
            .exact
            .cmp(&left.fitness.exact)
            .then(left.fitness.sum_abs_ulp.cmp(&right.fitness.sum_abs_ulp))
            .then(left.fitness.max_abs_ulp.cmp(&right.fitness.max_abs_ulp))
    });
    for candidate in candidates.iter().take(20) {
        println!(
            "scale=1e{:+} {:28} exact={}/{} sum_ulp={} max_ulp={}",
            candidate.scale_exponent,
            candidate.schedule,
            candidate.fitness.exact,
            candidate.fitness.total,
            candidate.fitness.sum_abs_ulp,
            candidate.fitness.max_abs_ulp
        );
    }

    let report = Report {
        schema_version: "w109.convert.length_effective_table_coordinate_fit.v2",
        function: "CONVERT",
        length_rows: rows.len(),
        neighborhood_radius: args.radius,
        candidates,
        capture_provenance: answer_set.capture_provenance,
    };
    if let Some(path) = args.out {
        std::fs::write(&path, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
        println!("wrote fitted length table -> {}", path.display());
    }
}
