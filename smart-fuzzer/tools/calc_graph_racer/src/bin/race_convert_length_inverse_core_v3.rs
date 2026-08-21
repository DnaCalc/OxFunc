//! Race the inverse-table length core exposed by the v3 Value2 readback.
//!
//! At the retired target `0x457bc2d00cc56eb2`, live `CONVERT(x,"m","m")`
//! returns `...eb1`.  That excludes the frozen-v2 `x*1e10/1e10` core
//! (`...eb3`) and is reproduced by an inverse-table graph such as
//! `(x / 1e-10) * 1e-10`.  This scorer searches public units-per-meter table
//! constructions, algebraically cancelling common decimal scales, and f64 or
//! PC64 operation schedules against all discovery plus explicitly retired
//! refinement evidence.

#[path = "convert_research/common.rs"]
mod common;

use common::{MetaDocument, ordered_bits};
use oxfunc_core::excel_numeric::research as rx;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

const CONSTRUCTIONS: [&str; 6] = [
    "hidden_div_physical_f64",
    "reciprocal_physical_div_scale_f64",
    "reciprocal_physical_mul_hidden_f64",
    "public_units_per_meter_div_scale_f64",
    "public_units_per_meter_mul_hidden_f64",
    "decimal_hidden_div_physical_pc64_store",
];

const SCHEDULES: [&str; 4] = [
    "f64_div_mul",
    "f64_div_mul_reversed_store",
    "pc64_div_mul_store_core",
    "pc64_div_mul_continuous_delta",
];

#[derive(Deserialize)]
struct WitnessSet {
    function: String,
    witnesses: Vec<Witness>,
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

#[derive(Clone)]
struct Row {
    id: String,
    dataset: String,
    number: f64,
    from: Resolved,
    to: Resolved,
    actual: u64,
}

#[derive(Default, Clone, Copy, Serialize)]
struct Fitness {
    exact: usize,
    total: usize,
    sum_abs_ulp: u128,
    max_abs_ulp: u128,
}

#[derive(Serialize)]
struct Candidate {
    scale_exponent: i32,
    construction: String,
    schedule: String,
    fitness: Fitness,
    dataset_fitness: BTreeMap<String, Fitness>,
    constants_bits: BTreeMap<String, String>,
    first_misses: Vec<String>,
}

#[derive(Serialize)]
struct Report {
    schema_version: &'static str,
    function: &'static str,
    rows: usize,
    candidates: Vec<Candidate>,
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
        other => panic!("unknown prefix {other}"),
    }
}

fn resolve(unit: &str) -> Resolved {
    if common::direct_unit(unit).is_some() {
        return Resolved {
            direct: unit.to_string(),
            prefix_exponent: 0,
        };
    }
    for (prefix, _) in common::PREFIXES {
        if unit == format!("{prefix}m") {
            return Resolved {
                direct: "m".to_string(),
                prefix_exponent: prefix_exponent(prefix),
            };
        }
    }
    panic!("cannot resolve length unit {unit}");
}

fn physical(unit: &str) -> &'static str {
    match unit {
        "m" => "1",
        "in" => "0.0254",
        "ft" => "0.3048",
        "yd" => "0.9144",
        "mi" => "1609.344",
        "Nmi" => "1852",
        other => panic!("no physical factor for {other}"),
    }
}

fn public_units_per_meter(unit: &str) -> &'static str {
    match unit {
        "m" => "1",
        "mi" => "6.2137119223733397E-04",
        "Nmi" => "5.3995680345572354E-04",
        "in" => "3.9370078740157480E01",
        "ft" => "3.2808398950131234E00",
        "yd" => "1.0936132983377078E00",
        other => panic!("no public direct constant for {other}"),
    }
}

