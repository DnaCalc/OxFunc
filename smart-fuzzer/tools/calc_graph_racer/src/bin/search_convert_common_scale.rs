//! Search the otherwise algebraically-cancelling category scale in CONVERT's
//! staged units-per-base table.
//!
//! A common scale is invisible in real arithmetic but observable in binary64
//! when conversion is evaluated as `(x / from_constant) * to_constant`.
//! Discovery identity probes show that this matters for Excel (notably `m`),
//! so this scorer enumerates decimal-power scales without fitting to the
//! held-out set.

#[path = "convert_research/common.rs"]
mod common;

use common::{MetaDocument, ordered_bits};
use oxfunc_core::excel_numeric::research as rx;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

const SCHEDULES: [&str; 6] = [
    "f64_div_mul",
    "f64_mul_div",
    "f64_ratio_mul",
    "x87_f64_div_mul_pc64",
    "x87_f64_mul_div_pc64",
    "x87_f64_ratio_mul_pc64",
];

#[derive(Deserialize)]
struct WitnessSet {
    function: String,
    witnesses: Vec<Witness>,
    #[serde(default)]
    capture_provenance: Value,
}

#[derive(Deserialize)]
struct Witness {
    id: String,
    expected_bits: String,
}

#[derive(Default, Clone, Serialize)]
struct CandidateScore {
    category: String,
    scale_exponent: i32,
    schedule: String,
    exact: usize,
    total: usize,
    max_abs_ulp: String,
    sum_abs_ulp: String,
    class_exact: BTreeMap<String, usize>,
    class_total: BTreeMap<String, usize>,
    pair_exact: BTreeMap<String, usize>,
    pair_total: BTreeMap<String, usize>,
}

#[derive(Serialize)]
struct Report {
    schema_version: &'static str,
    function: &'static str,
    fit_rows: usize,
    scored_numeric_rows: usize,
    best_by_category: BTreeMap<String, Vec<CandidateScore>>,
    all_candidates: Vec<CandidateScore>,
    fit_capture_provenance: Value,
    score_capture_provenance: Value,
}

struct Args {
    fit_meta: PathBuf,
    fit_answers: PathBuf,
    score_meta: PathBuf,
    score_answers: PathBuf,
    out: Option<PathBuf>,
}

fn args() -> Args {
    let mut fit_meta = None;
    let mut fit_answers = None;
    let mut score_meta = None;
    let mut score_answers = None;
    let mut out = None;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--fit-meta" => fit_meta = args.next().map(PathBuf::from),
            "--fit-answers" => fit_answers = args.next().map(PathBuf::from),
            "--score-meta" => score_meta = args.next().map(PathBuf::from),
            "--score-answers" => score_answers = args.next().map(PathBuf::from),
            "--out" => out = args.next().map(PathBuf::from),
            other => panic!("unknown argument {other}"),
        }
    }
    Args {
        fit_meta: fit_meta.expect("--fit-meta"),
        fit_answers: fit_answers.expect("--fit-answers"),
        score_meta: score_meta.expect("--score-meta"),
        score_answers: score_answers.expect("--score-answers"),
        out,
    }
}

fn bits(raw: &str) -> Option<u64> {
    let digits = raw.strip_prefix("0x")?;
    (digits.len() == 16)
        .then(|| u64::from_str_radix(digits, 16).ok())
        .flatten()
}

fn base(category: &str) -> &'static str {
    match category {
        "length" => "m",
        "mass" => "g",
        "time" => "sec",
        "pressure" => "Pa",
        "volume" => "l",
        _ => panic!("unknown category {category}"),
    }
}

fn answer_map(answers: &WitnessSet) -> BTreeMap<&str, &Witness> {
    answers
        .witnesses
        .iter()
        .map(|witness| (witness.id.as_str(), witness))
        .collect()
}

fn fit_effective_constants(
    metadata: &MetaDocument,
    answers: &WitnessSet,
) -> (BTreeMap<(String, String), f64>, BTreeSet<String>) {
    let by_id = answer_map(answers);
    let mut constants = BTreeMap::new();
    let mut fit_ids = BTreeSet::new();
    for row in &metadata.rows {
        if row.predictions.is_empty()
            || row.number_bits != "0x3ff0000000000000"
            || row.from_unit != base(&row.category)
        {
            continue;
        }
        let witness = by_id[row.id.as_str()];
        let Some(value_bits) = bits(&witness.expected_bits) else {
            continue;
        };
        constants.insert(
            (row.category.clone(), row.to_unit.clone()),
            f64::from_bits(value_bits),
        );
        fit_ids.insert(row.id.clone());
    }
    (constants, fit_ids)
}

fn predict(schedule: &str, number: f64, from: f64, to: f64) -> f64 {
    let cw = rx::CW_PC64_RN;
    match schedule {
        "f64_div_mul" => (number / from) * to,
        "f64_mul_div" => (number * to) / from,
        "f64_ratio_mul" => number * (to / from),
        "x87_f64_div_mul_pc64" => {
            let divided = rx::ext_div(&rx::ext_from_f64(number), &rx::ext_from_f64(from), cw);
            let result = rx::ext_mul(&divided, &rx::ext_from_f64(to), cw);
            rx::ext_to_f64(&result, cw)
        }
        "x87_f64_mul_div_pc64" => {
            let multiplied = rx::ext_mul(&rx::ext_from_f64(number), &rx::ext_from_f64(to), cw);
            let result = rx::ext_div(&multiplied, &rx::ext_from_f64(from), cw);
            rx::ext_to_f64(&result, cw)
        }
        "x87_f64_ratio_mul_pc64" => {
            let ratio = rx::ext_div(&rx::ext_from_f64(to), &rx::ext_from_f64(from), cw);
            let result = rx::ext_mul(&rx::ext_from_f64(number), &ratio, cw);
            rx::ext_to_f64(&result, cw)
        }
        _ => panic!("unknown schedule {schedule}"),
    }
}

