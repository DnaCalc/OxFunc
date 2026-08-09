//! Fit CONVERT's directed pair multiplier from the exhaustive discovery
//! `x = 1` matrix, then score only disjoint non-unit inputs.
//!
//! This tests whether Excel stores (or precomputes) a directed conversion
//! multiplier, as opposed to reconstructing every result from two public
//! unit constants.  The fitted matrix is discovery-only; a model selected
//! here must later survive the pre-generated random held-out battery.

#[path = "convert_research/common.rs"]
mod common;

use common::{MetaDocument, ordered_bits};
use oxfunc_core::excel_numeric::research as rx;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

const MODELS: [&str; 5] = [
    "f64_pair_mul",
    "x87_f64_pair_mul_pc64",
    "x87_f64_pair_mul_pc53",
    "length_km_f64_divmul_else_pair_mul",
    "length_km_f64_divmul_else_x87_pair_mul",
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
    args: Value,
    expected_bits: String,
}

#[derive(Default, Serialize)]
struct ModelScore {
    exact: usize,
    total: usize,
    max_abs_ulp: String,
    sum_abs_ulp: String,
    category_exact: BTreeMap<String, usize>,
    category_total: BTreeMap<String, usize>,
    class_exact: BTreeMap<String, usize>,
    class_total: BTreeMap<String, usize>,
    pair_exact: BTreeMap<String, usize>,
    pair_total: BTreeMap<String, usize>,
    first_misses: Vec<String>,
}

