//! Search whether the sole retired v2 CONVERT miss can be explained by the
//! binary64 meter-table constant alone, with the rest of the frozen v2 graph
//! unchanged.

#[path = "convert_research/common.rs"]
mod common;

use common::MetaDocument;
use oxfunc_core::excel_numeric::research as rx;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

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
    number: f64,
    from: Resolved,
    to: Resolved,
    actual: u64,
}

#[derive(Clone, Serialize)]
struct Candidate {
    meter_factor_bits: String,
    meter_factor: String,
    bit_offset: i64,
    exact: usize,
    misses: usize,
    first_misses: Vec<String>,
}

#[derive(Serialize)]
struct Report {
    schema_version: &'static str,
    function: &'static str,
    rows: usize,
    anchor_bits: String,
    search_radius: i64,
    best_exact: usize,
    best_offsets: Vec<i64>,
    zero_miss_offsets: Vec<i64>,
    decimal_power_candidates: Vec<Candidate>,
    representative_candidates: Vec<Candidate>,
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

fn fixed_factor(unit: &str) -> f64 {
    match unit {
        "in" => 254_000_000.0,
        "ft" => 3_048_000_000.0,
        "yd" => 9_144_000_000.0,
        "mi" => 16_093_440_000_000.0,
        "Nmi" => 18_520_000_000_000.0,
        other => panic!("no fixed factor for {other}"),
    }
}

fn factor(unit: &str, meter: f64) -> f64 {
    if unit == "m" {
        meter
    } else {
        fixed_factor(unit)
    }
}

fn pow10(exponent: i32) -> f64 {
    format!("1e{exponent}").parse().unwrap()
}

fn predict(row: &Row, meter: f64) -> u64 {
    let core = (row.number * factor(&row.from.direct, meter)) / factor(&row.to.direct, meter);
    let delta = pow10(row.from.prefix_exponent - row.to.prefix_exponent);
    let cw = rx::CW_PC64_RN;
    rx::ext_to_f64(
        &rx::ext_mul(&rx::ext_from_f64(core), &rx::ext_from_f64(delta), cw),
        cw,
    )
    .to_bits()
}

fn load(root: &Path, meta_name: &str, answer_name: &str) -> Vec<Row> {
    let metadata: MetaDocument =
        serde_json::from_slice(&std::fs::read(root.join(meta_name)).unwrap()).unwrap();
    let answers: WitnessSet =
        serde_json::from_slice(&std::fs::read(root.join(answer_name)).unwrap()).unwrap();
    assert_eq!(answers.function, "CONVERT");
    let by_id: BTreeMap<_, _> = answers
        .witnesses
        .iter()
        .map(|witness| (witness.id.as_str(), witness))
        .collect();
    metadata
        .rows
        .into_iter()
        .filter(|row| row.category == "length")
        .map(|row| Row {
            id: row.id.clone(),
            number: common::f64_from_hex(&row.number_bits).unwrap(),
            from: resolve(&row.from_unit),
            to: resolve(&row.to_unit),
            actual: bits(&by_id[row.id.as_str()].expected_bits).unwrap(),
        })
        .collect()
}

fn offset(anchor: f64, bit_offset: i64) -> f64 {
    let adjusted = if bit_offset >= 0 {
        anchor.to_bits().checked_add(bit_offset as u64).unwrap()
    } else {
        anchor
            .to_bits()
            .checked_sub(bit_offset.unsigned_abs())
            .unwrap()
    };
    f64::from_bits(adjusted)
}

fn score(rows: &[Row], meter: f64, anchor: f64) -> Candidate {
    let exact = rows
        .iter()
        .filter(|row| predict(row, meter) == row.actual)
        .count();
    let first_misses = rows
        .iter()
        .filter_map(|row| {
            let predicted = predict(row, meter);
            (predicted != row.actual).then(|| {
                format!(
                    "{} predicted=0x{predicted:016x} oracle=0x{:016x}",
                    row.id, row.actual
                )
            })
        })
        .take(12)
        .collect();
    Candidate {
        meter_factor_bits: format!("0x{:016x}", meter.to_bits()),
        meter_factor: format!("{meter:.17e}"),
        bit_offset: i64::try_from(i128::from(meter.to_bits()) - i128::from(anchor.to_bits()))
            .unwrap_or(i64::MIN),
        exact,
        misses: rows.len() - exact,
        first_misses,
    }
}

fn main() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../work/w109/G4-convert");
    let datasets = [
        (
            "batch-convert-discovery-20260809-meta.json",
            "answers-convert-discovery-20260809-clean.json",
        ),
        (
            "batch-convert-heldout-20260809-meta.json",
            "answers-convert-heldout-20260809.json",
        ),
        (
            "batch-convert-publication-heldout-v2-20260809-meta.json",
            "answers-convert-publication-heldout-v2-20260809.json",
        ),
    ];
    let rows: Vec<_> = datasets
        .into_iter()
        .flat_map(|(meta, answers)| load(&root, meta, answers))
        .collect();
    let anchor = 10_000_000_000.0_f64;
    let radius = 16_384_i64;
    let mut best_exact = 0;
    let mut best_offsets = Vec::new();
    let mut zero_miss_offsets = Vec::new();
    for bit_offset in -radius..=radius {
        let meter = offset(anchor, bit_offset);
        let exact = rows
            .iter()
            .filter(|row| predict(row, meter) == row.actual)
            .count();
        if exact > best_exact {
            best_exact = exact;
            best_offsets.clear();
            best_offsets.push(bit_offset);
        } else if exact == best_exact {
            best_offsets.push(bit_offset);
        }
        if exact == rows.len() {
            zero_miss_offsets.push(bit_offset);
        }
    }
    let decimal_power_candidates = (-3..=18)
        .map(|exponent| score(&rows, format!("1e{exponent}").parse().unwrap(), anchor))
        .collect();
    let mut representatives = vec![
        -radius, -8192, -4096, -2048, -1, 0, 1, 2048, 4096, 8192, radius,
    ];
    representatives.extend(best_offsets.iter().copied().take(16));
    representatives.sort_unstable();
    representatives.dedup();
    let representative_candidates = representatives
        .into_iter()
        .map(|bit_offset| score(&rows, offset(anchor, bit_offset), anchor))
        .collect();
    let report = Report {
        schema_version: "w109.convert.length_m_factor_search.v3",
        function: "CONVERT",
        rows: rows.len(),
        anchor_bits: format!("0x{:016x}", anchor.to_bits()),
        search_radius: radius,
        best_exact,
        best_offsets,
        zero_miss_offsets,
        decimal_power_candidates,
        representative_candidates,
    };
    let out = root.join("score-convert-length-m-factor-search-v3.json");
    std::fs::write(&out, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
    println!(
        "rows={} best={}/{} best_offsets={} zero_miss_offsets={}",
        rows.len(),
        best_exact,
        rows.len(),
        report.best_offsets.len(),
        report.zero_miss_offsets.len()
    );
    println!("wrote -> {}", out.display());
}
