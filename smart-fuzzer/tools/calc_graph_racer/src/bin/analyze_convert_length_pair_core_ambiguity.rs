//! Summarize, by directed length-unit pair, the retired evidence that
//! distinguishes the frozen v2 mul/div core from a ratio-first core.

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

#[derive(Default, Serialize)]
struct PairScore {
    rows: usize,
    disagreements: usize,
    primary_wins: usize,
    ratio_wins: usize,
    both_exact: usize,
    neither_exact: usize,
    witnesses: Vec<String>,
}

#[derive(Serialize)]
struct Report {
    schema_version: &'static str,
    function: &'static str,
    rows: usize,
    pairs: BTreeMap<String, PairScore>,
    conflict_free_ratio_switch_pairs: Vec<String>,
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

fn factor(unit: &str) -> f64 {
    match unit {
        "m" => 10_000_000_000.0,
        "in" => 254_000_000.0,
        "ft" => 3_048_000_000.0,
        "yd" => 9_144_000_000.0,
        "mi" => 16_093_440_000_000.0,
        "Nmi" => 18_520_000_000_000.0,
        other => panic!("no factor for {other}"),
    }
}

fn pow10(exponent: i32) -> f64 {
    format!("1e{exponent}").parse().unwrap()
}

fn finish(core: f64, delta: i32) -> u64 {
    let cw = rx::CW_PC64_RN;
    rx::ext_to_f64(
        &rx::ext_mul(&rx::ext_from_f64(core), &rx::ext_from_f64(pow10(delta)), cw),
        cw,
    )
    .to_bits()
}

fn process(
    root: &Path,
    meta_name: &str,
    answer_name: &str,
    pairs: &mut BTreeMap<String, PairScore>,
) -> usize {
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
    let mut rows = 0;
    for row in metadata.rows.iter().filter(|row| row.category == "length") {
        let actual = bits(&by_id[row.id.as_str()].expected_bits).unwrap();
        let number = common::f64_from_hex(&row.number_bits).unwrap();
        let from = resolve(&row.from_unit);
        let to = resolve(&row.to_unit);
        let delta = from.prefix_exponent - to.prefix_exponent;
        let primary = finish((number * factor(&from.direct)) / factor(&to.direct), delta);
        let ratio = finish(number * (factor(&from.direct) / factor(&to.direct)), delta);
        let pair = format!("{}->{}", row.from_unit, row.to_unit);
        let score = pairs.entry(pair).or_default();
        score.rows += 1;
        let p = primary == actual;
        let r = ratio == actual;
        match (p, r) {
            (true, true) => score.both_exact += 1,
            (true, false) => score.primary_wins += 1,
            (false, true) => score.ratio_wins += 1,
            (false, false) => score.neither_exact += 1,
        }
        if primary != ratio {
            score.disagreements += 1;
            if score.witnesses.len() < 12 {
                score.witnesses.push(format!(
                    "{} x={} primary=0x{primary:016x} ratio=0x{ratio:016x} oracle=0x{actual:016x}",
                    row.id, row.number_bits
                ));
            }
        }
        rows += 1;
    }
    rows
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
    let mut pairs = BTreeMap::new();
    let rows = datasets
        .into_iter()
        .map(|(m, a)| process(&root, m, a, &mut pairs))
        .sum();
    let conflict_free_ratio_switch_pairs = pairs
        .iter()
        .filter(|(_, score)| {
            score.ratio_wins > 0 && score.primary_wins == 0 && score.neither_exact == 0
        })
        .map(|(pair, _)| pair.clone())
        .collect::<Vec<_>>();
    let report = Report {
        schema_version: "w109.convert.length_pair_core_ambiguity.v1",
        function: "CONVERT",
        rows,
        pairs,
        conflict_free_ratio_switch_pairs,
    };
    let out = root.join("score-convert-length-pair-core-ambiguity.json");
    std::fs::write(&out, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
    println!(
        "rows={} conflict_free_switch_pairs={:?}",
        rows, report.conflict_free_ratio_switch_pairs
    );
    println!("wrote -> {}", out.display());
}
