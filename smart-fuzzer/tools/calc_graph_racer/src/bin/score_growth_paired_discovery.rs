//! Score the frozen paired LOGEST/GROWTH discovery bank.
//!
//! Three scoreboards are kept separate:
//! 1. LOGEST coefficient publication;
//! 2. GROWTH publication from the *observed Excel LOGEST cells* (which tests
//!    the prediction graph without allowing a coefficient error to cancel);
//! 3. complete end-to-end candidate graphs.

#[path = "growth_research/common.rs"]
mod common;

use common::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

const ROOT: &str = "../../work/w109/G3-04-growth";
const META: &str = "meta-paired-discovery-v1.json";
const LOGEST_BATCH: &str = "batch-logest-paired-discovery-v1.json";
const GROWTH_BATCH: &str = "batch-growth-paired-discovery-v1.json";
const LOGEST_ANSWERS: &str = "answers-logest-paired-discovery-v1.json";
const GROWTH_ANSWERS: &str = "answers-growth-paired-discovery-v1.json";

#[derive(Deserialize)]
struct DatasetBank {
    freeze_id: String,
    answer_blind: bool,
    datasets: Vec<DatasetRecord>,
}

#[derive(Deserialize)]
struct DatasetRecord {
    id: String,
    use_const: bool,
    x_bits: Vec<String>,
    y_bits: Vec<String>,
    new_x_bits: Vec<String>,
}

struct Dataset {
    id: String,
    use_const: bool,
    x: Vec<f64>,
    y: Vec<f64>,
    new_x: Vec<f64>,
}

#[derive(Deserialize)]
struct AnswerSet {
    function: String,
    witnesses: Vec<AnswerWitness>,
    capture_provenance: Value,
}

#[derive(Deserialize)]
struct AnswerWitness {
    id: String,
    args: Value,
    expected_bits: String,
}

#[derive(Clone, Debug, Default, Serialize)]
struct Score {
    exact: usize,
    total: usize,
    max_ulp: u64,
    sum_ulp: u128,
    structural: usize,
}

impl Score {
    fn record(&mut self, got: f64, expected: f64) {
        self.total += 1;
        let Some(distance) = ulp_distance(got, expected) else {
            self.structural += 1;
            return;
        };
        self.exact += usize::from(distance == 0);
        self.max_ulp = self.max_ulp.max(distance);
        self.sum_ulp = self.sum_ulp.saturating_add(distance as u128);
    }
}

#[derive(Serialize)]
struct RankedScore {
    id: String,
    score: Score,
}

fn parse_hex(text: &str) -> f64 {
    let digits = text.strip_prefix("0x").expect("0x-prefixed bits");
    f64::from_bits(u64::from_str_radix(digits, 16).expect("16-digit f64 bits"))
}

fn load_json<T: for<'de> Deserialize<'de>>(path: &Path) -> T {
    serde_json::from_str(
        &std::fs::read_to_string(path)
            .unwrap_or_else(|error| panic!("read {}: {error}", path.display())),
    )
    .unwrap_or_else(|error| panic!("parse {}: {error}", path.display()))
}

fn validate_provenance(provenance: &Value, label: &str) {
    let at = |pointer: &str| {
        provenance
            .pointer(pointer)
            .and_then(Value::as_str)
            .unwrap_or_else(|| panic!("{label}: missing provenance {pointer}"))
    };
    assert_eq!(at("/schema_version"), "w109-capture-provenance-v1");
    assert_eq!(at("/environment/excel_version"), "16.0");
    assert_eq!(at("/environment/excel_bitness"), "64-bit");
    assert_eq!(at("/environment/workbook_compatibility"), "2");
    let plumbing = at("/environment/excel_input_plumbing");
    assert!(
        plumbing == "cell_value2_bulk"
            || plumbing == "cell_value2_matrix"
            || plumbing == "cell_reference_value2",
        "{label}: unexpected input plumbing {plumbing}"
    );
    assert_eq!(at("/oracle_cache/mode"), "no_cache");
    assert_eq!(at("/runner/version"), "w109-bulk-batch-v2");
    assert_eq!(
        provenance["oracle_cache"]["hits"].as_u64(),
        Some(0),
        "{label}: NoCache capture must have zero hits"
    );
    assert_eq!(
        provenance["oracle_cache"]["misses"].as_u64(),
        Some(0),
        "{label}: NoCache capture must have zero misses"
    );
}

