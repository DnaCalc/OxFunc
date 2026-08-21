//! Fit the retired CONVERT evidence for prefix delta 10^-24 while keeping the
//! frozen v2 two-cell core unchanged.
//!
//! This clean-room search distinguishes a missing PC64 prefix constant from a
//! core-staging branch.  It searches x87-significand neighbors of the promoted
//! binary64 `1e-24` constant and reports every offset that reproduces all
//! discovery plus explicitly retired v1/v2 observations with delta -24.

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
    category: String,
    from_unit: String,
    to_unit: String,
    number_bits: String,
    core: f64,
    actual: u64,
}

#[derive(Serialize)]
struct OffsetScore {
    significand_offset: i64,
    exact: usize,
    misses: usize,
    first_misses: Vec<String>,
}

#[derive(Serialize)]
struct RowRace {
    id: String,
    category: String,
    from_unit: String,
    to_unit: String,
    number_bits: String,
    core_bits: String,
    oracle_bits: String,
    promoted_f64_bits: String,
    exact_decimal_bits: String,
}

#[derive(Serialize)]
struct Report {
    schema_version: &'static str,
    function: &'static str,
    delta_exponent: i32,
    rows: usize,
    anchor_f64_bits: String,
    anchor_ext80_hex: String,
    exact_decimal_ext80_hex: String,
    exact_decimal_offset_from_anchor: i64,
    best_exact: usize,
    best_offsets: Vec<i64>,
    zero_miss_offsets: Vec<i64>,
    representative_scores: Vec<OffsetScore>,
    row_races: Vec<RowRace>,
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

fn prefix_base(category: &str) -> Option<&'static str> {
    match category {
        "length" => Some("m"),
        "mass" => Some("g"),
        "time" => Some("sec"),
        "pressure" => Some("Pa"),
        "volume" => Some("l"),
        _ => None,
    }
}

fn resolve(unit: &str, category: &str) -> Resolved {
    if common::direct_unit(unit).is_some() {
        return Resolved {
            direct: unit.to_string(),
            prefix_exponent: 0,
        };
    }
    let base = prefix_base(category).unwrap();
    for (prefix, _) in common::PREFIXES {
        if unit == format!("{prefix}{base}") {
            return Resolved {
                direct: base.to_string(),
                prefix_exponent: prefix_exponent(prefix),
            };
        }
    }
    panic!("cannot resolve {category} unit {unit}");
}

fn factor(category: &str, unit: &str) -> f64 {
    if category == "length" {
        return match unit {
            "m" => 10_000_000_000.0,
            "in" => 254_000_000.0,
            "ft" => 3_048_000_000.0,
            "yd" => 9_144_000_000.0,
            "mi" => 16_093_440_000_000.0,
            "Nmi" => 18_520_000_000_000.0,
            other => panic!("no length factor for {other}"),
        };
    }
    if category == "pressure" {
        return match unit {
            "Pa" => 1.0,
            "atm" => 1.0 / "9.8692326671601280E-06".parse::<f64>().unwrap(),
            "psi" => 1.0 / "1.4503773773020920E-04".parse::<f64>().unwrap(),
            "bar" => f64::NAN,
            other => panic!("no pressure factor for {other}"),
        };
    }
    common::DIRECT_UNITS
        .iter()
        .find(|candidate| candidate.name == unit)
        .unwrap()
        .factor_decimal
        .parse()
        .unwrap()
}

fn ext_significand(value: &rx::Ext80) -> u64 {
    u64::from_le_bytes(value.0[..8].try_into().unwrap())
}

fn with_offset(anchor: &rx::Ext80, offset: i64) -> rx::Ext80 {
    let mut bytes = anchor.0;
    let significand = ext_significand(anchor);
    let adjusted = if offset >= 0 {
        significand.checked_add(offset as u64).unwrap()
    } else {
        significand.checked_sub(offset.unsigned_abs()).unwrap()
    };
    bytes[..8].copy_from_slice(&adjusted.to_le_bytes());
    rx::Ext80(bytes)
}

