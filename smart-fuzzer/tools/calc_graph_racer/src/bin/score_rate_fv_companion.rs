//! Reconstruct RATE's one-step publication from paired live FV operands.
//!
//! This scorer consumes discovery answers only.  It tests whether RATE builds
//! its objective and finite difference from the worksheet-visible FV surface,
//! while enumerating subtraction, division, reciprocal, and update stores.

use oxfunc_core::excel_numeric::research as rx;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::BTreeSet;
use std::path::PathBuf;

const ROOT: &str = "../../work/w109/G6-rate";
const FREEZE_ID: &str = "w109-g6-05-rate-fv-companion-v1-20260809";
const RATE_ROWS: usize = 256;
const FV_ROWS: usize = 512;
const CW: u16 = rx::CW_PC64_RN;

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

#[derive(Deserialize)]
struct CompanionRecord {
    id: String,
    source_rate_id: String,
    evaluation: String,
    requested_fv_bits: String,
    guess_bits: String,
}

#[derive(Deserialize)]
struct CompanionMeta {
    freeze_id: String,
    answer_blind: bool,
    records: Vec<CompanionRecord>,
}

#[derive(Clone, Copy, Debug, Serialize)]
enum Stored {
    F64,
    X87,
}

impl Stored {
    const ALL: [Self; 2] = [Self::F64, Self::X87];

    fn tag(self) -> &'static str {
        match self {
            Self::F64 => "f64",
            Self::X87 => "x87dr",
        }
    }

    fn mul(self, a: f64, b: f64) -> f64 {
        match self {
            Self::F64 => a * b,
            Self::X87 => x87_mul(a, b),
        }
    }

    fn sub(self, a: f64, b: f64) -> f64 {
        match self {
            Self::F64 => a - b,
            Self::X87 => x87_sub(a, b),
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize)]
enum Direction {
    SurfaceMinusRequested,
    RequestedMinusSurface,
}

impl Direction {
    const ALL: [Self; 2] = [Self::SurfaceMinusRequested, Self::RequestedMinusSurface];

    fn tag(self) -> &'static str {
        match self {
            Self::SurfaceMinusRequested => "fv-requested",
            Self::RequestedMinusSurface => "requested-fv",
        }
    }

    fn objective(self, surface: f64, requested: f64, op: Stored) -> f64 {
        match self {
            Self::SurfaceMinusRequested => op.sub(surface, requested),
            Self::RequestedMinusSurface => op.sub(requested, surface),
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize)]
enum DifferenceSource {
    Objective,
    Surface,
}

impl DifferenceSource {
    const ALL: [Self; 2] = [Self::Objective, Self::Surface];

    fn tag(self) -> &'static str {
        match self {
            Self::Objective => "f1-f0",
            Self::Surface => "fv1-fv0",
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize)]
enum Quotient {
    F64Divide,
    F64Reciprocal,
    X87DifferenceStoredDivide,
    X87DifferenceStoredReciprocal,
    F64DifferenceX87Divide,
    F64DifferenceX87Reciprocal,
    X87ContinuousDivide,
    X87ContinuousReciprocal,
}

impl Quotient {
    const ALL: [Self; 8] = [
        Self::F64Divide,
        Self::F64Reciprocal,
        Self::X87DifferenceStoredDivide,
        Self::X87DifferenceStoredReciprocal,
        Self::F64DifferenceX87Divide,
        Self::F64DifferenceX87Reciprocal,
        Self::X87ContinuousDivide,
        Self::X87ContinuousReciprocal,
    ];

    fn tag(self) -> &'static str {
        match self {
            Self::F64Divide => "diff/h-f64",
            Self::F64Reciprocal => "diff*(1/h)-f64",
            Self::X87DifferenceStoredDivide => "x87diff/h-x87dr",
            Self::X87DifferenceStoredReciprocal => "x87diff*(1/h)-x87dr",
            Self::F64DifferenceX87Divide => "f64diff/h-x87dr",
            Self::F64DifferenceX87Reciprocal => "f64diff*(1/h)-x87dr",
            Self::X87ContinuousDivide => "diff/h-x87cont",
            Self::X87ContinuousReciprocal => "diff*(1/h)-x87cont",
        }
    }

    fn eval(self, next: f64, current: f64, h: f64) -> f64 {
        match self {
            Self::F64Divide => (next - current) / h,
            Self::F64Reciprocal => (next - current) * (1.0 / h),
            Self::X87DifferenceStoredDivide => x87_div(x87_sub(next, current), h),
            Self::X87DifferenceStoredReciprocal => x87_mul(x87_sub(next, current), x87_div(1.0, h)),
            Self::F64DifferenceX87Divide => x87_div(next - current, h),
            Self::F64DifferenceX87Reciprocal => x87_mul(next - current, x87_div(1.0, h)),
            Self::X87ContinuousDivide => rx::ext_to_f64(
                &rx::ext_div(&rx::ext_sub(&e(next), &e(current), CW), &e(h), CW),
                CW,
            ),
            Self::X87ContinuousReciprocal => rx::ext_to_f64(
                &rx::ext_mul(
                    &rx::ext_sub(&e(next), &e(current), CW),
                    &rx::ext_div(&rx::ext_one(), &e(h), CW),
                    CW,
                ),
                CW,
            ),
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize)]
enum Update {
    F64Divide,
    F64Reciprocal,
    X87StoredDivide,
    X87StoredReciprocal,
    X87ContinuousDivide,
    X87ContinuousReciprocal,
}

