//! Generate the disjoint oracle-blind publication held-out for frozen CONVERT v3.
//!
//! The generator reads metadata, never prior or publication oracle answers. It
//! excludes every tuple used by discovery, retired v1/v2, refinement-only v3,
//! and the independent Value2/readback control.  The frozen candidate is then
//! challenged with all-pair coverage, power-of-two ladders, and separately
//! selected first- and second-operation PC64 staging discriminators.

#[path = "convert_research/common.rs"]
mod common;
#[path = "convert_research/model_v3.rs"]
mod model_v3;

use common::{
    Category, MetaDocument, MetaRow, OwnedUnitSpec, PREFIXES, PREFIX_BASES,
    direct_units_in, prefix_unit,
};
use model_v3::{CoreVariant, FREEZE_ID};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

const SPLIT: &str = "publication-heldout-v3";
const MODEL_NAMES: [&str; 5] = [
    "frozen_v3_all_linear_pc64_stores",
    "retired_v2_f64_core",
    "length_first_product_pc64",
    "all_linear_first_product_pc64",
    "length_each_operation_pc64",
];

#[derive(Clone, Serialize)]
struct Probe { id: String, args: [String; 3] }

#[derive(Clone, Serialize)]
struct ProbeEnvelope { probe: Probe }

#[derive(Serialize)]
struct ProbeBatch {
    schema_version: &'static str,
    function: &'static str,
    row_id: &'static str,
    arg_encoding: serde_json::Value,
    probes: Vec<ProbeEnvelope>,
}

#[derive(Deserialize)]
struct ReadbackDoc { rows: Vec<ReadbackRow> }

#[derive(Deserialize)]
struct ReadbackRow { requested_bits: String, from_unit: String, to_unit: String }

struct Builder {
    sequence: usize,
    seen: BTreeSet<(u64, String, String)>,
    probes: Vec<ProbeEnvelope>,
    rows: Vec<MetaRow>,
}

impl Builder {
    fn new(seen: BTreeSet<(u64, String, String)>) -> Self {
        Self { sequence: 0, seen, probes: Vec::new(), rows: Vec::new() }
    }

    fn contains(&self, number: f64, from: &str, to: &str) -> bool {
        self.seen.contains(&(number.to_bits(), from.to_string(), to.to_string()))
    }

