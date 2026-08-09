//! Freeze the first answer-blind paired LOGEST/GROWTH discriminator bank.
//!
//! Selection is performed entirely from disagreement among clean-room model
//! candidates.  No Excel answer is read.  The resulting batches deliberately
//! expose LOGEST's published factor/base cells separately from GROWTH's
//! predictions so coefficient and publication graphs cannot compensate for
//! one another.

#[path = "growth_research/common.rs"]
mod common;

use common::*;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

const FREEZE_ID: &str = "w109-g3-04-paired-discovery-v1-20260809";
const OUT_DIR: &str = "../../work/w109/G3-04-growth";
const GROUPS_PER_FAMILY: usize = 5;

#[derive(Clone)]
struct Dataset {
    id: String,
    family: String,
    metamer: String,
    use_const: bool,
    x: Vec<f64>,
    y: Vec<f64>,
    new_x: Vec<f64>,
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
struct Diversity {
    factor: usize,
    base: usize,
    prediction_sum: usize,
    prediction_min: usize,
    prediction_max: usize,
    score: usize,
}

#[derive(Serialize, Deserialize)]
struct DatasetRecord {
    id: String,
    family: String,
    metamer: String,
    use_const: bool,
    x_bits: Vec<String>,
    y_bits: Vec<String>,
    new_x_bits: Vec<String>,
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

fn next_up(value: f64) -> f64 {
    if value == 0.0 {
        return f64::from_bits(1);
    }
    if value.is_sign_negative() {
        f64::from_bits(value.to_bits() - 1)
    } else {
        f64::from_bits(value.to_bits() + 1)
    }
}

fn next_down(value: f64) -> f64 {
    if value == 0.0 {
        return -f64::from_bits(1);
    }
    if value.is_sign_negative() {
        f64::from_bits(value.to_bits() + 1)
    } else {
        f64::from_bits(value.to_bits() - 1)
    }
}

fn prediction_points(x: &[f64]) -> Vec<f64> {
    let min = x.iter().copied().fold(f64::INFINITY, f64::min);
    let max = x.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let span = max - min;
    let mean = x.iter().sum::<f64>() / x.len() as f64;
    vec![
        x[0],
        x[x.len() / 2],
        x[x.len() - 1],
        mean,
        min - 0.625 * span,
        max + 0.875 * span,
        min + 0.37109375 * span,
    ]
}

fn sorted_irregular_x(rng: &mut Rng, n: usize) -> Vec<f64> {
    let mut x = (0..n)
        .map(|index| -3.75 + 7.5 * rng.unit() + index as f64 * 0.000_976_562_5)
        .collect::<Vec<_>>();
    x.sort_by(f64::total_cmp);
    x
}

fn make_base(family: &str, index: usize, rng: &mut Rng) -> Dataset {
    let n = 3 + index % 7;
    let x = sorted_irregular_x(rng, n);
    let use_const = family != "no-const";
    let mut y = match family {
        "irregular" => (0..n)
            .map(|row| {
                let log_value = -1.75 + 3.5 * rng.unit() + (row as f64 - 2.0) * 0.003;
                log_value.exp()
            })
            .collect::<Vec<_>>(),
        "geometric" => {
            let intercept = -0.75 + 1.5 * rng.unit();
            let mut slope = -0.45 + 0.9 * rng.unit();
            if slope.abs() < 0.04 {
                slope = slope.copysign(0.04);
            }
            x.iter()
                .enumerate()
                .map(|(row, &xv)| {
                    let mut value = (intercept + slope * xv).exp();
                    if index % 3 == 1 && row == n / 2 {
                        value = next_up(value);
                    } else if index % 3 == 2 && row == n / 2 {
                        value = next_down(value);
                    }
                    value
                })
                .collect::<Vec<_>>()
        }
        "near-flat" => {
            let center = 1.0_f64.to_bits() as i128;
            (0..n)
                .map(|row| {
                    let signed = ((rng.next_u64() % 4_000_001) as i128) - 2_000_000;
                    let stagger = (row as i128 - n as i128 / 2) * 31_337;
                    f64::from_bits((center + signed + stagger) as u64)
                })
                .collect::<Vec<_>>()
        }
        "power-of-two" => (0..n)
            .map(|row| {
                let exponent = -12 + ((rng.next_u64() + row as u64 * 7) % 25) as i32;
                2.0_f64.powi(exponent)
            })
            .collect::<Vec<_>>(),
        "no-const" => {
            let mut slope = -0.35 + 0.7 * rng.unit();
            if slope.abs() < 0.03 {
                slope = slope.copysign(0.03);
            }
            x.iter()
                .enumerate()
                .map(|(row, &xv)| {
                    let noise = if row == n / 2 {
                        rng.signed() * 0.002
                    } else {
                        0.0
                    };
                    (slope * xv + noise).exp()
                })
                .collect::<Vec<_>>()
        }
        _ => unreachable!(),
    };
    // Keep every generated point strictly positive, finite, and non-uniform.
    for value in &mut y {
        assert!(value.is_finite() && *value > 0.0);
    }
    assert!(x.windows(2).all(|pair| pair[0] != pair[1]));
    Dataset {
        id: format!("pool-{family}-{index:03}"),
        family: family.to_owned(),
        metamer: "pool".to_owned(),
        use_const,
        new_x: prediction_points(&x),
        x,
        y,
    }
}

fn candidate_outputs(dataset: &Dataset) -> Vec<(LogCoefficients, (f64, f64))> {
    let variants = regression_variants();
    let mut outputs =
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
                outputs.push((coefficients, publish_coefficients(coefficients, exp)));
            }
        }
    }
    outputs
}

