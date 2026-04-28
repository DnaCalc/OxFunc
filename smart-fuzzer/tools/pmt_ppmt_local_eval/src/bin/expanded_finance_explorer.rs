use oxfunc_core::functions::surface_dispatch::{
    eval_surface_value_call, FUNC_ID_IPMT, FUNC_ID_PMT, FUNC_ID_PPMT,
};
use oxfunc_core::resolver::{RefResolutionError, ReferenceResolver, ResolverCapabilities};
use oxfunc_core::value::{CallArgValue, EvalValue, ReferenceLike, WorksheetErrorCode};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::time::Instant;

const FUNCTIONS: [&str; 3] = ["FUNC.PMT", "FUNC.PPMT", "FUNC.IPMT"];

struct NoResolver;

impl ReferenceResolver for NoResolver {
    fn capabilities(&self) -> ResolverCapabilities {
        ResolverCapabilities::permissive_local()
    }

    fn resolve_reference(
        &self,
        reference: &ReferenceLike,
    ) -> Result<EvalValue, RefResolutionError> {
        Err(RefResolutionError::UnresolvedReference {
            target: reference.target.clone(),
        })
    }
}

#[derive(Clone)]
struct GeneratedCase {
    case_id: String,
    function_id: &'static str,
    function_name: &'static str,
    args: Vec<f64>,
    formula_text: String,
    coverage_buckets: Vec<String>,
}

#[derive(Clone, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum Outcome {
    Number {
        value: f64,
        bits_hex: String,
        digest_payload: String,
    },
    Error {
        code: String,
        digest_payload: String,
    },
    Other {
        summary: String,
        digest_payload: String,
    },
}

#[derive(Serialize)]
struct CandidateRecord {
    schema_version: &'static str,
    case_id: String,
    function_id: String,
    function_name: String,
    generator_id: &'static str,
    formula_text: String,
    args: Vec<f64>,
    coverage_buckets: Vec<String>,
    local_outcome: Outcome,
    selection_reason: String,
}

struct Lcg {
    state: u64,
}

impl Lcg {
    fn new(seed: u64) -> Self {
        Self { state: seed | 1 }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.state
    }

    fn next_f64(&mut self) -> f64 {
        let bits = self.next_u64() >> 11;
        (bits as f64) / ((1_u64 << 53) as f64)
    }

    fn choose_usize(&mut self, len: usize) -> usize {
        (self.next_u64() as usize) % len
    }

    fn bool(&mut self, true_per_1024: u64) -> bool {
        self.next_u64() % 1024 < true_per_1024
    }
}

fn parse_args() -> Result<(PathBuf, usize, u64, usize), String> {
    let args: Vec<String> = env::args().collect();
    let mut run_dir = None;
    let mut cases = 10_000_000_usize;
    let mut seed = 8804_u64;
    let mut candidate_limit = 640_usize;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--run-dir" => {
                i += 1;
                run_dir = args.get(i).map(PathBuf::from);
            }
            "--cases" => {
                i += 1;
                cases = args
                    .get(i)
                    .ok_or("missing --cases value")?
                    .parse()
                    .map_err(|_| "invalid --cases value")?;
            }
            "--seed" => {
                i += 1;
                seed = args
                    .get(i)
                    .ok_or("missing --seed value")?
                    .parse()
                    .map_err(|_| "invalid --seed value")?;
            }
            "--candidate-limit" => {
                i += 1;
                candidate_limit = args
                    .get(i)
                    .ok_or("missing --candidate-limit value")?
                    .parse()
                    .map_err(|_| "invalid --candidate-limit value")?;
            }
            _ => return Err(format!("unknown argument: {}", args[i])),
        }
        i += 1;
    }
    Ok((
        run_dir.ok_or("missing --run-dir <path>")?,
        cases,
        seed,
        candidate_limit,
    ))
}

fn bucketed_rate(rng: &mut Lcg) -> (f64, &'static str) {
    match rng.choose_usize(8) {
        0 => (0.0, "rate:zero"),
        1 => {
            let exp = -12.0 + rng.next_f64() * 6.0;
            (10_f64.powf(exp), "rate:tiny_positive")
        }
        2 | 3 => {
            let annual = 0.0001 + rng.next_f64() * 0.24;
            (annual / 12.0, "rate:monthly_positive")
        }
        4 => {
            let annual = 0.0001 + rng.next_f64() * 0.40;
            (annual / 4.0, "rate:quarterly_positive")
        }
        5 => (-rng.next_f64() * 0.02 / 12.0, "rate:negative_small"),
        6 => (0.05 + rng.next_f64() * 0.95, "rate:large_positive"),
        _ => {
            let annual = 0.05 + (rng.next_f64() - 0.5) * 2.0e-14;
            (annual / 12.0, "rate:known_witness_neighborhood")
        }
    }
}

