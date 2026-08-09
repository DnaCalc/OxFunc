//! Search CONVERT pressure's hidden direct-table scale and prefix staging.
//!
//! The retired v1 held-out shows that a pre-rounded directed ratio is not the
//! full graph: even `psi -> psi` can move by one ULP.  This clean-room scorer
//! therefore keeps two direct table cells in the arithmetic path, searches
//! public pressure-unit anchors for an algebraically cancelling common scale,
//! and races f64/x87 final prefix staging.  No Excel implementation artifact
//! is inspected.

#[path = "convert_research/common.rs"]
mod common;

use common::{MetaDocument, ordered_bits};
use oxfunc_core::excel_numeric::research as rx;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

const CORE_SCHEDULES: [&str; 3] = ["mul_div", "div_mul", "ratio_mul"];
const PREFIX_SCHEDULES: [&str; 5] = [
    "f64_delta",
    "f64_from_div_to",
    "f64_div_to_mul_from",
    "f64_ratio",
    "x87_f64_delta_final",
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
    from_direct: String,
    to_direct: String,
    from_prefix: i32,
    to_prefix: i32,
    actual: u64,
}

#[derive(Clone)]
struct Scale {
    name: String,
    value: f64,
}

#[derive(Clone, Default, Serialize)]
struct Fitness {
    exact: usize,
    total: usize,
    sum_abs_ulp: String,
    max_abs_ulp: String,
    first_misses: Vec<String>,
    #[serde(skip)]
    sum_value: u128,
    #[serde(skip)]
    max_value: u128,
}

#[derive(Clone, Serialize)]
struct Candidate {
    table: String,
    scale_name: String,
    scale_bits: String,
    core_schedule: String,
    prefix_schedule: String,
    constants_bits: BTreeMap<String, String>,
    fitness: Fitness,
}

#[derive(Serialize)]
struct Report {
    schema_version: &'static str,
    function: &'static str,
    scored_rows: usize,
    candidates: usize,
    best: Vec<Candidate>,
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
        other => panic!("unknown prefix {other}"),
    }
}

fn resolve(unit: &str) -> (String, i32) {
    if matches!(unit, "Pa" | "atm" | "psi" | "bar") {
        return (unit.to_string(), 0);
    }
    for (prefix, _) in common::PREFIXES {
        if unit == format!("{prefix}Pa") {
            return ("Pa".to_string(), prefix_exponent(prefix));
        }
    }
    panic!("unknown pressure unit {unit}");
}

fn pow10(exponent: i32) -> f64 {
    format!("1e{exponent}").parse().unwrap()
}

fn physical(unit: &str) -> f64 {
    match unit {
        "Pa" => 1.0,
        "atm" => 101_325.0,
        "psi" => "6894.757293168361".parse().unwrap(),
        other => panic!("unsupported physical pressure unit {other}"),
    }
}

fn public_units_per_pa(unit: &str) -> f64 {
    match unit {
        "Pa" => 1.0,
        "atm" => "9.8692326671601280E-06".parse().unwrap(),
        "psi" => "1.4503773773020920E-04".parse().unwrap(),
        other => panic!("unsupported public pressure unit {other}"),
    }
}

fn scales() -> Vec<Scale> {
    let anchors = [
        ("one", 1.0),
        ("atm_pa", 101_325.0),
        ("pa_per_atm", 1.0 / 101_325.0),
        ("psi_pa", physical("psi")),
        ("pa_per_psi", 1.0 / physical("psi")),
        ("bar_pa", 100_000.0),
        ("pa_per_bar", 0.00001),
        ("torr_pa", "133.32236842105263".parse().unwrap()),
        ("pa_per_torr", 1.0 / "133.32236842105263".parse::<f64>().unwrap()),
    ];
    let mut seen = BTreeSet::new();
    let mut result = Vec::new();
    for (anchor_name, anchor) in anchors {
        for exponent in -24..=24 {
            for (order, value) in [
                ("anchor_mul_pow10", anchor * pow10(exponent)),
                ("pow10_mul_anchor", pow10(exponent) * anchor),
            ] {
                if value.is_finite() && value > 0.0 && seen.insert(value.to_bits()) {
                    result.push(Scale {
                        name: format!("{anchor_name}:{order}:1e{exponent:+}"),
                        value,
                    });
                }
            }
        }
    }
    result
}

fn direct_constants(table: &str, scale: f64, from: &str, to: &str) -> (f64, f64) {
    match table {
        // Return numerator and denominator constants for x*num/den.
        "physical_base_per_unit" => (physical(from) * scale, physical(to) * scale),
        "public_units_per_pa" => (
            public_units_per_pa(to) * scale,
            public_units_per_pa(from) * scale,
        ),
        "public_reciprocal_to_base" => (
            (1.0 / public_units_per_pa(from)) * scale,
            (1.0 / public_units_per_pa(to)) * scale,
        ),
        other => panic!("unknown table {other}"),
    }
}

fn core(schedule: &str, number: f64, numerator: f64, denominator: f64) -> f64 {
    match schedule {
        "mul_div" => (number * numerator) / denominator,
        "div_mul" => (number / denominator) * numerator,
        "ratio_mul" => number * (numerator / denominator),
        other => panic!("unknown core schedule {other}"),
    }
}