    fn push(&mut self, class: &str, category: &str, number: f64, from: &str, to: &str) {
        assert!(number.is_finite() && number != 0.0);
        assert!(
            self.seen.insert((number.to_bits(), from.to_string(), to.to_string())),
            "publication tuple repeats prior evidence: 0x{:016x} {from}->{to}",
            number.to_bits(),
        );
        let id = format!("conv-p3-{:05}", self.sequence);
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
                args: [format!("0x{:016x}", number.to_bits()), from.to_string(), to.to_string()],
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
    [Category::Length, Category::Mass, Category::Time, Category::Pressure, Category::Volume]
}

fn effective_units(category: Category) -> Vec<OwnedUnitSpec> {
    let mut units: Vec<_> = direct_units_in(category).map(|unit| OwnedUnitSpec {
        name: unit.name.to_string(),
        category: unit.category,
        factor_decimal: unit.factor_decimal.to_string(),
    }).collect();
    let base = PREFIX_BASES.iter().find(|(_, candidate)| *candidate == category).unwrap().0;
    units.extend(PREFIXES.iter().map(|(prefix, _)| prefix_unit(prefix, base).unwrap()));
    units.sort_by(|left, right| left.name.cmp(&right.name));
    units.dedup_by(|left, right| left.name == right.name);
    units
}

fn direct_units(category: Category) -> Vec<OwnedUnitSpec> {
    direct_units_in(category).map(|unit| OwnedUnitSpec {
        name: unit.name.to_string(),
        category: unit.category,
        factor_decimal: unit.factor_decimal.to_string(),
    }).collect()
}

fn next_random(seed: &mut u64) -> u64 {
    *seed ^= *seed << 13;
    *seed ^= *seed >> 7;
    *seed ^= *seed << 17;
    *seed
}

fn deterministic_value(seed: &mut u64, exponent_radius: i32) -> f64 {
    let raw = next_random(seed);
    let unbiased = ((raw >> 9) % u64::try_from(2 * exponent_radius + 1).unwrap()) as i32
        - exponent_radius;
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
    if value >= 0.0 { f64::from_bits(value.to_bits() + 1) }
    else { f64::from_bits(value.to_bits() - 1) }
}

fn next_down(value: f64) -> f64 {
    if value > 0.0 { f64::from_bits(value.to_bits() - 1) }
    else { f64::from_bits(value.to_bits() + 1) }
}

fn model_predictions(category: &str, number: f64, from: &str, to: &str) -> BTreeMap<String, String> {
    let variants = [
        CoreVariant::FrozenV3,
        CoreVariant::RetiredV2,
        CoreVariant::LengthFirstProductPc64,
        CoreVariant::FirstProductPc64AllLinear,
        CoreVariant::LengthSecondOperationPc64,
    ];
    MODEL_NAMES.iter().zip(variants).map(|(name, variant)| {
        ((*name).to_string(), model_v3::predict(number, category, from, to, variant).display())
    }).collect()
}

fn old_seen(root: &Path) -> BTreeSet<(u64, String, String)> {
    let mut seen = BTreeSet::new();
    for name in [
        "batch-convert-discovery-20260809-meta.json",
        "batch-convert-heldout-20260809-meta.json",
        "batch-convert-publication-heldout-v2-20260809-meta.json",
        "batch-convert-v3-length-discriminator-20260809-meta.json",
    ] {
        let document: MetaDocument = serde_json::from_slice(&std::fs::read(root.join(name)).unwrap()).unwrap();
        for row in document.rows {
            seen.insert((common::f64_from_hex(&row.number_bits).unwrap().to_bits(), row.from_unit, row.to_unit));
        }
    }
    let readback: ReadbackDoc = serde_json::from_slice(
        &std::fs::read(root.join("capture-convert-value2-readback-v2-20260809.json")).unwrap()
    ).unwrap();
    for row in readback.rows {
        seen.insert((common::f64_from_hex(&row.requested_bits).unwrap().to_bits(), row.from_unit, row.to_unit));
    }
    seen
}

fn find_discriminators(
    builder: &mut Builder,
    seed: &mut u64,
    category: Category,
    class: &str,
    left: CoreVariant,
    right: CoreVariant,
    target: usize,
) -> usize {
    let units: Vec<_> = effective_units(category)
        .into_iter()
        .filter(|unit| unit.name != "bar")
        .collect();
    let mut added = 0;
    let mut attempts = 0_u64;
    for _ in 0..3_000_000 {
        attempts += 1;
        if added == target { break; }
        let from = &units[(next_random(seed) as usize) % units.len()];
        let to = &units[(next_random(seed) as usize) % units.len()];
        let number = deterministic_value(seed, 240);
        if number == 0.0 || builder.contains(number, &from.name, &to.name) { continue; }
        let a = model_v3::predict(number, category.name(), &from.name, &to.name, left);
        let b = model_v3::predict(number, category.name(), &from.name, &to.name, right);
        if a != b {
            builder.push(class, category.name(), number, &from.name, &to.name);
            added += 1;
        }
    }
    println!("selected {added} {class} rows for {} after {attempts} attempts", category.name());
    added
}

fn output_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../work/w109/G4-convert")
}