fn bucketed_nper(rng: &mut Lcg) -> (f64, &'static str) {
    match rng.choose_usize(6) {
        0 => ((1 + rng.choose_usize(12)) as f64, "nper:short_1_12"),
        1 => ((13 + rng.choose_usize(108)) as f64, "nper:medium_13_120"),
        2 | 3 => ((121 + rng.choose_usize(360)) as f64, "nper:mortgage_121_480"),
        4 => ((481 + rng.choose_usize(1520)) as f64, "nper:long_481_2000"),
        _ => ((2001 + rng.choose_usize(7999)) as f64, "nper:very_long_2001_9999"),
    }
}

fn bucketed_pv(rng: &mut Lcg) -> (f64, &'static str) {
    let sign = if rng.bool(512) { 1.0 } else { -1.0 };
    match rng.choose_usize(6) {
        0 => (0.0, "pv:zero"),
        1 => (sign * (rng.next_f64() * 1_000.0), "pv:small"),
        2 | 3 => (sign * (1_000.0 + rng.next_f64() * 999_000.0), "pv:ordinary"),
        4 => (
            sign * (1_000_000.0 + rng.next_f64() * 999_000_000.0),
            "pv:large",
        ),
        _ => (
            sign * (1_000_000_000.0 + rng.next_f64() * 999_000_000_000.0),
            "pv:huge",
        ),
    }
}

fn bucketed_fv(rng: &mut Lcg) -> (f64, &'static str) {
    if rng.bool(768) {
        (0.0, "fv:zero")
    } else {
        let sign = if rng.bool(512) { 1.0 } else { -1.0 };
        (sign * rng.next_f64() * 1_000_000.0, "fv:nonzero")
    }
}

fn bucketed_period(rng: &mut Lcg, nper: f64) -> (f64, &'static str) {
    let n = nper.max(1.0) as usize;
    match rng.choose_usize(8) {
        0 => (0.0, "period:zero_invalid"),
        1 => ((n + 1 + rng.choose_usize(10)) as f64, "period:past_nper_invalid"),
        2 | 3 => (1.0, "period:first"),
        4 => (((n + 1) / 2).max(1) as f64, "period:middle"),
        5 => (n as f64, "period:last"),
        _ => ((1 + rng.choose_usize(n)) as f64, "period:interior_random"),
    }
}

fn fmt_num(value: f64) -> String {
    if value == 0.0 {
        "0".to_string()
    } else {
        format!("{value:.17}")
    }
}

fn build_formula(function_name: &str, args: &[f64]) -> String {
    let arg_text = args.iter().map(|v| fmt_num(*v)).collect::<Vec<_>>().join(",");
    format!("={function_name}({arg_text})")
}

fn generate_case(index: usize, rng: &mut Lcg) -> GeneratedCase {
    if index == 0 {
        return known_case(index, "PMT", vec![0.05 / 12.0, 360.0, 200000.0]);
    }
    if index == 1 {
        return known_case(index, "PPMT", vec![0.05 / 12.0, 1.0, 360.0, 200000.0]);
    }
    if index == 2 {
        return known_case(index, "IPMT", vec![0.05 / 12.0, 1.0, 360.0, 200000.0]);
    }

    let function_id = FUNCTIONS[rng.choose_usize(FUNCTIONS.len())];
    let function_name = function_id.trim_start_matches("FUNC.");
    let (rate, rate_bucket) = bucketed_rate(rng);
    let (nper, nper_bucket) = bucketed_nper(rng);
    let (pv, pv_bucket) = bucketed_pv(rng);
    let (fv, fv_bucket) = bucketed_fv(rng);
    let payment_type = if rng.bool(192) { 1.0 } else { 0.0 };
    let type_bucket = if payment_type == 1.0 { "type:beginning" } else { "type:end" };

    let mut coverage = vec![
        format!("function:{function_name}"),
        rate_bucket.to_string(),
        nper_bucket.to_string(),
        pv_bucket.to_string(),
        fv_bucket.to_string(),
        type_bucket.to_string(),
    ];

    let args = if function_id == FUNC_ID_PMT {
        if rng.bool(512) {
            coverage.push("arity:3".to_string());
            vec![rate, nper, pv]
        } else {
            coverage.push("arity:5".to_string());
            vec![rate, nper, pv, fv, payment_type]
        }
    } else {
        let (period, period_bucket) = bucketed_period(rng, nper);
        coverage.push(period_bucket.to_string());
        if rng.bool(512) {
            coverage.push("arity:4".to_string());
            vec![rate, period, nper, pv]
        } else {
            coverage.push("arity:6".to_string());
            vec![rate, period, nper, pv, fv, payment_type]
        }
    };

    GeneratedCase {
        case_id: format!("SFZ-EXP-{index:08}"),
        function_id,
        function_name,
        formula_text: build_formula(function_name, &args),
        args,
        coverage_buckets: coverage,
    }
}

