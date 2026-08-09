//! Score the frozen G3-04 refinement bank and the prior paired discovery bank.
//!
//! Scoreboards remain layer-separated so an error in coefficient publication
//! cannot compensate for an error in GROWTH's prediction graph.  The v2 bank
//! is discovery/refinement data, not heldout evidence.

#[path = "growth_research/common.rs"]
mod common;
#[path = "growth_research/refinement_v2.rs"]
mod refinement_v2;

use common::{Arithmetic, LogCoefficients, LogProvider, PREDICTION_GRAPHS, predict, ulp_distance};
use refinement_v2::{EXP_PROVIDERS, Intercept, Linear, fit, intercept, kernels, predict_argument};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

const ROOT: &str = "../../work/w109/G3-04-growth";
const REPORT: &str = "score-refinement-v2.json";

#[derive(Deserialize)]
struct Bank {
    freeze_id: String,
    answer_blind: bool,
    datasets: Vec<Record>,
}

#[derive(Deserialize)]
struct Record {
    id: String,
    #[serde(default)]
    lane: String,
    #[serde(default)]
    family: String,
    #[serde(default)]
    metamer: String,
    use_const: bool,
    x_bits: Vec<String>,
    y_bits: Vec<String>,
    new_x_bits: Vec<String>,
}

struct Dataset {
    corpus: &'static str,
    id: String,
    lane: String,
    metamer: String,
    use_const: bool,
    x: Vec<f64>,
    y: Vec<f64>,
    new_x: Vec<f64>,
}

#[derive(Deserialize)]
struct AnswerSet {
    function: String,
    witnesses: Vec<Witness>,
    capture_provenance: Value,
}

#[derive(Deserialize)]
struct Witness {
    id: String,
    args: Value,
    expected_bits: String,
}

#[derive(Clone, Copy)]
enum Expected {
    Numeric(f64),
    Structural,
}

#[derive(Clone, Copy, Debug, Default, Serialize)]
struct Score {
    exact: usize,
    total: usize,
    matched_structural: usize,
    structural: usize,
    max_ulp: u64,
    sum_ulp: u128,
}

impl Score {
    fn record(&mut self, got: f64, expected: Expected) {
        self.total += 1;
        let Expected::Numeric(expected) = expected else {
            if got.is_finite() {
                self.structural += 1;
            } else {
                self.matched_structural += 1;
            }
            return;
        };
        let Some(distance) = ulp_distance(got, expected) else {
            self.structural += 1;
            return;
        };
        self.exact += usize::from(distance == 0);
        self.max_ulp = self.max_ulp.max(distance);
        self.sum_ulp = self.sum_ulp.saturating_add(distance as u128);
    }

    fn merge(&mut self, other: Self) {
        self.exact += other.exact;
        self.total += other.total;
        self.matched_structural += other.matched_structural;
        self.structural += other.structural;
        self.max_ulp = self.max_ulp.max(other.max_ulp);
        self.sum_ulp = self.sum_ulp.saturating_add(other.sum_ulp);
    }
}

#[derive(Clone, Serialize)]
struct RankedScore {
    id: String,
    score: Score,
}

#[derive(Clone, Copy)]
enum ObservedLinear {
    F64,
    X87Stored,
    Fma,
}

impl ObservedLinear {
    const ALL: [Self; 3] = [Self::F64, Self::X87Stored, Self::Fma];

    fn tag(self) -> &'static str {
        match self {
            Self::F64 => "a+b*x-f64",
            Self::X87Stored => "a+b*x-x87dr",
            Self::Fma => "fma(b,x,a)",
        }
    }

    fn eval(self, a: f64, b: f64, x: f64) -> f64 {
        match self {
            Self::F64 => a + b * x,
            Self::X87Stored => Arithmetic::X87Stored.add(a, Arithmetic::X87Stored.mul(b, x)),
            Self::Fma => b.mul_add(x, a),
        }
    }
}

fn parse_hex(text: &str) -> f64 {
    let digits = text.strip_prefix("0x").expect("0x-prefixed bits");
    f64::from_bits(u64::from_str_radix(digits, 16).expect("16-digit f64 bits"))
}