fn main() {
    let root = output_root();
    let prior = old_seen(&root);
    let prior_count = prior.len();
    let mut builder = Builder::new(prior);
    let mut seed = 0x5033_434f_4e56_4552_u64;

    // Every supported same-category ordered pair (hence both forward and
    // reverse orientation), including every recognized prefix, twice.
    for category in categories() {
        let units = effective_units(category);
        for pass in 0..2 {
            for from in &units {
                for to in &units {
                    let number = next_unique(&builder, &mut seed, &from.name, &to.name);
                    builder.push(
                        if pass == 0 { "publication-all-pairs-a" } else { "publication-all-pairs-b" },
                        category.name(), number, &from.name, &to.name,
                    );
                }
            }
        }
    }

    // Signed immediate neighbors on a broad power-of-two ladder for every
    // direct ordered pair. This stresses normalization and magnitude without
    // reusing any oracle-observed number/pair tuple.
    for category in categories() {
        let units = direct_units(category);
        for from in &units {
            for to in &units {
                for exponent in [-220, -128, -64, -1, 0, 1, 64, 128, 220] {
                    let power = 2.0_f64.powi(exponent);
                    for number in [next_down(power), next_up(power), -next_down(power), -next_up(power)] {
                        if !builder.contains(number, &from.name, &to.name) {
                            builder.push("publication-direct-power-ladder", category.name(), number, &from.name, &to.name);
                        }
                    }
                }
            }
        }
    }

    // Separately kill the two tied staging axes. First-product rows compare a
    // global first store with the old f64 core; quotient rows compare the full
    // frozen graph with a graph whose second operation is native f64.
    for category in categories() {
        let _ = find_discriminators(
            &mut builder, &mut seed, category,
            "publication-first-product-pc64-discriminator",
            CoreVariant::FirstProductPc64AllLinear, CoreVariant::RetiredV2, 24,
        );
        let _ = find_discriminators(
            &mut builder, &mut seed, category,
            "publication-second-operation-pc64-discriminator",
            CoreVariant::FrozenV3, CoreVariant::FirstProductPc64AllLinear, 24,
        );
    }

    // Independent affine-temperature coverage; this path is intentionally
    // separate from generic linear-unit staging.
    for from in ["K", "C", "F"] {
        for to in ["K", "C", "F"] {
            for _ in 0..48 {
                let mut number = deterministic_value(&mut seed, 8).clamp(-10_000.0, 10_000.0);
                while number == 0.0 || builder.contains(number, from, to) {
                    number = deterministic_value(&mut seed, 8).clamp(-10_000.0, 10_000.0);
                }
                builder.push("publication-temperature-random", "temperature", number, from, to);
            }
        }
    }

    let batch = ProbeBatch {
        schema_version: "w109.convert.mixed_scalar_probe_batch.v3",
        function: "CONVERT",
        row_id: "convert-publication-heldout-v3-20260809",
        arg_encoding: json!({
            "number": "0x followed by exactly 16 IEEE-754 binary64 hex digits",
            "text": "verbatim non-hex JSON string",
            "plumbing": "Range.Value2 cells referenced by one shared formula; never formula literals"
        }),
        probes: builder.probes,
    };
    let metadata = MetaDocument {
        schema_version: "w109.convert.publication_heldout_predictions.v3".to_string(),
        function: "CONVERT".to_string(),
        selection_note: format!(
            "Oracle-blind deterministic publication set generated after {FREEZE_ID}; all tuples are disjoint from {prior_count} discovery, retired-v1/v2, refinement-v3, and Value2/readback tuples."
        ),
        model_names: MODEL_NAMES.iter().map(|name| (*name).to_string()).collect(),
        rows: builder.rows,
    };
    let batch_path = root.join("batch-convert-publication-heldout-v3-20260809.json");
    let meta_path = root.join("batch-convert-publication-heldout-v3-20260809-meta.json");
    let synthetic_path = root.join("synthetic-answers-convert-publication-heldout-v3-20260809.json");
    let synthetic_witnesses = metadata.rows.iter().map(|row| json!({
        "id": row.id,
        "args": [row.number_bits, row.from_unit, row.to_unit],
        "expected_bits": row.predictions["frozen_v3_all_linear_pc64_stores"]
    })).collect::<Vec<_>>();
    let synthetic_answers = json!({
        "function": "CONVERT",
        "witnesses": synthetic_witnesses,
        "capture_provenance": {"mode": "synthetic_offline_self_test", "freeze_id": FREEZE_ID}
    });
    std::fs::write(&batch_path, serde_json::to_vec_pretty(&batch).unwrap()).unwrap();
    std::fs::write(&meta_path, serde_json::to_vec_pretty(&metadata).unwrap()).unwrap();
    std::fs::write(&synthetic_path, serde_json::to_vec_pretty(&synthetic_answers).unwrap()).unwrap();
    println!("prior tuples excluded: {prior_count}");
    println!("wrote {} publication probes -> {}", batch.probes.len(), batch_path.display());
    println!("wrote publication metadata -> {}", meta_path.display());
    println!("wrote synthetic scorer self-test -> {}", synthetic_path.display());
    println!("offline candidate-informative rows: {}", metadata.rows.iter().filter(|row| row.informative).count());
    for class in [
        "publication-all-pairs-a", "publication-all-pairs-b", "publication-direct-power-ladder",
        "publication-first-product-pc64-discriminator", "publication-second-operation-pc64-discriminator",
        "publication-temperature-random",
    ] {
        println!("  {class}: {}", metadata.rows.iter().filter(|row| row.class == class).count());
    }
}
