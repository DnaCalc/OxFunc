//! Search simple, predeclared predicates for the retired v2 length kill.
//!
//! The primary graph is frozen v2.  Alternate cores are the factor-ratio core
//! and the effective-factor mul/div graph, both of which produce the killed
//! row.  Rules are evaluated only through their disagreement sets: a rule is
//! exact iff it switches every row where primary is wrong/alternate is right
//! and switches no row where primary is right/alternate is wrong.

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

#[derive(Clone, Serialize)]
struct Attributes {
    id: String,
    input_exponent: i32,
    delta_exponent: i32,
    from_prefix_exponent: i32,
    to_prefix_exponent: i32,
    same_direct: bool,
    from_unit: String,
    to_unit: String,
    number_bits: String,
}

#[derive(Clone, Serialize)]
struct ExactRule {
    alternate: String,
    family: String,
    input_threshold: i32,
    delta_threshold: Option<i32>,
    switched_harmless_rows: usize,
}

#[derive(Serialize)]
struct AlternateReport {
    needed_switches: Vec<Attributes>,
    harmful_switches: usize,
    nearest_harmful_by_input_exponent: Vec<Attributes>,
    exact_rules: Vec<ExactRule>,
}

#[derive(Serialize)]
struct Report {
    schema_version: &'static str,
    function: &'static str,
    rows: usize,
    primary_misses: usize,
    alternates: BTreeMap<String, AlternateReport>,
}

#[derive(Clone)]
struct Resolved {
    direct: String,
    prefix_exponent: i32,
}