fn validate_answers(batch_path: &Path, answer_path: &Path, function: &str) -> AnswerSet {
    let batch: Value = load_json(batch_path);
    let answers: AnswerSet = load_json(answer_path);
    assert_eq!(batch["function"], function);
    assert_eq!(answers.function, function);
    let probes = batch["probes"].as_array().expect("batch probes");
    assert_eq!(probes.len(), answers.witnesses.len());
    for (index, (ranked, witness)) in probes.iter().zip(&answers.witnesses).enumerate() {
        let probe = &ranked["probe"];
        assert_eq!(probe["id"], witness.id, "{function} id mismatch at {index}");
        assert_eq!(
            probe["args"], witness.args,
            "{function} argument mismatch at {index}"
        );
        assert!(
            witness.expected_bits.starts_with("0x"),
            "{function}/{} returned nonnumeric {}",
            witness.id,
            witness.expected_bits
        );
    }
    validate_provenance(&answers.capture_provenance, function);
    answers
}

fn answer_map(set: &AnswerSet) -> BTreeMap<String, f64> {
    set.witnesses
        .iter()
        .map(|witness| (witness.id.clone(), parse_hex(&witness.expected_bits)))
        .collect()
}

fn coefficient_models(dataset: &Dataset) -> Vec<(String, LogCoefficients, (f64, f64))> {
    let variants = regression_variants();
    let mut models =
        Vec::with_capacity(LOG_PROVIDERS.len() * variants.len() * COEFFICIENT_EXP_VARIANTS.len());
    for &ln in &LOG_PROVIDERS {
        let logged = dataset
            .y
            .iter()
            .copied()
            .map(|value| ln.eval(value))
            .collect::<Vec<_>>();
        for &regression in &variants {
            let coefficients = regress_model(&dataset.x, &logged, dataset.use_const, regression);
            for &exp in &COEFFICIENT_EXP_VARIANTS {
                models.push((
                    format!("{}|{}|{}", ln.tag(), regression.id(), exp.tag()),
                    coefficients,
                    publish_coefficients(coefficients, exp),
                ));
            }
        }
    }
    models
}

fn sorted(mut values: Vec<RankedScore>) -> Vec<RankedScore> {
    values.sort_by(|left, right| {
        left.score
            .structural
            .cmp(&right.score.structural)
            .then_with(|| right.score.exact.cmp(&left.score.exact))
            .then_with(|| left.score.max_ulp.cmp(&right.score.max_ulp))
            .then_with(|| left.score.sum_ulp.cmp(&right.score.sum_ulp))
            .then_with(|| left.id.cmp(&right.id))
    });
    values
}

fn print_top(label: &str, rankings: &[RankedScore], count: usize) {
    println!("\n{label}");
    for (rank, result) in rankings.iter().take(count).enumerate() {
        println!(
            "{:02} exact={}/{} structural={} max_ulp={} sum_ulp={} {}",
            rank + 1,
            result.score.exact,
            result.score.total,
            result.score.structural,
            result.score.max_ulp,
            result.score.sum_ulp,
            result.id
        );
    }
}