fn known_case(index: usize, function_name: &'static str, args: Vec<f64>) -> GeneratedCase {
    let function_id = match function_name {
        "PMT" => FUNC_ID_PMT,
        "PPMT" => FUNC_ID_PPMT,
        "IPMT" => FUNC_ID_IPMT,
        _ => unreachable!(),
    };
    GeneratedCase {
        case_id: format!("SFZ-EXP-{index:08}"),
        function_id,
        function_name,
        formula_text: build_formula(function_name, &args),
        args,
        coverage_buckets: vec![
            format!("function:{function_name}"),
            "seed:known_financial_exactness".to_string(),
            "rate:known_witness_neighborhood".to_string(),
        ],
    }
}

fn evaluate(case: &GeneratedCase) -> Outcome {
    let args = case
        .args
        .iter()
        .copied()
        .map(|v| CallArgValue::Eval(EvalValue::Number(v)))
        .collect::<Vec<_>>();
    let resolver = NoResolver;
    match eval_surface_value_call(case.function_id, &args, &resolver, None, None, None, None) {
        Ok(EvalValue::Number(value)) => {
            let bits_hex = format!("0x{:016x}", value.to_bits());
            Outcome::Number {
                value,
                bits_hex: bits_hex.clone(),
                digest_payload: format!("number:{bits_hex}"),
            }
        }
        Ok(EvalValue::Error(code)) => error_outcome(code),
        Ok(other) => Outcome::Other {
            summary: format!("{other:?}"),
            digest_payload: format!("other:{other:?}"),
        },
        Err(code) => error_outcome(code),
    }
}

fn error_outcome(code: WorksheetErrorCode) -> Outcome {
    let code = format!("{code:?}");
    Outcome::Error {
        digest_payload: format!("error:{code}"),
        code,
    }
}

fn outcome_bucket(outcome: &Outcome) -> String {
    match outcome {
        Outcome::Number { .. } => "outcome:number".to_string(),
        Outcome::Error { code, .. } => format!("outcome:error:{code}"),
        Outcome::Other { summary, .. } => format!("outcome:other:{summary}"),
    }
}

fn increment(map: &mut BTreeMap<String, u64>, key: impl Into<String>) {
    *map.entry(key.into()).or_default() += 1;
}