impl Update {
    const ALL: [Self; 6] = [
        Self::F64Divide,
        Self::F64Reciprocal,
        Self::X87StoredDivide,
        Self::X87StoredReciprocal,
        Self::X87ContinuousDivide,
        Self::X87ContinuousReciprocal,
    ];

    fn tag(self) -> &'static str {
        match self {
            Self::F64Divide => "x-f/d-f64",
            Self::F64Reciprocal => "x-f*(1/d)-f64",
            Self::X87StoredDivide => "x-(f/d)-x87dr",
            Self::X87StoredReciprocal => "x-f*(1/d)-x87dr",
            Self::X87ContinuousDivide => "x-f/d-x87cont",
            Self::X87ContinuousReciprocal => "x-f*(1/d)-x87cont",
        }
    }

    fn eval(self, x: f64, f: f64, derivative: f64) -> f64 {
        match self {
            Self::F64Divide => x - f / derivative,
            Self::F64Reciprocal => x - f * (1.0 / derivative),
            Self::X87StoredDivide => x87_sub(x, x87_div(f, derivative)),
            Self::X87StoredReciprocal => x87_sub(x, x87_mul(f, x87_div(1.0, derivative))),
            Self::X87ContinuousDivide => rx::ext_to_f64(
                &rx::ext_sub(&e(x), &rx::ext_div(&e(f), &e(derivative), CW), CW),
                CW,
            ),
            Self::X87ContinuousReciprocal => rx::ext_to_f64(
                &rx::ext_sub(
                    &e(x),
                    &rx::ext_mul(&e(f), &rx::ext_div(&rx::ext_one(), &e(derivative), CW), CW),
                    CW,
                ),
                CW,
            ),
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize)]
struct Model {
    objective_op: Stored,
    direction: Direction,
    h_op: Stored,
    difference_source: DifferenceSource,
    difference_op: Stored,
    quotient: Quotient,
    update: Update,
}

impl Model {
    fn id(self) -> String {
        format!(
            "objective-{}-{}|h-{}|{}-{}|{}|{}",
            self.direction.tag(),
            self.objective_op.tag(),
            self.h_op.tag(),
            self.difference_source.tag(),
            self.difference_op.tag(),
            self.quotient.tag(),
            self.update.tag(),
        )
    }
}

#[derive(Clone, Serialize)]
struct Score {
    model: Model,
    model_id: String,
    exact: usize,
    within_1_ulp: usize,
    within_4_ulp: usize,
    within_16_ulp: usize,
    max_ulp: u128,
    sum_ulp: u128,
}

fn e(value: f64) -> rx::Ext80 {
    rx::ext_from_f64(value)
}

fn x87_sub(a: f64, b: f64) -> f64 {
    rx::ext_to_f64(&rx::ext_sub(&e(a), &e(b), CW), CW)
}

fn x87_mul(a: f64, b: f64) -> f64 {
    rx::ext_to_f64(&rx::ext_mul(&e(a), &e(b), CW), CW)
}

fn x87_div(a: f64, b: f64) -> f64 {
    rx::ext_to_f64(&rx::ext_div(&e(a), &e(b), CW), CW)
}

fn parse_hex(value: &str) -> f64 {
    f64::from_bits(u64::from_str_radix(value.strip_prefix("0x").unwrap(), 16).unwrap())
}

fn ordered(bits: u64) -> i128 {
    if bits >> 63 == 0 {
        (bits | (1_u64 << 63)) as i128
    } else {
        (!bits) as i128
    }
}

