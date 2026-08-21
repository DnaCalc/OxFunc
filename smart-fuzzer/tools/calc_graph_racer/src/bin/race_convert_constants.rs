//! Score captured CONVERT bits against the W109 constant/operation models.
//!
//! Usage:
//!   cargo run --release --bin race_convert_constants -- \
//!     --meta <batch-meta.json> --answers <answered.json> [--out <report.json>]

#[path = "convert_research/common.rs"]
mod common;

use common::{MODEL_NAMES, MetaDocument, ordered_bits};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

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
struct ResidualBuckets {
    le_minus_3: usize,
    minus_2: usize,
    minus_1: usize,
    zero: usize,
    plus_1: usize,
    plus_2: usize,
    ge_plus_3: usize,
}

impl ResidualBuckets {
    fn add(&mut self, residual: i128) {
        match residual {
            i128::MIN..=-3 => self.le_minus_3 += 1,
            -2 => self.minus_2 += 1,
            -1 => self.minus_1 += 1,
            0 => self.zero += 1,
            1 => self.plus_1 += 1,
            2 => self.plus_2 += 1,
            3..=i128::MAX => self.ge_plus_3 += 1,
        }
    }
}

#[derive(Default, Serialize)]
struct ModelScore {
    modeled_numeric: usize,
    exact: usize,
    mismatch: usize,
    structural: usize,
    max_abs_ulp: String,
    sum_abs_ulp: String,
    residuals: ResidualBuckets,
    class_exact: BTreeMap<String, usize>,
    class_total: BTreeMap<String, usize>,
    category_exact: BTreeMap<String, usize>,
    category_total: BTreeMap<String, usize>,
    pair_exact: BTreeMap<String, usize>,
    pair_total: BTreeMap<String, usize>,
    first_misses: Vec<String>,
}

#[derive(Serialize)]
struct ScoreReport {
    schema_version: &'static str,
    function: String,
    split: String,
    metadata_rows: usize,
    answer_rows: usize,
    modeled_rows: usize,
    unmodeled_typed_controls: usize,
    numeric_answers: usize,
    nonnumeric_answers: usize,
    exact_survivors: Vec<String>,
    scores: BTreeMap<String, ModelScore>,
    prediction_equivalence_classes: Vec<Vec<String>>,
    capture_provenance: Value,
}

struct Args {
    meta: PathBuf,
    answers: PathBuf,
    out: Option<PathBuf>,
}

fn parse_args() -> Args {
    let mut meta = None;
    let mut answers = None;
    let mut out = None;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--meta" => meta = args.next().map(PathBuf::from),
            "--answers" => answers = args.next().map(PathBuf::from),
            "--out" => out = args.next().map(PathBuf::from),
            "-h" | "--help" => {
                println!(
                    "race_convert_constants --meta <meta.json> --answers <answers.json> [--out <report.json>]"
                );
                std::process::exit(0);
            }
            other => panic!("unknown argument {other}"),
        }
    }
    Args {
        meta: meta.expect("--meta is required"),
        answers: answers.expect("--answers is required"),
        out,
    }
}

fn parse_numeric_bits(raw: &str) -> Option<u64> {
    let digits = raw.strip_prefix("0x")?;
    if digits.len() != 16 {
        return None;
    }
    u64::from_str_radix(digits, 16).ok()
}

fn expected_args(meta: &common::MetaRow) -> Value {
    serde_json::json!([meta.number_bits, meta.from_unit, meta.to_unit])
}