fn parse_expected(text: &str) -> Expected {
    if text.starts_with("0x") {
        Expected::Numeric(parse_hex(text))
    } else {
        assert!(
            text.starts_with("error:"),
            "unexpected oracle result {text}"
        );
        Expected::Structural
    }
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
    assert_eq!(at("/environment/excel_build"), "20228");
    assert_eq!(at("/environment/excel_bitness"), "64-bit");
    assert_eq!(at("/environment/workbook_compatibility"), "2");
    assert_eq!(
        at("/environment/excel_input_plumbing"),
        "cell_value2_matrix"
    );
    assert_eq!(at("/oracle_cache/mode"), "no_cache");
    assert_eq!(at("/runner/version"), "w109-bulk-batch-v2");
    assert_eq!(provenance["oracle_cache"]["hits"].as_u64(), Some(0));
    assert_eq!(provenance["oracle_cache"]["misses"].as_u64(), Some(0));
}

fn validate_answers(batch_path: &Path, answer_path: &Path, function: &str) -> AnswerSet {
    let batch: Value = load_json(batch_path);
    let answers: AnswerSet = load_json(answer_path);
    assert_eq!(batch["function"], function);
    assert_eq!(answers.function, function);
    let probes = batch["probes"].as_array().expect("batch probes");
    assert_eq!(probes.len(), answers.witnesses.len());
    let mut ids = BTreeSet::new();
    for (index, (ranked, witness)) in probes.iter().zip(&answers.witnesses).enumerate() {
        let probe = &ranked["probe"];
        assert_eq!(probe["id"], witness.id, "{function} id mismatch {index}");
        assert_eq!(
            probe["args"], witness.args,
            "{function} argument mismatch {index}"
        );
        assert!(ids.insert(&witness.id), "duplicate {function} ID");
        let _ = parse_expected(&witness.expected_bits);
    }
    validate_provenance(&answers.capture_provenance, function);
    answers
}

fn answer_map(set: AnswerSet) -> BTreeMap<String, Expected> {
    set.witnesses
        .into_iter()
        .map(|witness| (witness.id, parse_expected(&witness.expected_bits)))
        .collect()
}

fn load_bank(path: &Path, corpus: &'static str) -> Vec<Dataset> {
    let bank: Bank = load_json(path);
    assert!(bank.answer_blind);
    assert!(!bank.freeze_id.is_empty());
    bank.datasets
        .into_iter()
        .map(|record| Dataset {
            corpus,
            id: record.id,
            lane: if record.lane.is_empty() {
                format!("v1-{}", record.family)
            } else {
                record.lane
            },
            metamer: record.metamer,
            use_const: record.use_const,
            x: record.x_bits.iter().map(|value| parse_hex(value)).collect(),
            y: record.y_bits.iter().map(|value| parse_hex(value)).collect(),
            new_x: record
                .new_x_bits
                .iter()
                .map(|value| parse_hex(value))
                .collect(),
        })
        .collect()
}

fn ranking(ids: Vec<String>) -> Vec<RankedScore> {
    ids.into_iter()
        .map(|id| RankedScore {
            id,
            score: Score::default(),
        })
        .collect()
}

fn sorted(mut scores: Vec<RankedScore>) -> Vec<RankedScore> {
    scores.sort_by(|left, right| {
        left.score
            .structural
            .cmp(&right.score.structural)
            .then_with(|| right.score.exact.cmp(&left.score.exact))
            .then_with(|| left.score.max_ulp.cmp(&right.score.max_ulp))
            .then_with(|| left.score.sum_ulp.cmp(&right.score.sum_ulp))
            .then_with(|| left.id.cmp(&right.id))
    });
    scores
}

fn top(scores: &[RankedScore], count: usize) -> Vec<RankedScore> {
    scores.iter().take(count).cloned().collect()
}

fn print_top(label: &str, scores: &[RankedScore], count: usize) {
    println!("\n{label}");
    for (rank, item) in scores.iter().take(count).enumerate() {
        println!(
            "{:02} numeric_exact={}/{} structural_match={} structural_mismatch={} max_ulp={} sum_ulp={} {}",
            rank + 1,
            item.score.exact,
            item.score.total,
            item.score.matched_structural,
            item.score.structural,
            item.score.max_ulp,
            item.score.sum_ulp,
            item.id
        );
    }
}

fn merge_rankings(left: &[RankedScore], right: &[RankedScore]) -> Vec<RankedScore> {
    assert_eq!(left.len(), right.len());
    left.iter()
        .zip(right)
        .map(|(left, right)| {
            assert_eq!(left.id, right.id);
            let mut score = left.score;
            score.merge(right.score);
            RankedScore {
                id: left.id.clone(),
                score,
            }
        })
        .collect()
}

