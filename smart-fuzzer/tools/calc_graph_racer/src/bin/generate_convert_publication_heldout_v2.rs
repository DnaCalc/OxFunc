//! Generate the disjoint oracle-blind publication held-out for CONVERT v2.
//!
//! The generator loads both earlier metadata sets and refuses every repeated
//! `(number_bits, from_unit, to_unit)` tuple.  Selection uses only deterministic
//! random coverage and offline disagreement between the frozen v2 graph and
//! already-retired controls; it never reads publication-oracle answers.

#[path = "convert_research/common.rs"]
mod common;

use common::{
    Category, MetaDocument, MetaRow, OwnedUnitSpec, PREFIX_BASES, PREFIXES, direct_units_in,
    prefix_unit,
};
use oxfunc_core::excel_numeric::research as rx;
use serde::Serialize;
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

const SPLIT: &str = "publication-heldout-v2";
const MODEL_NAMES: [&str; 3] = ["unified_v2", "f64_final_control", "retired_v1_control"];

#[derive(Clone, Serialize)]
struct Probe {
    id: String,
    args: [String; 3],
}

#[derive(Clone, Serialize)]
struct ProbeEnvelope {
    probe: Probe,
}

#[derive(Serialize)]
struct ProbeBatch {
    schema_version: &'static str,
    function: &'static str,
    row_id: &'static str,
    arg_encoding: serde_json::Value,
    probes: Vec<ProbeEnvelope>,
}

#[derive(Clone)]
struct ResolvedUnit {
    direct: String,
    prefix_exponent: i32,
}

struct Builder {
    sequence: usize,
    seen: BTreeSet<(u64, String, String)>,
    probes: Vec<ProbeEnvelope>,
    rows: Vec<MetaRow>,
}

impl Builder {
    fn new(seen: BTreeSet<(u64, String, String)>) -> Self {
        Self {
            sequence: 0,
            seen,
            probes: Vec::new(),
            rows: Vec::new(),
        }
    }

    fn contains(&self, number: f64, from: &str, to: &str) -> bool {
        self.seen
            .contains(&(number.to_bits(), from.to_string(), to.to_string()))
    }

    fn push(&mut self, class: &str, category: &str, number: f64, from: &str, to: &str) {
        assert!(number.is_finite());
        assert_ne!(number, 0.0);
        assert!(
            self.seen
                .insert((number.to_bits(), from.to_string(), to.to_string())),
            "publication tuple repeats prior evidence: 0x{:016x} {from}->{to}",
            number.to_bits(),
        );
        let id = format!("conv-p2-{:05}", self.sequence);
        self.sequence += 1;
        let predictions = model_predictions(category, number, from, to);
        let informative = {
            let mut values = predictions.values();
            let first = values.next().unwrap();
            values.any(|value| value != first)
        };
        self.probes.push(ProbeEnvelope {
            probe: Probe {
                id: id.clone(),
                args: [
                    format!("0x{:016x}", number.to_bits()),
                    from.to_string(),
                    to.to_string(),
                ],
            },
        });
        self.rows.push(MetaRow {
            id,
            split: SPLIT.to_string(),
            class: class.to_string(),
            category: category.to_string(),
            number_bits: format!("0x{:016x}", number.to_bits()),
            from_unit: from.to_string(),
            to_unit: to.to_string(),
            informative,
            predictions,
        });
    }
}

fn categories() -> [Category; 5] {
    [
        Category::Length,
        Category::Mass,
        Category::Time,
        Category::Pressure,
        Category::Volume,
    ]
}

fn effective_units(category: Category) -> Vec<OwnedUnitSpec> {
    let mut units: Vec<_> = direct_units_in(category)
        .map(|unit| OwnedUnitSpec {
            name: unit.name.to_string(),
            category: unit.category,
            factor_decimal: unit.factor_decimal.to_string(),
        })
        .collect();
    let base = PREFIX_BASES
        .iter()
        .find(|(_, candidate)| *candidate == category)
        .unwrap()
        .0;
    units.extend(
        PREFIXES
            .iter()
            .map(|(prefix, _)| prefix_unit(prefix, base).unwrap()),
    );
    units.sort_by(|left, right| left.name.cmp(&right.name));
    units.dedup_by(|left, right| left.name == right.name);
    units
}

fn direct_units(category: Category) -> Vec<OwnedUnitSpec> {
    direct_units_in(category)
        .map(|unit| OwnedUnitSpec {
            name: unit.name.to_string(),
            category: unit.category,
            factor_decimal: unit.factor_decimal.to_string(),
        })
        .collect()
}

