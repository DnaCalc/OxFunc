//! Freeze answer-blind RATE one-step discovery and sealed-heldout inputs.
//!
//! Every selected row has a first residual below the observed `1e-7` stop
//! threshold for every graph in the frozen grammar.  RATE therefore publishes
//! exactly one Newton update, making balance, forward-difference, and update
//! staging observable without a trajectory confounder.

#[path = "rate_research/one_step.rs"]
mod one_step;

use one_step::{
    Annuity, BalanceSchedule, DerivativeGraph, StoredOp, SumPair, TermAssoc, UpdateGraph, balance,
    canonical_balance, models, one_step,
};
use serde::Serialize;
use serde_json::json;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

const ROOT: &str = "../../work/w109/G6-rate";
const FREEZE_ID: &str = "w109-g6-05-rate-one-step-v2-20260809";
const TARGET_ROWS: usize = 256;
const MAX_ATTEMPTS: usize = 250_000;

#[derive(Serialize)]
struct Probe {
    id: String,
    args: [String; 6],
}

#[derive(Serialize)]
struct RankedProbe {
    probe: Probe,
    distinct_outputs: usize,
    prediction_span_ulp: i128,
}

#[derive(Serialize)]
struct Batch {
    function: &'static str,
    row_id: String,
    probes: Vec<RankedProbe>,
}

#[derive(Serialize)]
struct Record {
    id: String,
    split: String,
    args_bits: [String; 6],
    distinct_predictions: usize,
    prediction_span_ulp: i128,
    max_abs_first_residual: f64,
}

#[derive(Serialize)]
struct DatasetBank {
    schema_version: &'static str,
    freeze_id: &'static str,
    answer_blind: bool,
    split: String,
    seed: String,
    records: Vec<Record>,
}

fn xorshift(seed: &mut u64) -> u64 {
    *seed ^= *seed << 13;
    *seed ^= *seed >> 7;
    *seed ^= *seed << 17;
    *seed
}

fn unit(seed: &mut u64) -> f64 {
    let mantissa = xorshift(seed) >> 11;
    (mantissa as f64) * (1.0 / ((1_u64 << 53) as f64))
}

fn signed_scale(seed: &mut u64, min_exp: i32, max_exp: i32) -> f64 {
    let sign = if xorshift(seed) & 1 == 0 { 1.0 } else { -1.0 };
    let exponent = min_exp + (xorshift(seed) % ((max_exp - min_exp + 1) as u64)) as i32;
    sign * (0.5 + unit(seed)) * 2.0_f64.powi(exponent)
}

fn hex(value: f64) -> String {
    format!("0x{:016x}", value.to_bits())
}

fn ordered(bits: u64) -> i128 {
    if bits >> 63 == 0 {
        (bits | (1_u64 << 63)) as i128
    } else {
        (!bits) as i128
    }
}

fn write_frozen(path: &Path, bytes: &[u8]) {
    if let Ok(existing) = std::fs::read(path) {
        assert_eq!(
            existing,
            bytes,
            "refusing to overwrite frozen artifact {} with different bytes",
            path.display()
        );
        println!("verified frozen {}", path.display());
        return;
    }
    std::fs::create_dir_all(path.parent().expect("artifact parent")).unwrap();
    std::fs::write(path, bytes).unwrap();
    println!("wrote frozen {}", path.display());
}

fn pretty<T: Serialize>(value: &T) -> Vec<u8> {
    let mut bytes = serde_json::to_vec_pretty(value).unwrap();
    bytes.push(b'\n');
    bytes
}