fn prefix(schedule: &str, value: f64, from_exponent: i32, to_exponent: i32) -> f64 {
    let from = pow10(from_exponent);
    let to = pow10(to_exponent);
    let delta = pow10(from_exponent - to_exponent);
    match schedule {
        "f64_delta" => value * delta,
        "f64_from_div_to" => (value * from) / to,
        "f64_div_to_mul_from" => (value / to) * from,
        "f64_ratio" => value * (from / to),
        "x87_f64_delta_final" => {
            let cw = rx::CW_PC64_RN;
            let product = rx::ext_mul(
                &rx::ext_from_f64(value),
                &rx::ext_from_f64(delta),
                cw,
            );
            rx::ext_to_f64(&product, cw)
        }
        other => panic!("unknown prefix schedule {other}"),
    }
}

fn score(rows: &[Row], table: &str, scale: &Scale, core_name: &str, prefix_name: &str) -> Candidate {
    let mut fitness = Fitness {
        total: rows.len(),
        ..Fitness::default()
    };
    for row in rows {
        let (numerator, denominator) = direct_constants(
            table,
            scale.value,
            &row.from_direct,
            &row.to_direct,
        );
        let predicted = prefix(
            prefix_name,
            core(core_name, row.number, numerator, denominator),
            row.from_prefix,
            row.to_prefix,
        );
        let residual = ordered_bits(row.actual) - ordered_bits(predicted.to_bits());
        if residual == 0 {
            fitness.exact += 1;
        } else {
            let abs = residual.unsigned_abs();
            fitness.sum_value = fitness.sum_value.saturating_add(abs);
            fitness.max_value = fitness.max_value.max(abs);
            if fitness.first_misses.len() < 24 {
                fitness.first_misses.push(format!(
                    "{} {}->{} x=0x{:016x} residual={:+} predicted=0x{:016x} oracle=0x{:016x}",
                    row.id,
                    row.from_direct,
                    row.to_direct,
                    row.number.to_bits(),
                    residual,
                    predicted.to_bits(),
                    row.actual,
                ));
            }
        }
    }
    fitness.sum_abs_ulp = fitness.sum_value.to_string();
    fitness.max_abs_ulp = fitness.max_value.to_string();
    let constants_bits = ["Pa", "atm", "psi"]
        .into_iter()
        .map(|unit| {
            let value = match table {
                "physical_base_per_unit" => physical(unit) * scale.value,
                "public_units_per_pa" => public_units_per_pa(unit) * scale.value,
                "public_reciprocal_to_base" => (1.0 / public_units_per_pa(unit)) * scale.value,
                _ => unreachable!(),
            };
            (unit.to_string(), format!("0x{:016x}", value.to_bits()))
        })
        .collect();
    Candidate {
        table: table.to_string(),
        scale_name: scale.name.clone(),
        scale_bits: format!("0x{:016x}", scale.value.to_bits()),
        core_schedule: core_name.to_string(),
        prefix_schedule: prefix_name.to_string(),
        constants_bits,
        fitness,
    }
}

fn better(left: &Candidate, right: &Candidate) -> bool {
    left.fitness.exact > right.fitness.exact
        || (left.fitness.exact == right.fitness.exact
            && left.fitness.sum_value < right.fitness.sum_value)
        || (left.fitness.exact == right.fitness.exact
            && left.fitness.sum_value == right.fitness.sum_value
            && left.fitness.max_value < right.fitness.max_value)
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
    let mut rows = Vec::new();
    for row in &metadata.rows {
        if row.category != "pressure" {
            continue;
        }
        let Some(actual) = bits(&by_id[row.id.as_str()].expected_bits) else {
            continue;
        };
        let (from_direct, from_prefix) = resolve(&row.from_unit);
        let (to_direct, to_prefix) = resolve(&row.to_unit);
        assert_ne!(from_direct, "bar");
        assert_ne!(to_direct, "bar");
        rows.push(Row {
            id: row.id.clone(),
            number: common::f64_from_hex(&row.number_bits).unwrap(),
            from_direct,
            to_direct,
            from_prefix,
            to_prefix,
            actual,
        });
    }

    let scales = scales();
    let tables = [
        "physical_base_per_unit",
        "public_units_per_pa",
        "public_reciprocal_to_base",
    ];
    let mut best: Vec<Candidate> = Vec::new();
    let mut count = 0;
    for table in tables {
        for scale in &scales {
            for core_name in CORE_SCHEDULES {
                for prefix_name in PREFIX_SCHEDULES {
                    count += 1;
                    let candidate = score(&rows, table, scale, core_name, prefix_name);
                    let position = best
                        .iter()
                        .position(|existing| better(&candidate, existing))
                        .unwrap_or(best.len());
                    best.insert(position, candidate);
                    best.truncate(100);
                }
            }
        }
    }
    for candidate in best.iter().take(20) {
        println!(
            "{}/{} exact sum={} table={} scale={} core={} prefix={}",
            candidate.fitness.exact,
            candidate.fitness.total,
            candidate.fitness.sum_abs_ulp,
            candidate.table,
            candidate.scale_name,
            candidate.core_schedule,
            candidate.prefix_schedule,
        );
    }
    let report = Report {
        schema_version: "w109.convert.pressure_hidden_scale_search.v1",
        function: "CONVERT",
        scored_rows: rows.len(),
        candidates: count,
        best,
        capture_provenance: answers.capture_provenance,
    };
    if let Some(path) = args.out {
        std::fs::write(&path, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
        println!("wrote pressure scale search -> {}", path.display());
    }
}
