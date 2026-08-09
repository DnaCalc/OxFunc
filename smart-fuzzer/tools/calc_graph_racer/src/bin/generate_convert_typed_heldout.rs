//! Generate the oracle-blind W109 CONVERT discovery and held-out batteries.
//!
//! Numeric arguments are exact-bits hex strings; unit arguments are verbatim
//! non-hex strings.  The intended runner contract is the hardened
//! `Run-W109BulkBatch.ps1`/`CellRefBatch.psm1` Value2 path: strings must be
//! written to argument cells as strings, while `0x[0-9a-f]{16}` values are
//! decoded to binary64 first.  No numeric argument is placed in formula text.

#[path = "convert_research/common.rs"]
mod common;

use common::{
    Category, DIRECT_UNITS, MODEL_NAMES, MetaDocument, MetaRow, OwnedUnitSpec, PREFIX_BASES,
    PREFIXES, direct_units_in, hex, predictions, predictions_are_informative, prefix_unit,
};
use serde::Serialize;
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

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
    row_id: String,
    arg_encoding: serde_json::Value,
    probes: Vec<ProbeEnvelope>,
}

struct SplitBuilder {
    split: &'static str,
    sequence: usize,
    seen: BTreeSet<(u64, String, String)>,
    probes: Vec<ProbeEnvelope>,
    rows: Vec<MetaRow>,
}

impl SplitBuilder {
    fn new(split: &'static str) -> Self {
        Self {
            split,
            sequence: 0,
            seen: BTreeSet::new(),
            probes: Vec::new(),
            rows: Vec::new(),
        }
    }

    fn push_linear(
        &mut self,
        class: &str,
        number: f64,
        from: &OwnedUnitSpec,
        to: &OwnedUnitSpec,
    ) {
        assert_eq!(from.category, to.category);
        if !number.is_finite() || number == 0.0 {
            return;
        }
        if !self
            .seen
            .insert((number.to_bits(), from.name.clone(), to.name.clone()))
        {
            return;
        }
        let id = format!("conv-{}-{:05}", &self.split[..2], self.sequence);
        self.sequence += 1;
        let model_predictions = predictions(number, from, to);
        let informative = predictions_are_informative(&model_predictions);
        self.probes.push(ProbeEnvelope {
            probe: Probe {
                id: id.clone(),
                args: [hex(number), from.name.clone(), to.name.clone()],
            },
        });
        self.rows.push(MetaRow {
            id,
            split: self.split.to_string(),
            class: class.to_string(),
            category: from.category.name().to_string(),
            number_bits: hex(number),
            from_unit: from.name.clone(),
            to_unit: to.name.clone(),
            informative,
            predictions: model_predictions,
        });
    }

    fn push_unmodeled(&mut self, class: &str, number: f64, from: &str, to: &str) {
        if !self
            .seen
            .insert((number.to_bits(), from.to_string(), to.to_string()))
        {
            return;
        }
        let id = format!("conv-{}-{:05}", &self.split[..2], self.sequence);
        self.sequence += 1;
        self.probes.push(ProbeEnvelope {
            probe: Probe {
                id: id.clone(),
                args: [hex(number), from.to_string(), to.to_string()],
            },
        });
        self.rows.push(MetaRow {
            id,
            split: self.split.to_string(),
            class: class.to_string(),
            category: "temperature".to_string(),
            number_bits: hex(number),
            from_unit: from.to_string(),
            to_unit: to.to_string(),
            informative: false,
            predictions: BTreeMap::new(),
        });
    }
}

fn next_up(value: f64) -> f64 {
    if value == f64::INFINITY {
        value
    } else if value == -0.0 {
        f64::from_bits(1)
    } else if value >= 0.0 {
        f64::from_bits(value.to_bits() + 1)
    } else {
        f64::from_bits(value.to_bits() - 1)
    }
}

