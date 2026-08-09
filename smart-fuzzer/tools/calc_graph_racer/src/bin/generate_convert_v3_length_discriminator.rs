//! Generate the frozen, oracle-blind CONVERT v3 length-staging discriminator.
//!
//! This is a refinement set, not a publication held-out.  It targets the sole
//! retired-v2 `nm -> Pm` residual and separates input-cell mutation, f64
//! mul/div core staging, ratio-first core staging, continuous PC64 staging,
//! effective-factor staging, and reciprocal prefix schedules.  Except for an
//! explicit replay of the retired kill, every tuple is disjoint from all three
//! prior CONVERT evidence sets.

#[path = "convert_research/common.rs"]
mod common;

use common::{MetaDocument, MetaRow};
use oxfunc_core::excel_numeric::research as rx;
use serde::Serialize;
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

const SPLIT: &str = "refinement-discriminator-v3";
const MODEL_NAMES: [&str; 8] = [
    "v2_f64_core_pc64_delta",
    "ratio_core_pc64_delta",
    "pc64_core_store_pc64_delta",
    "pc64_core_continuous_delta",
    "effective_factor_f64_mul_div",
    "v2_core_pc64_separate_prefixes",
    "v2_core_pc64_reciprocal_delta",
    "g15_input_v2_graph",
];

#[derive(Clone, Serialize)]
struct Probe {
    id: String,
    args: [String; 3],
}

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

#[derive(Clone)]
struct Resolved { direct: String, prefix_exponent: i32 }

struct Builder {
    sequence: usize,
    prior: BTreeSet<(u64, String, String)>,
    emitted: BTreeSet<(u64, String, String)>,
    probes: Vec<ProbeEnvelope>,
    rows: Vec<MetaRow>,
}

impl Builder {
    fn new(prior: BTreeSet<(u64, String, String)>) -> Self {
        Self { sequence: 0, prior, emitted: BTreeSet::new(), probes: Vec::new(), rows: Vec::new() }
    }

    fn contains(&self, number: f64, from: &str, to: &str) -> bool {
        let key = (number.to_bits(), from.to_string(), to.to_string());
        self.prior.contains(&key) || self.emitted.contains(&key)
    }

    fn push(&mut self, class: &str, number: f64, from: &str, to: &str, allow_prior: bool) -> bool {
        assert!(number.is_finite() && number != 0.0);
        let key = (number.to_bits(), from.to_string(), to.to_string());
        if self.emitted.contains(&key) || (!allow_prior && self.prior.contains(&key)) {
            return false;
        }
        self.emitted.insert(key);
        let predictions = predictions(number, from, to);
        let informative = {
            let mut values = predictions.values();
            let first = values.next().unwrap();
            values.any(|value| value != first)
        };
        let id = format!("conv-v3d-{:05}", self.sequence);
        self.sequence += 1;
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
            category: "length".to_string(),
            number_bits: format!("0x{:016x}", number.to_bits()),
            from_unit: from.to_string(),
            to_unit: to.to_string(),
            informative,
            predictions,
        });
        true
    }
}

fn prefix_exponent(prefix: &str) -> i32 {
    match prefix {
        "Y" => 24, "Z" => 21, "E" => 18, "P" => 15, "T" => 12,
        "G" => 9, "M" => 6, "k" => 3, "h" => 2, "da" => 1,
        "d" => -1, "c" => -2, "m" => -3, "u" => -6, "n" => -9,
        "p" => -12, "f" => -15, other => panic!("unknown prefix {other}"),
    }
}