fn diversity(dataset: &Dataset) -> Diversity {
    let outputs = candidate_outputs(dataset);
    let factor = outputs
        .iter()
        .map(|(_, published)| published.0.to_bits())
        .collect::<BTreeSet<_>>()
        .len();
    let base = outputs
        .iter()
        .map(|(_, published)| published.1.to_bits())
        .collect::<BTreeSet<_>>()
        .len();
    let mut prediction_counts = Vec::new();
    for &new_x in &dataset.new_x {
        let mut bits = BTreeSet::new();
        for &(coefficients, published) in &outputs {
            for &graph in &PREDICTION_GRAPHS {
                bits.insert(predict(coefficients, published, new_x, graph).to_bits());
            }
        }
        prediction_counts.push(bits.len());
    }
    let prediction_sum = prediction_counts.iter().sum();
    let prediction_min = prediction_counts.iter().copied().min().unwrap_or(0);
    let prediction_max = prediction_counts.iter().copied().max().unwrap_or(0);
    Diversity {
        factor,
        base,
        prediction_sum,
        prediction_min,
        prediction_max,
        score: factor * 7 + base * 7 + prediction_sum,
    }
}

fn transform(base: &Dataset, metamer: &str, ordinal: usize) -> Dataset {
    let mut dataset = base.clone();
    dataset.metamer = metamer.to_owned();
    dataset.id = format!(
        "g3-04-d{ordinal:03}-{}-{}",
        base.family.replace('-', ""),
        metamer
    );
    match metamer {
        "original" => {}
        "reversed" => {
            dataset.x.reverse();
            dataset.y.reverse();
        }
        "translated" => {
            let offset = if ordinal & 1 == 0 { 16.0 } else { -32.0 };
            for value in &mut dataset.x {
                *value += offset;
            }
            for value in &mut dataset.new_x {
                *value += offset;
            }
        }
        "scaled" => {
            let scale = if ordinal & 1 == 0 { 4.0 } else { 0.25 };
            for value in &mut dataset.x {
                *value *= scale;
            }
            for value in &mut dataset.new_x {
                *value *= scale;
            }
        }
        _ => unreachable!(),
    }
    dataset
}

fn bits(values: &[f64]) -> Vec<String> {
    values.iter().copied().map(hex).collect()
}

fn as_record(dataset: &Dataset, diversity: Diversity) -> DatasetRecord {
    DatasetRecord {
        id: dataset.id.clone(),
        family: dataset.family.clone(),
        metamer: dataset.metamer.clone(),
        use_const: dataset.use_const,
        x_bits: bits(&dataset.x),
        y_bits: bits(&dataset.y),
        new_x_bits: bits(&dataset.new_x),
        diversity,
    }
}

fn coefficient_distinct(dataset: &Dataset, factor: bool) -> usize {
    candidate_outputs(dataset)
        .iter()
        .map(|(_, published)| {
            if factor {
                published.0.to_bits()
            } else {
                published.1.to_bits()
            }
        })
        .collect::<BTreeSet<_>>()
        .len()
}