fn main() {
    let root = PathBuf::from(ROOT);
    let bank: DatasetBank = load_json(&root.join(META));
    assert_eq!(bank.freeze_id, "w109-g3-04-paired-discovery-v1-20260809");
    assert!(bank.answer_blind);
    assert_eq!(bank.datasets.len(), 100);
    let datasets = bank
        .datasets
        .into_iter()
        .map(|record| Dataset {
            id: record.id,
            use_const: record.use_const,
            x: record.x_bits.iter().map(|value| parse_hex(value)).collect(),
            y: record.y_bits.iter().map(|value| parse_hex(value)).collect(),
            new_x: record
                .new_x_bits
                .iter()
                .map(|value| parse_hex(value))
                .collect(),
        })
        .collect::<Vec<_>>();

    let logest = validate_answers(
        &root.join(LOGEST_BATCH),
        &root.join(LOGEST_ANSWERS),
        "LOGEST",
    );
    let growth = validate_answers(
        &root.join(GROWTH_BATCH),
        &root.join(GROWTH_ANSWERS),
        "GROWTH",
    );
    let logest = answer_map(&logest);
    let growth = answer_map(&growth);

    // Coefficient candidates have a stable order for every dataset.
    let first_models = coefficient_models(&datasets[0]);
    let mut coefficient_scores = first_models
        .iter()
        .map(|(id, _, _)| RankedScore {
            id: id.clone(),
            score: Score::default(),
        })
        .collect::<Vec<_>>();

    // End-to-end is the Cartesian product with the nine prediction graphs.
    let mut end_to_end_scores = first_models
        .iter()
        .flat_map(|(id, _, _)| {
            PREDICTION_GRAPHS.iter().map(move |graph| RankedScore {
                id: format!("{}|{}", id, graph.tag()),
                score: Score::default(),
            })
        })
        .collect::<Vec<_>>();
    assert_eq!(coefficient_scores.len(), 2_592);
    assert_eq!(end_to_end_scores.len(), 23_328);

    // Prediction-only scores use the actual observed Excel LOGEST cells.
    let published_graphs = &PREDICTION_GRAPHS[..6];
    let mut prediction_scores = published_graphs
        .iter()
        .map(|graph| RankedScore {
            id: graph.tag().to_owned(),
            score: Score::default(),
        })
        .collect::<Vec<_>>();

    for dataset in &datasets {
        let expected_factor = logest[&format!("{}-factor", dataset.id)];
        let expected_base = logest[&format!("{}-base", dataset.id)];
        let models = coefficient_models(dataset);
        assert_eq!(models.len(), coefficient_scores.len());
        for (index, (_, coefficients, published)) in models.iter().enumerate() {
            coefficient_scores[index]
                .score
                .record(published.0, expected_factor);
            coefficient_scores[index]
                .score
                .record(published.1, expected_base);
            for (position, &new_x) in dataset.new_x.iter().enumerate() {
                let expected = growth[&format!("{}-pred-{position:02}", dataset.id)];
                for (graph_index, &graph) in PREDICTION_GRAPHS.iter().enumerate() {
                    let score_index = index * PREDICTION_GRAPHS.len() + graph_index;
                    let got = predict(*coefficients, *published, new_x, graph);
                    end_to_end_scores[score_index].score.record(got, expected);
                }
            }
        }

        let observed = (expected_factor, expected_base);
        let dummy = LogCoefficients {
            slope: 0.0,
            intercept: 0.0,
        };
        for (position, &new_x) in dataset.new_x.iter().enumerate() {
            let expected = growth[&format!("{}-pred-{position:02}", dataset.id)];
            for (index, &graph) in published_graphs.iter().enumerate() {
                prediction_scores[index]
                    .score
                    .record(predict(dummy, observed, new_x, graph), expected);
            }
        }
    }

    let coefficient_scores = sorted(coefficient_scores);
    let prediction_scores = sorted(prediction_scores);
    let end_to_end_scores = sorted(end_to_end_scores);
    print_top("LOGEST coefficient publication", &coefficient_scores, 30);
    print_top(
        "GROWTH prediction from observed LOGEST coefficients",
        &prediction_scores,
        prediction_scores.len(),
    );
    print_top("end-to-end GROWTH", &end_to_end_scores, 30);

    let report = serde_json::json!({
        "schema_version": "oxfunc.w109.growth_paired_score.v1",
        "freeze_id": bank.freeze_id,
        "dataset_count": datasets.len(),
        "logest_cell_count": logest.len(),
        "growth_cell_count": growth.len(),
        "coefficient_rankings": coefficient_scores,
        "prediction_from_observed_coefficient_rankings": prediction_scores,
        "end_to_end_rankings": end_to_end_scores,
    });
    let mut bytes = serde_json::to_vec_pretty(&report).unwrap();
    bytes.push(b'\n');
    std::fs::write(root.join("score-paired-discovery-v1.json"), bytes).unwrap();
    println!(
        "\nwrote {}",
        root.join("score-paired-discovery-v1.json").display()
    );
}
