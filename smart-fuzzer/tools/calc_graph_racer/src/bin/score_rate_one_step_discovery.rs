//! Score the frozen RATE one-step graph grammar against discovery answers.
//!
//! This tool intentionally knows only the discovery batch.  It validates the
//! live-capture contract before evaluating candidates and emits a durable,
//! deterministic score report.  It never opens the sealed heldout artifacts.

#[path = "rate_research/one_step.rs"]
mod one_step;

use one_step::{Model, models, one_step};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

const ROOT: &str = "../../work/w109/G6-rate";
const FREEZE_ID: &str = "w109-g6-05-rate-one-step-v2-20260809";
const EXPECTED_ROWS: usize = 256;

#[derive(Deserialize)]
struct Probe {
    id: String,
    args: Vec<String>,
}

#[derive(Deserialize)]
struct RankedProbe {
    probe: Probe,
}

#[derive(Deserialize)]
struct Batch {
    function: String,
    row_id: String,
    probes: Vec<RankedProbe>,
}

#[derive(Deserialize)]
struct Witness {
    id: String,
    args: Vec<String>,
    expected_bits: Option<String>,
    expected_error: Option<String>,
}

#[derive(Deserialize)]
struct Environment {
    excel_version: String,
    excel_build: String,
    excel_bitness: String,
    workbook_compatibility: String,
    excel_input_plumbing: String,
}

#[derive(Deserialize)]
struct OracleCache {
    mode: String,
    hits: u64,
    misses: u64,
}

#[derive(Deserialize)]
struct Runner {
    name: String,
    version: String,
}

#[derive(Deserialize)]
struct Provenance {
    schema_version: String,
    captured_utc: String,
    environment: Environment,
    oracle_cache: OracleCache,
    runner: Runner,
}

#[derive(Deserialize)]
struct WitnessSet {
    function: String,
    witnesses: Vec<Witness>,
    capture_provenance: Provenance,
}

#[derive(Clone, Serialize)]
struct Score {
    rank: usize,
    model: Model,
    model_id: String,
    exact: usize,
    within_1_ulp: usize,
    within_2_ulp: usize,
    within_4_ulp: usize,
    within_16_ulp: usize,
    max_ulp: u128,
    sum_ulp: u128,
    signed_negative: usize,
    signed_zero: usize,
    signed_positive: usize,
}

fn parse_hex(value: &str) -> f64 {
    let raw = value.strip_prefix("0x").expect("hex prefix");
    assert_eq!(raw.len(), 16, "expected one binary64 bit pattern");
    f64::from_bits(u64::from_str_radix(raw, 16).expect("binary64 bits"))
}

fn ordered(bits: u64) -> i128 {
    if bits >> 63 == 0 {
        (bits | (1_u64 << 63)) as i128
    } else {
        (!bits) as i128
    }
}

fn ulp_delta(got: f64, want: f64) -> i128 {
    ordered(got.to_bits()) - ordered(want.to_bits())
}

fn write_frozen(path: &Path, bytes: &[u8]) {
    if let Ok(existing) = std::fs::read(path) {
        assert_eq!(
            existing,
            bytes,
            "refusing to overwrite score report {} with different bytes",
            path.display()
        );
        println!("verified report {}", path.display());
        return;
    }
    std::fs::write(path, bytes).unwrap();
    println!("wrote report {}", path.display());
}