fn ext_hex(value: &rx::Ext80) -> String {
    value
        .0
        .iter()
        .rev()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn predict(core: f64, multiplier: &rx::Ext80) -> u64 {
    let cw = rx::CW_PC64_RN;
    rx::ext_to_f64(&rx::ext_mul(&rx::ext_from_f64(core), multiplier, cw), cw).to_bits()
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
        .filter_map(|row| {
            if !matches!(
                row.category.as_str(),
                "length" | "mass" | "time" | "pressure" | "volume"
            ) {
                return None;
            }
            let actual = bits(&by_id[row.id.as_str()].expected_bits)?;
            let from = resolve(&row.from_unit, &row.category);
            let to = resolve(&row.to_unit, &row.category);
            if from.prefix_exponent - to.prefix_exponent != -24
                || from.direct == "bar"
                || to.direct == "bar"
            {
                return None;
            }
            let number = common::f64_from_hex(&row.number_bits).unwrap();
            let core =
                (number * factor(&row.category, &from.direct)) / factor(&row.category, &to.direct);
            Some(Row {
                id: row.id,
                category: row.category,
                from_unit: row.from_unit,
                to_unit: row.to_unit,
                number_bits: row.number_bits,
                core,
                actual,
            })
        })
        .collect()
}

fn score(rows: &[Row], anchor: &rx::Ext80, offset: i64) -> OffsetScore {
    let multiplier = with_offset(anchor, offset);
    let mut first_misses = Vec::new();
    let exact = rows
        .iter()
        .filter(|row| {
            let predicted = predict(row.core, &multiplier);
            let is_exact = predicted == row.actual;
            if !is_exact && first_misses.len() < 8 {
                first_misses.push(format!(
                    "{} {} {}->{} x={} predicted=0x{predicted:016x} oracle=0x{:016x}",
                    row.id, row.category, row.from_unit, row.to_unit, row.number_bits, row.actual,
                ));
            }
            is_exact
        })
        .count();
    OffsetScore {
        significand_offset: offset,
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
    let anchor_f64 = "1e-24".parse::<f64>().unwrap();
    let anchor = rx::ext_from_f64(anchor_f64);
    let decimal = common::ext_from_decimal("1e-24");
    assert_eq!(anchor.0[8..], decimal.0[8..]);
    let decimal_offset =
        i128::from(ext_significand(&decimal)) - i128::from(ext_significand(&anchor));
    let decimal_offset = i64::try_from(decimal_offset).unwrap();

    let mut best_exact = 0;
    let mut best_offsets = Vec::new();
    let mut zero_miss_offsets = Vec::new();
    for offset in -16_384..=16_384 {
        let exact = rows
            .iter()
            .filter(|row| predict(row.core, &with_offset(&anchor, offset)) == row.actual)
            .count();
        if exact > best_exact {
            best_exact = exact;
            best_offsets.clear();
            best_offsets.push(offset);
        } else if exact == best_exact {
            best_offsets.push(offset);
        }
        if exact == rows.len() {
            zero_miss_offsets.push(offset);
        }
    }

    let mut representative_offsets = vec![
        -16_384,
        -8_192,
        -4_096,
        -2_048,
        -1,
        0,
        1,
        2_048,
        4_096,
        8_192,
        16_384,
        decimal_offset,
    ];
    representative_offsets.extend(best_offsets.iter().copied().take(8));
    representative_offsets.sort_unstable();
    representative_offsets.dedup();
    let representative_scores = representative_offsets
        .into_iter()
        .map(|offset| score(&rows, &anchor, offset))
        .collect();

    let promoted = anchor;
    let row_races = rows
        .iter()
        .map(|row| RowRace {
            id: row.id.clone(),
            category: row.category.clone(),
            from_unit: row.from_unit.clone(),
            to_unit: row.to_unit.clone(),
            number_bits: row.number_bits.clone(),
            core_bits: format!("0x{:016x}", row.core.to_bits()),
            oracle_bits: format!("0x{:016x}", row.actual),
            promoted_f64_bits: format!("0x{:016x}", predict(row.core, &promoted)),
            exact_decimal_bits: format!("0x{:016x}", predict(row.core, &decimal)),
        })
        .collect();

    let report = Report {
        schema_version: "w109.convert.delta_minus24_constant_search.v1",
        function: "CONVERT",
        delta_exponent: -24,
        rows: rows.len(),
        anchor_f64_bits: format!("0x{:016x}", anchor_f64.to_bits()),
        anchor_ext80_hex: ext_hex(&anchor),
        exact_decimal_ext80_hex: ext_hex(&decimal),
        exact_decimal_offset_from_anchor: decimal_offset,
        best_exact,
        best_offsets,
        zero_miss_offsets,
        representative_scores,
        row_races,
    };
    let out = root.join("score-convert-delta-minus24-constant-search.json");
    std::fs::write(&out, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
    println!(
        "delta=-24 rows={} best={}/{} best_offsets={} zero_miss_offsets={}",
        rows.len(),
        best_exact,
        rows.len(),
        report.best_offsets.len(),
        report.zero_miss_offsets.len(),
    );
    println!("decimal_offset={decimal_offset}");
    println!("wrote -> {}", out.display());
}