fn main() {
    let args = parse_args();
    let metadata: MetaDocument =
        serde_json::from_slice(&std::fs::read(&args.meta).unwrap()).unwrap();
    let answers: WitnessSet =
        serde_json::from_slice(&std::fs::read(&args.answers).unwrap()).unwrap();
    assert_eq!(metadata.function, "CONVERT");
    assert_eq!(answers.function, "CONVERT");

    let expected_models: BTreeSet<_> = MODEL_NAMES.iter().copied().collect();
    let declared_models: BTreeSet<_> = metadata.model_names.iter().map(String::as_str).collect();
    assert_eq!(declared_models, expected_models, "metadata model set drift");

    let mut answer_by_id = BTreeMap::new();
    for witness in &answers.witnesses {
        assert!(
            answer_by_id.insert(witness.id.as_str(), witness).is_none(),
            "duplicate answer id {}",
            witness.id
        );
    }
    assert_eq!(
        metadata.rows.len(),
        answer_by_id.len(),
        "metadata/answer row count mismatch"
    );

    let split = metadata
        .rows
        .first()
        .map(|row| row.split.clone())
        .unwrap_or_default();
    let mut scores: BTreeMap<String, ModelScore> = MODEL_NAMES
        .iter()
        .map(|name| ((*name).to_string(), ModelScore::default()))
        .collect();
    let mut modeled_rows = 0;
    let mut controls = 0;
    let mut numeric_answers = 0;
    let mut nonnumeric_answers = 0;

    // Each model's full prediction signature supports a useful collapse audit:
    // if two candidates are identical over the entire batch, the batch cannot
    // distinguish them even if one scores perfectly.
    let mut signatures: BTreeMap<String, Vec<String>> = MODEL_NAMES
        .iter()
        .map(|name| ((*name).to_string(), Vec::new()))
        .collect();

    for row in &metadata.rows {
        let witness = answer_by_id
            .get(row.id.as_str())
            .unwrap_or_else(|| panic!("missing answer id {}", row.id));
        assert_eq!(
            witness.args,
            expected_args(row),
            "argument alignment mismatch for {}",
            row.id
        );
        if row.predictions.is_empty() {
            controls += 1;
            if parse_numeric_bits(&witness.expected_bits).is_some() {
                numeric_answers += 1;
            } else {
                nonnumeric_answers += 1;
            }
            continue;
        }
        modeled_rows += 1;
        let answer_bits = parse_numeric_bits(&witness.expected_bits);
        if answer_bits.is_some() {
            numeric_answers += 1;
        } else {
            nonnumeric_answers += 1;
        }
        for model_name in MODEL_NAMES {
            let predicted_raw = row
                .predictions
                .get(model_name)
                .unwrap_or_else(|| panic!("{} lacks prediction {model_name}", row.id));
            signatures
                .get_mut(model_name)
                .unwrap()
                .push(predicted_raw.clone());
            let score = scores.get_mut(model_name).unwrap();
            *score.class_total.entry(row.class.clone()).or_default() += 1;
            *score
                .category_total
                .entry(row.category.clone())
                .or_default() += 1;
            let pair = format!("{}->{}", row.from_unit, row.to_unit);
            *score.pair_total.entry(pair.clone()).or_default() += 1;
            let Some(answer_bits) = answer_bits else {
                score.structural += 1;
                if score.first_misses.len() < 12 {
                    score.first_misses.push(format!(
                        "{} {}({},{},{}) predicted={} oracle={}",
                        row.id,
                        row.class,
                        row.number_bits,
                        row.from_unit,
                        row.to_unit,
                        predicted_raw,
                        witness.expected_bits
                    ));
                }
                continue;
            };
            let predicted_bits = parse_numeric_bits(predicted_raw).unwrap();
            let residual = ordered_bits(answer_bits) - ordered_bits(predicted_bits);
            score.modeled_numeric += 1;
            score.residuals.add(residual);
            if residual == 0 {
                score.exact += 1;
                *score.class_exact.entry(row.class.clone()).or_default() += 1;
                *score
                    .category_exact
                    .entry(row.category.clone())
                    .or_default() += 1;
                *score.pair_exact.entry(pair).or_default() += 1;
            } else {
                score.mismatch += 1;
                let abs = residual.unsigned_abs();
                let old_max = score.max_abs_ulp.parse::<u128>().unwrap_or(0);
                if abs > old_max {
                    score.max_abs_ulp = abs.to_string();
                }
                let old_sum = score.sum_abs_ulp.parse::<u128>().unwrap_or(0);
                score.sum_abs_ulp = old_sum.saturating_add(abs).to_string();
                if score.first_misses.len() < 12 {
                    score.first_misses.push(format!(
                        "{} {}({},{},{}) residual={:+} predicted={} oracle=0x{:016x}",
                        row.id,
                        row.class,
                        row.number_bits,
                        row.from_unit,
                        row.to_unit,
                        residual,
                        predicted_raw,
                        answer_bits
                    ));
                }
            }
        }
    }

    for score in scores.values_mut() {
        if score.max_abs_ulp.is_empty() {
            score.max_abs_ulp = "0".to_string();
        }
        if score.sum_abs_ulp.is_empty() {
            score.sum_abs_ulp = "0".to_string();
        }
    }

    let mut signature_groups: BTreeMap<Vec<String>, Vec<String>> = BTreeMap::new();
    for (model, signature) in signatures {
        signature_groups.entry(signature).or_default().push(model);
    }
    let equivalence_classes: Vec<Vec<String>> = signature_groups.into_values().collect();
    let survivors = scores
        .iter()
        .filter(|(_, score)| score.mismatch == 0 && score.structural == 0)
        .map(|(name, _)| name.clone())
        .collect::<Vec<_>>();

    println!(
        "CONVERT {split}: {} modeled rows, {} typed controls, {} numeric / {} nonnumeric answers",
        modeled_rows, controls, numeric_answers, nonnumeric_answers
    );
    println!(
        "{:<48} {:>8} {:>8} {:>10}",
        "model", "exact", "total", "max_abs_ulp"
    );
    for model in MODEL_NAMES {
        let score = &scores[model];
        println!(
            "{:<48} {:>8} {:>8} {:>10}",
            model,
            score.exact,
            score.modeled_numeric + score.structural,
            score.max_abs_ulp
        );
    }
    if survivors.is_empty() {
        println!("exact survivors: none");
    } else {
        println!("exact survivors: {}", survivors.join(", "));
    }
    println!(
        "prediction equivalence classes: {}",
        equivalence_classes.len()
    );
    for group in &equivalence_classes {
        if group.len() > 1 {
            println!("  collapsed: {}", group.join(" == "));
        }
    }

    let report = ScoreReport {
        schema_version: "w109.convert.model_score.v1",
        function: "CONVERT".to_string(),
        split,
        metadata_rows: metadata.rows.len(),
        answer_rows: answers.witnesses.len(),
        modeled_rows,
        unmodeled_typed_controls: controls,
        numeric_answers,
        nonnumeric_answers,
        exact_survivors: survivors,
        scores,
        prediction_equivalence_classes: equivalence_classes,
        capture_provenance: answers.capture_provenance,
    };
    if let Some(path) = args.out {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(&path, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
        println!("wrote score report -> {}", path.display());
    }
}
