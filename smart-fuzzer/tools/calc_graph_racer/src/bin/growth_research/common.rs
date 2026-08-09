//! Shared offline model space for the W109 G3-04 GROWTH/LOGEST lane.
//!
//! This module deliberately contains only clean-room candidate graphs.  It is
//! research tooling, not a production implementation.

#![allow(dead_code)]

use oxfunc_core::excel_numeric::research as rx;
use serde::{Deserialize, Serialize};

pub const CW: u16 = rx::CW_PC64_RN;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Arithmetic {
    F64,
    X87Stored,
}

impl Arithmetic {
    pub fn tag(self) -> &'static str {
        match self {
            Self::F64 => "f64",
            Self::X87Stored => "x87dr",
        }
    }

    pub fn add(self, a: f64, b: f64) -> f64 {
        match self {
            Self::F64 => a + b,
            Self::X87Stored => rx::ext_to_f64(
                &rx::ext_add(&rx::ext_from_f64(a), &rx::ext_from_f64(b), CW),
                CW,
            ),
        }
    }

    pub fn sub(self, a: f64, b: f64) -> f64 {
        match self {
            Self::F64 => a - b,
            Self::X87Stored => rx::ext_to_f64(
                &rx::ext_sub(&rx::ext_from_f64(a), &rx::ext_from_f64(b), CW),
                CW,
            ),
        }
    }

    pub fn mul(self, a: f64, b: f64) -> f64 {
        match self {
            Self::F64 => a * b,
            Self::X87Stored => rx::ext_to_f64(
                &rx::ext_mul(&rx::ext_from_f64(a), &rx::ext_from_f64(b), CW),
                CW,
            ),
        }
    }

    pub fn div(self, a: f64, b: f64) -> f64 {
        match self {
            Self::F64 => a / b,
            Self::X87Stored => rx::ext_to_f64(
                &rx::ext_div(&rx::ext_from_f64(a), &rx::ext_from_f64(b), CW),
                CW,
            ),
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Order {
    Forward,
    Reverse,
    Pairwise,
}

impl Order {
    pub fn tag(self) -> &'static str {
        match self {
            Self::Forward => "fwd",
            Self::Reverse => "rev",
            Self::Pairwise => "pair",
        }
    }
}

fn sum_pairwise(values: &[f64], arith: Arithmetic) -> f64 {
    match values.len() {
        0 => 0.0,
        1 => values[0],
        _ => {
            let middle = values.len() / 2;
            arith.add(
                sum_pairwise(&values[..middle], arith),
                sum_pairwise(&values[middle..], arith),
            )
        }
    }
}

pub fn sum(values: &[f64], order: Order, arith: Arithmetic) -> f64 {
    match order {
        Order::Forward => values
            .iter()
            .copied()
            .fold(0.0, |acc, value| arith.add(acc, value)),
        Order::Reverse => values
            .iter()
            .rev()
            .copied()
            .fold(0.0, |acc, value| arith.add(acc, value)),
        Order::Pairwise => sum_pairwise(values, arith),
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum LogProvider {
    Platform,
    WorksheetX87,
}

impl LogProvider {
    pub fn tag(self) -> &'static str {
        match self {
            Self::Platform => "ln-platform",
            Self::WorksheetX87 => "ln-x87",
        }
    }

    pub fn eval(self, value: f64) -> f64 {
        match self {
            Self::Platform => value.ln(),
            Self::WorksheetX87 => rx::excel_ln(value),
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum RegressionFamily {
    /// Means, then one centered cross-product/variance loop.
    Centered,
    /// Raw computational sums `(n*Sxy-Sx*Sy)/(n*Sxx-Sx*Sx)`.
    RawNormal,
    /// One-pass Welford/Youngs-Cramer covariance update.
    Welford,
    /// Normal equations solved as a two-by-two determinant.
    Determinant,
    /// Modified Gram-Schmidt over design columns `[x, 1]`.
    MgsXConst,
    /// Modified Gram-Schmidt over design columns `[1, x]`.
    MgsConstX,
    /// Householder QR over design columns `[x, 1]`.
    HouseholderXConst,
    /// Householder QR over design columns `[1, x]`.
    HouseholderConstX,
}

impl RegressionFamily {
    pub fn tag(self) -> &'static str {
        match self {
            Self::Centered => "centered",
            Self::RawNormal => "raw-ne",
            Self::Welford => "welford",
            Self::Determinant => "det-ne",
            Self::MgsXConst => "mgs-x1",
            Self::MgsConstX => "mgs-1x",
            Self::HouseholderXConst => "hh-x1",
            Self::HouseholderConstX => "hh-1x",
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum InterceptForm {
    MeanMinusSlopeMean,
    SumResidualOverN,
    MeanResidual,
}

impl InterceptForm {
    pub fn tag(self) -> &'static str {
        match self {
            Self::MeanMinusSlopeMean => "a=my-bmx",
            Self::SumResidualOverN => "a=(sy-b.sx)/n",
            Self::MeanResidual => "a=mean(y-bx)",
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct RegressionVariant {
    pub family: RegressionFamily,
    pub arith: Arithmetic,
    pub mean_order: Order,
    pub moment_order: Order,
    pub intercept: InterceptForm,
}

impl RegressionVariant {
    pub fn id(self) -> String {
        format!(
            "{}-{}-mean-{}-mom-{}-{}",
            self.family.tag(),
            self.arith.tag(),
            self.mean_order.tag(),
            self.moment_order.tag(),
            self.intercept.tag()
        )
    }
}

#[derive(Clone, Copy, Debug)]
pub struct LogCoefficients {
    pub slope: f64,
    pub intercept: f64,
}

fn intercept_from(
    x: &[f64],
    y: &[f64],
    slope: f64,
    mx: f64,
    my: f64,
    variant: RegressionVariant,
) -> f64 {
    let a = variant.arith;
    match variant.intercept {
        InterceptForm::MeanMinusSlopeMean => a.sub(my, a.mul(slope, mx)),
        InterceptForm::SumResidualOverN => {
            let sx = sum(x, variant.mean_order, a);
            let sy = sum(y, variant.mean_order, a);
            a.div(a.sub(sy, a.mul(slope, sx)), x.len() as f64)
        }
        InterceptForm::MeanResidual => {
            let residuals = x
                .iter()
                .copied()
                .zip(y.iter().copied())
                .map(|(xv, yv)| a.sub(yv, a.mul(slope, xv)))
                .collect::<Vec<_>>();
            a.div(
                sum(&residuals, variant.mean_order, a),
                residuals.len() as f64,
            )
        }
    }
}

fn centered(x: &[f64], y: &[f64], variant: RegressionVariant) -> LogCoefficients {
    let a = variant.arith;
    let n = x.len() as f64;
    let mx = a.div(sum(x, variant.mean_order, a), n);
    let my = a.div(sum(y, variant.mean_order, a), n);
    let mut xx = Vec::with_capacity(x.len());
    let mut xy = Vec::with_capacity(x.len());
    for (&xv, &yv) in x.iter().zip(y) {
        let dx = a.sub(xv, mx);
        let dy = a.sub(yv, my);
        xx.push(a.mul(dx, dx));
        xy.push(a.mul(dx, dy));
    }
    let slope = a.div(
        sum(&xy, variant.moment_order, a),
        sum(&xx, variant.moment_order, a),
    );
    LogCoefficients {
        slope,
        intercept: intercept_from(x, y, slope, mx, my, variant),
    }
}

fn raw_normal(x: &[f64], y: &[f64], variant: RegressionVariant) -> LogCoefficients {
    let a = variant.arith;
    let n = x.len() as f64;
    let sx = sum(x, variant.mean_order, a);
    let sy = sum(y, variant.mean_order, a);
    let xx = x.iter().copied().map(|v| a.mul(v, v)).collect::<Vec<_>>();
    let xy = x
        .iter()
        .copied()
        .zip(y.iter().copied())
        .map(|(xv, yv)| a.mul(xv, yv))
        .collect::<Vec<_>>();
    let numerator = a.sub(a.mul(n, sum(&xy, variant.moment_order, a)), a.mul(sx, sy));
    let denominator = a.sub(a.mul(n, sum(&xx, variant.moment_order, a)), a.mul(sx, sx));
    let slope = a.div(numerator, denominator);
    let mx = a.div(sx, n);
    let my = a.div(sy, n);
    LogCoefficients {
        slope,
        intercept: intercept_from(x, y, slope, mx, my, variant),
    }
}

fn determinant(x: &[f64], y: &[f64], variant: RegressionVariant) -> LogCoefficients {
    let a = variant.arith;
    let n = x.len() as f64;
    let sx = sum(x, variant.mean_order, a);
    let sy = sum(y, variant.mean_order, a);
    let xx = x.iter().copied().map(|v| a.mul(v, v)).collect::<Vec<_>>();
    let xy = x
        .iter()
        .copied()
        .zip(y.iter().copied())
        .map(|(xv, yv)| a.mul(xv, yv))
        .collect::<Vec<_>>();
    let sxx = sum(&xx, variant.moment_order, a);
    let sxy = sum(&xy, variant.moment_order, a);
    let det = a.sub(a.mul(sxx, n), a.mul(sx, sx));
    let slope = a.div(a.sub(a.mul(sxy, n), a.mul(sx, sy)), det);
    let raw_intercept = a.div(a.sub(a.mul(sxx, sy), a.mul(sx, sxy)), det);
    let mx = a.div(sx, n);
    let my = a.div(sy, n);
    LogCoefficients {
        slope,
        intercept: match variant.intercept {
            InterceptForm::MeanMinusSlopeMean => raw_intercept,
            _ => intercept_from(x, y, slope, mx, my, variant),
        },
    }
}

fn welford(x: &[f64], y: &[f64], variant: RegressionVariant) -> LogCoefficients {
    let a = variant.arith;
    let indices = match variant.moment_order {
        Order::Reverse => (0..x.len()).rev().collect::<Vec<_>>(),
        _ => (0..x.len()).collect::<Vec<_>>(),
    };
    let mut count = 0.0;
    let mut mx = 0.0;
    let mut my = 0.0;
    let mut xx = 0.0;
    let mut xy = 0.0;
    for index in indices {
        count = a.add(count, 1.0);
        let dx = a.sub(x[index], mx);
        let dy = a.sub(y[index], my);
        mx = a.add(mx, a.div(dx, count));
        my = a.add(my, a.div(dy, count));
        xx = a.add(xx, a.mul(dx, a.sub(x[index], mx)));
        xy = a.add(xy, a.mul(dx, a.sub(y[index], my)));
    }
    let slope = a.div(xy, xx);
    LogCoefficients {
        slope,
        intercept: intercept_from(x, y, slope, mx, my, variant),
    }
}

fn mgs(x: &[f64], y: &[f64], variant: RegressionVariant, const_first: bool) -> LogCoefficients {
    let a = variant.arith;
    let indices = match variant.moment_order {
        Order::Reverse => (0..x.len()).rev().collect::<Vec<_>>(),
        _ => (0..x.len()).collect::<Vec<_>>(),
    };
    let (mut c0, mut c1) = if const_first {
        (vec![1.0; x.len()], x.to_vec())
    } else {
        (x.to_vec(), vec![1.0; x.len()])
    };
    let dot = |left: &[f64], right: &[f64]| {
        let products = indices
            .iter()
            .map(|&i| a.mul(left[i], right[i]))
            .collect::<Vec<_>>();
        sum(&products, variant.moment_order, a)
    };
    let r00 = dot(&c0, &c0).sqrt();
    for value in &mut c0 {
        *value = a.div(*value, r00);
    }
    let r01 = dot(&c0, &c1);
    for i in 0..c1.len() {
        c1[i] = a.sub(c1[i], a.mul(r01, c0[i]));
    }
    let r11 = dot(&c1, &c1).sqrt();
    for value in &mut c1 {
        *value = a.div(*value, r11);
    }
    let qy0 = dot(&c0, y);
    let qy1 = dot(&c1, y);
    let beta1 = a.div(qy1, r11);
    let beta0 = a.div(a.sub(qy0, a.mul(r01, beta1)), r00);
    if const_first {
        LogCoefficients {
            slope: beta1,
            intercept: beta0,
        }
    } else {
        LogCoefficients {
            slope: beta0,
            intercept: beta1,
        }
    }
}

fn householder(
    x: &[f64],
    y: &[f64],
    variant: RegressionVariant,
    const_first: bool,
) -> LogCoefficients {
    let a = variant.arith;
    let mut matrix = if const_first {
        x.iter().copied().map(|v| [1.0, v]).collect::<Vec<_>>()
    } else {
        x.iter().copied().map(|v| [v, 1.0]).collect::<Vec<_>>()
    };
    let mut rhs = y.to_vec();
    for col in 0..2 {
        let norm2 = (col..matrix.len())
            .map(|row| a.mul(matrix[row][col], matrix[row][col]))
            .collect::<Vec<_>>();
        let norm = sum(&norm2, variant.moment_order, a).sqrt();
        if norm == 0.0 {
            return LogCoefficients {
                slope: f64::NAN,
                intercept: f64::NAN,
            };
        }
        let sign = if matrix[col][col].is_sign_negative() {
            -1.0
        } else {
            1.0
        };
        let mut v = vec![0.0; matrix.len()];
        for row in col..matrix.len() {
            v[row] = matrix[row][col];
        }
        v[col] = a.add(v[col], a.mul(sign, norm));
        let vv = (col..matrix.len())
            .map(|row| a.mul(v[row], v[row]))
            .collect::<Vec<_>>();
        let beta = a.div(2.0, sum(&vv, variant.moment_order, a));
        for target_col in col..2 {
            let dots = (col..matrix.len())
                .map(|row| a.mul(v[row], matrix[row][target_col]))
                .collect::<Vec<_>>();
            let scale = a.mul(beta, sum(&dots, variant.moment_order, a));
            for row in col..matrix.len() {
                matrix[row][target_col] = a.sub(matrix[row][target_col], a.mul(scale, v[row]));
            }
        }
        let dots = (col..matrix.len())
            .map(|row| a.mul(v[row], rhs[row]))
            .collect::<Vec<_>>();
        let scale = a.mul(beta, sum(&dots, variant.moment_order, a));
        for row in col..matrix.len() {
            rhs[row] = a.sub(rhs[row], a.mul(scale, v[row]));
        }
    }
    let beta1 = a.div(rhs[1], matrix[1][1]);
    let beta0 = a.div(a.sub(rhs[0], a.mul(matrix[0][1], beta1)), matrix[0][0]);
    if const_first {
        LogCoefficients {
            slope: beta1,
            intercept: beta0,
        }
    } else {
        LogCoefficients {
            slope: beta0,
            intercept: beta1,
        }
    }
}

pub fn regress(x: &[f64], log_y: &[f64], variant: RegressionVariant) -> LogCoefficients {
    match variant.family {
        RegressionFamily::Centered => centered(x, log_y, variant),
        RegressionFamily::RawNormal => raw_normal(x, log_y, variant),
        RegressionFamily::Welford => welford(x, log_y, variant),
        RegressionFamily::Determinant => determinant(x, log_y, variant),
        RegressionFamily::MgsXConst => mgs(x, log_y, variant, false),
        RegressionFamily::MgsConstX => mgs(x, log_y, variant, true),
        RegressionFamily::HouseholderXConst => householder(x, log_y, variant, false),
        RegressionFamily::HouseholderConstX => householder(x, log_y, variant, true),
    }
}

fn regress_no_const(x: &[f64], log_y: &[f64], variant: RegressionVariant) -> LogCoefficients {
    let a = variant.arith;
    let products = x
        .iter()
        .copied()
        .zip(log_y.iter().copied())
        .map(|(xv, yv)| a.mul(xv, yv))
        .collect::<Vec<_>>();
    let squares = x
        .iter()
        .copied()
        .map(|xv| a.mul(xv, xv))
        .collect::<Vec<_>>();
    let direct = a.div(
        sum(&products, variant.moment_order, a),
        sum(&squares, variant.moment_order, a),
    );
    let slope = match variant.family {
        RegressionFamily::MgsXConst
        | RegressionFamily::MgsConstX
        | RegressionFamily::HouseholderXConst
        | RegressionFamily::HouseholderConstX => {
            // A one-column QR publication: norm=sqrt(x.x), q=x/norm,
            // beta=(q.y)/norm.  It is algebraically the same as the direct
            // ratio but has different rounding sites.
            let norm = sum(&squares, variant.moment_order, a).sqrt();
            let q = x
                .iter()
                .copied()
                .map(|xv| a.div(xv, norm))
                .collect::<Vec<_>>();
            let qy = q
                .iter()
                .copied()
                .zip(log_y.iter().copied())
                .map(|(qv, yv)| a.mul(qv, yv))
                .collect::<Vec<_>>();
            a.div(sum(&qy, variant.moment_order, a), norm)
        }
        _ => direct,
    };
    LogCoefficients {
        slope,
        intercept: 0.0,
    }
}

pub fn regress_model(
    x: &[f64],
    log_y: &[f64],
    use_const: bool,
    variant: RegressionVariant,
) -> LogCoefficients {
    if use_const {
        regress(x, log_y, variant)
    } else {
        regress_no_const(x, log_y, variant)
    }
}

pub fn regression_variants() -> Vec<RegressionVariant> {
    let mut variants = Vec::new();
    for family in [
        RegressionFamily::Centered,
        RegressionFamily::RawNormal,
        RegressionFamily::Welford,
        RegressionFamily::Determinant,
        RegressionFamily::MgsXConst,
        RegressionFamily::MgsConstX,
        RegressionFamily::HouseholderXConst,
        RegressionFamily::HouseholderConstX,
    ] {
        for arith in [Arithmetic::F64, Arithmetic::X87Stored] {
            for mean_order in [Order::Forward, Order::Reverse, Order::Pairwise] {
                for moment_order in [Order::Forward, Order::Reverse, Order::Pairwise] {
                    for intercept in [
                        InterceptForm::MeanMinusSlopeMean,
                        InterceptForm::SumResidualOverN,
                        InterceptForm::MeanResidual,
                    ] {
                        variants.push(RegressionVariant {
                            family,
                            arith,
                            mean_order,
                            moment_order,
                            intercept,
                        });
                    }
                }
            }
        }
    }
    variants
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum CoefficientExp {
    Platform,
    WorksheetX87,
    X87RoundTowardZero,
}

impl CoefficientExp {
    pub fn tag(self) -> &'static str {
        match self {
            Self::Platform => "exp-platform",
            Self::WorksheetX87 => "exp-x87",
            Self::X87RoundTowardZero => "exp-x87-rz",
        }
    }

    pub fn eval(self, value: f64) -> f64 {
        match self {
            Self::Platform => value.exp(),
            Self::WorksheetX87 => rx::excel_exp(value),
            Self::X87RoundTowardZero => rx::excel_exp_rz(value),
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum PredictionGraph {
    PublishedPlatformPowF64Mul,
    PublishedPlatformPowX87Mul,
    PublishedWorksheetPowerF64Mul,
    PublishedWorksheetPowerX87Mul,
    PublishedRawX87PowerF64Mul,
    PublishedRawX87PowerX87Mul,
    LogPlatformF64,
    LogWorksheetF64,
    LogWorksheetX87Stored,
}

impl PredictionGraph {
    pub fn tag(self) -> &'static str {
        match self {
            Self::PublishedPlatformPowF64Mul => "b*powf(m,x)-f64",
            Self::PublishedPlatformPowX87Mul => "b*powf(m,x)-x87dr",
            Self::PublishedWorksheetPowerF64Mul => "b*POWER(m,x)-f64",
            Self::PublishedWorksheetPowerX87Mul => "b*POWER(m,x)-x87dr",
            Self::PublishedRawX87PowerF64Mul => "b*rawpow(m,x)-f64",
            Self::PublishedRawX87PowerX87Mul => "b*rawpow(m,x)-x87dr",
            Self::LogPlatformF64 => "exp-platform(a+b*x)",
            Self::LogWorksheetF64 => "exp-x87(a+b*x-f64)",
            Self::LogWorksheetX87Stored => "exp-x87(a+b*x-x87dr)",
        }
    }
}

pub fn publish_coefficients(coefficients: LogCoefficients, exp: CoefficientExp) -> (f64, f64) {
    (
        exp.eval(coefficients.slope),
        exp.eval(coefficients.intercept),
    )
}

pub fn predict(
    coefficients: LogCoefficients,
    published: (f64, f64),
    new_x: f64,
    graph: PredictionGraph,
) -> f64 {
    let (factor, base) = published;
    let worksheet_power = |power_base: f64, exponent: f64| {
        if exponent == 0.0 {
            1.0
        } else if exponent.is_sign_negative() {
            rx::x87_recip(rx::excel_pow_positive(power_base, -exponent))
        } else {
            rx::excel_pow_positive(power_base, exponent)
        }
    };
    match graph {
        PredictionGraph::PublishedPlatformPowF64Mul => base * factor.powf(new_x),
        PredictionGraph::PublishedPlatformPowX87Mul => {
            Arithmetic::X87Stored.mul(base, factor.powf(new_x))
        }
        PredictionGraph::PublishedWorksheetPowerF64Mul => base * worksheet_power(factor, new_x),
        PredictionGraph::PublishedWorksheetPowerX87Mul => {
            Arithmetic::X87Stored.mul(base, worksheet_power(factor, new_x))
        }
        PredictionGraph::PublishedRawX87PowerF64Mul => base * rx::excel_pow_chain(factor, new_x),
        PredictionGraph::PublishedRawX87PowerX87Mul => {
            Arithmetic::X87Stored.mul(base, rx::excel_pow_chain(factor, new_x))
        }
        PredictionGraph::LogPlatformF64 => {
            (coefficients.intercept + coefficients.slope * new_x).exp()
        }
        PredictionGraph::LogWorksheetF64 => {
            rx::excel_exp(coefficients.intercept + coefficients.slope * new_x)
        }
        PredictionGraph::LogWorksheetX87Stored => {
            let arith = Arithmetic::X87Stored;
            rx::excel_exp(arith.add(coefficients.intercept, arith.mul(coefficients.slope, new_x)))
        }
    }
}

pub const LOG_PROVIDERS: [LogProvider; 2] = [LogProvider::Platform, LogProvider::WorksheetX87];
pub const COEFFICIENT_EXP_VARIANTS: [CoefficientExp; 3] = [
    CoefficientExp::Platform,
    CoefficientExp::WorksheetX87,
    CoefficientExp::X87RoundTowardZero,
];
pub const PREDICTION_GRAPHS: [PredictionGraph; 9] = [
    PredictionGraph::PublishedPlatformPowF64Mul,
    PredictionGraph::PublishedPlatformPowX87Mul,
    PredictionGraph::PublishedWorksheetPowerF64Mul,
    PredictionGraph::PublishedWorksheetPowerX87Mul,
    PredictionGraph::PublishedRawX87PowerF64Mul,
    PredictionGraph::PublishedRawX87PowerX87Mul,
    PredictionGraph::LogPlatformF64,
    PredictionGraph::LogWorksheetF64,
    PredictionGraph::LogWorksheetX87Stored,
];

pub fn ulp_distance(a: f64, b: f64) -> Option<u64> {
    if a.is_nan() || b.is_nan() {
        return None;
    }
    let key = |value: f64| {
        let bits = value.to_bits();
        if bits >> 63 != 0 {
            !bits
        } else {
            bits | (1_u64 << 63)
        }
    };
    Some(key(a).abs_diff(key(b)))
}

pub fn hex(value: f64) -> String {
    format!("0x{:016x}", value.to_bits())
}