fn main() {
    let args = args();
    let fit_meta: MetaDocument =
        serde_json::from_slice(&std::fs::read(&args.fit_meta).unwrap()).unwrap();
    let fit_answers: WitnessSet =
        serde_json::from_slice(&std::fs::read(&args.fit_answers).unwrap()).unwrap();
    let score_meta: MetaDocument =
        serde_json::from_slice(&std::fs::read(&args.score_meta).unwrap()).unwrap();
    let score_answers: WitnessSet =
        serde_json::from_slice(&std::fs::read(&args.score_answers).unwrap()).unwrap();
    assert_eq!(fit_answers.function, "CONVERT");
    assert_eq!(score_answers.function, "CONVERT");
    // Fit every supported effective-unit ratio from the base-unit row.  This
    // keeps prefix construction itself under test: Excel may store a rounded
    // effective constant rather than recomputing it from the public prefix.
    let (effective, fit_ids) = fit_effective_constants(&fit_meta, &fit_answers);
    let score_by_id = answer_map(&score_answers);
    let same = args.fit_meta == args.score_meta && args.fit_answers == args.score_answers;
    let categories = ["length", "mass", "time", "pressure", "volume"];
    let mut candidates = Vec::new();
    let mut numeric_rows = 0;

    for category in categories {
        for scale_exponent in -24..=24 {
            let scale = format!("1e{scale_exponent}").parse::<f64>().unwrap();
            for schedule in SCHEDULES {
                let mut score = CandidateScore {
                    category: category.to_string(),
                    scale_exponent,
                    schedule: schedule.to_string(),
                    ..CandidateScore::default()
                };
                for row in &score_meta.rows {
                    if row.category != category
                        || row.predictions.is_empty()
                        || (same && fit_ids.contains(&row.id))
                    {
                        continue;
                    }
                    let Some(actual) = bits(&score_by_id[row.id.as_str()].expected_bits) else {
                        continue;
                    };
                    let Some(from_effective) =
                        effective.get(&(category.to_string(), row.from_unit.clone()))
                    else {
                        continue;
                    };
                    let Some(to_effective) =
                        effective.get(&(category.to_string(), row.to_unit.clone()))
                    else {
                        continue;
                    };
                    // Units per hidden category base.  Each division is kept
                    // explicit because the candidate is about staged rounding.
                    let from_constant = *from_effective / scale;
                    let to_constant = *to_effective / scale;
                    let number = common::f64_from_hex(&row.number_bits).unwrap();
                    let predicted = predict(schedule, number, from_constant, to_constant);
                    let residual = ordered_bits(actual) - ordered_bits(predicted.to_bits());
                    score.total += 1;
                    *score.class_total.entry(row.class.clone()).or_default() += 1;
                    let pair = format!("{}->{}", row.from_unit, row.to_unit);
                    *score.pair_total.entry(pair.clone()).or_default() += 1;
                    if residual == 0 {
                        score.exact += 1;
                        *score.class_exact.entry(row.class.clone()).or_default() += 1;
                        *score.pair_exact.entry(pair).or_default() += 1;
                    } else {
                        let abs = residual.unsigned_abs();
                        let old_max = score.max_abs_ulp.parse::<u128>().unwrap_or(0);
                        if abs > old_max {
                            score.max_abs_ulp = abs.to_string();
                        }
                        let old_sum = score.sum_abs_ulp.parse::<u128>().unwrap_or(0);
                        score.sum_abs_ulp = old_sum.saturating_add(abs).to_string();
                    }
                }
                if score.max_abs_ulp.is_empty() {
                    score.max_abs_ulp = "0".to_string();
                }
                if score.sum_abs_ulp.is_empty() {
                    score.sum_abs_ulp = "0".to_string();
                }
                numeric_rows = numeric_rows.max(score.total);
                candidates.push(score);
            }
        }
    }

    let mut best = BTreeMap::new();
    for category in categories {
        let category_candidates: Vec<_> = candidates
            .iter()
            .filter(|candidate| candidate.category == category)
            .cloned()
            .collect();
        let best_exact = category_candidates
            .iter()
            .map(|candidate| candidate.exact)
            .max()
            .unwrap();
        let mut survivors: Vec<_> = category_candidates
            .into_iter()
            .filter(|candidate| candidate.exact == best_exact)
            .collect();
        survivors.sort_by(|a, b| {
            a.sum_abs_ulp
                .parse::<u128>()
                .unwrap()
                .cmp(&b.sum_abs_ulp.parse::<u128>().unwrap())
        });
        println!("{category}: best {best_exact}/{}", survivors[0].total);
        for survivor in survivors.iter().take(8) {
            println!(
                "  scale=1e{:+} schedule={} sum_ulp={}",
                survivor.scale_exponent, survivor.schedule, survivor.sum_abs_ulp
            );
        }
        best.insert(category.to_string(), survivors);
    }

    let report = Report {
        schema_version: "w109.convert.common_scale_search.v2_effective_unit_fit",
        function: "CONVERT",
        fit_rows: fit_ids.len(),
        scored_numeric_rows: numeric_rows,
        best_by_category: best,
        all_candidates: candidates,
        fit_capture_provenance: fit_answers.capture_provenance,
        score_capture_provenance: score_answers.capture_provenance,
    };
    if let Some(path) = args.out {
        std::fs::write(&path, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
        println!("wrote scale search -> {}", path.display());
    }
}