fn resolve(unit: &str) -> Resolved {
    if common::direct_unit(unit).is_some() {
        return Resolved { direct: unit.to_string(), prefix_exponent: 0 };
    }
    for (prefix, _) in common::PREFIXES {
        if unit == format!("{prefix}m") {
            return Resolved { direct: "m".to_string(), prefix_exponent: prefix_exponent(prefix) };
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
        other => panic!("no length factor for {other}"),
    }
}

fn pow10(exponent: i32) -> f64 { format!("1e{exponent}").parse().unwrap() }

fn pc64_mul(left: &rx::Ext80, right: &rx::Ext80) -> rx::Ext80 {
    rx::ext_mul(left, right, rx::CW_PC64_RN)
}

fn finish_pc64(core: f64, delta: i32) -> f64 {
    rx::ext_to_f64(
        &pc64_mul(&rx::ext_from_f64(core), &rx::ext_from_f64(pow10(delta))),
        rx::CW_PC64_RN,
    )
}

fn g15(value: f64) -> f64 {
    format!("{value:.14e}").parse().unwrap()
}

fn predictions(number: f64, from_raw: &str, to_raw: &str) -> BTreeMap<String, String> {
    let from = resolve(from_raw);
    let to = resolve(to_raw);
    let ff = factor(&from.direct);
    let tf = factor(&to.direct);
    let delta = from.prefix_exponent - to.prefix_exponent;
    let f64_core = (number * ff) / tf;
    let ratio_core = number * (ff / tf);
    let cw = rx::CW_PC64_RN;
    let pc64_product = rx::ext_mul(&rx::ext_from_f64(number), &rx::ext_from_f64(ff), cw);
    let pc64_core = rx::ext_div(&pc64_product, &rx::ext_from_f64(tf), cw);
    let pc64_core_stored = rx::ext_to_f64(&pc64_core, cw);
    let continuous = rx::ext_to_f64(
        &rx::ext_mul(&pc64_core, &rx::ext_from_f64(pow10(delta)), cw),
        cw,
    );
    let effective_from = ff * pow10(from.prefix_exponent);
    let effective_to = tf * pow10(to.prefix_exponent);
    let effective = (number * effective_from) / effective_to;
    let separate_divided = rx::ext_div(
        &rx::ext_from_f64(f64_core),
        &rx::ext_from_f64(pow10(to.prefix_exponent)),
        cw,
    );
    let separate = rx::ext_to_f64(
        &rx::ext_mul(
            &separate_divided,
            &rx::ext_from_f64(pow10(from.prefix_exponent)),
            cw,
        ),
        cw,
    );
    let reciprocal = if delta >= 0 {
        finish_pc64(f64_core, delta)
    } else {
        let multiplier = 1.0 / pow10(-delta);
        rx::ext_to_f64(
            &rx::ext_mul(&rx::ext_from_f64(f64_core), &rx::ext_from_f64(multiplier), cw),
            cw,
        )
    };
    let g15_number = g15(number);
    let g15_core = (g15_number * ff) / tf;
    let values = [
        finish_pc64(f64_core, delta),
        finish_pc64(ratio_core, delta),
        finish_pc64(pc64_core_stored, delta),
        continuous,
        effective,
        separate,
        reciprocal,
        finish_pc64(g15_core, delta),
    ];
    MODEL_NAMES.iter().zip(values).map(|(name, value)| {
        ((*name).to_string(), format!("0x{:016x}", value.to_bits()))
    }).collect()
}

fn informative(number: f64, from: &str, to: &str) -> bool {
    let values = predictions(number, from, to);
    let mut iter = values.values();
    let first = iter.next().unwrap();
    iter.any(|value| value != first)
}

fn next_random(seed: &mut u64) -> u64 {
    *seed ^= *seed << 13;
    *seed ^= *seed >> 7;
    *seed ^= *seed << 17;
    *seed
}

fn candidate(seed: &mut u64, exponent: i32) -> f64 {
    assert!((-1022..=1023).contains(&exponent));
    let exponent_bits = u64::try_from(exponent + 1023).unwrap() << 52;
    let mantissa = (next_random(seed) >> 12) & ((1_u64 << 52) - 1);
    let sign = (next_random(seed) & 1) << 63;
    f64::from_bits(sign | exponent_bits | mantissa)
}

fn next_up(value: f64) -> f64 {
    if value >= 0.0 { f64::from_bits(value.to_bits() + 1) } else { f64::from_bits(value.to_bits() - 1) }
}

fn next_down(value: f64) -> f64 {
    if value > 0.0 { f64::from_bits(value.to_bits() - 1) } else { f64::from_bits(value.to_bits() + 1) }
}

fn prior_seen(root: &Path) -> BTreeSet<(u64, String, String)> {
    let mut seen = BTreeSet::new();
    for name in [
        "batch-convert-discovery-20260809-meta.json",
        "batch-convert-heldout-20260809-meta.json",
        "batch-convert-publication-heldout-v2-20260809-meta.json",
    ] {
        let document: MetaDocument = serde_json::from_slice(&std::fs::read(root.join(name)).unwrap()).unwrap();
        for row in document.rows {
            seen.insert((common::f64_from_hex(&row.number_bits).unwrap().to_bits(), row.from_unit, row.to_unit));
        }
    }
    seen
}

fn main() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../work/w109/G4-convert");
    let prior = prior_seen(&root);
    let prior_count = prior.len();
    let mut builder = Builder::new(prior);
    let mut seed = 0x5633_4449_5343_4f4e_u64;

    // Explicitly replay the one retired v2 kill, plus exact bit neighbors.
    let target = u64::from_str_radix("457bc2d00cc56eb2", 16).unwrap();
    for offset in -64_i64..=64 {
        let bits = if offset >= 0 { target + offset as u64 } else { target - offset.unsigned_abs() };
        builder.push("retired-v2-kill-adjacent", f64::from_bits(bits), "nm", "Pm", true);
        builder.push("retired-v2-kill-adjacent-negative", f64::from_bits(bits | (1_u64 << 63)), "nm", "Pm", false);
    }

    // Every same-base prefix pair realizing +/-24 decimal exponents.
    let delta24_pairs = [
        ("fm", "Gm"), ("pm", "Tm"), ("nm", "Pm"),
        ("um", "Em"), ("mm", "Zm"), ("m", "Ym"),
    ];
    let exponents: Vec<i32> = (-128..=160).step_by(4).collect();
    for (from, to) in delta24_pairs {
        for (left, right, direction) in [(from, to, "forward"), (to, from, "reverse")] {
            for exponent in &exponents {
                let mut accepted = 0;
                for _ in 0..10_000 {
                    let value = candidate(&mut seed, *exponent);
                    if builder.contains(value, left, right) || !informative(value, left, right) {
                        continue;
                    }
                    builder.push(&format!("delta24-{direction}-disagreement"), value, left, right, false);
                    accepted += 1;
                    if accepted == 2 { break; }
                }
                assert_eq!(accepted, 2, "could not fill {left}->{right} exponent {exponent}");
            }
        }
    }

    // Exact powers and adjacent binary64 values around the only observed
    // magnitude band, shared across the delta-24 family and nearby controls.
    let boundary_exponents = [29, 52, 53, 63, 64, 79, 80, 87, 88, 89, 90, 95, 96, 104, 127];
    let control_pairs = [
        ("nm", "Tm"), ("nm", "Em"), ("nm", "Zm"),
        ("pm", "Pm"), ("pm", "Em"), ("um", "Pm"),
        ("um", "Zm"), ("mm", "Pm"), ("mm", "Em"),
        ("fm", "Pm"), ("fm", "Em"), ("m", "Zm"),
    ];
    for (from, to) in delta24_pairs.into_iter().chain(control_pairs) {
        for exponent in boundary_exponents {
            let power = 2.0_f64.powi(exponent);
            for value in [next_down(power), power, next_up(power), -next_down(power), -power, -next_up(power)] {
                builder.push("power-boundary-control", value, from, to, false);
            }
        }
    }

    // Hold the exact retired-kill mantissa fixed while changing prefix pairs;
    // this separates a pair-specific path from magnitude/delta-wide guards.
    let target_value = f64::from_bits(target);
    for (from, to) in delta24_pairs.into_iter().chain(control_pairs) {
        for offset in -8_i64..=8 {
            let bits = if offset >= 0 { target + offset as u64 } else { target - offset.unsigned_abs() };
            for value in [f64::from_bits(bits), f64::from_bits(bits | (1_u64 << 63))] {
                builder.push("fixed-kill-mantissa-cross-pair", value, from, to, false);
            }
        }
        // Forward/reverse controls use the same exact source number.
        builder.push("fixed-kill-mantissa-reverse", target_value, to, from, false);
    }

    assert!(builder.rows.iter().any(|row| row.number_bits == "0x457bc2d00cc56eb2" && row.from_unit == "nm" && row.to_unit == "Pm"));
    let informative_count = builder.rows.iter().filter(|row| row.informative).count();
    let batch = ProbeBatch {
        schema_version: "w109.convert.typed_probe_batch.v3-discriminator",
        function: "CONVERT",
        row_id: "probe.id",
        arg_encoding: json!({
            "kind": "typed_vector",
            "types": ["f64_bits", "string", "string"],
            "f64_bits_format": "0x + exactly 16 hexadecimal digits",
            "string_policy": "literal UTF-16 cell text"
        }),
        probes: builder.probes,
    };
    let metadata = MetaDocument {
        schema_version: "w109.convert.v3_length_discriminator_meta.v1".to_string(),
        function: "CONVERT".to_string(),
        selection_note: format!(
            "oracle-blind deterministic length-staging discriminator; prior tuples={prior_count}; only explicit retired-kill replay may overlap"
        ),
        model_names: MODEL_NAMES.iter().map(|name| (*name).to_string()).collect(),
        rows: builder.rows,
    };
    let batch_path = root.join("batch-convert-v3-length-discriminator-20260809.json");
    let meta_path = root.join("batch-convert-v3-length-discriminator-20260809-meta.json");
    std::fs::write(&batch_path, serde_json::to_vec_pretty(&batch).unwrap()).unwrap();
    std::fs::write(&meta_path, serde_json::to_vec_pretty(&metadata).unwrap()).unwrap();
    println!(
        "v3 discriminator rows={} informative={} prior={} -> {}",
        metadata.rows.len(), informative_count, prior_count, batch_path.display()
    );
    println!("metadata -> {}", meta_path.display());
}