fn validate_capture(answers: &WitnessSet, function: &str, rows: usize) {
    assert_eq!(answers.function, function);
    assert_eq!(answers.witnesses.len(), rows);
    assert_eq!(
        answers.capture_provenance.schema_version,
        "w109-capture-provenance-v1"
    );
    let environment = &answers.capture_provenance.environment;
    assert_eq!(environment.excel_version, "16.0");
    assert_eq!(environment.excel_build, "20228");
    assert_eq!(environment.excel_bitness, "64-bit");
    assert_eq!(environment.workbook_compatibility, "2");
    assert_eq!(environment.excel_input_plumbing, "cell_value2_bulk");
    let cache = &answers.capture_provenance.oracle_cache;
    assert_eq!(cache.mode, "no_cache");
    assert_eq!((cache.hits, cache.misses), (0, 0));
    let runner = &answers.capture_provenance.runner;
    assert_eq!(runner.name, "Run-W109BulkBatch.ps1");
    assert_eq!(runner.version, "w109-bulk-batch-v2");
}

fn models() -> Vec<Model> {
    let mut models = Vec::new();
    for objective_op in Stored::ALL {
        for direction in Direction::ALL {
            for h_op in Stored::ALL {
                for difference_source in DifferenceSource::ALL {
                    for difference_op in Stored::ALL {
                        for quotient in Quotient::ALL {
                            for update in Update::ALL {
                                models.push(Model {
                                    objective_op,
                                    direction,
                                    h_op,
                                    difference_source,
                                    difference_op,
                                    quotient,
                                    update,
                                });
                            }
                        }
                    }
                }
            }
        }
    }
    models
}