#[derive(Clone)]
struct Difference {
    attrs: Attributes,
    primary_exact: bool,
    alternate_exact: bool,
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

fn final_prefix(core: f64, delta: i32) -> f64 {
    let cw = rx::CW_PC64_RN;
    rx::ext_to_f64(
        &rx::ext_mul(&rx::ext_from_f64(core), &rx::ext_from_f64(pow10(delta)), cw),
        cw,
    )
}

fn exponent(value: f64) -> i32 {
    i32::from(((value.to_bits() >> 52) & 0x7ff) as u16) - 1023
}

fn load(root: &Path, meta_name: &str, answer_name: &str) -> Vec<(common::MetaRow, u64)> {
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
        .map(|row| {
            let actual = bits(&by_id[row.id.as_str()].expected_bits).unwrap();
            (row, actual)
        })
        .collect()
}

fn rule_matches(family: &str, attrs: &Attributes, input: i32, delta: i32) -> bool {
    match family {
        "input_ge" => attrs.input_exponent >= input,
        "same_direct_input_ge" => attrs.same_direct && attrs.input_exponent >= input,
        "input_ge_delta_le" => attrs.input_exponent >= input && attrs.delta_exponent <= delta,
        "same_direct_input_ge_delta_le" => {
            attrs.same_direct && attrs.input_exponent >= input && attrs.delta_exponent <= delta
        }
        "same_direct_input_ge_delta_eq" => {
            attrs.same_direct && attrs.input_exponent >= input && attrs.delta_exponent == delta
        }
        _ => panic!("unknown rule family {family}"),
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
    let mut differences: BTreeMap<String, Vec<Difference>> = [
        "ratio_core".to_string(),
        "effective_factor_mul_div".to_string(),
    ]
    .into_iter()
    .map(|name| (name, Vec::new()))
    .collect();
    let mut primary_misses = 0;
    for (row, actual) in &rows {
        let from = resolve(&row.from_unit);
        let to = resolve(&row.to_unit);
        let number = common::f64_from_hex(&row.number_bits).unwrap();
        let delta = from.prefix_exponent - to.prefix_exponent;
        let primary_core = (number * angstroms(&from.direct)) / angstroms(&to.direct);
        let primary = final_prefix(primary_core, delta).to_bits();
        if primary != *actual {
            primary_misses += 1;
        }
        let ratio = final_prefix(
            number * (angstroms(&from.direct) / angstroms(&to.direct)),
            delta,
        )
        .to_bits();
        let effective_from = angstroms(&from.direct) * pow10(from.prefix_exponent);
        let effective_to = angstroms(&to.direct) * pow10(to.prefix_exponent);
        let effective = ((number * effective_from) / effective_to).to_bits();
        let attrs = Attributes {
            id: row.id.clone(),
            input_exponent: exponent(number.abs()),
            delta_exponent: delta,
            from_prefix_exponent: from.prefix_exponent,
            to_prefix_exponent: to.prefix_exponent,
            same_direct: from.direct == to.direct,
            from_unit: row.from_unit.clone(),
            to_unit: row.to_unit.clone(),
            number_bits: row.number_bits.clone(),
        };
        for (name, alternate) in [
            ("ratio_core", ratio),
            ("effective_factor_mul_div", effective),
        ] {
            if alternate != primary {
                differences.get_mut(name).unwrap().push(Difference {
                    attrs: attrs.clone(),
                    primary_exact: primary == *actual,
                    alternate_exact: alternate == *actual,
                });
            }
        }
    }

    let families = [
        "input_ge",
        "same_direct_input_ge",
        "input_ge_delta_le",
        "same_direct_input_ge_delta_le",
        "same_direct_input_ge_delta_eq",
    ];
    let mut alternates = BTreeMap::new();
    for (alternate, diffs) in differences {
        let needed: Vec<_> = diffs
            .iter()
            .filter(|row| !row.primary_exact && row.alternate_exact)
            .map(|row| row.attrs.clone())
            .collect();
        let harmful: Vec<_> = diffs
            .iter()
            .filter(|row| row.primary_exact && !row.alternate_exact)
            .collect();
        let mut exact_rules = Vec::new();
        for family in families {
            for input_threshold in -300..=300 {
                let deltas: Vec<i32> = if family.contains("delta") {
                    (-39..=39).collect()
                } else {
                    vec![0]
                };
                for delta_threshold in deltas {
                    let switches_needed = needed
                        .iter()
                        .all(|row| rule_matches(family, row, input_threshold, delta_threshold));
                    if !switches_needed {
                        continue;
                    }
                    let harmful_count = harmful
                        .iter()
                        .filter(|row| {
                            rule_matches(family, &row.attrs, input_threshold, delta_threshold)
                        })
                        .count();
                    if harmful_count == 0 {
                        exact_rules.push(ExactRule {
                            alternate: alternate.clone(),
                            family: family.to_string(),
                            input_threshold,
                            delta_threshold: family.contains("delta").then_some(delta_threshold),
                            switched_harmless_rows: diffs
                                .iter()
                                .filter(|row| {
                                    row.primary_exact
                                        && row.alternate_exact
                                        && rule_matches(
                                            family,
                                            &row.attrs,
                                            input_threshold,
                                            delta_threshold,
                                        )
                                })
                                .count(),
                        });
                    }
                }
            }
        }
        let mut nearest = harmful
            .iter()
            .map(|row| row.attrs.clone())
            .collect::<Vec<_>>();
        let target_exponent = needed.first().map(|row| row.input_exponent).unwrap_or(0);
        nearest.sort_by_key(|row| (row.input_exponent - target_exponent).abs());
        nearest.truncate(64);
        println!(
            "{alternate}: needed={} harmful={} exact_rules={}",
            needed.len(),
            harmful.len(),
            exact_rules.len(),
        );
        for rule in exact_rules.iter().take(20) {
            println!(
                "  {} input>={} delta={:?}",
                rule.family, rule.input_threshold, rule.delta_threshold
            );
        }
        alternates.insert(
            alternate,
            AlternateReport {
                needed_switches: needed,
                harmful_switches: harmful.len(),
                nearest_harmful_by_input_exponent: nearest,
                exact_rules,
            },
        );
    }
    let report = Report {
        schema_version: "w109.convert.length_v3_switch_search.v1",
        function: "CONVERT",
        rows: rows.len(),
        primary_misses,
        alternates,
    };
    let out = root.join("score-convert-length-v3-switch-search.json");
    std::fs::write(&out, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
    println!("wrote switch search -> {}", out.display());
}
