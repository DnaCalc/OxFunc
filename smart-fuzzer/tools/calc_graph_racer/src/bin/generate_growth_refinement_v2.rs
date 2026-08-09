//! Freeze the answer-blind G3-04 LOGEST/GROWTH refinement bank.
//!
//! The first paired discovery bank selected a centered, ordinary-f64
//! substrate but left the LN provider, length-dependent reduction schedule,
//! coefficient EXP publication, and GROWTH association graph open.  This
//! generator spends a fixed 80-dataset budget on those four orthogonal lanes.
//! Selection uses only disagreement among clean-room candidates; no oracle
//! answer is read.

#[path = "growth_research/common.rs"]
mod common;
#[path = "growth_research/refinement_v2.rs"]
mod refinement_v2;

use common::{LogProvider, PredictionGraph, hex};
use refinement_v2::{
    EXP_PROVIDERS, Intercept, Kernel, Linear, MeanFinish, MeanOrder, Reduction, fit, intercept,
    kernels, predict_argument,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

const FREEZE_ID: &str = "w109-g3-04-growth-refinement-v2-20260809";
const OUT_DIR: &str = "../../work/w109/G3-04-growth";
const DATASET_COUNT: usize = 80;

#[derive(Clone)]
struct Dataset {
    id: String,
    lane: String,
    metamer: String,
    use_const: bool,
    x: Vec<f64>,
    y: Vec<f64>,
    new_x: Vec<f64>,
    selection_score: usize,
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
struct Diversity {
    factor_distinct: usize,
    base_distinct: usize,
    prediction_distinct_sum: usize,
    prediction_distinct_min: usize,
    prediction_distinct_max: usize,
    kernel_slope_distinct: usize,
    provider_pair_differences: usize,
    exp_publication_differences: usize,
    linear_graph_distinct_sum: usize,
}

#[derive(Serialize, Deserialize)]
struct DatasetRecord {
    id: String,
    lane: String,
    metamer: String,
    use_const: bool,
    x_bits: Vec<String>,
    y_bits: Vec<String>,
    new_x_bits: Vec<String>,
    selection_score: usize,
    diversity: Diversity,
}

#[derive(Serialize)]
struct Probe {
    id: String,
    args: Vec<Value>,
    result_index: Vec<usize>,
}

#[derive(Serialize)]
struct RankedProbe {
    probe: Probe,
    distinct_outputs: usize,
    outputs: Vec<String>,
}

#[derive(Serialize)]
struct Batch {
    function: &'static str,
    row_id: &'static str,
    probes: Vec<RankedProbe>,
}

#[derive(Deserialize)]
struct PriorBank {
    datasets: Vec<PriorRecord>,
}

#[derive(Deserialize)]
struct PriorRecord {
    x_bits: Vec<String>,
    y_bits: Vec<String>,
}

struct Rng(u64);

impl Rng {
    fn next_u64(&mut self) -> u64 {
        let mut value = self.0;
        value ^= value >> 12;
        value ^= value << 25;
        value ^= value >> 27;
        self.0 = value;
        value.wrapping_mul(0x2545_f491_4f6c_dd1d)
    }

    fn unit(&mut self) -> f64 {
        ((self.next_u64() >> 11) as f64) * (1.0 / ((1_u64 << 53) as f64))
    }

    fn signed(&mut self) -> f64 {
        2.0 * self.unit() - 1.0
    }
}

fn bits(values: &[f64]) -> Vec<String> {
    values.iter().copied().map(hex).collect()
}

fn rotate<T>(values: &mut [T], amount: usize) {
    let amount = amount % values.len();
    values.rotate_left(amount);
}

fn reference_kernel(log: LogProvider, moments: Reduction) -> Kernel {
    Kernel {
        log,
        mean_order: MeanOrder::Forward,
        mean_finish: MeanFinish::Divide,
        moments,
    }
}

fn generic_prediction_points(x: &[f64], state_intercept: Option<(f64, f64)>) -> Vec<f64> {
    let min = x.iter().copied().fold(f64::INFINITY, f64::min);
    let max = x.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let span = max - min;
    let mean = x.iter().sum::<f64>() / x.len() as f64;
    let cancellation = state_intercept
        .filter(|(slope, _)| slope.abs() > 1.0e-14)
        .map(|(slope, intercept)| -intercept / slope)
        .unwrap_or(mean + 0.3125 * span);
    vec![
        0.0,
        mean,
        min,
        max,
        min - 0.75 * span,
        max + 1.25 * span,
        cancellation,
    ]
}

fn candidate_diversity(dataset: &Dataset) -> Diversity {
    let all_kernels = kernels();
    let mut factor_bits = BTreeSet::new();
    let mut base_bits = BTreeSet::new();
    let mut slope_bits = BTreeSet::new();
    let mut prediction_bits = dataset
        .new_x
        .iter()
        .map(|_| BTreeSet::new())
        .collect::<Vec<_>>();
    let mut provider_pair_differences = 0;
    let mut exp_publication_differences = 0;
    let mut linear_graph_distinct_sum = 0;

    for kernel in &all_kernels {
        let state = fit(&dataset.x, &dataset.y, dataset.use_const, *kernel);
        if !state.slope.is_finite() {
            continue;
        }
        slope_bits.insert(state.slope.to_bits());
        let factor_outputs = EXP_PROVIDERS
            .iter()
            .map(|provider| provider.eval(state.slope).to_bits())
            .collect::<BTreeSet<_>>();
        exp_publication_differences += factor_outputs.len().saturating_sub(1);
        factor_bits.extend(factor_outputs);
        for &form in &Intercept::ALL {
            let a = intercept(
                &dataset.x,
                &dataset.y,
                dataset.use_const,
                *kernel,
                state,
                form,
            );
            if !a.is_finite() {
                continue;
            }
            let base_outputs = EXP_PROVIDERS
                .iter()
                .map(|provider| provider.eval(a).to_bits())
                .collect::<BTreeSet<_>>();
            exp_publication_differences += base_outputs.len().saturating_sub(1);
            base_bits.extend(base_outputs);
            for (position, &new_x) in dataset.new_x.iter().enumerate() {
                let mut local = BTreeSet::new();
                for &linear in &Linear::ALL {
                    let argument = predict_argument(state, a, new_x, dataset.use_const, linear);
                    if argument.is_finite() {
                        for &provider in &EXP_PROVIDERS {
                            let value = provider.eval(argument);
                            if value.is_finite() {
                                local.insert(value.to_bits());
                                prediction_bits[position].insert(value.to_bits());
                            }
                        }
                    }
                }
                linear_graph_distinct_sum += local.len();
            }
        }
    }

    for mean_order in MeanOrder::ALL {
        for mean_finish in MeanFinish::ALL {
            for moments in Reduction::ALL {
                let platform = reference_kernel(LogProvider::Platform, moments);
                let x87 = Kernel {
                    log: LogProvider::WorksheetX87,
                    mean_order,
                    mean_finish,
                    moments,
                };
                let platform = Kernel {
                    mean_order,
                    mean_finish,
                    ..platform
                };
                let left = fit(&dataset.x, &dataset.y, dataset.use_const, platform);
                let right = fit(&dataset.x, &dataset.y, dataset.use_const, x87);
                provider_pair_differences +=
                    usize::from(left.slope.to_bits() != right.slope.to_bits());
            }
        }
    }

    let counts = prediction_bits
        .iter()
        .map(BTreeSet::len)
        .collect::<Vec<_>>();
    Diversity {
        factor_distinct: factor_bits.len(),
        base_distinct: base_bits.len(),
        prediction_distinct_sum: counts.iter().sum(),
        prediction_distinct_min: counts.iter().copied().min().unwrap_or(0),
        prediction_distinct_max: counts.iter().copied().max().unwrap_or(0),
        kernel_slope_distinct: slope_bits.len(),
        provider_pair_differences,
        exp_publication_differences,
        linear_graph_distinct_sum,
    }
}

fn provider_anchors() -> Vec<u64> {
    // Find moderate values at which the public platform ln() and clean-room
    // worksheet-x87 model publish different binary64 results.  The scan is
    // deterministic and answer-blind; it does not touch Excel.
    let mut anchors = BTreeSet::new();
    let known = [
        0x3ff1_5524_d1fd_5a72_u64,
        0x4000_1c2a_6126_b88b,
        0x4028_5d6f_d932_0a4b,
    ];
    for bits in known {
        let value = f64::from_bits(bits);
        if LogProvider::Platform.eval(value).to_bits()
            != LogProvider::WorksheetX87.eval(value).to_bits()
        {
            anchors.insert(bits);
        }
    }
    let mut rng = Rng(0x6c6e_2d70_726f_7669);
    for _ in 0..2_000_000 {
        let exponent = 1_021 + (rng.next_u64() % 7);
        let mantissa = rng.next_u64() & ((1_u64 << 52) - 1);
        let bits = (exponent << 52) | mantissa;
        let value = f64::from_bits(bits);
        if LogProvider::Platform.eval(value).to_bits()
            != LogProvider::WorksheetX87.eval(value).to_bits()
        {
            anchors.insert(bits);
            if anchors.len() == 64 {
                break;
            }
        }
    }
    assert_eq!(anchors.len(), 64, "insufficient offline LN discriminators");
    anchors.into_iter().collect()
}

fn provider_lane() -> Vec<Dataset> {
    // Adjacent values are included so selection can favor neighborhoods over
    // isolated provider coincidences.
    let anchors = provider_anchors();
    let exact_x = [-5.0, -2.0, -0.5, 1.0, 3.0, 6.0];
    let mut pool = Vec::new();
    for index in 0..anchors.len() * 9 {
        let anchor = anchors[index % anchors.len()];
        let delta = (index / anchors.len()) as i64 % 9;
        let signed_delta = delta - 4;
        let y_anchor = f64::from_bits((anchor as i128 + signed_delta as i128) as u64);
        let n = 4 + index % 3;
        let mut x = exact_x[..n].to_vec();
        let position = (index * 5 + 1) % n;
        let mut y = (0..n)
            .map(|row| {
                if row == position {
                    y_anchor
                } else {
                    let exponent = ((row * 7 + index) % 11) as i32 - 5;
                    2.0_f64.powi(exponent)
                }
            })
            .collect::<Vec<_>>();
        if index & 1 != 0 {
            rotate(&mut x, index % n);
            rotate(&mut y, index % n);
        }
        let use_const = index % 5 != 0;
        let provisional = Dataset {
            id: format!("provider-pool-{index:03}"),
            lane: "ln-provider".to_owned(),
            metamer: "original".to_owned(),
            use_const,
            new_x: generic_prediction_points(&x, None),
            x,
            y,
            selection_score: 0,
        };
        let diversity = candidate_diversity(&provisional);
        let score = diversity.provider_pair_differences * 10_000
            + diversity.kernel_slope_distinct * 100
            + diversity.prediction_distinct_sum;
        pool.push((score, provisional));
    }
    pool.sort_by(|left, right| {
        right
            .0
            .cmp(&left.0)
            .then_with(|| left.1.id.cmp(&right.1.id))
    });
    pool.into_iter()
        .take(16)
        .enumerate()
        .map(|(rank, (score, mut dataset))| {
            dataset.id = format!("g3-04-v2-provider-{rank:02}");
            dataset.selection_score = score;
            dataset
        })
        .collect()
}

fn make_unroll_master(rng: &mut Rng, pool_index: usize) -> (Vec<f64>, Vec<f64>) {
    let n = 18;
    let offset = match pool_index % 4 {
        0 => 0.0,
        1 => 1024.0,
        2 => -65_536.0,
        _ => 1_048_576.0,
    };
    let scale = match pool_index % 3 {
        0 => 1.0,
        1 => 0.03125,
        _ => 8.0,
    };
    let mut x = (0..n)
        .map(|row| {
            let alternating = if row & 1 == 0 { 1.0 } else { -1.0 };
            offset
                + scale * (alternating * (0.25 + 6.0 * rng.unit()) + row as f64 * 0.000_244_140_625)
        })
        .collect::<Vec<_>>();
    let center = offset;
    let slope = -0.08 + 0.16 * rng.unit();
    let intercept = -0.6 + 1.2 * rng.unit();
    let y = x
        .iter()
        .enumerate()
        .map(|(row, &xv)| {
            let noise = if row % 3 == 1 {
                rng.signed() * 0.025
            } else {
                rng.signed() * 0.002
            };
            (intercept + slope * ((xv - center) / scale) + noise).exp()
        })
        .collect::<Vec<_>>();
    rotate(&mut x, pool_index % n);
    (x, y)
}

fn unroll_score(x: &[f64], y: &[f64]) -> usize {
    let mut total = 0;
    for n in 3..=18 {
        for reverse in [false, true] {
            let mut px = x[..n].to_vec();
            let mut py = y[..n].to_vec();
            if reverse {
                px.reverse();
                py.reverse();
            }
            let slopes = Reduction::ALL
                .iter()
                .map(|&moments| {
                    fit(
                        &px,
                        &py,
                        true,
                        reference_kernel(LogProvider::Platform, moments),
                    )
                    .slope
                    .to_bits()
                })
                .collect::<BTreeSet<_>>();
            total += slopes.len() * slopes.len();
        }
    }
    total
}

fn unroll_lane(rng: &mut Rng) -> Vec<Dataset> {
    let mut pool = (0..128)
        .map(|index| {
            let (x, y) = make_unroll_master(rng, index);
            (unroll_score(&x, &y), index, x, y)
        })
        .collect::<Vec<_>>();
    pool.sort_by(|left, right| right.0.cmp(&left.0).then_with(|| left.1.cmp(&right.1)));
    let (master_score, _, master_x, master_y) = pool.remove(0);
    let mut datasets = Vec::new();
    for n in 3..=18 {
        for reverse in [false, true] {
            let mut x = master_x[..n].to_vec();
            let mut y = master_y[..n].to_vec();
            let metamer = if reverse { "reversed" } else { "original" };
            if reverse {
                x.reverse();
                y.reverse();
            }
            let state = fit(
                &x,
                &y,
                true,
                reference_kernel(LogProvider::Platform, Reduction::Pairwise),
            );
            let a = intercept(
                &x,
                &y,
                true,
                reference_kernel(LogProvider::Platform, Reduction::Pairwise),
                state,
                Intercept::MeanMinus,
            );
            datasets.push(Dataset {
                id: format!("g3-04-v2-unroll-n{n:02}-{metamer}"),
                lane: "length-unroll".to_owned(),
                metamer: metamer.to_owned(),
                use_const: true,
                new_x: generic_prediction_points(&x, Some((state.slope, a))),
                x,
                y,
                selection_score: master_score,
            });
        }
    }
    assert_eq!(datasets.len(), 32);
    datasets
}

fn random_symmetric_dataset(rng: &mut Rng, index: usize, lane: &str) -> Dataset {
    let n = 3 + index % 7;
    let center = match index % 4 {
        0 => 0.0,
        1 => 8.0,
        2 => -32.0,
        _ => 256.0,
    };
    let scale = [0.125, 0.5, 2.0, 8.0][(index / 4) % 4];
    let mut x = (0..n)
        .map(|row| center + scale * (row as f64 - (n - 1) as f64 * 0.5))
        .collect::<Vec<_>>();
    let slope = -0.45 + 0.9 * rng.unit();
    let intercept_value = -1.25 + 2.5 * rng.unit();
    let mut y = x
        .iter()
        .enumerate()
        .map(|(row, &xv)| {
            let normalized = (xv - center) / scale;
            let perturbation = if row == (index * 3 + 1) % n {
                rng.signed() * 0.015625
            } else {
                rng.signed() * 0.000_122_070_312_5
            };
            (intercept_value + slope * normalized + perturbation).exp()
        })
        .collect::<Vec<_>>();
    let amount = (index * 5 + 2) % n;
    rotate(&mut x, amount);
    rotate(&mut y, amount);
    let kernel = reference_kernel(LogProvider::Platform, Reduction::Pairwise);
    let state = fit(&x, &y, true, kernel);
    let a = intercept(&x, &y, true, kernel, state, Intercept::MeanMinus);
    Dataset {
        id: format!("{lane}-pool-{index:03}"),
        lane: lane.to_owned(),
        metamer: "original".to_owned(),
        use_const: index % 11 != 0,
        new_x: generic_prediction_points(&x, Some((state.slope, a))),
        x,
        y,
        selection_score: 0,
    }
}

fn select_lane(rng: &mut Rng, lane: &str, count: usize) -> Vec<Dataset> {
    let mut pool = (0..256)
        .map(|index| {
            let dataset = random_symmetric_dataset(rng, index, lane);
            let diversity = candidate_diversity(&dataset);
            let score = if lane == "coefficient-publication" {
                diversity.exp_publication_differences * 10_000
                    + (diversity.factor_distinct + diversity.base_distinct) * 100
                    + diversity.kernel_slope_distinct
            } else {
                diversity.linear_graph_distinct_sum * 100
                    + diversity.prediction_distinct_sum * 10
                    + diversity.prediction_distinct_min
            };
            (score, dataset)
        })
        .collect::<Vec<_>>();
    pool.sort_by(|left, right| {
        right
            .0
            .cmp(&left.0)
            .then_with(|| left.1.id.cmp(&right.1.id))
    });
    pool.into_iter()
        .take(count)
        .enumerate()
        .map(|(rank, (score, mut dataset))| {
            dataset.id = format!("g3-04-v2-{}-{rank:02}", lane.replace('-', ""));
            dataset.selection_score = score;
            dataset
        })
        .collect()
}

fn coefficient_distinct(dataset: &Dataset, factor: bool) -> usize {
    let mut outputs = BTreeSet::new();
    for kernel in kernels() {
        let state = fit(&dataset.x, &dataset.y, dataset.use_const, kernel);
        if factor {
            for &provider in &EXP_PROVIDERS {
                outputs.insert(provider.eval(state.slope).to_bits());
            }
        } else {
            for &form in &Intercept::ALL {
                let a = intercept(
                    &dataset.x,
                    &dataset.y,
                    dataset.use_const,
                    kernel,
                    state,
                    form,
                );
                for &provider in &EXP_PROVIDERS {
                    outputs.insert(provider.eval(a).to_bits());
                }
            }
        }
    }
    outputs.len()
}

fn prediction_distinct(dataset: &Dataset, new_x: f64) -> usize {
    let mut outputs = BTreeSet::new();
    for kernel in kernels() {
        let state = fit(&dataset.x, &dataset.y, dataset.use_const, kernel);
        for &form in &Intercept::ALL {
            let a = intercept(
                &dataset.x,
                &dataset.y,
                dataset.use_const,
                kernel,
                state,
                form,
            );
            for &linear in &Linear::ALL {
                let argument = predict_argument(state, a, new_x, dataset.use_const, linear);
                for &provider in &EXP_PROVIDERS {
                    let value = provider.eval(argument);
                    if value.is_finite() {
                        outputs.insert(value.to_bits());
                    }
                }
            }
        }
    }
    outputs.len()
}

fn record(dataset: &Dataset) -> DatasetRecord {
    DatasetRecord {
        id: dataset.id.clone(),
        lane: dataset.lane.clone(),
        metamer: dataset.metamer.clone(),
        use_const: dataset.use_const,
        x_bits: bits(&dataset.x),
        y_bits: bits(&dataset.y),
        new_x_bits: bits(&dataset.new_x),
        selection_score: dataset.selection_score,
        diversity: candidate_diversity(dataset),
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

fn main() {
    let mut rng = Rng(0x6733_3034_7632_7265);
    let mut datasets = provider_lane();
    datasets.extend(unroll_lane(&mut rng));
    datasets.extend(select_lane(&mut rng, "coefficient-publication", 16));
    datasets.extend(select_lane(&mut rng, "prediction-association", 16));
    assert_eq!(datasets.len(), DATASET_COUNT);
    assert!(datasets.iter().all(|dataset| {
        dataset.x.len() == dataset.y.len()
            && dataset.x.len() >= 3
            && dataset.x.iter().all(|value| value.is_finite())
            && dataset
                .y
                .iter()
                .all(|value| value.is_finite() && *value > 0.0)
            && dataset.new_x.len() == 7
            && dataset.new_x.iter().all(|value| value.is_finite())
    }));

    // Enforce exact input-bit disjointness from the complete v1 discovery
    // bank.  Reading its answer-blind metadata does not expose oracle output.
    let root = PathBuf::from(OUT_DIR);
    let prior: PriorBank = serde_json::from_str(
        &std::fs::read_to_string(root.join("meta-paired-discovery-v1.json")).unwrap(),
    )
    .unwrap();
    let prior_pairs = prior
        .datasets
        .into_iter()
        .map(|record| (record.x_bits, record.y_bits))
        .collect::<BTreeSet<_>>();
    for dataset in &datasets {
        assert!(!prior_pairs.contains(&(bits(&dataset.x), bits(&dataset.y))));
    }

    let records = datasets.iter().map(record).collect::<Vec<_>>();
    let mut logest_probes = Vec::new();
    let mut growth_probes = Vec::new();
    for dataset in &datasets {
        let logest_args = vec![
            json!(bits(&dataset.y)),
            json!(bits(&dataset.x)),
            json!(hex(if dataset.use_const { 1.0 } else { 0.0 })),
            json!(hex(0.0)),
        ];
        logest_probes.push(RankedProbe {
            probe: Probe {
                id: format!("{}-factor", dataset.id),
                args: logest_args.clone(),
                result_index: vec![1, 1],
            },
            distinct_outputs: coefficient_distinct(dataset, true),
            outputs: Vec::new(),
        });
        logest_probes.push(RankedProbe {
            probe: Probe {
                id: format!("{}-base", dataset.id),
                args: logest_args,
                result_index: vec![1, 2],
            },
            distinct_outputs: coefficient_distinct(dataset, false),
            outputs: Vec::new(),
        });

        let growth_args = vec![
            json!(bits(&dataset.y)),
            json!(bits(&dataset.x)),
            json!(bits(&dataset.new_x)),
            json!(hex(if dataset.use_const { 1.0 } else { 0.0 })),
        ];
        for (position, &new_x) in dataset.new_x.iter().enumerate() {
            growth_probes.push(RankedProbe {
                probe: Probe {
                    id: format!("{}-pred-{position:02}", dataset.id),
                    args: growth_args.clone(),
                    result_index: vec![position + 1, 1],
                },
                distinct_outputs: prediction_distinct(dataset, new_x),
                outputs: Vec::new(),
            });
        }
    }

    let all_kernels = kernels();
    let factor_candidates = all_kernels.len() * EXP_PROVIDERS.len();
    let coefficient_candidates = all_kernels.len() * Intercept::ALL.len() * EXP_PROVIDERS.len();
    let end_to_end_candidates = coefficient_candidates * Linear::ALL.len();
    assert_eq!(all_kernels.len(), 108);
    assert_eq!(factor_candidates, 324);
    assert_eq!(coefficient_candidates, 2_268);
    assert_eq!(end_to_end_candidates, 13_608);

    let manifest = json!({
        "schema_version": "oxfunc.w109.growth_candidate_manifest.v2",
        "freeze_id": FREEZE_ID,
        "answer_blind": true,
        "clean_room": true,
        "candidate_space": {
            "kernels": all_kernels,
            "intercepts": Intercept::ALL,
            "linear_forms": Linear::ALL,
            "exp_providers": EXP_PROVIDERS,
            "factor_candidate_count": factor_candidates,
            "coefficient_candidate_count": coefficient_candidates,
            "end_to_end_candidate_count": end_to_end_candidates,
            "published_prediction_controls": [
                PredictionGraph::PublishedPlatformPowF64Mul,
                PredictionGraph::PublishedPlatformPowX87Mul,
                PredictionGraph::PublishedWorksheetPowerF64Mul,
                PredictionGraph::PublishedWorksheetPowerX87Mul,
                PredictionGraph::PublishedRawX87PowerF64Mul,
                PredictionGraph::PublishedRawX87PowerX87Mul,
            ],
        },
        "selection": {
            "method": "candidate-output-disagreement only; no oracle answers read",
            "lanes": {
                "ln-provider": 16,
                "length-unroll": 32,
                "coefficient-publication": 16,
                "prediction-association": 16,
            },
            "length_unroll_design": "one selected 18-row master prefix, n=3..18, original/reversed pairs",
            "dataset_count": datasets.len(),
            "logest_calls": logest_probes.len(),
            "growth_calls": growth_probes.len(),
        },
        "prior_disjointness": {
            "excluded_bank": "meta-paired-discovery-v1.json",
            "method": "exact x/y input-bit tuple comparison before freeze",
        },
    });
    let metadata = json!({
        "schema_version": "oxfunc.w109.growth_refinement_dataset_bank.v2",
        "freeze_id": FREEZE_ID,
        "answer_blind": true,
        "datasets": records,
    });
    let logest = Batch {
        function: "LOGEST",
        row_id: "G3-04-growth-logest-refinement-v2-20260809",
        probes: logest_probes,
    };
    let growth = Batch {
        function: "GROWTH",
        row_id: "G3-04-growth-refinement-v2-20260809",
        probes: growth_probes,
    };
    for (name, value) in [
        (
            "candidate-manifest-refinement-v2.json",
            serde_json::to_value(manifest).unwrap(),
        ),
        (
            "meta-refinement-v2.json",
            serde_json::to_value(metadata).unwrap(),
        ),
        (
            "batch-logest-refinement-v2.json",
            serde_json::to_value(logest).unwrap(),
        ),
        (
            "batch-growth-refinement-v2.json",
            serde_json::to_value(growth).unwrap(),
        ),
    ] {
        let mut bytes = serde_json::to_vec_pretty(&value).unwrap();
        bytes.push(b'\n');
        write_frozen(&root.join(name), &bytes);
    }
    println!(
        "freeze_id={FREEZE_ID} datasets={} LOGEST_calls={} GROWTH_calls={} factor_candidates={} coefficient_candidates={} end_to_end_candidates={}",
        datasets.len(),
        datasets.len() * 2,
        datasets.len() * 7,
        factor_candidates,
        coefficient_candidates,
        end_to_end_candidates,
    );
}