#[derive(Serialize)]
struct MetamerSummary {
    paired_lengths: usize,
    factor_equal: usize,
    base_equal: usize,
    prediction_comparable: usize,
    prediction_equal: usize,
    structural_pair_equal: usize,
}

fn same_expected(left: Expected, right: Expected) -> Option<bool> {
    match (left, right) {
        (Expected::Numeric(left), Expected::Numeric(right)) => {
            Some(left.to_bits() == right.to_bits())
        }
        (Expected::Structural, Expected::Structural) => Some(true),
        _ => None,
    }
}

fn unroll_metamers(
    datasets: &[Dataset],
    logest: &BTreeMap<String, Expected>,
    growth: &BTreeMap<String, Expected>,
) -> MetamerSummary {
    let mut result = MetamerSummary {
        paired_lengths: 0,
        factor_equal: 0,
        base_equal: 0,
        prediction_comparable: 0,
        prediction_equal: 0,
        structural_pair_equal: 0,
    };
    for original in datasets.iter().filter(|dataset| {
        dataset.corpus == "v2-refinement"
            && dataset.lane == "length-unroll"
            && dataset.metamer == "original"
    }) {
        let reversed_id = original.id.replace("-original", "-reversed");
        let reversed = datasets
            .iter()
            .find(|dataset| dataset.id == reversed_id)
            .expect("reversed unroll metamer");
        result.paired_lengths += 1;
        result.factor_equal += usize::from(
            same_expected(
                logest[&format!("{}-factor", original.id)],
                logest[&format!("{}-factor", reversed.id)],
            ) == Some(true),
        );
        result.base_equal += usize::from(
            same_expected(
                logest[&format!("{}-base", original.id)],
                logest[&format!("{}-base", reversed.id)],
            ) == Some(true),
        );
        for position in 0..original.new_x.len() {
            if original.new_x[position].to_bits() != reversed.new_x[position].to_bits() {
                continue;
            }
            result.prediction_comparable += 1;
            let left = growth[&format!("{}-pred-{position:02}", original.id)];
            let right = growth[&format!("{}-pred-{position:02}", reversed.id)];
            if same_expected(left, right) == Some(true) {
                result.prediction_equal += 1;
                result.structural_pair_equal += usize::from(matches!(
                    (left, right),
                    (Expected::Structural, Expected::Structural)
                ));
            }
        }
    }
    result
}

#[derive(Serialize)]
struct PublishedControlSummary {
    x_zero_occurrences: usize,
    x_zero_exact_logest_base: usize,
    x_zero_numeric_mismatch: usize,
    x_zero_both_structural: usize,
    x_zero_mixed_kind: usize,
    x_one_occurrences: usize,
    x_one_numeric_comparable: usize,
    x_one_base_times_factor_f64_exact: usize,
    x_one_base_times_factor_x87_exact: usize,
}

fn published_controls(
    datasets: &[Dataset],
    logest: &BTreeMap<String, Expected>,
    growth: &BTreeMap<String, Expected>,
) -> PublishedControlSummary {
    let mut result = PublishedControlSummary {
        x_zero_occurrences: 0,
        x_zero_exact_logest_base: 0,
        x_zero_numeric_mismatch: 0,
        x_zero_both_structural: 0,
        x_zero_mixed_kind: 0,
        x_one_occurrences: 0,
        x_one_numeric_comparable: 0,
        x_one_base_times_factor_f64_exact: 0,
        x_one_base_times_factor_x87_exact: 0,
    };
    for dataset in datasets
        .iter()
        .filter(|dataset| dataset.corpus == "v2-refinement")
    {
        let factor = logest[&format!("{}-factor", dataset.id)];
        let base = logest[&format!("{}-base", dataset.id)];
        for (position, &new_x) in dataset.new_x.iter().enumerate() {
            let prediction = growth[&format!("{}-pred-{position:02}", dataset.id)];
            if new_x == 0.0 {
                result.x_zero_occurrences += 1;
                match (base, prediction) {
                    (Expected::Numeric(base), Expected::Numeric(prediction)) => {
                        if base.to_bits() == prediction.to_bits() {
                            result.x_zero_exact_logest_base += 1;
                        } else {
                            result.x_zero_numeric_mismatch += 1;
                        }
                    }
                    (Expected::Structural, Expected::Structural) => {
                        result.x_zero_both_structural += 1;
                    }
                    _ => result.x_zero_mixed_kind += 1,
                }
            }
            if new_x == 1.0 {
                result.x_one_occurrences += 1;
                if let (
                    Expected::Numeric(factor),
                    Expected::Numeric(base),
                    Expected::Numeric(prediction),
                ) = (factor, base, prediction)
                {
                    result.x_one_numeric_comparable += 1;
                    result.x_one_base_times_factor_f64_exact +=
                        usize::from((base * factor).to_bits() == prediction.to_bits());
                    result.x_one_base_times_factor_x87_exact += usize::from(
                        Arithmetic::X87Stored.mul(base, factor).to_bits() == prediction.to_bits(),
                    );
                }
            }
        }
    }
    result
}