#[derive(Serialize)]
struct Report {
    schema_version: &'static str,
    function: &'static str,
    fitted_pairs: usize,
    excluded_fit_rows: usize,
    unmodeled_or_structural_rows: usize,
    scores: BTreeMap<String, ModelScore>,
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
    let mut values = std::env::args().skip(1);
    while let Some(arg) = values.next() {
        match arg.as_str() {
            "--fit-meta" => fit_meta = values.next().map(PathBuf::from),
            "--fit-answers" => fit_answers = values.next().map(PathBuf::from),
            "--score-meta" => score_meta = values.next().map(PathBuf::from),
            "--score-answers" => score_answers = values.next().map(PathBuf::from),
            "--out" => out = values.next().map(PathBuf::from),
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

fn answers(set: &WitnessSet) -> BTreeMap<&str, &Witness> {
    set.witnesses
        .iter()
        .map(|witness| (witness.id.as_str(), witness))
        .collect()
}

fn expected_args(row: &common::MetaRow) -> Value {
    serde_json::json!([row.number_bits, row.from_unit, row.to_unit])
}

type PairKey = (String, String, String);

fn fit_pairs(
    metadata: &MetaDocument,
    answer_set: &WitnessSet,
) -> (BTreeMap<PairKey, f64>, BTreeSet<String>) {
    let by_id = answers(answer_set);
    let mut pairs = BTreeMap::new();
    let mut ids = BTreeSet::new();
    for row in &metadata.rows {
        if row.predictions.is_empty() || row.number_bits != "0x3ff0000000000000" {
            continue;
        }
        let witness = by_id[row.id.as_str()];
        assert_eq!(witness.args, expected_args(row));
        let Some(raw) = bits(&witness.expected_bits) else {
            continue;
        };
        let key = (
            row.category.clone(),
            row.from_unit.clone(),
            row.to_unit.clone(),
        );
        assert!(pairs.insert(key, f64::from_bits(raw)).is_none());
        ids.insert(row.id.clone());
    }
    (pairs, ids)
}

fn x87_mul(number: f64, multiplier: f64, cw: u16) -> f64 {
    rx::ext_to_f64(
        &rx::ext_mul(
            &rx::ext_from_f64(number),
            &rx::ext_from_f64(multiplier),
            cw,
        ),
        cw,
    )
}

fn length_km_divmul(number: f64, multiplier: f64) -> f64 {
    // Discovery's m identity and m->ft neighbor probes expose a hidden
    // units-per-kilometre staging candidate: C_from=1000 and
    // C_to=round(multiplier/0.001).  Apply it to every length pair here as a
    // falsifiable category-wide model, not as a pair-specific exception.
    let from = 1000.0;
    let to = multiplier / 0.001;
    (number / from) * to
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
    let (pairs, fit_ids) = fit_pairs(&fit_meta, &fit_answers);
    let score_by_id = answers(&score_answers);
    let same = args.fit_meta == args.score_meta && args.fit_answers == args.score_answers;
    let mut scores: BTreeMap<String, ModelScore> = MODELS
        .iter()
        .map(|name| ((*name).to_string(), ModelScore::default()))
        .collect();
    let mut excluded = 0;
    let mut unmodeled = 0;

    for row in &score_meta.rows {
        if row.predictions.is_empty() || (same && fit_ids.contains(&row.id)) {
            if same && fit_ids.contains(&row.id) {
                excluded += 1;
            } else {
                unmodeled += 1;
            }
            continue;
        }
        let witness = score_by_id[row.id.as_str()];
        assert_eq!(witness.args, expected_args(row));
        let Some(actual) = bits(&witness.expected_bits) else {
            unmodeled += 1;
            continue;
        };
        let key = (
            row.category.clone(),
            row.from_unit.clone(),
            row.to_unit.clone(),
        );
        let Some(multiplier) = pairs.get(&key).copied() else {
            unmodeled += 1;
            continue;
        };
        let number = common::f64_from_hex(&row.number_bits).unwrap();
        let f64_pair = number * multiplier;
        let x87_pair = x87_mul(number, multiplier, rx::CW_PC64_RN);
        let x87_pair_pc53 = x87_mul(number, multiplier, rx::CW_PC53_RN);
        let km = if row.category == "length" {
            length_km_divmul(number, multiplier)
        } else {
            f64_pair
        };
        let km_x87_else = if row.category == "length" {
            length_km_divmul(number, multiplier)
        } else {
            x87_pair
        };
        let predictions = [f64_pair, x87_pair, x87_pair_pc53, km, km_x87_else];

        for (name, predicted) in MODELS.iter().zip(predictions) {
            let score = scores.get_mut(*name).unwrap();
            score.total += 1;
            *score.category_total.entry(row.category.clone()).or_default() += 1;
            *score.class_total.entry(row.class.clone()).or_default() += 1;
            let pair = format!("{}->{}", row.from_unit, row.to_unit);
            *score.pair_total.entry(pair.clone()).or_default() += 1;
            let residual = ordered_bits(actual) - ordered_bits(predicted.to_bits());
            if residual == 0 {
                score.exact += 1;
                *score.category_exact.entry(row.category.clone()).or_default() += 1;
                *score.class_exact.entry(row.class.clone()).or_default() += 1;
                *score.pair_exact.entry(pair).or_default() += 1;
            } else {
                let abs = residual.unsigned_abs();
                let max = score.max_abs_ulp.parse::<u128>().unwrap_or(0).max(abs);
                let sum = score
                    .sum_abs_ulp
                    .parse::<u128>()
                    .unwrap_or(0)
                    .saturating_add(abs);
                score.max_abs_ulp = max.to_string();
                score.sum_abs_ulp = sum.to_string();
                if score.first_misses.len() < 10_000 {
                    score.first_misses.push(format!(
                        "{} {}({},{},{}) residual={:+} predicted=0x{:016x} oracle=0x{:016x}",
                        row.id,
                        row.class,
                        row.number_bits,
                        row.from_unit,
                        row.to_unit,
                        residual,
                        predicted.to_bits(),
                        actual
                    ));
                }
            }
        }
    }

    for (name, score) in &mut scores {
        if score.max_abs_ulp.is_empty() {
            score.max_abs_ulp = "0".to_string();
        }
        if score.sum_abs_ulp.is_empty() {
            score.sum_abs_ulp = "0".to_string();
        }
        println!(
            "{name}: {}/{} exact, max_ulp={}, sum_ulp={}",
            score.exact, score.total, score.max_abs_ulp, score.sum_abs_ulp
        );
        for category in ["length", "mass", "time", "pressure", "volume"] {
            if let Some(total) = score.category_total.get(category) {
                println!(
                    "  {category}: {}/{}",
                    score.category_exact.get(category).copied().unwrap_or(0),
                    total
                );
            }
        }
    }

    let report = Report {
        schema_version: "w109.convert.pair_multiplier_score.v1",
        function: "CONVERT",
        fitted_pairs: pairs.len(),
        excluded_fit_rows: excluded,
        unmodeled_or_structural_rows: unmodeled,
        scores,
        fit_capture_provenance: fit_answers.capture_provenance,
        score_capture_provenance: score_answers.capture_provenance,
    };
    if let Some(path) = args.out {
        std::fs::write(&path, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
        println!("wrote pair-multiplier score -> {}", path.display());
    }
}