fn constant(unit: &str, scale_exponent: i32, construction: &str) -> f64 {
    let scale_decimal = format!("1e{scale_exponent}");
    let hidden_decimal = format!("1e{}", -scale_exponent);
    let scale = scale_decimal.parse::<f64>().unwrap();
    let hidden = hidden_decimal.parse::<f64>().unwrap();
    let p = physical(unit).parse::<f64>().unwrap();
    let public = public_units_per_meter(unit).parse::<f64>().unwrap();
    match construction {
        "hidden_div_physical_f64" => hidden / p,
        "reciprocal_physical_div_scale_f64" => (1.0 / p) / scale,
        "reciprocal_physical_mul_hidden_f64" => (1.0 / p) * hidden,
        "public_units_per_meter_div_scale_f64" => public / scale,
        "public_units_per_meter_mul_hidden_f64" => public * hidden,
        "decimal_hidden_div_physical_pc64_store" => {
            let cw = rx::CW_PC64_RN;
            rx::ext_to_f64(
                &rx::ext_div(
                    &common::ext_from_decimal(&hidden_decimal),
                    &common::ext_from_decimal(physical(unit)),
                    cw,
                ),
                cw,
            )
        }
        other => panic!("unknown construction {other}"),
    }
}

fn pow10(exponent: i32) -> f64 {
    format!("1e{exponent}").parse().unwrap()
}

fn predict(row: &Row, constants: &BTreeMap<String, f64>, schedule: &str) -> u64 {
    let from = constants[&row.from.direct];
    let to = constants[&row.to.direct];
    let delta = row.from.prefix_exponent - row.to.prefix_exponent;
    let cw = rx::CW_PC64_RN;
    match schedule {
        "f64_div_mul" => {
            let core = (row.number / from) * to;
            rx::ext_to_f64(
                &rx::ext_mul(&rx::ext_from_f64(core), &rx::ext_from_f64(pow10(delta)), cw),
                cw,
            )
            .to_bits()
        }
        "f64_div_mul_reversed_store" => {
            let divided = row.number / from;
            let core = divided * to;
            rx::ext_to_f64(
                &rx::ext_mul(&rx::ext_from_f64(core), &rx::ext_from_f64(pow10(delta)), cw),
                cw,
            )
            .to_bits()
        }
        "pc64_div_mul_store_core" => {
            let divided = rx::ext_div(&rx::ext_from_f64(row.number), &rx::ext_from_f64(from), cw);
            let core = rx::ext_mul(&divided, &rx::ext_from_f64(to), cw);
            let stored = rx::ext_to_f64(&core, cw);
            rx::ext_to_f64(
                &rx::ext_mul(
                    &rx::ext_from_f64(stored),
                    &rx::ext_from_f64(pow10(delta)),
                    cw,
                ),
                cw,
            )
            .to_bits()
        }
        "pc64_div_mul_continuous_delta" => {
            let divided = rx::ext_div(&rx::ext_from_f64(row.number), &rx::ext_from_f64(from), cw);
            let core = rx::ext_mul(&divided, &rx::ext_from_f64(to), cw);
            rx::ext_to_f64(&rx::ext_mul(&core, &rx::ext_from_f64(pow10(delta)), cw), cw).to_bits()
        }
        other => panic!("unknown schedule {other}"),
    }
}

fn add_fit(fit: &mut Fitness, predicted: u64, actual: u64) {
    fit.total += 1;
    let residual = ordered_bits(actual) - ordered_bits(predicted);
    if residual == 0 {
        fit.exact += 1;
    } else {
        let abs = residual.unsigned_abs();
        fit.sum_abs_ulp = fit.sum_abs_ulp.saturating_add(abs);
        fit.max_abs_ulp = fit.max_abs_ulp.max(abs);
    }
}