fn main() {
    let root = PathBuf::from(ROOT);
    let rate_batch: Batch = serde_json::from_str(
        &std::fs::read_to_string(root.join("batch-rate-one-step-discovery-v2.json")).unwrap(),
    )
    .unwrap();
    let rate_answers: WitnessSet = serde_json::from_str(
        &std::fs::read_to_string(root.join("answers-rate-one-step-discovery-v2.json")).unwrap(),
    )
    .unwrap();
    let fv_batch: Batch = serde_json::from_str(
        &std::fs::read_to_string(root.join("batch-rate-fv-companion-discovery-v1.json")).unwrap(),
    )
    .unwrap();
    let fv_answers: WitnessSet = serde_json::from_str(
        &std::fs::read_to_string(root.join("answers-rate-fv-companion-discovery-v1.json")).unwrap(),
    )
    .unwrap();
    let meta: CompanionMeta = serde_json::from_str(
        &std::fs::read_to_string(root.join("meta-rate-fv-companion-discovery-v1.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(meta.freeze_id, FREEZE_ID);
    assert!(meta.answer_blind);
    assert_eq!(meta.records.len(), FV_ROWS);
    assert_eq!(
        (rate_batch.function.as_str(), fv_batch.function.as_str()),
        ("RATE", "FV")
    );
    validate_capture(&rate_answers, "RATE", RATE_ROWS);
    validate_capture(&fv_answers, "FV", FV_ROWS);
    assert_eq!(rate_batch.probes.len(), RATE_ROWS);
    assert_eq!(fv_batch.probes.len(), FV_ROWS);
    for (batch, answer) in fv_batch.probes.iter().zip(&fv_answers.witnesses) {
        assert_eq!(batch.probe.id, answer.id);
        assert_eq!(batch.probe.args, answer.args);
        assert!(answer.expected_error.is_none());
        assert!(answer.expected_bits.is_some());
    }
    assert_eq!(
        fv_answers
            .witnesses
            .iter()
            .map(|witness| witness.id.as_str())
            .collect::<BTreeSet<_>>()
            .len(),
        FV_ROWS
    );

    let mut rows = Vec::new();
    let mut residual_under_tolerance = 0;
    let mut max_abs_surface_residual = 0.0_f64;
    for row in 0..RATE_ROWS {
        let rate_input = &rate_batch.probes[row].probe;
        let rate_answer = &rate_answers.witnesses[row];
        assert_eq!(rate_input.id, rate_answer.id);
        assert_eq!(rate_input.args, rate_answer.args);
        let guess_record = &meta.records[2 * row];
        let next_record = &meta.records[2 * row + 1];
        assert_eq!(guess_record.source_rate_id, rate_input.id);
        assert_eq!(next_record.source_rate_id, rate_input.id);
        assert_eq!(guess_record.evaluation, "guess");
        assert!(next_record.evaluation.starts_with("x-plus-h-"));
        assert_eq!(guess_record.id, fv_answers.witnesses[2 * row].id);
        assert_eq!(next_record.id, fv_answers.witnesses[2 * row + 1].id);
        assert_eq!(guess_record.guess_bits, rate_input.args[5]);
        assert_eq!(next_record.guess_bits, rate_input.args[5]);
        assert_eq!(guess_record.requested_fv_bits, rate_input.args[3]);
        assert_eq!(next_record.requested_fv_bits, rate_input.args[3]);
        let x = parse_hex(&rate_input.args[5]);
        let requested = parse_hex(&rate_input.args[3]);
        let fv0 = parse_hex(
            fv_answers.witnesses[2 * row]
                .expected_bits
                .as_deref()
                .unwrap(),
        );
        let fv1 = parse_hex(
            fv_answers.witnesses[2 * row + 1]
                .expected_bits
                .as_deref()
                .unwrap(),
        );
        let want = parse_hex(rate_answer.expected_bits.as_deref().unwrap());
        let residual = fv0 - requested;
        residual_under_tolerance += usize::from(residual.abs() < 1.0e-7);
        max_abs_surface_residual = max_abs_surface_residual.max(residual.abs());
        rows.push((x, requested, fv0, fv1, want));
    }

    let candidates = models();
    let mut scores = candidates
        .par_iter()
        .map(|&model| {
            let mut exact = 0;
            let mut within_1 = 0;
            let mut within_4 = 0;
            let mut within_16 = 0;
            let mut max_ulp = 0_u128;
            let mut sum_ulp = 0_u128;
            for &(x, requested, fv0, fv1, want) in &rows {
                let f0 = model
                    .direction
                    .objective(fv0, requested, model.objective_op);
                let f1 = model
                    .direction
                    .objective(fv1, requested, model.objective_op);
                let h = model.h_op.mul(1.0e-6, x);
                let (next, current) = match model.difference_source {
                    DifferenceSource::Objective => (f1, f0),
                    DifferenceSource::Surface => match model.direction {
                        Direction::SurfaceMinusRequested => (fv1, fv0),
                        Direction::RequestedMinusSurface => (fv0, fv1),
                    },
                };
                let difference = model.difference_op.sub(next, current);
                let derivative = model.quotient.eval(difference, 0.0, h);
                let got = model.update.eval(x, f0, derivative);
                assert!(got.is_finite());
                let distance = (ordered(got.to_bits()) - ordered(want.to_bits())).unsigned_abs();
                exact += usize::from(distance == 0);
                within_1 += usize::from(distance <= 1);
                within_4 += usize::from(distance <= 4);
                within_16 += usize::from(distance <= 16);
                max_ulp = max_ulp.max(distance);
                sum_ulp += distance;
            }
            Score {
                model,
                model_id: model.id(),
                exact,
                within_1_ulp: within_1,
                within_4_ulp: within_4,
                within_16_ulp: within_16,
                max_ulp,
                sum_ulp,
            }
        })
        .collect::<Vec<_>>();
    scores.sort_by(|a, b| {
        b.exact
            .cmp(&a.exact)
            .then_with(|| b.within_1_ulp.cmp(&a.within_1_ulp))
            .then_with(|| b.within_4_ulp.cmp(&a.within_4_ulp))
            .then_with(|| b.within_16_ulp.cmp(&a.within_16_ulp))
            .then_with(|| a.max_ulp.cmp(&b.max_ulp))
            .then_with(|| a.sum_ulp.cmp(&b.sum_ulp))
            .then_with(|| a.model_id.cmp(&b.model_id))
    });
    let exact_survivors = scores
        .iter()
        .filter(|score| score.exact == RATE_ROWS)
        .cloned()
        .collect::<Vec<_>>();
    let report = json!({
        "schema_version": "oxfunc.w109.rate_from_fv_companion_scores.v1",
        "freeze_id": FREEZE_ID,
        "scope_status": "discovery_only",
        "rate_rows": RATE_ROWS,
        "fv_rows": FV_ROWS,
        "fv_surface_residual": {
            "under_abs_1e_7": residual_under_tolerance,
            "max_abs": max_abs_surface_residual,
        },
        "candidate_count": scores.len(),
        "exact_survivor_count": exact_survivors.len(),
        "exact_survivors": exact_survivors,
        "top_scores": scores.iter().take(128).collect::<Vec<_>>(),
    });
    let mut bytes = serde_json::to_vec_pretty(&report).unwrap();
    bytes.push(b'\n');
    std::fs::write(
        root.join("report-rate-fv-companion-discovery-v1.json"),
        bytes,
    )
    .unwrap();
    println!(
        "rate_rows={RATE_ROWS} fv_rows={FV_ROWS} surface_residual_under_tol={residual_under_tolerance} max_abs_surface_residual={max_abs_surface_residual:e} candidates={} exact_survivors={}",
        scores.len(),
        exact_survivors.len(),
    );
    println!("exact <=1 <=4 <=16 max sum model");
    for score in scores.iter().take(20) {
        println!(
            "{:>5} {:>3} {:>3} {:>4} {:>8} {:>12} {}",
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