fn main() {
    let root = PathBuf::from(ROOT);
    let mut datasets = load_bank(&root.join("meta-paired-discovery-v1.json"), "v1-discovery");
    let v1_count = datasets.len();
    datasets.extend(load_bank(
        &root.join("meta-refinement-v2.json"),
        "v2-refinement",
    ));
    assert_eq!(v1_count, 100);
    assert_eq!(datasets.len(), 180);

    let mut logest = answer_map(validate_answers(
        &root.join("batch-logest-paired-discovery-v1.json"),
        &root.join("answers-logest-paired-discovery-v1.json"),
        "LOGEST",
    ));
    logest.extend(answer_map(validate_answers(
        &root.join("batch-logest-refinement-v2.json"),
        &root.join("answers-logest-refinement-v2.json"),
        "LOGEST",
    )));
    let mut growth = answer_map(validate_answers(
        &root.join("batch-growth-paired-discovery-v1.json"),
        &root.join("answers-growth-paired-discovery-v1.json"),
        "GROWTH",
    ));
    growth.extend(answer_map(validate_answers(
        &root.join("batch-growth-refinement-v2.json"),
        &root.join("answers-growth-refinement-v2.json"),
        "GROWTH",
    )));

    let all_kernels = kernels();
    let factor_ids = all_kernels
        .iter()
        .flat_map(|kernel| {
            EXP_PROVIDERS
                .iter()
                .map(move |exp| format!("{}|{}", kernel.id(), exp.tag()))
        })
        .collect::<Vec<_>>();
    let coefficient_ids = all_kernels
        .iter()
        .flat_map(|kernel| {
            Intercept::ALL.iter().flat_map(move |form| {
                EXP_PROVIDERS
                    .iter()
                    .map(move |exp| format!("{}|{}|{}", kernel.id(), form.tag(), exp.tag()))
            })
        })
        .collect::<Vec<_>>();
    let growth_ids = all_kernels
        .iter()
        .flat_map(|kernel| {
            Intercept::ALL.iter().flat_map(move |form| {
                Linear::ALL.iter().flat_map(move |linear| {
                    EXP_PROVIDERS.iter().map(move |exp| {
                        format!(
                            "{}|{}|{}|{}",
                            kernel.id(),
                            form.tag(),
                            linear.tag(),
                            exp.tag()
                        )
                    })
                })
            })
        })
        .collect::<Vec<_>>();
    let internal_published_ids = all_kernels
        .iter()
        .flat_map(|kernel| {
            Intercept::ALL.iter().flat_map(move |form| {
                EXP_PROVIDERS.iter().flat_map(move |exp| {
                    PREDICTION_GRAPHS[..6].iter().map(move |graph| {
                        format!(
                            "{}|{}|{}|{}",
                            kernel.id(),
                            form.tag(),
                            exp.tag(),
                            graph.tag()
                        )
                    })
                })
            })
        })
        .collect::<Vec<_>>();
    assert_eq!(factor_ids.len(), 324);
    assert_eq!(coefficient_ids.len(), 2_268);
    assert_eq!(growth_ids.len(), 13_608);
    assert_eq!(internal_published_ids.len(), 13_608);

    let published_graphs = &PREDICTION_GRAPHS[..6];
    let published_ids = published_graphs
        .iter()
        .map(|graph| graph.tag().to_owned())
        .collect::<Vec<_>>();
    let reconstructed_ids = [LogProvider::Platform, LogProvider::WorksheetX87]
        .iter()
        .flat_map(|log| {
            ObservedLinear::ALL.iter().flat_map(move |linear| {
                EXP_PROVIDERS
                    .iter()
                    .map(move |exp| format!("{}|{}|{}", log.tag(), linear.tag(), exp.tag()))
            })
        })
        .collect::<Vec<_>>();

    let mut corpus_factor = [ranking(factor_ids.clone()), ranking(factor_ids.clone())];
    let mut corpus_base = [
        ranking(coefficient_ids.clone()),
        ranking(coefficient_ids.clone()),
    ];
    let mut corpus_coefficients = [
        ranking(coefficient_ids.clone()),
        ranking(coefficient_ids.clone()),
    ];
    let mut corpus_growth = [ranking(growth_ids.clone()), ranking(growth_ids.clone())];
    let mut corpus_joint = [ranking(growth_ids.clone()), ranking(growth_ids.clone())];
    let mut corpus_internal_published = [
        ranking(internal_published_ids.clone()),
        ranking(internal_published_ids.clone()),
    ];
    let mut corpus_published = [
        ranking(published_ids.clone()),
        ranking(published_ids.clone()),
    ];
    let mut corpus_reconstructed = [
        ranking(reconstructed_ids.clone()),
        ranking(reconstructed_ids.clone()),
    ];
    let lane_names = [
        "ln-provider",
        "length-unroll",
        "coefficient-publication",
        "prediction-association",
    ];
    let mut lane_factor = lane_names.map(|_| ranking(factor_ids.clone()));
    let mut lane_base = lane_names.map(|_| ranking(coefficient_ids.clone()));
    let mut lane_coefficients = lane_names.map(|_| ranking(coefficient_ids.clone()));
    let mut lane_growth = lane_names.map(|_| ranking(growth_ids.clone()));
    let mut lane_joint = lane_names.map(|_| ranking(growth_ids.clone()));
    let mut lane_internal_published = lane_names.map(|_| ranking(internal_published_ids.clone()));
    let mut lane_published = lane_names.map(|_| ranking(published_ids.clone()));

    for dataset in &datasets {
        let corpus = usize::from(dataset.corpus == "v2-refinement");
        let lane = lane_names.iter().position(|name| *name == dataset.lane);
        let expected_factor = logest[&format!("{}-factor", dataset.id)];
        let expected_base = logest[&format!("{}-base", dataset.id)];

        let mut factor_index = 0;
        let mut coefficient_index = 0;
        let mut growth_index = 0;
        let mut internal_published_index = 0;
        for &kernel in &all_kernels {
            let state = fit(&dataset.x, &dataset.y, dataset.use_const, kernel);
            for &exp in &EXP_PROVIDERS {
                let predicted = exp.eval(state.slope);
                corpus_factor[corpus][factor_index]
                    .score
                    .record(predicted, expected_factor);
                if let Some(lane) = lane {
                    lane_factor[lane][factor_index]
                        .score
                        .record(predicted, expected_factor);
                }
                factor_index += 1;
            }
            for &form in &Intercept::ALL {
                let a = intercept(
                    &dataset.x,
                    &dataset.y,
                    dataset.use_const,
                    kernel,
                    state,
                    form,
                );
                for &exp in &EXP_PROVIDERS {
                    let factor = exp.eval(state.slope);
                    let base = exp.eval(a);
                    corpus_base[corpus][coefficient_index]
                        .score
                        .record(base, expected_base);
                    corpus_coefficients[corpus][coefficient_index]
                        .score
                        .record(factor, expected_factor);
                    corpus_coefficients[corpus][coefficient_index]
                        .score
                        .record(base, expected_base);
                    if let Some(lane) = lane {
                        lane_base[lane][coefficient_index]
                            .score
                            .record(base, expected_base);
                        lane_coefficients[lane][coefficient_index]
                            .score
                            .record(factor, expected_factor);
                        lane_coefficients[lane][coefficient_index]
                            .score
                            .record(base, expected_base);
                    }
                    coefficient_index += 1;
                }
                for &linear in &Linear::ALL {
                    for &exp in &EXP_PROVIDERS {
                        let factor = exp.eval(state.slope);
                        let base = exp.eval(a);
                        corpus_joint[corpus][growth_index]
                            .score
                            .record(factor, expected_factor);
                        corpus_joint[corpus][growth_index]
                            .score
                            .record(base, expected_base);
                        if let Some(lane) = lane {
                            lane_joint[lane][growth_index]
                                .score
                                .record(factor, expected_factor);
                            lane_joint[lane][growth_index]
                                .score
                                .record(base, expected_base);
                        }
                        for (position, &new_x) in dataset.new_x.iter().enumerate() {
                            let expected = growth[&format!("{}-pred-{position:02}", dataset.id)];
                            let argument =
                                predict_argument(state, a, new_x, dataset.use_const, linear);
                            let predicted = exp.eval(argument);
                            corpus_growth[corpus][growth_index]
                                .score
                                .record(predicted, expected);
                            corpus_joint[corpus][growth_index]
                                .score
                                .record(predicted, expected);
                            if let Some(lane) = lane {
                                lane_growth[lane][growth_index]
                                    .score
                                    .record(predicted, expected);
                                lane_joint[lane][growth_index]
                                    .score
                                    .record(predicted, expected);
                            }
                        }
                        growth_index += 1;
                    }
                }
                for &exp in &EXP_PROVIDERS {
                    let published = (exp.eval(state.slope), exp.eval(a));
                    for &graph in published_graphs {
                        for (position, &new_x) in dataset.new_x.iter().enumerate() {
                            let expected = growth[&format!("{}-pred-{position:02}", dataset.id)];
                            let predicted = predict(
                                LogCoefficients {
                                    slope: state.slope,
                                    intercept: a,
                                },
                                published,
                                new_x,
                                graph,
                            );
                            corpus_internal_published[corpus][internal_published_index]
                                .score
                                .record(predicted, expected);
                            if let Some(lane) = lane {
                                lane_internal_published[lane][internal_published_index]
                                    .score
                                    .record(predicted, expected);
                            }
                        }
                        internal_published_index += 1;
                    }
                }
            }
        }
        assert_eq!(factor_index, 324);
        assert_eq!(coefficient_index, 2_268);
        assert_eq!(growth_index, 13_608);
        assert_eq!(internal_published_index, 13_608);

        if let (Expected::Numeric(factor), Expected::Numeric(base)) =
            (expected_factor, expected_base)
        {
            for (index, &graph) in published_graphs.iter().enumerate() {
                for (position, &new_x) in dataset.new_x.iter().enumerate() {
                    let expected = growth[&format!("{}-pred-{position:02}", dataset.id)];
                    let predicted = predict(
                        LogCoefficients {
                            slope: 0.0,
                            intercept: 0.0,
                        },
                        (factor, base),
                        new_x,
                        graph,
                    );
                    corpus_published[corpus][index]
                        .score
                        .record(predicted, expected);
                    if let Some(lane) = lane {
                        lane_published[lane][index]
                            .score
                            .record(predicted, expected);
                    }
                }
            }
            let mut index = 0;
            for log in [LogProvider::Platform, LogProvider::WorksheetX87] {
                let b = log.eval(factor);
                let a = log.eval(base);
                for linear in ObservedLinear::ALL {
                    for exp in EXP_PROVIDERS {
                        for (position, &new_x) in dataset.new_x.iter().enumerate() {
                            let expected = growth[&format!("{}-pred-{position:02}", dataset.id)];
                            corpus_reconstructed[corpus][index]
                                .score
                                .record(exp.eval(linear.eval(a, b, new_x)), expected);
                        }
                        index += 1;
                    }
                }
            }
            assert_eq!(index, 18);
        } else {
            for score in &mut corpus_published[corpus] {
                for (position, _) in dataset.new_x.iter().enumerate() {
                    score.score.record(
                        f64::NAN,
                        growth[&format!("{}-pred-{position:02}", dataset.id)],
                    );
                }
            }
            if let Some(lane) = lane {
                for score in &mut lane_published[lane] {
                    for (position, _) in dataset.new_x.iter().enumerate() {
                        score.score.record(
                            f64::NAN,
                            growth[&format!("{}-pred-{position:02}", dataset.id)],
                        );
                    }
                }
            }
            for score in &mut corpus_reconstructed[corpus] {
                for (position, _) in dataset.new_x.iter().enumerate() {
                    score.score.record(
                        f64::NAN,
                        growth[&format!("{}-pred-{position:02}", dataset.id)],
                    );
                }
            }
        }
    }

    let combined_factor = sorted(merge_rankings(&corpus_factor[0], &corpus_factor[1]));
    let combined_base = sorted(merge_rankings(&corpus_base[0], &corpus_base[1]));
    let combined_coefficients = sorted(merge_rankings(
        &corpus_coefficients[0],
        &corpus_coefficients[1],
    ));
    let combined_growth = sorted(merge_rankings(&corpus_growth[0], &corpus_growth[1]));
    let combined_joint = sorted(merge_rankings(&corpus_joint[0], &corpus_joint[1]));
    let combined_internal_published = sorted(merge_rankings(
        &corpus_internal_published[0],
        &corpus_internal_published[1],
    ));
    let combined_published = sorted(merge_rankings(&corpus_published[0], &corpus_published[1]));
    let combined_reconstructed = sorted(merge_rankings(
        &corpus_reconstructed[0],
        &corpus_reconstructed[1],
    ));
    for scores in &mut corpus_factor {
        *scores = sorted(std::mem::take(scores));
    }
    for scores in &mut corpus_base {
        *scores = sorted(std::mem::take(scores));
    }
    for scores in &mut corpus_coefficients {
        *scores = sorted(std::mem::take(scores));
    }
    for scores in &mut corpus_growth {
        *scores = sorted(std::mem::take(scores));
    }
    for scores in &mut corpus_joint {
        *scores = sorted(std::mem::take(scores));
    }
    for scores in &mut corpus_internal_published {
        *scores = sorted(std::mem::take(scores));
    }
    for scores in &mut corpus_published {
        *scores = sorted(std::mem::take(scores));
    }
    for scores in &mut corpus_reconstructed {
        *scores = sorted(std::mem::take(scores));
    }
    for scores in &mut lane_factor {
        *scores = sorted(std::mem::take(scores));
    }
    for scores in &mut lane_base {
        *scores = sorted(std::mem::take(scores));
    }
    for scores in &mut lane_coefficients {
        *scores = sorted(std::mem::take(scores));
    }
    for scores in &mut lane_growth {
        *scores = sorted(std::mem::take(scores));
    }
    for scores in &mut lane_joint {
        *scores = sorted(std::mem::take(scores));
    }
    for scores in &mut lane_internal_published {
        *scores = sorted(std::mem::take(scores));
    }
    for scores in &mut lane_published {
        *scores = sorted(std::mem::take(scores));
    }

    print_top("COMBINED LOGEST factor", &combined_factor, 12);
    print_top("COMBINED LOGEST base", &combined_base, 12);
    print_top(
        "COMBINED LOGEST paired coefficients",
        &combined_coefficients,
        12,
    );
    print_top("COMBINED GROWTH direct internal log", &combined_growth, 12);
    print_top(
        "COMBINED GROWTH internal coefficients then power/product",
        &combined_internal_published,
        12,
    );
    print_top("COMBINED joint LOGEST+GROWTH", &combined_joint, 12);
    print_top(
        "COMBINED GROWTH from observed published LOGEST",
        &combined_published,
        6,
    );
    print_top(
        "COMBINED GROWTH from reconstructed observed LOGEST logs",
        &combined_reconstructed,
        12,
    );
    print_top("V2 LOGEST paired coefficients", &corpus_coefficients[1], 8);
    print_top("V2 GROWTH direct internal log", &corpus_growth[1], 8);
    for (index, name) in lane_names.iter().enumerate() {
        print_top(
            &format!("V2 lane {name}: LOGEST factor"),
            &lane_factor[index],
            4,
        );
        print_top(
            &format!("V2 lane {name}: LOGEST paired coefficients"),
            &lane_coefficients[index],
            4,
        );
        print_top(
            &format!("V2 lane {name}: GROWTH direct internal log"),
            &lane_growth[index],
            4,
        );
        print_top(
            &format!("V2 lane {name}: GROWTH internal coefficients then power/product"),
            &lane_internal_published[index],
            4,
        );
        print_top(
            &format!("V2 lane {name}: GROWTH published controls"),
            &lane_published[index],
            3,
        );
    }

    let metamers = unroll_metamers(&datasets, &logest, &growth);
    let published_controls = published_controls(&datasets, &logest, &growth);
    println!(
        "\nunroll metamers: pairs={} factor_equal={} base_equal={} predictions_equal={}/{} structural_pair_equal={}",
        metamers.paired_lengths,
        metamers.factor_equal,
        metamers.base_equal,
        metamers.prediction_equal,
        metamers.prediction_comparable,
        metamers.structural_pair_equal
    );

    let lane_reports = lane_names
        .iter()
        .enumerate()
        .map(|(index, name)| {
            (
                *name,
                json!({
                    "factor": top(&lane_factor[index], 100),
                    "base": top(&lane_base[index], 100),
                    "paired_coefficients": top(&lane_coefficients[index], 100),
                    "growth": top(&lane_growth[index], 100),
                    "internal_coefficients_then_power_product": top(&lane_internal_published[index], 100),
                    "joint": top(&lane_joint[index], 100),
                    "published_prediction_controls": top(&lane_published[index], 6),
                }),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let report = json!({
        "schema_version": "oxfunc.w109.growth_refinement_score.v2",
        "status": "discovery_refinement_not_heldout",
        "status_axes": {
            "scope_completeness": "scope_partial",
            "target_completeness": "target_partial",
            "integration_completeness": "partial",
        },
        "no_exact_single_predictor_survivor": true,
        "coverage_boundary": {
            "single_predictor_numeric_datasets": 180,
            "const_true_datasets": 157,
            "const_false_datasets": 23,
            "row_counts": "3..18",
            "v2_status": "discovery/refinement; intentionally not heldout",
        },
        "open_lanes": [
            "exact single-predictor LOGEST coefficient kernel and length-dependent association schedule",
            "exact GROWTH coefficient publication under normal/subnormal/overflow regimes",
            "exact fractional-exponent POWER/product association and store graph",
            "multivariate GROWTH/LOGEST",
            "full const=false characterization",
            "omitted/default known_x and new_x",
            "row/column orientation and output shape",
            "coercion, errors, and publication ordering",
            "prior-disjoint heldout gate",
            "production, tests, formal alignment, and integration",
        ],
        "corpora": {
            "v1_discovery": {"datasets": 100, "logest_cells": 200, "growth_cells": 700},
            "v2_refinement": {"datasets": 80, "logest_cells": 160, "growth_cells": 560},
            "combined": {"datasets": 180, "logest_cells": 360, "growth_cells": 1260},
        },
        "candidate_counts": {
            "factor": 324,
            "coefficient": 2268,
            "growth_direct_internal_log": 13608,
            "growth_internal_coefficients_then_power_product": 13608,
            "published_prediction_controls": 6,
            "reconstructed_observed_prediction_controls": 18,
        },
        "combined_top": {
            "factor": top(&combined_factor, 200),
            "base": top(&combined_base, 200),
            "paired_coefficients": top(&combined_coefficients, 200),
            "growth": top(&combined_growth, 200),
            "internal_coefficients_then_power_product": top(&combined_internal_published, 200),
            "joint": top(&combined_joint, 200),
            "published_prediction_controls": top(&combined_published, 6),
            "reconstructed_observed_prediction_controls": top(&combined_reconstructed, 18),
        },
        "v1_top": {
            "factor": top(&corpus_factor[0], 100),
            "base": top(&corpus_base[0], 100),
            "paired_coefficients": top(&corpus_coefficients[0], 100),
            "growth": top(&corpus_growth[0], 100),
            "internal_coefficients_then_power_product": top(&corpus_internal_published[0], 100),
            "joint": top(&corpus_joint[0], 100),
            "published_prediction_controls": top(&corpus_published[0], 6),
            "reconstructed_observed_prediction_controls": top(&corpus_reconstructed[0], 18),
        },
        "v2_top": {
            "factor": top(&corpus_factor[1], 100),
            "base": top(&corpus_base[1], 100),
            "paired_coefficients": top(&corpus_coefficients[1], 100),
            "growth": top(&corpus_growth[1], 100),
            "internal_coefficients_then_power_product": top(&corpus_internal_published[1], 100),
            "joint": top(&corpus_joint[1], 100),
            "published_prediction_controls": top(&corpus_published[1], 6),
            "reconstructed_observed_prediction_controls": top(&corpus_reconstructed[1], 18),
        },
        "v2_lanes": lane_reports,
        "unroll_reversal_metamers": metamers,
        "published_integer_controls": published_controls,
        "structural_discriminator": {
            "observation": "20 GROWTH #NUM! cells are all matched by internal-coefficient POWER/product candidates; best direct-log candidates match only 2 and mismatch 18",
            "inference": "GROWTH exposes separately published coefficient/power intermediates in the translated/subnormal slice; direct exp(a+b*x) is not the complete graph",
            "clean_room_status": "behavioral inference from frozen public-interface oracle captures",
        },
    });
    let mut bytes = serde_json::to_vec_pretty(&report).unwrap();
    bytes.push(b'\n');
    std::fs::write(root.join(REPORT), bytes).unwrap();
    println!("wrote {}", root.join(REPORT).display());
}
