//! Residual and metamer analysis for the frozen G3-04 paired discovery bank.

#[path = "growth_research/common.rs"]
mod common;

use common::*;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::PathBuf;

const ROOT: &str = "../../work/w109/G3-04-growth";

#[derive(Deserialize)]
struct Bank {
    datasets: Vec<Record>,
}

#[derive(Deserialize)]
struct Record {
    id: String,
    family: String,
    metamer: String,
    use_const: bool,
    x_bits: Vec<String>,
    y_bits: Vec<String>,
    new_x_bits: Vec<String>,
}

#[derive(Deserialize)]
struct Answers {
    witnesses: Vec<Witness>,
}

#[derive(Deserialize)]
struct Witness {
    id: String,
    expected_bits: String,
}

#[derive(Clone, Copy, Default)]
struct Stats {
    exact: usize,
    total: usize,
    positive: usize,
    negative: usize,
    max_abs: u64,
    sum_abs: u128,
}

impl Stats {
    fn record(&mut self, got: f64, expected: f64) {
        self.total += 1;
        let signed = signed_ulp(got, expected);
        self.exact += usize::from(signed == 0);
        self.positive += usize::from(signed > 0);
        self.negative += usize::from(signed < 0);
        let absolute = signed.unsigned_abs() as u64;
        self.max_abs = self.max_abs.max(absolute);
        self.sum_abs += absolute as u128;
    }

    fn render(self) -> String {
        format!(
            "exact={}/{} +/-={}/{} max={} sum={}",
            self.exact, self.total, self.positive, self.negative, self.max_abs, self.sum_abs
        )
    }
}

fn parse_hex(text: &str) -> f64 {
    f64::from_bits(u64::from_str_radix(text.trim_start_matches("0x"), 16).unwrap())
}

fn ordered(value: f64) -> i128 {
    let bits = value.to_bits();
    let key = if bits >> 63 == 0 {
        bits | (1_u64 << 63)
    } else {
        !bits
    };
    key as i128
}

/// Positive means the candidate is above Excel.
fn signed_ulp(got: f64, expected: f64) -> i128 {
    ordered(got) - ordered(expected)
}

fn load<T: for<'de> Deserialize<'de>>(path: PathBuf) -> T {
    serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap()
}

fn map(answers: Answers) -> BTreeMap<String, f64> {
    answers
        .witnesses
        .into_iter()
        .map(|witness| (witness.id, parse_hex(&witness.expected_bits)))
        .collect()
}

fn group_id(id: &str) -> String {
    for suffix in ["-original", "-reversed", "-translated", "-scaled"] {
        if let Some(prefix) = id.strip_suffix(suffix) {
            return prefix.to_owned();
        }
    }
    panic!("unrecognized metamer id {id}")
}