fn next_random(seed: &mut u64) -> u64 {
    *seed ^= *seed << 13;
    *seed ^= *seed >> 7;
    *seed ^= *seed << 17;
    *seed
}

fn deterministic_value(seed: &mut u64, exponent_radius: i32) -> f64 {
    let raw = next_random(seed);
    let unbiased =
        ((raw >> 9) % u64::try_from(2 * exponent_radius + 1).unwrap()) as i32 - exponent_radius;
    let exponent = u64::try_from(unbiased + 1023).unwrap();
    let mantissa = (next_random(seed) >> 12) & ((1_u64 << 52) - 1);
    let sign = (next_random(seed) & 1) << 63;
    f64::from_bits(sign | (exponent << 52) | mantissa)
}

fn next_unique(builder: &Builder, seed: &mut u64, from: &str, to: &str) -> f64 {
    for _ in 0..1_000_000 {
        let value = deterministic_value(seed, 240);
        if value != 0.0 && !builder.contains(value, from, to) {
            return value;
        }
    }
    panic!("could not generate unique value for {from}->{to}");
}

fn next_up(value: f64) -> f64 {
    if value >= 0.0 {
        f64::from_bits(value.to_bits() + 1)
    } else {
        f64::from_bits(value.to_bits() - 1)
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

fn resolve(unit: &str, category: &str) -> ResolvedUnit {
    if common::direct_unit(unit).is_some() {
        return ResolvedUnit {
            direct: unit.to_string(),
            prefix_exponent: 0,
        };
    }
    let base = prefix_base(category).unwrap();
    for (prefix, _) in PREFIXES {
        if unit == format!("{prefix}{base}") {
            return ResolvedUnit {
                direct: base.to_string(),
                prefix_exponent: prefix_exponent(prefix),
            };
        }
    }
    panic!("cannot resolve {category} unit {unit}");
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

fn physical_factor(unit: &str) -> f64 {
    common::DIRECT_UNITS
        .iter()
        .find(|candidate| candidate.name == unit)
        .unwrap()
        .factor_decimal
        .parse()
        .unwrap()
}

fn pressure_factor(unit: &str) -> f64 {
    match unit {
        "Pa" => 1.0,
        "atm" => 1.0 / "9.8692326671601280E-06".parse::<f64>().unwrap(),
        "psi" => 1.0 / "1.4503773773020920E-04".parse::<f64>().unwrap(),
        other => panic!("no supported pressure factor for {other}"),
    }
}

fn factor(category: &str, unit: &str) -> f64 {
    match category {
        "length" => angstroms(unit),
        "mass" | "time" | "volume" => physical_factor(unit),
        "pressure" => pressure_factor(unit),
        other => panic!("not linear: {other}"),
    }
}

fn pow10(exponent: i32) -> f64 {
    format!("1e{exponent}").parse().unwrap()
}

fn temperature(number: f64, from: &str, to: &str) -> f64 {
    match (from, to) {
        (a, b) if a == b => number,
        ("K", "C") => number - 273.15,
        ("C", "K") => number + 273.15,
        ("K", "F") => (number - 273.15) * 1.8 + 32.0,
        ("F", "K") => (number - 32.0) / 1.8 + 273.15,
        ("C", "F") => number * 1.8 + 32.0,
        ("F", "C") => (number - 32.0) / 1.8,
        _ => panic!("unknown temperature pair {from}->{to}"),
    }
}

fn prediction_string(value: Option<f64>) -> String {
    match value {
        Some(value) => format!("0x{:016x}", value.to_bits()),
        None => "error:NA".to_string(),
    }
}

fn linear_values(
    category: &str,
    number: f64,
    from: &str,
    to: &str,
) -> (Option<f64>, Option<f64>, Option<f64>) {
    let from = resolve(from, category);
    let to = resolve(to, category);
    if category == "pressure" && (from.direct == "bar" || to.direct == "bar") {
        return (None, None, None);
    }
    let core = (number * factor(category, &from.direct)) / factor(category, &to.direct);
    let delta = pow10(from.prefix_exponent - to.prefix_exponent);
    let cw = rx::CW_PC64_RN;
    let extended = rx::ext_mul(&rx::ext_from_f64(core), &rx::ext_from_f64(delta), cw);
    let unified = rx::ext_to_f64(&extended, cw);
    let f64_final = core * delta;
    let retired = match category {
        "length" => f64_final,
        "mass" | "time" | "volume" => {
            let direct_ratio = physical_factor(&from.direct) / physical_factor(&to.direct);
            (number * direct_ratio) * delta
        }
        "pressure" => {
            let units_per_pa = |unit: &str| match unit {
                "Pa" => 1.0,
                "atm" => "9.8692326671601280E-06".parse::<f64>().unwrap(),
                "psi" => "1.4503773773020920E-04".parse::<f64>().unwrap(),
                _ => unreachable!(),
            };
            (number * (units_per_pa(&to.direct) / units_per_pa(&from.direct))) * delta
        }
        _ => unreachable!(),
    };
    (Some(unified), Some(f64_final), Some(retired))
}

fn model_predictions(
    category: &str,
    number: f64,
    from: &str,
    to: &str,
) -> BTreeMap<String, String> {
    let values = if category == "temperature" {
        let value = Some(temperature(number, from, to));
        (value, value, value)
    } else {
        linear_values(category, number, from, to)
    };
    MODEL_NAMES
        .iter()
        .zip([values.0, values.1, values.2])
        .map(|(name, value)| ((*name).to_string(), prediction_string(value)))
        .collect()
}

fn old_seen(root: &Path) -> BTreeSet<(u64, String, String)> {
    let mut seen = BTreeSet::new();
    for name in [
        "batch-convert-discovery-20260809-meta.json",
        "batch-convert-heldout-20260809-meta.json",
    ] {
        let document: MetaDocument =
            serde_json::from_slice(&std::fs::read(root.join(name)).unwrap()).unwrap();
        for row in document.rows {
            seen.insert((
                common::f64_from_hex(&row.number_bits).unwrap().to_bits(),
                row.from_unit,
                row.to_unit,
            ));
        }
    }
    seen
}

fn output_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../work/w109/G4-convert")
}

fn main() {
    let root = output_root();
    let prior = old_seen(&root);
    let prior_count = prior.len();
    let mut builder = Builder::new(prior);
    let mut seed = 0x5032_434f_4e56_4552_u64;

    // Two independent mantissas for every supported/structural pair.
    for category in categories() {
        let units = effective_units(category);
        for pass in 0..2 {
            for from in &units {
                for to in &units {
                    let number = next_unique(&builder, &mut seed, &from.name, &to.name);
                    builder.push(
                        if pass == 0 {
                            "publication-full-pair-a"
                        } else {
                            "publication-full-pair-b"
                        },
                        category.name(),
                        number,
                        &from.name,
                        &to.name,
                    );
                }
            }
        }
    }

    // Extra independent direct-unit mantissas and far power-neighbor rows.
    for category in categories() {
        let units = direct_units(category);
        for from in &units {
            for to in &units {
                for _ in 0..3 {
                    let number = next_unique(&builder, &mut seed, &from.name, &to.name);
                    builder.push(
                        "publication-direct-extra",
                        category.name(),
                        number,
                        &from.name,
                        &to.name,
                    );
                }
                for exponent in [-200, -127, 127, 200] {
                    let number = next_up(2.0_f64.powi(exponent));
                    if !builder.contains(number, &from.name, &to.name) {
                        builder.push(
                            "publication-direct-power-neighbor",
                            category.name(),
                            number,
                            &from.name,
                            &to.name,
                        );
                    }
                }
            }
        }
    }

    // Frozen v2 versus f64-final double-rounding discriminators.
    for category in categories() {
        let units: Vec<_> = effective_units(category)
            .into_iter()
            .filter(|unit| unit.name != "bar")
            .collect();
        let mut added = 0;
        for _ in 0..5_000_000 {
            if added == 128 {
                break;
            }
            let from = &units[(next_random(&mut seed) as usize) % units.len()];
            let to = &units[(next_random(&mut seed) as usize) % units.len()];
            let number = deterministic_value(&mut seed, 240);
            if number == 0.0 || builder.contains(number, &from.name, &to.name) {
                continue;
            }
            let (unified, f64_final, _) =
                linear_values(category.name(), number, &from.name, &to.name);
            if unified.unwrap().to_bits() != f64_final.unwrap().to_bits() {
                builder.push(
                    "publication-x87-final-discriminator",
                    category.name(),
                    number,
                    &from.name,
                    &to.name,
                );
                added += 1;
            }
        }
        assert_eq!(
            added,
            128,
            "insufficient x87-final discriminators for {}",
            category.name()
        );
    }

    // Additional reciprocal-table versus retired-ratio pressure kills.
    let pressure_units: Vec<_> = effective_units(Category::Pressure)
        .into_iter()
        .filter(|unit| unit.name != "bar")
        .collect();
    let mut pressure_added = 0;
    for _ in 0..5_000_000 {
        if pressure_added == 128 {
            break;
        }
        let from = &pressure_units[(next_random(&mut seed) as usize) % pressure_units.len()];
        let to = &pressure_units[(next_random(&mut seed) as usize) % pressure_units.len()];
        let number = deterministic_value(&mut seed, 240);
        if number == 0.0 || builder.contains(number, &from.name, &to.name) {
            continue;
        }
        let (unified, _, retired) = linear_values("pressure", number, &from.name, &to.name);
        if unified.unwrap().to_bits() != retired.unwrap().to_bits() {
            builder.push(
                "publication-pressure-reciprocal-discriminator",
                "pressure",
                number,
                &from.name,
                &to.name,
            );
            pressure_added += 1;
        }
    }
    assert_eq!(
        pressure_added, 128,
        "insufficient pressure reciprocal discriminators"
    );

    // Independent affine temperature mantissas.
    for from in ["K", "C", "F"] {
        for to in ["K", "C", "F"] {
            for _ in 0..32 {
                let mut number = deterministic_value(&mut seed, 8).clamp(-10_000.0, 10_000.0);
                while number == 0.0 || builder.contains(number, from, to) {
                    number = deterministic_value(&mut seed, 8).clamp(-10_000.0, 10_000.0);
                }
                builder.push(
                    "publication-temperature-random",
                    "temperature",
                    number,
                    from,
                    to,
                );
            }
        }
    }

    let batch = ProbeBatch {
        schema_version: "w109.convert.mixed_scalar_probe_batch.v2",
        function: "CONVERT",
        row_id: "convert-publication-heldout-v2-20260809",
        arg_encoding: json!({
            "number": "0x followed by exactly 16 IEEE-754 binary64 hex digits",
            "text": "verbatim non-hex JSON string",
            "plumbing": "Range.Value2 cells referenced by one shared formula; never formula literals"
        }),
        probes: builder.probes,
    };
    let metadata = MetaDocument {
        schema_version: "w109.convert.publication_heldout_predictions.v2".to_string(),
        function: "CONVERT".to_string(),
        selection_note: format!(
            "Oracle-blind deterministic publication set generated after v2 freeze; all tuples are disjoint from {prior_count} prior discovery/retired tuples. Full-pair random coverage plus frozen-v2/control disagreements."
        ),
        model_names: MODEL_NAMES.iter().map(|name| (*name).to_string()).collect(),
        rows: builder.rows,
    };
    let batch_path = root.join("batch-convert-publication-heldout-v2-20260809.json");
    let meta_path = root.join("batch-convert-publication-heldout-v2-20260809-meta.json");
    let synthetic_path =
        root.join("synthetic-answers-convert-publication-heldout-v2-20260809.json");
    let synthetic_witnesses = metadata
        .rows
        .iter()
        .map(|row| {
            json!({
                "id": row.id,
                "args": [row.number_bits, row.from_unit, row.to_unit],
                "expected_bits": row.predictions["unified_v2"]
            })
        })
        .collect::<Vec<_>>();
    let synthetic_answers = json!({
        "function": "CONVERT",
        "witnesses": synthetic_witnesses,
        "capture_provenance": {
            "mode": "synthetic_offline_self_test",
            "freeze_id": "g4-05.convert.unified.20260809.v2"
        }
    });
    std::fs::write(&batch_path, serde_json::to_vec_pretty(&batch).unwrap()).unwrap();
    std::fs::write(&meta_path, serde_json::to_vec_pretty(&metadata).unwrap()).unwrap();
    std::fs::write(
        &synthetic_path,
        serde_json::to_vec_pretty(&synthetic_answers).unwrap(),
    )
    .unwrap();
    println!("prior tuples excluded: {prior_count}");
    println!(
        "wrote {} publication probes -> {}",
        batch.probes.len(),
        batch_path.display()
    );
    println!("wrote publication metadata -> {}", meta_path.display());
    println!(
        "wrote synthetic scorer self-test -> {}",
        synthetic_path.display()
    );
    let informative = metadata.rows.iter().filter(|row| row.informative).count();
    println!("offline candidate-informative rows: {informative}");
}