fn main() {
    let root = PathBuf::from(ROOT);
    let batch: Batch = serde_json::from_str(
        &std::fs::read_to_string(root.join("batch-rate-one-step-discovery-v2.json")).unwrap(),
    )
    .unwrap();
    let answers: WitnessSet = serde_json::from_str(
        &std::fs::read_to_string(root.join("answers-rate-one-step-discovery-v2.json")).unwrap(),
    )
    .unwrap();

    assert_eq!(batch.function, "RATE");
    assert_eq!(answers.function, "RATE");
    assert_eq!(batch.probes.len(), EXPECTED_ROWS);
    assert_eq!(answers.witnesses.len(), EXPECTED_ROWS);
    assert_eq!(
        answers
            .witnesses
            .iter()
            .map(|witness| witness.id.as_str())
            .collect::<BTreeSet<_>>()
            .len(),
        EXPECTED_ROWS
    );
    for (input, answer) in batch.probes.iter().zip(&answers.witnesses) {
        assert_eq!(input.probe.id, answer.id, "ordered ID mismatch");
        assert_eq!(input.probe.args, answer.args, "argument-bit mismatch");
        assert_eq!(input.probe.args.len(), 6);
        assert!(answer.expected_error.is_none(), "unexpected Excel error");
        assert!(answer.expected_bits.is_some(), "missing numeric answer");
    }

    let provenance = &answers.capture_provenance;
    assert_eq!(provenance.schema_version, "w109-capture-provenance-v1");
    assert_eq!(provenance.environment.excel_version, "16.0");
    assert_eq!(provenance.environment.excel_build, "20228");
    assert_eq!(provenance.environment.excel_bitness, "64-bit");
    assert_eq!(provenance.environment.workbook_compatibility, "2");
    assert_eq!(
        provenance.environment.excel_input_plumbing,
        "cell_value2_bulk"
    );
    assert_eq!(provenance.oracle_cache.mode, "no_cache");
    assert_eq!(provenance.oracle_cache.hits, 0);
    assert_eq!(provenance.oracle_cache.misses, 0);
    assert_eq!(provenance.runner.name, "Run-W109BulkBatch.ps1");
    assert_eq!(provenance.runner.version, "w109-bulk-batch-v2");

    let rows = batch
        .probes
        .iter()
        .zip(&answers.witnesses)
        .map(|(input, answer)| {
            let args: [f64; 6] = input
                .probe
                .args
                .iter()
                .map(|value| parse_hex(value))
                .collect::<Vec<_>>()
                .try_into()
                .unwrap();
            let want = parse_hex(answer.expected_bits.as_deref().unwrap());
            assert!(want.is_finite());
            (args, want)
        })
        .collect::<Vec<_>>();

    let candidates = models();
    assert_eq!(candidates.len(), 13_824);
    let mut scores = candidates
        .par_iter()
        .map(|&model| {
            let mut exact = 0;
            let mut within_1 = 0;
            let mut within_2 = 0;
            let mut within_4 = 0;
            let mut within_16 = 0;
            let mut max_ulp = 0_u128;
            let mut sum_ulp = 0_u128;
            let mut negative = 0;
            let mut zero = 0;
            let mut positive = 0;
            for &(args, want) in &rows {
                let got = one_step(args, model)
                    .expect("frozen model was valid during selection")
                    .published;
                let delta = ulp_delta(got, want);
                let distance = delta.unsigned_abs();
                exact += usize::from(distance == 0);
                within_1 += usize::from(distance <= 1);
                within_2 += usize::from(distance <= 2);
                within_4 += usize::from(distance <= 4);
                within_16 += usize::from(distance <= 16);
                max_ulp = max_ulp.max(distance);
                sum_ulp += distance;
                negative += usize::from(delta < 0);
                zero += usize::from(delta == 0);
                positive += usize::from(delta > 0);
            }
            Score {
                rank: 0,
                model,
                model_id: model.id(),
                exact,
                within_1_ulp: within_1,
                within_2_ulp: within_2,
                within_4_ulp: within_4,
                within_16_ulp: within_16,
                max_ulp,
                sum_ulp,
                signed_negative: negative,
                signed_zero: zero,
                signed_positive: positive,
            }
        })
        .collect::<Vec<_>>();
    scores.sort_by(|a, b| {
        b.exact
            .cmp(&a.exact)
            .then_with(|| b.within_1_ulp.cmp(&a.within_1_ulp))
            .then_with(|| b.within_2_ulp.cmp(&a.within_2_ulp))
            .then_with(|| b.within_4_ulp.cmp(&a.within_4_ulp))
            .then_with(|| b.within_16_ulp.cmp(&a.within_16_ulp))
            .then_with(|| a.max_ulp.cmp(&b.max_ulp))
            .then_with(|| a.sum_ulp.cmp(&b.sum_ulp))
            .then_with(|| a.model_id.cmp(&b.model_id))
    });
    for (index, score) in scores.iter_mut().enumerate() {
        score.rank = index + 1;
    }

    let exact_survivors = scores
        .iter()
        .filter(|score| score.exact == EXPECTED_ROWS)
        .cloned()
        .collect::<Vec<_>>();
    let mut by_within_1 = scores.clone();
    by_within_1.sort_by(|a, b| {
        b.within_1_ulp
            .cmp(&a.within_1_ulp)
            .then_with(|| b.exact.cmp(&a.exact))
            .then_with(|| b.within_4_ulp.cmp(&a.within_4_ulp))
            .then_with(|| a.max_ulp.cmp(&b.max_ulp))
            .then_with(|| a.sum_ulp.cmp(&b.sum_ulp))
            .then_with(|| a.model_id.cmp(&b.model_id))
    });
    let mut by_within_16 = scores.clone();
    by_within_16.sort_by(|a, b| {
        b.within_16_ulp
            .cmp(&a.within_16_ulp)
            .then_with(|| b.within_4_ulp.cmp(&a.within_4_ulp))
            .then_with(|| b.within_1_ulp.cmp(&a.within_1_ulp))
            .then_with(|| a.max_ulp.cmp(&b.max_ulp))
            .then_with(|| a.sum_ulp.cmp(&b.sum_ulp))
            .then_with(|| a.model_id.cmp(&b.model_id))
    });
    let mut by_sum = scores.clone();
    by_sum.sort_by(|a, b| {
        a.sum_ulp
            .cmp(&b.sum_ulp)
            .then_with(|| a.max_ulp.cmp(&b.max_ulp))
            .then_with(|| b.exact.cmp(&a.exact))
            .then_with(|| a.model_id.cmp(&b.model_id))
    });
    let report = json!({
        "schema_version": "oxfunc.w109.rate_one_step_discovery_scores.v2",
        "freeze_id": FREEZE_ID,
        "scope_status": "discovery_only",
        "batch_row_id": batch.row_id,
        "capture": {
            "captured_utc": provenance.captured_utc,
            "excel_version": provenance.environment.excel_version,
            "excel_build": provenance.environment.excel_build,
            "excel_bitness": provenance.environment.excel_bitness,
            "workbook_compatibility": provenance.environment.workbook_compatibility,
            "input_plumbing": provenance.environment.excel_input_plumbing,
            "cache_mode": provenance.oracle_cache.mode,
            "runner": provenance.runner.name,
            "runner_version": provenance.runner.version,
        },
        "alignment": {
            "function": "RATE",
            "row_count": EXPECTED_ROWS,
            "unique_ids": EXPECTED_ROWS,
            "ordered_id_argument_bit_mismatches": 0,
            "numeric_results": EXPECTED_ROWS,
            "error_results": 0,
        },
        "candidate_count": scores.len(),
        "exact_survivor_count": exact_survivors.len(),
        "exact_survivors": exact_survivors,
        "top_by_exact": scores.iter().take(64).collect::<Vec<_>>(),
        "top_by_within_1_ulp": by_within_1.iter().take(64).collect::<Vec<_>>(),
        "top_by_within_16_ulp": by_within_16.iter().take(64).collect::<Vec<_>>(),
        "top_by_sum_ulp": by_sum.iter().take(64).collect::<Vec<_>>(),
    });
    let mut bytes = serde_json::to_vec_pretty(&report).unwrap();
    bytes.push(b'\n');
    write_frozen(
        &root.join("report-rate-one-step-discovery-v2-classification.json"),
        &bytes,
    );

    println!(
        "rows={} candidates={} exact_survivors={}",
        EXPECTED_ROWS,
        scores.len(),
        exact_survivors.len()
    );
    println!("rank exact <=1 <=2 <=4 <=16 max_ulp sum_ulp model");
    for score in scores.iter().take(20) {
        println!(
            "{:>4} {:>5} {:>3} {:>3} {:>3} {:>4} {:>7} {:>10} {}",
            score.rank,
            score.exact,
            score.within_1_ulp,
            score.within_2_ulp,
            score.within_4_ulp,
            score.within_16_ulp,
            score.max_ulp,
            score.sum_ulp,
            score.model_id,
        );
    }
    for (label, score) in [
        ("best_exact", &scores[0]),
        ("best_within_1", &by_within_1[0]),
        ("best_within_16", &by_within_16[0]),
        ("best_sum", &by_sum[0]),
    ] {
        println!(
            "{label}: exact={} <=1={} <=4={} <=16={} max={} sum={} {}",
            score.exact,
            score.within_1_ulp,
            score.within_4_ulp,
            score.within_16_ulp,
            score.max_ulp,
            score.sum_ulp,
            score.model_id,
        );
    }
}