fn candidate_key(case: &GeneratedCase, outcome: &Outcome) -> String {
    let mut buckets = case.coverage_buckets.clone();
    buckets.sort();
    let rate = buckets
        .iter()
        .find(|b| b.starts_with("rate:"))
        .cloned()
        .unwrap_or_else(|| "rate:unknown".to_string());
    let arity = buckets
        .iter()
        .find(|b| b.starts_with("arity:"))
        .cloned()
        .unwrap_or_else(|| "arity:unknown".to_string());
    let period = buckets
        .iter()
        .find(|b| b.starts_with("period:"))
        .cloned()
        .unwrap_or_else(|| "period:none".to_string());
    format!(
        "{}|{}|{}|{}|{}",
        case.function_id,
        rate,
        arity,
        period,
        outcome_bucket(outcome)
    )
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (run_dir, case_count, seed, candidate_limit) =
        parse_args().map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;
    fs::create_dir_all(run_dir.join("candidates"))?;
    fs::create_dir_all(run_dir.join("logs"))?;

    let started = Instant::now();
    let mut rng = Lcg::new(seed);
    let mut by_function = BTreeMap::new();
    let mut by_bucket = BTreeMap::new();
    let mut by_outcome = BTreeMap::new();
    let mut by_area = BTreeMap::new();
    let mut selected_keys = BTreeSet::new();
    let mut candidates = Vec::new();

    for index in 0..case_count {
        let case = generate_case(index, &mut rng);
        let outcome = evaluate(&case);
        increment(&mut by_function, case.function_id);
        increment(&mut by_outcome, outcome_bucket(&outcome));
        for bucket in &case.coverage_buckets {
            increment(&mut by_bucket, bucket);
        }

        let key = candidate_key(&case, &outcome);
        increment(&mut by_area, &key);
        let must_select = index < 3 || key.contains("outcome:error") || key.contains("rate:known_witness_neighborhood");
        if candidates.len() < candidate_limit && (must_select || selected_keys.insert(key.clone())) {
            selected_keys.insert(key.clone());
            candidates.push(CandidateRecord {
                schema_version: "oxfunc.smart_fuzzer.expanded_candidate.v0",
                case_id: case.case_id.clone(),
                function_id: case.function_id.to_string(),
                function_name: case.function_name.to_string(),
                generator_id: "expanded_finance_local_10m.v0",
                formula_text: case.formula_text.clone(),
                args: case.args.clone(),
                coverage_buckets: case.coverage_buckets.clone(),
                local_outcome: outcome.clone(),
                selection_reason: if index < 3 {
                    "known_reference_seed".to_string()
                } else if key.contains("outcome:error") {
                    "local_error_area_sample".to_string()
                } else {
                    "new_coverage_area_sample".to_string()
                },
            });
        }
    }

    let candidates_path = run_dir.join("candidates").join("excel_candidates.jsonl");
    let mut writer = BufWriter::new(File::create(&candidates_path)?);
    for candidate in &candidates {
        serde_json::to_writer(&mut writer, candidate)?;
        writer.write_all(b"\n")?;
    }
    writer.flush()?;

    let elapsed = started.elapsed().as_secs_f64();
    let mut top_area_keys = by_area.iter().collect::<Vec<_>>();
    top_area_keys.sort_by(|a, b| b.1.cmp(a.1).then_with(|| a.0.cmp(b.0)));
    top_area_keys.truncate(40);

    let local_rollup = serde_json::json!({
        "schema_version": "oxfunc.smart_fuzzer.expanded_local_rollup.v0",
        "run_kind": "expanded_finance_local_10m",
        "seed": seed,
        "generated": case_count,
        "local_evaluated": case_count,
        "candidate_count": candidates.len(),
        "local_wall_seconds": elapsed,
        "local_cases_per_second": (case_count as f64) / elapsed.max(f64::MIN_POSITIVE),
        "by_function": by_function,
        "by_outcome": by_outcome,
        "by_coverage_bucket": by_bucket,
        "area_key_count": by_area.len(),
        "top_area_keys": top_area_keys,
    });
    fs::write(
        run_dir.join("local_rollup.json"),
        serde_json::to_string_pretty(&local_rollup)?,
    )?;

    let roadmap = serde_json::json!({
        "schema_version": "oxfunc.smart_fuzzer.roadmap_trace.v0",
        "run_kind": "expanded_finance_local_10m",
        "explored_space": [
            "function: PMT, PPMT, IPMT",
            "arity: PMT 3/5; PPMT/IPMT 4/6",
            "rate: zero, tiny positive, monthly positive, quarterly positive, negative small, large positive, known witness neighborhood",
            "nper: short, medium, mortgage, long, very long",
            "pv: zero, small, ordinary, large, huge with both signs",
            "fv/type: omitted/default and explicit nonzero fv plus beginning/end payment timing",
            "period: invalid zero, invalid past nper, first, middle, last, interior random"
        ],
        "case_count": case_count,
        "candidate_count": candidates.len(),
        "known_expected_deviation": "PMT/PPMT/IPMT non-zero-rate exact Value2 drift remains expected pending further investigation",
    });
    fs::write(
        run_dir.join("roadmap_trace.json"),
        serde_json::to_string_pretty(&roadmap)?,
    )?;

    let mut md = String::new();
    md.push_str("# Expanded Finance Exploration Trace\n\n");
    md.push_str("Status: `generated_local_pending_or_with_excel_sample`\n\n");
    md.push_str("## Areas Explored\n\n");
    md.push_str("1. Functions: `PMT`, `PPMT`, `IPMT`.\n");
    md.push_str("2. Arities: PMT `3`/`5`, PPMT/IPMT `4`/`6`.\n");
    md.push_str("3. Rates: zero, tiny positive, ordinary monthly/quarterly, negative small, large positive, known witness neighborhood.\n");
    md.push_str("4. Horizons: short, medium, mortgage-like, long, very long.\n");
    md.push_str("5. Amounts: zero/small/ordinary/large/huge PV with both signs, zero and nonzero FV.\n");
    md.push_str("6. Timing and period lanes: end/beginning timing, invalid periods, first/middle/last/interior periods.\n\n");
    md.push_str("## Local Summary\n\n");
    md.push_str(&format!("- Generated/evaluated locally: `{case_count}`\n"));
    md.push_str(&format!("- Excel candidate samples selected: `{}`\n", candidates.len()));
    md.push_str(&format!("- Local throughput: `{:.2}` cases/sec\n", (case_count as f64) / elapsed.max(f64::MIN_POSITIVE)));
    md.push_str("- Known PMT/PPMT/IPMT non-zero-rate exactness drift is treated as expected reference behavior for this run.\n");
    fs::write(run_dir.join("roadmap_trace.md"), md)?;

    println!(
        "expanded finance explorer: generated {case_count}, selected {}, local cps {:.2}",
        candidates.len(),
        (case_count as f64) / elapsed.max(f64::MIN_POSITIVE)
    );
    Ok(())
}