fn load(root: &Path, dataset: &str, meta_name: &str, answer_name: &str) -> Vec<Row> {
    let metadata: MetaDocument =
        serde_json::from_slice(&std::fs::read(root.join(meta_name)).unwrap()).unwrap();
    let answers: WitnessSet =
        serde_json::from_slice(&std::fs::read(root.join(answer_name)).unwrap()).unwrap();
    assert_eq!(answers.function, "CONVERT");
    let by_id: BTreeMap<_, _> = answers
        .witnesses
        .iter()
        .map(|w| (w.id.as_str(), w))
        .collect();
    metadata
        .rows
        .into_iter()
        .filter(|row| row.category == "length")
        .map(|row| Row {
            id: row.id.clone(),
            dataset: dataset.to_string(),
            number: common::f64_from_hex(&row.number_bits).unwrap(),
            from: resolve(&row.from_unit),
            to: resolve(&row.to_unit),
            actual: bits(&by_id[row.id.as_str()].expected_bits).unwrap(),
        })
        .collect()
}

fn main() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../work/w109/G4-convert");
    let datasets = [
        (
            "discovery",
            "batch-convert-discovery-20260809-meta.json",
            "answers-convert-discovery-20260809-clean.json",
        ),
        (
            "retired-v1",
            "batch-convert-heldout-20260809-meta.json",
            "answers-convert-heldout-20260809.json",
        ),
        (
            "retired-v2",
            "batch-convert-publication-heldout-v2-20260809-meta.json",
            "answers-convert-publication-heldout-v2-20260809.json",
        ),
        (
            "v3-refinement",
            "batch-convert-v3-length-discriminator-20260809-meta.json",
            "answers-convert-v3-length-discriminator-20260809.json",
        ),
    ];
    let rows: Vec<_> = datasets
        .into_iter()
        .flat_map(|(dataset, meta, answers)| load(&root, dataset, meta, answers))
        .collect();
    let units = ["m", "in", "ft", "yd", "mi", "Nmi"];
    let mut candidates = Vec::new();
    for scale_exponent in -24..=24 {
        for construction in CONSTRUCTIONS {
            let constants: BTreeMap<_, _> = units
                .iter()
                .map(|unit| {
                    (
                        (*unit).to_string(),
                        constant(unit, scale_exponent, construction),
                    )
                })
                .collect();
            for schedule in SCHEDULES {
                let mut fitness = Fitness::default();
                let mut dataset_fitness = BTreeMap::new();
                let mut first_misses = Vec::new();
                for row in &rows {
                    let predicted = predict(row, &constants, schedule);
                    add_fit(&mut fitness, predicted, row.actual);
                    add_fit(
                        dataset_fitness.entry(row.dataset.clone()).or_default(),
                        predicted,
                        row.actual,
                    );
                    if predicted != row.actual && first_misses.len() < 16 {
                        first_misses.push(format!(
                            "{} {} predicted=0x{predicted:016x} oracle=0x{:016x}",
                            row.dataset, row.id, row.actual
                        ));
                    }
                }
                candidates.push(Candidate {
                    scale_exponent,
                    construction: construction.to_string(),
                    schedule: schedule.to_string(),
                    fitness,
                    dataset_fitness,
                    constants_bits: constants
                        .iter()
                        .map(|(unit, value)| (unit.clone(), format!("0x{:016x}", value.to_bits())))
                        .collect(),
                    first_misses,
                });
            }
        }
    }
    candidates.sort_by_key(|candidate| {
        (
            std::cmp::Reverse(candidate.fitness.exact),
            candidate.fitness.sum_abs_ulp,
            candidate.fitness.max_abs_ulp,
        )
    });
    for candidate in candidates.iter().take(20) {
        println!(
            "scale={} {} {}: {}/{} sum={} max={}",
            candidate.scale_exponent,
            candidate.construction,
            candidate.schedule,
            candidate.fitness.exact,
            candidate.fitness.total,
            candidate.fitness.sum_abs_ulp,
            candidate.fitness.max_abs_ulp
        );
    }
    let report = Report {
        schema_version: "w109.convert.length_inverse_core_race.v3",
        function: "CONVERT",
        rows: rows.len(),
        candidates,
    };
    let out = root.join("score-convert-length-inverse-core-v3.json");
    std::fs::write(&out, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
    println!("wrote -> {}", out.display());
}
