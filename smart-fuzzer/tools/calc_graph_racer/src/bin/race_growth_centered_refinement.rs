//! Discovery-trained refinement of the G3-04 centered regression graph.
//!
//! This search expands only arithmetic and association axes suggested by the
//! frozen paired bank: mean formation, centered cross-product identity,
//! summation tree, correction term, divide/reciprocal staging, intercept
//! publication, and direct-log prediction association.  It never reads or
//! writes Excel and its winners require a separately frozen held-out gate.

#[path = "growth_research/common.rs"]
mod common;

use common::{Arithmetic, CoefficientExp, LogProvider, ulp_distance};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
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

struct Dataset {
    id: String,
    family: String,
    metamer: String,
    use_const: bool,
    x: Vec<f64>,
    y: Vec<f64>,
    new_x: Vec<f64>,
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

#[derive(Clone, Copy, Debug, Default, Serialize)]
struct Score {
    exact: usize,
    total: usize,
    structural: usize,
    max_ulp: u64,
    sum_ulp: u128,
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

fn better(left: &Score, right: &Score) -> std::cmp::Ordering {
    left.structural
        .cmp(&right.structural)
        .then_with(|| right.exact.cmp(&left.exact))
        .then_with(|| left.max_ulp.cmp(&right.max_ulp))
        .then_with(|| left.sum_ulp.cmp(&right.sum_ulp))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SumKind {
    Forward,
    Reverse,
    Pairwise,
    Kahan,
    Neumaier,
    Lanes2,
    Lanes4,
    ReverseLanes2,
    ReverseLanes4,
}

impl SumKind {
    const ALL: [Self; 9] = [
        Self::Forward,
        Self::Reverse,
        Self::Pairwise,
        Self::Kahan,
        Self::Neumaier,
        Self::Lanes2,
        Self::Lanes4,
        Self::ReverseLanes2,
        Self::ReverseLanes4,
    ];

    fn tag(self) -> &'static str {
        match self {
            Self::Forward => "fwd",
            Self::Reverse => "rev",
            Self::Pairwise => "pair",
            Self::Kahan => "kahan",
            Self::Neumaier => "neumaier",
            Self::Lanes2 => "lanes2",
            Self::Lanes4 => "lanes4",
            Self::ReverseLanes2 => "revlanes2",
            Self::ReverseLanes4 => "revlanes4",
        }
    }
}

fn pairwise(values: &[f64], arithmetic: Arithmetic) -> f64 {
    match values.len() {
        0 => 0.0,
        1 => values[0],
        _ => {
            let middle = values.len() / 2;
            arithmetic.add(
                pairwise(&values[..middle], arithmetic),
                pairwise(&values[middle..], arithmetic),
            )
        }
    }
}

fn lanes(values: &[f64], count: usize, reverse: bool, arithmetic: Arithmetic) -> f64 {
    let mut accumulators = vec![0.0; count];
    if reverse {
        for (position, &value) in values.iter().rev().enumerate() {
            let lane = position % count;
            accumulators[lane] = arithmetic.add(accumulators[lane], value);
        }
    } else {
        for (position, &value) in values.iter().enumerate() {
            let lane = position % count;
            accumulators[lane] = arithmetic.add(accumulators[lane], value);
        }
    }
    accumulators
        .into_iter()
        .fold(0.0, |acc, value| arithmetic.add(acc, value))
}

fn sum(values: &[f64], kind: SumKind, arithmetic: Arithmetic) -> f64 {
    match kind {
        SumKind::Forward => values
            .iter()
            .copied()
            .fold(0.0, |acc, value| arithmetic.add(acc, value)),
        SumKind::Reverse => values
            .iter()
            .rev()
            .copied()
            .fold(0.0, |acc, value| arithmetic.add(acc, value)),
        SumKind::Pairwise => pairwise(values, arithmetic),
        SumKind::Kahan => {
            let mut total = 0.0;
            let mut correction = 0.0;
            for &value in values {
                let adjusted = arithmetic.sub(value, correction);
                let next = arithmetic.add(total, adjusted);
                correction = arithmetic.sub(arithmetic.sub(next, total), adjusted);
                total = next;
            }
            total
        }
        SumKind::Neumaier => {
            let mut total = 0.0;
            let mut correction = 0.0;
            for &value in values {
                let next = arithmetic.add(total, value);
                let delta = if total.abs() >= value.abs() {
                    arithmetic.add(arithmetic.sub(total, next), value)
                } else {
                    arithmetic.add(arithmetic.sub(value, next), total)
                };
                correction = arithmetic.add(correction, delta);
                total = next;
            }
            arithmetic.add(total, correction)
        }
        SumKind::Lanes2 => lanes(values, 2, false, arithmetic),
        SumKind::Lanes4 => lanes(values, 4, false, arithmetic),
        SumKind::ReverseLanes2 => lanes(values, 2, true, arithmetic),
        SumKind::ReverseLanes4 => lanes(values, 4, true, arithmetic),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MeanFinish {
    DivF64,
    DivX87,
    ReciprocalF64,
    ReciprocalX87,
}

impl MeanFinish {
    const ALL: [Self; 4] = [
        Self::DivF64,
        Self::DivX87,
        Self::ReciprocalF64,
        Self::ReciprocalX87,
    ];

    fn eval(self, total: f64, n: f64) -> f64 {
        match self {
            Self::DivF64 => total / n,
            Self::DivX87 => Arithmetic::X87Stored.div(total, n),
            Self::ReciprocalF64 => total * (1.0 / n),
            Self::ReciprocalX87 => {
                Arithmetic::X87Stored.mul(total, Arithmetic::X87Stored.div(1.0, n))
            }
        }
    }

    fn tag(self) -> &'static str {
        match self {
            Self::DivF64 => "div",
            Self::DivX87 => "div-x87",
            Self::ReciprocalF64 => "recip",
            Self::ReciprocalX87 => "recip-x87",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CrossForm {
    DxDy,
    DxY,
    XDy,
}

impl CrossForm {
    const ALL: [Self; 3] = [Self::DxDy, Self::DxY, Self::XDy];
    fn tag(self) -> &'static str {
        match self {
            Self::DxDy => "dxdy",
            Self::DxY => "dxy",
            Self::XDy => "xdy",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SquareForm {
    DxDx,
    DxX,
}

impl SquareForm {
    const ALL: [Self; 2] = [Self::DxDx, Self::DxX];
    fn tag(self) -> &'static str {
        match self {
            Self::DxDx => "dxdx",
            Self::DxX => "dxx",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Correction {
    None,
    ResidualMean,
}

impl Correction {
    const ALL: [Self; 2] = [Self::None, Self::ResidualMean];
    fn tag(self) -> &'static str {
        match self {
            Self::None => "nocorr",
            Self::ResidualMean => "corr",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SlopeFinish {
    DivF64,
    DivX87,
    ReciprocalF64,
}

impl SlopeFinish {
    const ALL: [Self; 3] = [Self::DivF64, Self::DivX87, Self::ReciprocalF64];
    fn eval(self, numerator: f64, denominator: f64) -> f64 {
        match self {
            Self::DivF64 => numerator / denominator,
            Self::DivX87 => Arithmetic::X87Stored.div(numerator, denominator),
            Self::ReciprocalF64 => numerator * (1.0 / denominator),
        }
    }
    fn tag(self) -> &'static str {
        match self {
            Self::DivF64 => "div",
            Self::DivX87 => "div-x87",
            Self::ReciprocalF64 => "recip",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SlopeVariant {
    log: LogProvider,
    mean_order: SumKind,
    mean_sum_arith: Arithmetic,
    mean_finish: MeanFinish,
    body_arith: Arithmetic,
    cross: CrossForm,
    square: SquareForm,
    moment_order: SumKind,
    moment_sum_arith: Arithmetic,
    correction: Correction,
    slope_finish: SlopeFinish,
}

impl SlopeVariant {
    fn id(self) -> String {
        format!(
            "{}|mean-{}-{}-{}|body-{}|{}-{}|mom-{}-{}|{}|slope-{}",
            self.log.tag(),
            self.mean_order.tag(),
            self.mean_sum_arith.tag(),
            self.mean_finish.tag(),
            self.body_arith.tag(),
            self.cross.tag(),
            self.square.tag(),
            self.moment_order.tag(),
            self.moment_sum_arith.tag(),
            self.correction.tag(),
            self.slope_finish.tag(),
        )
    }
}

#[derive(Clone, Copy)]
struct State {
    slope: f64,
    mean_x: f64,
    mean_y: f64,
    sum_x: f64,
    sum_y: f64,
}

fn slope_variants() -> Vec<SlopeVariant> {
    let mut variants = Vec::new();
    for log in [LogProvider::Platform, LogProvider::WorksheetX87] {
        for mean_order in [SumKind::Forward, SumKind::Reverse, SumKind::Pairwise] {
            for mean_sum_arith in [Arithmetic::F64, Arithmetic::X87Stored] {
                for mean_finish in MeanFinish::ALL {
                    for body_arith in [Arithmetic::F64, Arithmetic::X87Stored] {
                        for cross in CrossForm::ALL {
                            for square in SquareForm::ALL {
                                for moment_order in SumKind::ALL {
                                    for moment_sum_arith in [Arithmetic::F64, Arithmetic::X87Stored]
                                    {
                                        for correction in Correction::ALL {
                                            for slope_finish in SlopeFinish::ALL {
                                                variants.push(SlopeVariant {
                                                    log,
                                                    mean_order,
                                                    mean_sum_arith,
                                                    mean_finish,
                                                    body_arith,
                                                    cross,
                                                    square,
                                                    moment_order,
                                                    moment_sum_arith,
                                                    correction,
                                                    slope_finish,
                                                });
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    variants
}

fn state(dataset: &Dataset, variant: SlopeVariant) -> State {
    let log_y = dataset
        .y
        .iter()
        .copied()
        .map(|value| variant.log.eval(value))
        .collect::<Vec<_>>();
    let n = dataset.x.len() as f64;
    let sum_x = sum(&dataset.x, variant.mean_order, variant.mean_sum_arith);
    let sum_y = sum(&log_y, variant.mean_order, variant.mean_sum_arith);
    if !dataset.use_const {
        let numerator = dataset
            .x
            .iter()
            .copied()
            .zip(log_y.iter().copied())
            .map(|(x, y)| variant.body_arith.mul(x, y))
            .collect::<Vec<_>>();
        let denominator = dataset
            .x
            .iter()
            .copied()
            .map(|x| variant.body_arith.mul(x, x))
            .collect::<Vec<_>>();
        return State {
            slope: variant.slope_finish.eval(
                sum(&numerator, variant.moment_order, variant.moment_sum_arith),
                sum(&denominator, variant.moment_order, variant.moment_sum_arith),
            ),
            mean_x: 0.0,
            mean_y: 0.0,
            sum_x,
            sum_y,
        };
    }
    let mean_x = variant.mean_finish.eval(sum_x, n);
    let mean_y = variant.mean_finish.eval(sum_y, n);
    let mut cross_left = Vec::with_capacity(dataset.x.len());
    let mut cross_right = Vec::with_capacity(dataset.x.len());
    let mut square_left = Vec::with_capacity(dataset.x.len());
    let mut square_right = Vec::with_capacity(dataset.x.len());
    for (&x, &y) in dataset.x.iter().zip(&log_y) {
        let dx = variant.body_arith.sub(x, mean_x);
        let dy = variant.body_arith.sub(y, mean_y);
        let (left, right) = match variant.cross {
            CrossForm::DxDy => (dx, dy),
            CrossForm::DxY => (dx, y),
            CrossForm::XDy => (x, dy),
        };
        cross_left.push(left);
        cross_right.push(right);
        let (left, right) = match variant.square {
            SquareForm::DxDx => (dx, dx),
            SquareForm::DxX => (dx, x),
        };
        square_left.push(left);
        square_right.push(right);
    }
    let products = |left: &[f64], right: &[f64]| {
        left.iter()
            .copied()
            .zip(right.iter().copied())
            .map(|(a, b)| variant.body_arith.mul(a, b))
            .collect::<Vec<_>>()
    };
    let corrected = |left: &[f64], right: &[f64]| {
        let raw = sum(
            &products(left, right),
            variant.moment_order,
            variant.moment_sum_arith,
        );
        match variant.correction {
            Correction::None => raw,
            Correction::ResidualMean => {
                let left_sum = sum(left, variant.moment_order, variant.moment_sum_arith);
                let right_sum = sum(right, variant.moment_order, variant.moment_sum_arith);
                variant.body_arith.sub(
                    raw,
                    variant
                        .body_arith
                        .div(variant.body_arith.mul(left_sum, right_sum), n),
                )
            }
        }
    };
    let numerator = corrected(&cross_left, &cross_right);
    let denominator = corrected(&square_left, &square_right);
    State {
        slope: variant.slope_finish.eval(numerator, denominator),
        mean_x,
        mean_y,
        sum_x,
        sum_y,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum InterceptVariant {
    MeanMinus {
        multiply: Arithmetic,
        subtract: Arithmetic,
    },
    MeanPlusNegative {
        multiply: Arithmetic,
        add: Arithmetic,
    },
    Fma,
    SumMinus {
        multiply: Arithmetic,
        subtract: Arithmetic,
        finish: MeanFinish,
    },
    MeanResidual {
        multiply: Arithmetic,
        subtract: Arithmetic,
        order: SumKind,
        sum_arith: Arithmetic,
        finish: MeanFinish,
    },
}

impl InterceptVariant {
    fn id(self) -> String {
        match self {
            Self::MeanMinus { multiply, subtract } => {
                format!("my-{}mul-{}sub", multiply.tag(), subtract.tag())
            }
            Self::MeanPlusNegative { multiply, add } => {
                format!("my+neg-{}mul-{}add", multiply.tag(), add.tag())
            }
            Self::Fma => "fma(-b,mx,my)".to_owned(),
            Self::SumMinus {
                multiply,
                subtract,
                finish,
            } => format!(
                "(sy-{}mul-{}sub)/n-{}",
                multiply.tag(),
                subtract.tag(),
                finish.tag()
            ),
            Self::MeanResidual {
                multiply,
                subtract,
                order,
                sum_arith,
                finish,
            } => format!(
                "meanres-{}mul-{}sub-{}-{}-{}",
                multiply.tag(),
                subtract.tag(),
                order.tag(),
                sum_arith.tag(),
                finish.tag()
            ),
        }
    }
}

fn intercept_variants() -> Vec<InterceptVariant> {
    let mut variants = Vec::new();
    for multiply in [Arithmetic::F64, Arithmetic::X87Stored] {
        for other in [Arithmetic::F64, Arithmetic::X87Stored] {
            variants.push(InterceptVariant::MeanMinus {
                multiply,
                subtract: other,
            });
            variants.push(InterceptVariant::MeanPlusNegative {
                multiply,
                add: other,
            });
            for finish in MeanFinish::ALL {
                variants.push(InterceptVariant::SumMinus {
                    multiply,
                    subtract: other,
                    finish,
                });
            }
            for order in SumKind::ALL {
                for sum_arith in [Arithmetic::F64, Arithmetic::X87Stored] {
                    for finish in MeanFinish::ALL {
                        variants.push(InterceptVariant::MeanResidual {
                            multiply,
                            subtract: other,
                            order,
                            sum_arith,
                            finish,
                        });
                    }
                }
            }
        }
    }
    variants.push(InterceptVariant::Fma);
    variants
}

fn intercept(dataset: &Dataset, state: State, variant: InterceptVariant, log: LogProvider) -> f64 {
    if !dataset.use_const {
        return 0.0;
    }
    match variant {
        InterceptVariant::MeanMinus { multiply, subtract } => {
            subtract.sub(state.mean_y, multiply.mul(state.slope, state.mean_x))
        }
        InterceptVariant::MeanPlusNegative { multiply, add } => {
            add.add(state.mean_y, multiply.mul(-state.slope, state.mean_x))
        }
        InterceptVariant::Fma => (-state.slope).mul_add(state.mean_x, state.mean_y),
        InterceptVariant::SumMinus {
            multiply,
            subtract,
            finish,
        } => finish.eval(
            subtract.sub(state.sum_y, multiply.mul(state.slope, state.sum_x)),
            dataset.x.len() as f64,
        ),
        InterceptVariant::MeanResidual {
            multiply,
            subtract,
            order,
            sum_arith,
            finish,
        } => {
            let logged = dataset
                .y
                .iter()
                .copied()
                .map(|value| log.eval(value))
                .collect::<Vec<_>>();
            let residuals = dataset
                .x
                .iter()
                .copied()
                .zip(logged)
                .map(|(x, y)| subtract.sub(y, multiply.mul(state.slope, x)))
                .collect::<Vec<_>>();
            finish.eval(sum(&residuals, order, sum_arith), dataset.x.len() as f64)
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LinearForm {
    Intercept {
        multiply: Arithmetic,
        add: Arithmetic,
    },
    InterceptFma,
    Centered {
        subtract: Arithmetic,
        multiply: Arithmetic,
        add: Arithmetic,
    },
    CenteredFma {
        subtract: Arithmetic,
    },
}

impl LinearForm {
    fn id(self) -> String {
        match self {
            Self::Intercept { multiply, add } => {
                format!("a+bx-{}mul-{}add", multiply.tag(), add.tag())
            }
            Self::InterceptFma => "fma(b,x,a)".to_owned(),
            Self::Centered {
                subtract,
                multiply,
                add,
            } => format!(
                "my+b(x-mx)-{}sub-{}mul-{}add",
                subtract.tag(),
                multiply.tag(),
                add.tag()
            ),
            Self::CenteredFma { subtract } => {
                format!("fma(b,{}sub(x,mx),my)", subtract.tag())
            }
        }
    }
}

fn linear_forms() -> Vec<LinearForm> {
    let mut forms = Vec::new();
    for multiply in [Arithmetic::F64, Arithmetic::X87Stored] {
        for add in [Arithmetic::F64, Arithmetic::X87Stored] {
            forms.push(LinearForm::Intercept { multiply, add });
        }
    }
    forms.push(LinearForm::InterceptFma);
    for subtract in [Arithmetic::F64, Arithmetic::X87Stored] {
        for multiply in [Arithmetic::F64, Arithmetic::X87Stored] {
            for add in [Arithmetic::F64, Arithmetic::X87Stored] {
                forms.push(LinearForm::Centered {
                    subtract,
                    multiply,
                    add,
                });
            }
        }
        forms.push(LinearForm::CenteredFma { subtract });
    }
    forms
}

fn linear(state: State, intercept: f64, x: f64, form: LinearForm, use_const: bool) -> f64 {
    match form {
        LinearForm::Intercept { multiply, add } => add.add(intercept, multiply.mul(state.slope, x)),
        LinearForm::InterceptFma => state.slope.mul_add(x, intercept),
        LinearForm::Centered {
            subtract,
            multiply,
            add,
        } if use_const => add.add(
            state.mean_y,
            multiply.mul(state.slope, subtract.sub(x, state.mean_x)),
        ),
        LinearForm::CenteredFma { subtract } if use_const => state
            .slope
            .mul_add(subtract.sub(x, state.mean_x), state.mean_y),
        _ => state.slope * x,
    }
}

fn exp(value: f64, provider: CoefficientExp) -> f64 {
    provider.eval(value)
}

fn parse_hex(text: &str) -> f64 {
    f64::from_bits(u64::from_str_radix(text.trim_start_matches("0x"), 16).unwrap())
}

fn load<T: for<'de> Deserialize<'de>>(path: PathBuf) -> T {
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

fn answer_map(answers: Answers) -> BTreeMap<String, f64> {
    answers
        .witnesses
        .into_iter()
        .map(|witness| (witness.id, parse_hex(&witness.expected_bits)))
        .collect()
}

fn print_ranked(label: &str, ranked: &[(String, Score)], count: usize) {
    println!("\n{label}");
    for (rank, (id, score)) in ranked.iter().take(count).enumerate() {
        println!(
            "{:02} exact={}/{} structural={} max={} sum={} {}",
            rank + 1,
            score.exact,
            score.total,
            score.structural,
            score.max_ulp,
            score.sum_ulp,
            id
        );
    }
}

fn main() {
    let root = PathBuf::from(ROOT);
    let bank: Bank = load(root.join("meta-paired-discovery-v1.json"));
    let datasets = bank
        .datasets
        .into_iter()
        .map(|record| Dataset {
            id: record.id,
            family: record.family,
            metamer: record.metamer,
            use_const: record.use_const,
            x: record.x_bits.iter().map(|v| parse_hex(v)).collect(),
            y: record.y_bits.iter().map(|v| parse_hex(v)).collect(),
            new_x: record.new_x_bits.iter().map(|v| parse_hex(v)).collect(),
        })
        .collect::<Vec<_>>();
    let logest = answer_map(load(root.join("answers-logest-paired-discovery-v1.json")));
    let growth = answer_map(load(root.join("answers-growth-paired-discovery-v1.json")));
    let variants = slope_variants();
    println!("refinement slope variants={}", variants.len());

    let coefficient_exps = [
        CoefficientExp::Platform,
        CoefficientExp::WorksheetX87,
        CoefficientExp::X87RoundTowardZero,
    ];
    let mut factor_scores = vec![Score::default(); variants.len() * coefficient_exps.len()];
    for (variant_index, &variant) in variants.iter().enumerate() {
        for dataset in &datasets {
            let state = state(dataset, variant);
            let expected = logest[&format!("{}-factor", dataset.id)];
            for (exp_index, &provider) in coefficient_exps.iter().enumerate() {
                factor_scores[variant_index * coefficient_exps.len() + exp_index]
                    .record(exp(state.slope, provider), expected);
            }
        }
    }
    let mut factor_indices = (0..factor_scores.len()).collect::<Vec<_>>();
    factor_indices.sort_by(|&left, &right| {
        better(&factor_scores[left], &factor_scores[right]).then_with(|| left.cmp(&right))
    });
    let factor_ranked = factor_indices
        .iter()
        .take(100)
        .map(|&index| {
            let variant_index = index / coefficient_exps.len();
            let exp_index = index % coefficient_exps.len();
            (
                format!(
                    "{}|{}",
                    variants[variant_index].id(),
                    coefficient_exps[exp_index].tag()
                ),
                factor_scores[index],
            )
        })
        .collect::<Vec<_>>();
    print_ranked("LOGEST factor refinement", &factor_ranked, 30);

    let mut selected_slope_indices = Vec::new();
    let mut seen_variants = BTreeSet::new();
    let mut seen_slope_signatures = BTreeSet::new();
    for &factor_index in &factor_indices {
        let variant_index = factor_index / coefficient_exps.len();
        if !seen_variants.insert(variant_index) {
            continue;
        }
        let signature = datasets
            .iter()
            .map(|dataset| state(dataset, variants[variant_index]).slope.to_bits())
            .collect::<Vec<_>>();
        if seen_slope_signatures.insert(signature) {
            selected_slope_indices.push(variant_index);
        }
        if selected_slope_indices.len() == 128 {
            break;
        }
    }

    let intercepts = intercept_variants();
    println!(
        "selected slope states={} intercept variants={}",
        selected_slope_indices.len(),
        intercepts.len()
    );
    let base_candidate_count =
        selected_slope_indices.len() * intercepts.len() * coefficient_exps.len();
    let mut base_scores = vec![Score::default(); base_candidate_count];
    for (slope_slot, &variant_index) in selected_slope_indices.iter().enumerate() {
        let variant = variants[variant_index];
        for dataset in &datasets {
            let state = state(dataset, variant);
            let expected = logest[&format!("{}-base", dataset.id)];
            for (intercept_index, &intercept_variant) in intercepts.iter().enumerate() {
                let log_base = intercept(dataset, state, intercept_variant, variant.log);
                for (exp_index, &provider) in coefficient_exps.iter().enumerate() {
                    let index = (slope_slot * intercepts.len() + intercept_index)
                        * coefficient_exps.len()
                        + exp_index;
                    base_scores[index].record(exp(log_base, provider), expected);
                }
            }
        }
    }
    let mut base_indices = (0..base_scores.len()).collect::<Vec<_>>();
    base_indices.sort_by(|&left, &right| {
        better(&base_scores[left], &base_scores[right]).then_with(|| left.cmp(&right))
    });
    let base_decode = |index: usize| {
        let exp_index = index % coefficient_exps.len();
        let pair = index / coefficient_exps.len();
        let intercept_index = pair % intercepts.len();
        let slope_slot = pair / intercepts.len();
        (slope_slot, intercept_index, exp_index)
    };
    let base_ranked = base_indices
        .iter()
        .take(100)
        .map(|&index| {
            let (slope_slot, intercept_index, exp_index) = base_decode(index);
            (
                format!(
                    "{}|{}|{}",
                    variants[selected_slope_indices[slope_slot]].id(),
                    intercepts[intercept_index].id(),
                    coefficient_exps[exp_index].tag()
                ),
                base_scores[index],
            )
        })
        .collect::<Vec<_>>();
    print_ranked("LOGEST base refinement", &base_ranked, 30);

    let mut selected_model_pairs = Vec::new();
    let mut seen_pairs = BTreeSet::new();
    let mut seen_model_signatures = BTreeSet::new();
    for &base_index in &base_indices {
        let (slope_slot, intercept_index, _) = base_decode(base_index);
        if !seen_pairs.insert((slope_slot, intercept_index)) {
            continue;
        }
        let variant = variants[selected_slope_indices[slope_slot]];
        let intercept_variant = intercepts[intercept_index];
        let signature = datasets
            .iter()
            .flat_map(|dataset| {
                let state = state(dataset, variant);
                [
                    state.slope.to_bits(),
                    intercept(dataset, state, intercept_variant, variant.log).to_bits(),
                ]
            })
            .collect::<Vec<_>>();
        if seen_model_signatures.insert(signature) {
            selected_model_pairs.push((slope_slot, intercept_index));
        }
        if selected_model_pairs.len() == 128 {
            break;
        }
    }

    let forms = linear_forms();
    let growth_candidate_count = selected_model_pairs.len() * forms.len() * coefficient_exps.len();
    let mut growth_scores = vec![Score::default(); growth_candidate_count];
    for (model_slot, &(slope_slot, intercept_index)) in selected_model_pairs.iter().enumerate() {
        let variant = variants[selected_slope_indices[slope_slot]];
        let intercept_variant = intercepts[intercept_index];
        for dataset in &datasets {
            let state = state(dataset, variant);
            let log_intercept = intercept(dataset, state, intercept_variant, variant.log);
            for (position, &point) in dataset.new_x.iter().enumerate() {
                let expected = growth[&format!("{}-pred-{position:02}", dataset.id)];
                for (form_index, &form) in forms.iter().enumerate() {
                    let argument = linear(state, log_intercept, point, form, dataset.use_const);
                    for (exp_index, &provider) in coefficient_exps.iter().enumerate() {
                        let index = (model_slot * forms.len() + form_index)
                            * coefficient_exps.len()
                            + exp_index;
                        growth_scores[index].record(exp(argument, provider), expected);
                    }
                }
            }
        }
    }
    let mut growth_indices = (0..growth_scores.len()).collect::<Vec<_>>();
    growth_indices.sort_by(|&left, &right| {
        better(&growth_scores[left], &growth_scores[right]).then_with(|| left.cmp(&right))
    });
    let growth_decode = |index: usize| {
        let exp_index = index % coefficient_exps.len();
        let pair = index / coefficient_exps.len();
        let form_index = pair % forms.len();
        let model_slot = pair / forms.len();
        (model_slot, form_index, exp_index)
    };
    let growth_ranked = growth_indices
        .iter()
        .take(100)
        .map(|&index| {
            let (model_slot, form_index, exp_index) = growth_decode(index);
            let (slope_slot, intercept_index) = selected_model_pairs[model_slot];
            (
                format!(
                    "{}|{}|{}|{}",
                    variants[selected_slope_indices[slope_slot]].id(),
                    intercepts[intercept_index].id(),
                    forms[form_index].id(),
                    coefficient_exps[exp_index].tag()
                ),
                growth_scores[index],
            )
        })
        .collect::<Vec<_>>();
    print_ranked("GROWTH direct-log refinement", &growth_ranked, 30);

    let report = serde_json::json!({
        "schema_version": "oxfunc.w109.growth_centered_refinement.v1",
        "status": "discovery_trained_not_heldout",
        "slope_variant_count": variants.len(),
        "factor_candidate_count": factor_scores.len(),
        "selected_slope_state_count": selected_slope_indices.len(),
        "intercept_variant_count": intercepts.len(),
        "base_candidate_count": base_scores.len(),
        "selected_model_pair_count": selected_model_pairs.len(),
        "linear_form_count": forms.len(),
        "growth_candidate_count": growth_scores.len(),
        "factor_top": factor_ranked,
        "base_top": base_ranked,
        "growth_top": growth_ranked,
        "capture_slices": {
            "families": datasets.iter().map(|d| d.family.clone()).collect::<BTreeSet<_>>(),
            "metamers": datasets.iter().map(|d| d.metamer.clone()).collect::<BTreeSet<_>>(),
        },
    });
    let mut bytes = serde_json::to_vec_pretty(&report).unwrap();
    bytes.push(b'\n');
    std::fs::write(root.join("centered-refinement-v1.json"), bytes).unwrap();
    println!(
        "\nwrote {}",
        root.join("centered-refinement-v1.json").display()
    );
}