fn main() {
    let root = PathBuf::from(ROOT);
    let bank: Bank = load(root.join("meta-paired-discovery-v1.json"));
    let logest = map(load(root.join("answers-logest-paired-discovery-v1.json")));
    let growth = map(load(root.join("answers-growth-paired-discovery-v1.json")));

    let coefficient_variant = RegressionVariant {
        family: RegressionFamily::Centered,
        arith: Arithmetic::F64,
        mean_order: Order::Forward,
        moment_order: Order::Pairwise,
        intercept: InterceptForm::MeanMinusSlopeMean,
    };
    let growth_variant = RegressionVariant {
        moment_order: Order::Reverse,
        ..coefficient_variant
    };
    let mut coefficient_slices: BTreeMap<String, Stats> = BTreeMap::new();
    let mut direct_growth_slices: BTreeMap<String, Stats> = BTreeMap::new();
    let mut centered_growth_slices: BTreeMap<String, Stats> = BTreeMap::new();
    let mut observed_prediction_slices: BTreeMap<String, Stats> = BTreeMap::new();

    for record in &bank.datasets {
        let x = record
            .x_bits
            .iter()
            .map(|v| parse_hex(v))
            .collect::<Vec<_>>();
        let y = record
            .y_bits
            .iter()
            .map(|v| parse_hex(v))
            .collect::<Vec<_>>();
        let new_x = record
            .new_x_bits
            .iter()
            .map(|v| parse_hex(v))
            .collect::<Vec<_>>();
        let logged = y.iter().copied().map(f64::ln).collect::<Vec<_>>();
        let coefficients = regress_model(&x, &logged, record.use_const, coefficient_variant);
        let published = publish_coefficients(coefficients, CoefficientExp::WorksheetX87);
        let expected_factor = logest[&format!("{}-factor", record.id)];
        let expected_base = logest[&format!("{}-base", record.id)];
        for (cell, got, expected) in [
            ("factor", published.0, expected_factor),
            ("base", published.1, expected_base),
        ] {
            for key in [
                "all".to_owned(),
                format!("family:{}", record.family),
                format!("metamer:{}", record.metamer),
                format!("cell:{cell}"),
                format!("family:{}|cell:{cell}", record.family),
                format!("metamer:{}|cell:{cell}", record.metamer),
            ] {
                coefficient_slices
                    .entry(key)
                    .or_default()
                    .record(got, expected);
            }
        }

        let growth_coefficients = regress_model(&x, &logged, record.use_const, growth_variant);
        let mean_x = x.iter().sum::<f64>() / x.len() as f64;
        let mean_y = if record.use_const {
            logged.iter().sum::<f64>() / logged.len() as f64
        } else {
            0.0
        };
        let observed = (expected_factor, expected_base);
        for (position, &point) in new_x.iter().enumerate() {
            let expected = growth[&format!("{}-pred-{position:02}", record.id)];
            let direct = predict(
                growth_coefficients,
                publish_coefficients(growth_coefficients, CoefficientExp::Platform),
                point,
                PredictionGraph::LogWorksheetF64,
            );
            let centered = if record.use_const {
                oxfunc_core::excel_numeric::research::excel_exp(
                    mean_y + growth_coefficients.slope * (point - mean_x),
                )
            } else {
                oxfunc_core::excel_numeric::research::excel_exp(growth_coefficients.slope * point)
            };
            let from_observed = predict(
                LogCoefficients {
                    slope: 0.0,
                    intercept: 0.0,
                },
                observed,
                point,
                PredictionGraph::PublishedRawX87PowerX87Mul,
            );
            for key in [
                "all".to_owned(),
                format!("family:{}", record.family),
                format!("metamer:{}", record.metamer),
                format!("position:{position:02}"),
                format!("family:{}|position:{position:02}", record.family),
            ] {
                direct_growth_slices
                    .entry(key.clone())
                    .or_default()
                    .record(direct, expected);
                centered_growth_slices
                    .entry(key.clone())
                    .or_default()
                    .record(centered, expected);
                observed_prediction_slices
                    .entry(key)
                    .or_default()
                    .record(from_observed, expected);
            }
        }
    }

    println!("champion LOGEST coefficient candidate slices");
    for (key, stats) in &coefficient_slices {
        if key == "all"
            || key.starts_with("family:")
            || key.starts_with("metamer:")
            || key.starts_with("cell:")
        {
            println!("{key:42} {}", stats.render());
        }
    }
    println!("\nchampion direct-log GROWTH candidate slices");
    for (key, stats) in &direct_growth_slices {
        if key == "all"
            || key.starts_with("family:")
            || key.starts_with("metamer:")
            || key.starts_with("position:")
        {
            println!("{key:42} {}", stats.render());
        }
    }
    println!("\nchampion observed-LOGEST publication candidate slices");
    for (key, stats) in &observed_prediction_slices {
        if key == "all"
            || key.starts_with("family:")
            || key.starts_with("metamer:")
            || key.starts_with("position:")
        {
            println!("{key:42} {}", stats.render());
        }
    }
    println!("\nchampion centered-prediction GROWTH candidate slices");
    for (key, stats) in &centered_growth_slices {
        if key == "all"
            || key.starts_with("family:")
            || key.starts_with("metamer:")
            || key.starts_with("position:")
        {
            println!("{key:42} {}", stats.render());
        }
    }

    let records = bank
        .datasets
        .iter()
        .map(|record| (record.id.clone(), record))
        .collect::<BTreeMap<_, _>>();
    let mut reversed_logest = Stats::default();
    let mut reversed_growth = Stats::default();
    let mut scaled_base = Stats::default();
    let mut scaled_growth = Stats::default();
    for record in &bank.datasets {
        if record.metamer != "original" {
            continue;
        }
        let group = group_id(&record.id);
        let reversed = records[&format!("{group}-reversed")];
        let scaled = records[&format!("{group}-scaled")];
        for cell in ["factor", "base"] {
            reversed_logest.record(
                logest[&format!("{}-{cell}", record.id)],
                logest[&format!("{}-{cell}", reversed.id)],
            );
        }
        scaled_base.record(
            logest[&format!("{}-base", record.id)],
            logest[&format!("{}-base", scaled.id)],
        );
        for position in 0..record.new_x_bits.len() {
            reversed_growth.record(
                growth[&format!("{}-pred-{position:02}", record.id)],
                growth[&format!("{}-pred-{position:02}", reversed.id)],
            );
            scaled_growth.record(
                growth[&format!("{}-pred-{position:02}", record.id)],
                growth[&format!("{}-pred-{position:02}", scaled.id)],
            );
        }
    }
    println!("\nExcel metamer equality (left scored against right)");
    println!(
        "LOGEST original vs reversed cells  {}",
        reversed_logest.render()
    );
    println!(
        "GROWTH original vs reversed preds {}",
        reversed_growth.render()
    );
    println!("LOGEST original vs x-scaled base  {}", scaled_base.render());
    println!(
        "GROWTH original vs x-scaled preds {}",
        scaled_growth.render()
    );

    let report = serde_json::json!({
        "schema_version": "oxfunc.w109.growth_residual_slices.v1",
        "coefficient_candidate": "ln-platform|centered-f64-mean-fwd-mom-pair-a=my-bmx|exp-x87",
        "direct_growth_candidate": "ln-platform|centered-f64-mean-fwd-mom-rev-a=my-bmx|exp-x87(a+b*x-f64)",
        "observed_coefficient_prediction_candidate": "b*rawpow(m,x)-x87dr",
        "metamer_equality": {
            "logest_original_vs_reversed": reversed_logest.render(),
            "growth_original_vs_reversed": reversed_growth.render(),
            "logest_original_vs_x_scaled_base": scaled_base.render(),
            "growth_original_vs_x_scaled": scaled_growth.render(),
        },
    });
    let mut bytes = serde_json::to_vec_pretty(&report).unwrap();
    bytes.push(b'\n');
    std::fs::write(root.join("residual-slices-paired-discovery-v1.json"), bytes).unwrap();
}