fn next_down(value: f64) -> f64 {
    if value == f64::NEG_INFINITY {
        value
    } else if value == 0.0 {
        -f64::from_bits(1)
    } else if value > 0.0 {
        f64::from_bits(value.to_bits() - 1)
    } else {
        f64::from_bits(value.to_bits() + 1)
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
    let mut units: Vec<OwnedUnitSpec> = direct_units_in(category)
        .map(|unit| OwnedUnitSpec {
            name: unit.name.to_string(),
            category: unit.category,
            factor_decimal: unit.factor_decimal.to_string(),
        })
        .collect();
    let prefix_base = PREFIX_BASES
        .iter()
        .find(|(_, candidate_category)| *candidate_category == category)
        .unwrap()
        .0;
    units.extend(
        PREFIXES
            .iter()
            .map(|(prefix, _)| prefix_unit(prefix, prefix_base).unwrap()),
    );
    units.sort_by(|a, b| a.name.cmp(&b.name));
    units.dedup_by(|a, b| a.name == b.name);
    units
}

fn direct_owned(name: &str) -> OwnedUnitSpec {
    let unit = DIRECT_UNITS.iter().find(|unit| unit.name == name).unwrap();
    OwnedUnitSpec {
        name: unit.name.to_string(),
        category: unit.category,
        factor_decimal: unit.factor_decimal.to_string(),
    }
}

fn deterministic_value(seed: &mut u64, allow_negative: bool) -> f64 {
    *seed ^= *seed << 13;
    *seed ^= *seed >> 7;
    *seed ^= *seed << 17;
    let unbiased = ((*seed >> 9) % 161) as i32 - 80;
    let exponent = u64::try_from(unbiased + 1023).unwrap();
    let mantissa = (*seed >> 12) & ((1_u64 << 52) - 1);
    let sign = if allow_negative && (*seed & 1) != 0 {
        1_u64 << 63
    } else {
        0
    };
    f64::from_bits(sign | (exponent << 52) | mantissa)
}

fn add_discovery(discovery: &mut SplitBuilder) -> Vec<(OwnedUnitSpec, OwnedUnitSpec)> {
    let mut reveal_pairs = Vec::new();

    // Exhaustive current-surface cross-product at input=1.  This includes
    // every direct unit, every supported decimal prefix spelling, both
    // directions, and identity controls.
    for category in categories() {
        let units = effective_units(category);
        for from in &units {
            for to in &units {
                discovery.push_linear("all-supported-pairs-at-one", 1.0, from, to);
            }
        }
    }

    // Identify direct-unit pairs where any fixed arithmetic/constant model
    // differs.  Selection remains oracle-free.
    for category in categories() {
        let units: Vec<_> = direct_units_in(category)
            .map(|unit| direct_owned(unit.name))
            .collect();
        for from in &units {
            for to in &units {
                if from.name == to.name {
                    continue;
                }
                let model_predictions = predictions(1.0, from, to);
                if predictions_are_informative(&model_predictions) {
                    reveal_pairs.push((from.clone(), to.clone()));
                    discovery.push_linear("direct-reveal-neighbor", next_down(1.0), from, to);
                    discovery.push_linear("direct-reveal-neighbor", next_up(1.0), from, to);
                }
            }
        }
    }

    // Exact powers preserve the input mantissa, exposing constant low bits and
    // final-rounding behavior.  The whole -60..=60 ladder is retained for
    // every direct reveal pair.  Adjacent-bit probes at structural exponents
    // separate ratio-first from multiply-then-divide schedules.
    for (from, to) in &reveal_pairs {
        for exponent in -60..=60 {
            discovery.push_linear(
                "direct-power-of-two-ladder",
                2.0_f64.powi(exponent),
                from,
                to,
            );
        }
        for exponent in [-60, -53, -1, 0, 1, 53, 60] {
            let power = 2.0_f64.powi(exponent);
            discovery.push_linear("direct-power-neighbor", next_down(power), from, to);
            discovery.push_linear("direct-power-neighbor", next_up(power), from, to);
        }
    }

    // Known clean-ratio ISA/staging probes are included whether or not the
    // fixed model set happened to disagree at x=1.
    for (from_name, to_name) in [
        ("ft", "yd"),
        ("yd", "ft"),
        ("in", "ft"),
        ("ft", "in"),
        ("tsp", "tbs"),
        ("tbs", "tsp"),
        ("oz", "cup"),
        ("cup", "oz"),
        ("pt", "gal"),
        ("gal", "pt"),
    ] {
        let from = direct_owned(from_name);
        let to = direct_owned(to_name);
        for exponent in -60..=60 {
            discovery.push_linear(
                "exact-ratio-power-of-two-ladder",
                2.0_f64.powi(exponent),
                &from,
                &to,
            );
        }
    }

    // Prefix controls: base<->every prefix, identity, adjacent prefixes, and
    // the full Y<->f span.  The exhaustive x=1 cross-product above is the
    // completeness guard; these extra values diagnose prefix decomposition.
    for (base, category) in PREFIX_BASES {
        let base_unit = direct_owned(base);
        let prefixed: Vec<_> = PREFIXES
            .iter()
            .map(|(prefix, _)| prefix_unit(prefix, base).unwrap())
            .collect();
        for unit in &prefixed {
            for number in [next_down(1.0), next_up(1.0)] {
                discovery.push_linear("prefix-base-neighbor", number, unit, &base_unit);
                discovery.push_linear("prefix-base-neighbor", number, &base_unit, unit);
                discovery.push_linear("prefix-identity-neighbor", number, unit, unit);
            }
        }
        for pair in prefixed.windows(2) {
            discovery.push_linear("prefix-adjacent-forward", 1.0, &pair[0], &pair[1]);
            discovery.push_linear("prefix-adjacent-reverse", 1.0, &pair[1], &pair[0]);
        }
        let largest = prefix_unit("Y", base).unwrap();
        let smallest = prefix_unit("f", base).unwrap();
        for exponent in [-60, -1, 0, 1, 60] {
            let number = 2.0_f64.powi(exponent);
            discovery.push_linear("prefix-extreme-ladder", number, &largest, &smallest);
            discovery.push_linear("prefix-extreme-ladder", number, &smallest, &largest);
        }
        assert_eq!(base_unit.category, category);
    }

    // The affine temperature lane is a typed-control block.  It is captured
    // alongside the linear table but intentionally excluded from the constant
    // model scores.
    for from in ["K", "C", "F"] {
        for to in ["K", "C", "F"] {
            for number in [-459.67, -273.15, -40.0, 0.0, 32.0, 68.0, 100.0, 212.0, 273.15, 373.15] {
                discovery.push_unmodeled("temperature-affine-control", number, from, to);
            }
        }
    }

    reveal_pairs
}

fn add_heldout(heldout: &mut SplitBuilder, reveal_pairs: &[(OwnedUnitSpec, OwnedUnitSpec)]) {
    let mut seed = 0x434f_4e56_4552_5409_u64;

    // One oracle-blind mantissa/exponent per member of the full supported
    // cross-product.  These inputs are disjoint from the discovery constants.
    for category in categories() {
        let units = effective_units(category);
        for from in &units {
            for to in &units {
                let number = deterministic_value(&mut seed, true);
                heldout.push_linear("heldout-all-supported-pairs", number, from, to);
            }
        }
    }

    // Direct pairs get a second independent mantissa, ensuring the held-out
    // result is not carried by prefix-heavy collapse controls.
    for category in categories() {
        let units: Vec<_> = direct_units_in(category)
            .map(|unit| direct_owned(unit.name))
            .collect();
        for from in &units {
            for to in &units {
                let number = deterministic_value(&mut seed, true);
                heldout.push_linear("heldout-direct-second-mantissa", number, from, to);
            }
        }
    }

    // Adjacent-to-power inputs were not used by the exact-power ladder.  Odd
    // exponents form a disjoint held-out discriminator for all reveal pairs.
    for (from, to) in reveal_pairs {
        for exponent in (-59..=59).step_by(2) {
            let power = 2.0_f64.powi(exponent);
            heldout.push_linear("heldout-power-neighbor-up", next_up(power), from, to);
            heldout.push_linear("heldout-power-neighbor-down", next_down(power), from, to);
        }
    }

    for from in ["K", "C", "F"] {
        for to in ["K", "C", "F"] {
            let number = deterministic_value(&mut seed, true).clamp(-400.0, 400.0);
            heldout.push_unmodeled("heldout-temperature-affine", number, from, to);
        }
    }
}

fn write_split(root: &Path, split: SplitBuilder) {
    let batch_path = root.join(format!("batch-convert-{}-20260809.json", split.split));
    let meta_path = root.join(format!("batch-convert-{}-20260809-meta.json", split.split));
    let batch = ProbeBatch {
        schema_version: "w109.convert.mixed_scalar_probe_batch.v1",
        function: "CONVERT",
        row_id: format!("convert-{}-20260809", split.split),
        arg_encoding: json!({
            "number": "0x followed by exactly 16 IEEE-754 binary64 hex digits",
            "text": "verbatim non-hex JSON string",
            "plumbing": "Range.Value2 cells referenced by one shared formula; never formula literals"
        }),
        probes: split.probes,
    };
    let meta = MetaDocument {
        schema_version: "w109.convert.model_predictions.v1".to_string(),
        function: "CONVERT".to_string(),
        selection_note: "Oracle-free candidate-disagreement selection. Discovery and held-out IDs/inputs are disjoint; temperature rows are typed controls and have no linear-model predictions.".to_string(),
        model_names: MODEL_NAMES.iter().map(|name| (*name).to_string()).collect(),
        rows: split.rows,
    };
    let synthetic_path = root.join(format!(
        "synthetic-answers-convert-{}-x87-decimal-cont-pc64.json",
        split.split
    ));
    let synthetic_witnesses = meta
        .rows
        .iter()
        .map(|row| {
            let expected = row
                .predictions
                .get("x87_decimal_cont_pc64")
                .cloned()
                .unwrap_or_else(|| "control:unmodeled".to_string());
            json!({
                "id": row.id,
                "args": [row.number_bits, row.from_unit, row.to_unit],
                "expected_bits": expected
            })
        })
        .collect::<Vec<_>>();
    let synthetic_answers = json!({
        "function": "CONVERT",
        "witnesses": synthetic_witnesses,
        "capture_provenance": {
            "mode": "synthetic_offline_self_test",
            "model": "x87_decimal_cont_pc64"
        }
    });
    std::fs::write(&batch_path, serde_json::to_vec_pretty(&batch).unwrap()).unwrap();
    std::fs::write(&meta_path, serde_json::to_vec_pretty(&meta).unwrap()).unwrap();
    std::fs::write(
        &synthetic_path,
        serde_json::to_vec_pretty(&synthetic_answers).unwrap(),
    )
    .unwrap();
    println!(
        "wrote {} probes -> {}",
        batch.probes.len(),
        batch_path.display()
    );
    println!("wrote model metadata -> {}", meta_path.display());
    println!("wrote synthetic scorer self-test -> {}", synthetic_path.display());
}

fn write_smoke(root: &Path, discovery: &SplitBuilder) {
    let wanted = [
        (1.0_f64.to_bits(), "m", "ft"),
        (1.0_f64.to_bits(), "ft", "yd"),
        (1.0_f64.to_bits(), "l", "gal"),
        (1.0_f64.to_bits(), "km", "m"),
        (68.0_f64.to_bits(), "F", "C"),
    ];
    let mut probes = Vec::new();
    let mut rows = Vec::new();
    for (number_bits, from, to) in wanted {
        let (index, row) = discovery
            .rows
            .iter()
            .enumerate()
            .find(|(_, row)| {
                row.number_bits == format!("0x{number_bits:016x}")
                    && row.from_unit == from
                    && row.to_unit == to
            })
            .unwrap_or_else(|| panic!("smoke row missing: {from}->{to}"));
        probes.push(discovery.probes[index].clone());
        rows.push(row.clone());
    }
    let batch = ProbeBatch {
        schema_version: "w109.convert.mixed_scalar_probe_batch.v1",
        function: "CONVERT",
        row_id: "convert-typed-smoke-20260809".to_string(),
        arg_encoding: json!({
            "number": "0x followed by exactly 16 IEEE-754 binary64 hex digits",
            "text": "verbatim non-hex JSON string",
            "plumbing": "Range.Value2 cells referenced by one shared formula; never formula literals"
        }),
        probes,
    };
    let meta = MetaDocument {
        schema_version: "w109.convert.model_predictions.v1".to_string(),
        function: "CONVERT".to_string(),
        selection_note: "Five-row mixed numeric/text live plumbing smoke extracted from the discovery batch.".to_string(),
        model_names: MODEL_NAMES.iter().map(|name| (*name).to_string()).collect(),
        rows,
    };
    let batch_path = root.join("batch-convert-smoke-20260809.json");
    let meta_path = root.join("batch-convert-smoke-20260809-meta.json");
    std::fs::write(&batch_path, serde_json::to_vec_pretty(&batch).unwrap()).unwrap();
    std::fs::write(&meta_path, serde_json::to_vec_pretty(&meta).unwrap()).unwrap();
    println!("wrote {}-probe typed smoke -> {}", batch.probes.len(), batch_path.display());
}

fn output_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../work/w109/G4-convert")
}

fn main() {
    let root = output_root();
    std::fs::create_dir_all(&root).unwrap();
    let mut discovery = SplitBuilder::new("discovery");
    let reveal_pairs = add_discovery(&mut discovery);
    let mut heldout = SplitBuilder::new("heldout");
    add_heldout(&mut heldout, &reveal_pairs);

    println!(
        "oracle-free direct reveal pairs selected: {} of {} directed non-identity direct pairs",
        reveal_pairs.len(),
        categories()
            .iter()
            .map(|category| {
                let count = direct_units_in(*category).count();
                count * (count - 1)
            })
            .sum::<usize>()
    );
    write_smoke(&root, &discovery);
    write_split(&root, discovery);
    write_split(&root, heldout);
}