fn generate(split: &str, initial_seed: u64) -> (Batch, DatasetBank) {
    let candidates = models();
    let mut seed = initial_seed;
    let mut probes = Vec::new();
    let mut records = Vec::new();
    let mut seen = BTreeSet::new();

    for attempt in 0..MAX_ATTEMPTS {
        if probes.len() == TARGET_ROWS {
            break;
        }

        let periods = match attempt % 10 {
            0 => 1.0,
            1 => 2.0,
            2 => 3.0,
            3 => 7.0,
            4 => 12.0,
            5 => 37.0,
            6 => 120.0,
            7 => 360.0,
            _ => 1.25 + unit(&mut seed) * 80.0,
        };
        let timing = if xorshift(&mut seed) & 1 == 0 {
            0.0
        } else {
            1.0
        };
        let guess = match attempt % 12 {
            0 => -0.80 + unit(&mut seed) * 0.10,
            1 => -0.50 + unit(&mut seed) * 0.15,
            2 => -0.20 + unit(&mut seed) * 0.15,
            3 => 2.0_f64.powi(-18 - (xorshift(&mut seed) % 22) as i32),
            4 => 0.005 + unit(&mut seed) * 0.015,
            5 => 0.04 + unit(&mut seed) * 0.04,
            6 => 0.09 + unit(&mut seed) * 0.03,
            7 => 0.25 + unit(&mut seed) * 0.25,
            8 => 0.75 + unit(&mut seed) * 0.75,
            _ => -0.85 + unit(&mut seed) * 2.35,
        };
        if guess == 0.0 || guess <= -1.0 || !guess.is_finite() {
            continue;
        }
        let payment = signed_scale(&mut seed, -4, 4);
        let present = signed_scale(&mut seed, -4, 4);

        let mut seed_args = [periods, payment, present, 0.0, timing, guess];
        let tail = balance(seed_args, guess, canonical_balance());
        if !tail.is_finite() || tail.abs() > 1.0e7 {
            continue;
        }
        let delta_exp = -38 + (xorshift(&mut seed) % 12) as i32;
        let delta_sign = if xorshift(&mut seed) & 1 == 0 {
            1.0
        } else {
            -1.0
        };
        let delta =
            delta_sign * (1.0 + (xorshift(&mut seed) % 31) as f64) * 2.0_f64.powi(delta_exp);
        seed_args[3] = -tail + delta;
        let args = seed_args;
        if !seen.insert(args.map(f64::to_bits)) {
            continue;
        }

        let mut predictions = BTreeSet::new();
        let mut max_abs_residual = 0.0_f64;
        let mut valid = true;
        for &model in &candidates {
            let Some(trace) = one_step(args, model) else {
                valid = false;
                break;
            };
            predictions.insert(trace.published.to_bits());
            max_abs_residual = max_abs_residual.max(trace.residual.abs());
        }
        if !valid || predictions.len() < 8 {
            continue;
        }
        let low = predictions.iter().map(|bits| ordered(*bits)).min().unwrap();
        let high = predictions.iter().map(|bits| ordered(*bits)).max().unwrap();
        let span = high - low;
        if span < 7 {
            continue;
        }

        let id = format!("rate-one-step-{split}-v2-{:04}", probes.len());
        let args_bits = args.map(hex);
        probes.push(RankedProbe {
            probe: Probe {
                id: id.clone(),
                args: args_bits.clone(),
            },
            distinct_outputs: predictions.len(),
            prediction_span_ulp: span,
        });
        records.push(Record {
            id,
            split: split.to_owned(),
            args_bits,
            distinct_predictions: predictions.len(),
            prediction_span_ulp: span,
            max_abs_first_residual: max_abs_residual,
        });
    }

    assert_eq!(
        probes.len(),
        TARGET_ROWS,
        "insufficient discriminators after {MAX_ATTEMPTS} attempts"
    );
    (
        Batch {
            function: "RATE",
            row_id: format!("G6-05-rate-one-step-{split}-v2-20260809"),
            probes,
        },
        DatasetBank {
            schema_version: "oxfunc.w109.rate_one_step_dataset_bank.v2",
            freeze_id: FREEZE_ID,
            answer_blind: true,
            split: split.to_owned(),
            seed: format!("0x{initial_seed:016x}"),
            records,
        },
    )
}

fn main() {
    let candidates = models();
    assert_eq!(candidates.len(), 13_824);
    let manifest = json!({
        "schema_version": "oxfunc.w109.rate_one_step_candidate_manifest.v2",
        "freeze_id": FREEZE_ID,
        "answer_blind": true,
        "clean_room": true,
        "known_fixed_nodes": {
            "power": "excel_pow_chain: binary64 base -> raw x87 LN/product/EXP -> binary64",
            "solver": "forward-difference Newton in rate space",
            "step": "h = 1e-6*x",
            "stop": "abs(first residual) < 1e-7; publish stepped iterate",
        },
        "candidate_space": {
            "balance_schedules": BalanceSchedule::ALL,
            "annuity_forms": Annuity::ALL,
            "term_associations": TermAssoc::ALL,
            "sum_pairings": SumPair::ALL,
            "h_store_ops": StoredOp::ALL,
            "next_input_store_ops": StoredOp::ALL,
            "derivative_graphs": DerivativeGraph::ALL,
            "update_graphs": UpdateGraph::ALL,
            "candidate_count": candidates.len(),
        },
        "selection": {
            "method": "candidate-output disagreement only; no oracle answers read",
            "rows_per_split": TARGET_ROWS,
            "minimum_distinct_predictions": 8,
            "minimum_prediction_span_ulp": 7,
            "all_candidate_first_residuals_below": 1.0e-7,
            "discovery_seed": "0x52415445d15c0a11",
            "sealed_heldout_seed": "0x524154454e11d0a7",
        },
        "scope": {
            "discovery": "one-step numeric RATE publication only",
            "heldout": "sealed until one exact coherent discovery survivor is frozen",
            "excluded": [
                "full multi-iteration trajectory closure",
                "omitted/default argument and coercion behavior",
                "error and domain boundary characterization",
                "cross-build and compatibility-version sweeps"
            ],
        },
    });
    let (discovery, discovery_meta) = generate("discovery", 0x5241_5445_d15c_0a11);
    let (heldout, heldout_meta) = generate("heldout", 0x5241_5445_4e11_d0a7);
    let root = PathBuf::from(ROOT);
    for (name, bytes) in [
        (
            "candidate-manifest-rate-one-step-v2.json",
            pretty(&manifest),
        ),
        (
            "meta-rate-one-step-discovery-v2.json",
            pretty(&discovery_meta),
        ),
        ("batch-rate-one-step-discovery-v2.json", pretty(&discovery)),
        ("meta-rate-one-step-heldout-v2.json", pretty(&heldout_meta)),
        ("batch-rate-one-step-heldout-v2.json", pretty(&heldout)),
    ] {
        write_frozen(&root.join(name), &bytes);
    }
    println!(
        "freeze_id={FREEZE_ID} candidate_graphs={} discovery_calls={} sealed_heldout_calls={}",
        candidates.len(),
        TARGET_ROWS,
        TARGET_ROWS
    );
}