fn prediction_distinct(dataset: &Dataset, new_x: f64) -> usize {
    let mut unique = BTreeSet::new();
    for (coefficients, published) in candidate_outputs(dataset) {
        for &graph in &PREDICTION_GRAPHS {
            unique.insert(predict(coefficients, published, new_x, graph).to_bits());
        }
    }
    unique.len()
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
    let families = [
        "irregular",
        "geometric",
        "near-flat",
        "power-of-two",
        "no-const",
    ];
    let mut rng = Rng(0x6733_3034_6772_6f77);
    let mut chosen_bases = Vec::new();
    let mut selection_rows = Vec::new();
    for family in families {
        let mut pool = (0..36)
            .map(|index| {
                let dataset = make_base(family, index, &mut rng);
                let measure = diversity(&dataset);
                (measure, dataset)
            })
            .collect::<Vec<_>>();
        pool.sort_by(|left, right| {
            right
                .0
                .score
                .cmp(&left.0.score)
                .then_with(|| left.1.id.cmp(&right.1.id))
        });
        for (rank, (measure, dataset)) in pool.into_iter().take(GROUPS_PER_FAMILY).enumerate() {
            selection_rows.push(json!({
                "family": family,
                "rank_within_family": rank + 1,
                "pool_id": dataset.id,
                "answer_blind_diversity": measure,
            }));
            chosen_bases.push(dataset);
        }
    }

    let mut datasets = Vec::new();
    for (index, base) in chosen_bases.iter().enumerate() {
        for metamer in ["original", "reversed", "translated", "scaled"] {
            datasets.push(transform(base, metamer, index + 1));
        }
    }
    assert_eq!(datasets.len(), families.len() * GROUPS_PER_FAMILY * 4);

    // Candidate selection was prior-disjoint from every banked numeric GROWTH
    // input.  These explicit assertions prevent accidental regression to the
    // two July recon rows or the old {2;3} structural control.
    let legacy = [
        (vec![1.0, 3.0, 2.0, 5.0], vec![1.0, 2.0, 3.0, 4.0]),
        (vec![2.0, 4.0, 8.0, 16.0], vec![1.0, 2.0, 3.0, 4.0]),
        (vec![2.0, 3.0], vec![1.0, 2.0]),
    ];
    for dataset in &datasets {
        assert!(
            legacy
                .iter()
                .all(|(y, x)| &dataset.y != y || &dataset.x != x)
        );
    }

    let mut records = Vec::new();
    let mut logest_probes = Vec::new();
    let mut growth_probes = Vec::new();
    for dataset in &datasets {
        let measure = diversity(dataset);
        records.push(as_record(dataset, measure));
        let common_args = vec![
            json!(bits(&dataset.y)),
            json!(bits(&dataset.x)),
            json!(hex(if dataset.use_const { 1.0 } else { 0.0 })),
            json!(hex(0.0)),
        ];
        logest_probes.push(RankedProbe {
            probe: Probe {
                id: format!("{}-factor", dataset.id),
                args: common_args.clone(),
                result_index: vec![1, 1],
            },
            distinct_outputs: coefficient_distinct(dataset, true),
            outputs: Vec::new(),
        });
        logest_probes.push(RankedProbe {
            probe: Probe {
                id: format!("{}-base", dataset.id),
                args: common_args,
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

    let variants = regression_variants();
    let coefficient_count = LOG_PROVIDERS.len() * variants.len() * COEFFICIENT_EXP_VARIANTS.len();
    let end_to_end_count = coefficient_count * PREDICTION_GRAPHS.len();
    assert_eq!(coefficient_count, 2_592);
    assert_eq!(end_to_end_count, 23_328);

    let manifest = json!({
        "schema_version": "oxfunc.w109.growth_candidate_manifest.v1",
        "freeze_id": FREEZE_ID,
        "answer_blind": true,
        "clean_room": true,
        "candidate_space": {
            "log_providers": LOG_PROVIDERS,
            "regression_variants": variants,
            "coefficient_exp_variants": COEFFICIENT_EXP_VARIANTS,
            "prediction_graphs": PREDICTION_GRAPHS,
            "coefficient_candidate_count": coefficient_count,
            "end_to_end_candidate_count": end_to_end_count,
        },
        "selection": {
            "families": families,
            "pool_per_family": 36,
            "selected_groups_per_family": GROUPS_PER_FAMILY,
            "metamers_per_group": ["original", "reversed", "translated", "scaled"],
            "dataset_count": datasets.len(),
            "logest_calls": logest_probes.len(),
            "growth_calls": growth_probes.len(),
            "rows": selection_rows,
        },
        "prior_disjointness": {
            "excluded_banked_rows": ["growth-catalog", "growth-geometric", "unswept-growth-{2;3}"],
            "method": "exact x/y input-bit tuple comparison before freeze",
        },
    });
    let metadata = json!({
        "schema_version": "oxfunc.w109.growth_paired_dataset_bank.v1",
        "freeze_id": FREEZE_ID,
        "answer_blind": true,
        "datasets": records,
    });
    let logest = Batch {
        function: "LOGEST",
        row_id: "G3-04-growth-logest-paired-discovery-20260809",
        probes: logest_probes,
    };
    let growth = Batch {
        function: "GROWTH",
        row_id: "G3-04-growth-paired-discovery-20260809",
        probes: growth_probes,
    };

    let root = PathBuf::from(OUT_DIR);
    for (name, value) in [
        (
            "candidate-manifest-paired-discovery-v1.json",
            serde_json::to_value(manifest).unwrap(),
        ),
        (
            "meta-paired-discovery-v1.json",
            serde_json::to_value(metadata).unwrap(),
        ),
        (
            "batch-logest-paired-discovery-v1.json",
            serde_json::to_value(logest).unwrap(),
        ),
        (
            "batch-growth-paired-discovery-v1.json",
            serde_json::to_value(growth).unwrap(),
        ),
    ] {
        let mut bytes = serde_json::to_vec_pretty(&value).unwrap();
        bytes.push(b'\n');
        write_frozen(&root.join(name), &bytes);
    }
    println!(
        "freeze_id={FREEZE_ID} datasets={} LOGEST_calls={} GROWTH_calls={} coefficient_candidates={} end_to_end_candidates={}",
        datasets.len(),
        datasets.len() * 2,
        datasets.len() * 7,
        coefficient_count,
        end_to_end_count
    );
}
