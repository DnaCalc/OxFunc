//! Narrow, discovery-selected GROWTH/LOGEST refinement grammar.
//!
//! The broad v1 search selected ordinary-f64 centered arithmetic.  This v2
//! grammar retains the still-discriminating axes: log provider, mean order,
//! unrolled moment reduction, intercept formation, linear prediction form,
//! and EXP publication.

#![allow(dead_code)]

use crate::common::{Arithmetic, CoefficientExp, LogProvider};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Reduction {
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

impl Reduction {
    pub const ALL: [Self; 9] = [
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

    pub fn tag(self) -> &'static str {
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

fn pairwise(values: &[f64]) -> f64 {
    match values.len() {
        0 => 0.0,
        1 => values[0],
        _ => {
            let middle = values.len() / 2;
            pairwise(&values[..middle]) + pairwise(&values[middle..])
        }
    }
}

fn lanes(values: &[f64], count: usize, reverse: bool) -> f64 {
    let mut accumulators = vec![0.0; count];
    if reverse {
        for (position, &value) in values.iter().rev().enumerate() {
            accumulators[position % count] += value;
        }
    } else {
        for (position, &value) in values.iter().enumerate() {
            accumulators[position % count] += value;
        }
    }
    accumulators.into_iter().sum()
}

pub fn reduce(values: &[f64], reduction: Reduction) -> f64 {
    match reduction {
        Reduction::Forward => values.iter().sum(),
        Reduction::Reverse => values.iter().rev().sum(),
        Reduction::Pairwise => pairwise(values),
        Reduction::Kahan => {
            let mut total = 0.0;
            let mut correction = 0.0;
            for &value in values {
                let adjusted = value - correction;
                let next = total + adjusted;
                correction = (next - total) - adjusted;
                total = next;
            }
            total
        }
        Reduction::Neumaier => {
            let mut total = 0.0;
            let mut correction = 0.0;
            for &value in values {
                let next = total + value;
                correction += if total.abs() >= value.abs() {
                    (total - next) + value
                } else {
                    (value - next) + total
                };
                total = next;
            }
            total + correction
        }
        Reduction::Lanes2 => lanes(values, 2, false),
        Reduction::Lanes4 => lanes(values, 4, false),
        Reduction::ReverseLanes2 => lanes(values, 2, true),
        Reduction::ReverseLanes4 => lanes(values, 4, true),
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum MeanOrder {
    Forward,
    Reverse,
    Pairwise,
}

impl MeanOrder {
    pub const ALL: [Self; 3] = [Self::Forward, Self::Reverse, Self::Pairwise];
    pub fn reduction(self) -> Reduction {
        match self {
            Self::Forward => Reduction::Forward,
            Self::Reverse => Reduction::Reverse,
            Self::Pairwise => Reduction::Pairwise,
        }
    }
    pub fn tag(self) -> &'static str {
        match self {
            Self::Forward => "fwd",
            Self::Reverse => "rev",
            Self::Pairwise => "pair",
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum MeanFinish {
    Divide,
    ReciprocalMultiply,
}

impl MeanFinish {
    pub const ALL: [Self; 2] = [Self::Divide, Self::ReciprocalMultiply];
    pub fn eval(self, total: f64, n: f64) -> f64 {
        match self {
            Self::Divide => total / n,
            Self::ReciprocalMultiply => total * (1.0 / n),
        }
    }
    pub fn tag(self) -> &'static str {
        match self {
            Self::Divide => "div",
            Self::ReciprocalMultiply => "recip",
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Kernel {
    pub log: LogProvider,
    pub mean_order: MeanOrder,
    pub mean_finish: MeanFinish,
    pub moments: Reduction,
}

impl Kernel {
    pub fn id(self) -> String {
        format!(
            "{}|mean-{}-{}|mom-{}",
            self.log.tag(),
            self.mean_order.tag(),
            self.mean_finish.tag(),
            self.moments.tag()
        )
    }
}

pub fn kernels() -> Vec<Kernel> {
    let mut values = Vec::new();
    for log in [LogProvider::Platform, LogProvider::WorksheetX87] {
        for mean_order in MeanOrder::ALL {
            for mean_finish in MeanFinish::ALL {
                for moments in Reduction::ALL {
                    values.push(Kernel {
                        log,
                        mean_order,
                        mean_finish,
                        moments,
                    });
                }
            }
        }
    }
    values
}

#[derive(Clone, Copy, Debug)]
pub struct State {
    pub slope: f64,
    pub mean_x: f64,
    pub mean_y: f64,
    pub sum_x: f64,
    pub sum_y: f64,
}

pub fn fit(x: &[f64], y: &[f64], use_const: bool, kernel: Kernel) -> State {
    let logged = y
        .iter()
        .copied()
        .map(|value| kernel.log.eval(value))
        .collect::<Vec<_>>();
    let n = x.len() as f64;
    let sum_x = reduce(x, kernel.mean_order.reduction());
    let sum_y = reduce(&logged, kernel.mean_order.reduction());
    if !use_const {
        let xy = x
            .iter()
            .copied()
            .zip(logged.iter().copied())
            .map(|(x, y)| x * y)
            .collect::<Vec<_>>();
        let xx = x.iter().copied().map(|x| x * x).collect::<Vec<_>>();
        return State {
            slope: reduce(&xy, kernel.moments) / reduce(&xx, kernel.moments),
            mean_x: 0.0,
            mean_y: 0.0,
            sum_x,
            sum_y,
        };
    }
    let mean_x = kernel.mean_finish.eval(sum_x, n);
    let mean_y = kernel.mean_finish.eval(sum_y, n);
    let xy = x
        .iter()
        .copied()
        .zip(logged.iter().copied())
        .map(|(x, y)| (x - mean_x) * (y - mean_y))
        .collect::<Vec<_>>();
    let xx = x
        .iter()
        .copied()
        .map(|x| {
            let delta = x - mean_x;
            delta * delta
        })
        .collect::<Vec<_>>();
    State {
        slope: reduce(&xy, kernel.moments) / reduce(&xx, kernel.moments),
        mean_x,
        mean_y,
        sum_x,
        sum_y,
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Intercept {
    MeanMinus,
    MeanMinusX87,
    Fma,
    SumMinus,
    MeanResidualForward,
    MeanResidualReverse,
    MeanResidualPairwise,
}

impl Intercept {
    pub const ALL: [Self; 7] = [
        Self::MeanMinus,
        Self::MeanMinusX87,
        Self::Fma,
        Self::SumMinus,
        Self::MeanResidualForward,
        Self::MeanResidualReverse,
        Self::MeanResidualPairwise,
    ];
    pub fn tag(self) -> &'static str {
        match self {
            Self::MeanMinus => "my-bmx",
            Self::MeanMinusX87 => "my-bmx-x87dr",
            Self::Fma => "fma(-b,mx,my)",
            Self::SumMinus => "(sy-b.sx)/n",
            Self::MeanResidualForward => "meanres-fwd",
            Self::MeanResidualReverse => "meanres-rev",
            Self::MeanResidualPairwise => "meanres-pair",
        }
    }
}

pub fn intercept(
    x: &[f64],
    y: &[f64],
    use_const: bool,
    kernel: Kernel,
    state: State,
    form: Intercept,
) -> f64 {
    if !use_const {
        return 0.0;
    }
    match form {
        Intercept::MeanMinus => state.mean_y - state.slope * state.mean_x,
        Intercept::MeanMinusX87 => Arithmetic::X87Stored.sub(
            state.mean_y,
            Arithmetic::X87Stored.mul(state.slope, state.mean_x),
        ),
        Intercept::Fma => (-state.slope).mul_add(state.mean_x, state.mean_y),
        Intercept::SumMinus => (state.sum_y - state.slope * state.sum_x) / x.len() as f64,
        Intercept::MeanResidualForward
        | Intercept::MeanResidualReverse
        | Intercept::MeanResidualPairwise => {
            let logged = y
                .iter()
                .copied()
                .map(|value| kernel.log.eval(value))
                .collect::<Vec<_>>();
            let residuals = x
                .iter()
                .copied()
                .zip(logged)
                .map(|(x, y)| y - state.slope * x)
                .collect::<Vec<_>>();
            let reduction = match form {
                Intercept::MeanResidualForward => Reduction::Forward,
                Intercept::MeanResidualReverse => Reduction::Reverse,
                Intercept::MeanResidualPairwise => Reduction::Pairwise,
                _ => unreachable!(),
            };
            reduce(&residuals, reduction) / x.len() as f64
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Linear {
    InterceptF64,
    InterceptX87,
    InterceptFma,
    CenteredF64,
    CenteredX87,
    CenteredFma,
}

impl Linear {
    pub const ALL: [Self; 6] = [
        Self::InterceptF64,
        Self::InterceptX87,
        Self::InterceptFma,
        Self::CenteredF64,
        Self::CenteredX87,
        Self::CenteredFma,
    ];
    pub fn tag(self) -> &'static str {
        match self {
            Self::InterceptF64 => "a+b*x",
            Self::InterceptX87 => "a+b*x-x87dr",
            Self::InterceptFma => "fma(b,x,a)",
            Self::CenteredF64 => "my+b*(x-mx)",
            Self::CenteredX87 => "my+b*(x-mx)-x87dr",
            Self::CenteredFma => "fma(b,x-mx,my)",
        }
    }
}

pub fn predict_argument(
    state: State,
    intercept: f64,
    x: f64,
    use_const: bool,
    form: Linear,
) -> f64 {
    match form {
        Linear::InterceptF64 => intercept + state.slope * x,
        Linear::InterceptX87 => {
            Arithmetic::X87Stored.add(intercept, Arithmetic::X87Stored.mul(state.slope, x))
        }
        Linear::InterceptFma => state.slope.mul_add(x, intercept),
        Linear::CenteredF64 if use_const => state.mean_y + state.slope * (x - state.mean_x),
        Linear::CenteredX87 if use_const => Arithmetic::X87Stored.add(
            state.mean_y,
            Arithmetic::X87Stored.mul(state.slope, Arithmetic::X87Stored.sub(x, state.mean_x)),
        ),
        Linear::CenteredFma if use_const => state.slope.mul_add(x - state.mean_x, state.mean_y),
        _ => state.slope * x,
    }
}

pub const EXP_PROVIDERS: [CoefficientExp; 3] = [
    CoefficientExp::Platform,
    CoefficientExp::WorksheetX87,
    CoefficientExp::X87RoundTowardZero,
];
