//! Lane-local clean-room coefficient graphs for the G3-03 LINEST/TREND audit.
//!
//! The canonical GROWTH/LOGEST helper remains byte-stable.  This module reuses
//! only its public arithmetic/order primitives and owns the corrected regression
//! family enumeration needed by `race_regression_linest_offline`.

#![allow(dead_code)]

#[path = "../growth_research/common.rs"]
mod growth_common;

pub use growth_common::{Arithmetic, Order, sum};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RegressionFamily {
    Centered,
    RawNormal,
    Welford,
    Determinant,
    MgsXConst,
    MgsConstX,
    HouseholderXConst,
    HouseholderConstX,
}

impl RegressionFamily {
    fn tag(self) -> &'static str {
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InterceptForm {
    MeanMinusSlopeMean,
    SumResidualOverN,
    MeanResidual,
    /// Intercept produced directly by the two-by-two determinant solve.
    RawDeterminant,
    /// Intercept coefficient returned by a direct QR solve.
    SolverCoefficient,
}

impl InterceptForm {
    pub fn tag(self) -> &'static str {
        match self {
            Self::MeanMinusSlopeMean => "a=my-bmx",
            Self::SumResidualOverN => "a=(sy-b.sx)/n",
            Self::MeanResidual => "a=mean(y-bx)",
            Self::RawDeterminant => "a=raw-det",
            Self::SolverCoefficient => "a=solver",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RegressionVariant {
    pub family: RegressionFamily,
    pub arith: Arithmetic,
    pub mean_order: Order,
    pub moment_order: Order,
    pub intercept: InterceptForm,
}

impl RegressionVariant {
    pub fn id(self) -> String {
        match self.family {
            RegressionFamily::MgsXConst
            | RegressionFamily::MgsConstX
            | RegressionFamily::HouseholderXConst
            | RegressionFamily::HouseholderConstX => format!(
                "{}-{}-mom-{}-{}",
                self.family.tag(),
                self.arith.tag(),
                self.moment_order.tag(),
                self.intercept.tag()
            ),
            RegressionFamily::Welford if self.intercept == InterceptForm::MeanMinusSlopeMean => {
                format!(
                    "{}-{}-mom-{}-{}",
                    self.family.tag(),
                    self.arith.tag(),
                    self.moment_order.tag(),
                    self.intercept.tag()
                )
            }
            _ => format!(
                "{}-{}-mean-{}-mom-{}-{}",
                self.family.tag(),
                self.arith.tag(),
                self.mean_order.tag(),
                self.moment_order.tag(),
                self.intercept.tag()
            ),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Coefficients {
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
        InterceptForm::RawDeterminant | InterceptForm::SolverCoefficient => {
            panic!(
                "intercept form {} is not a post-slope graph",
                variant.intercept.tag()
            )
        }
    }
}

fn centered(x: &[f64], y: &[f64], variant: RegressionVariant) -> Coefficients {
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
    Coefficients {
        slope,
        intercept: intercept_from(x, y, slope, mx, my, variant),
    }
}

fn raw_normal(x: &[f64], y: &[f64], variant: RegressionVariant) -> Coefficients {
    let a = variant.arith;
    let n = x.len() as f64;
    let sx = sum(x, variant.mean_order, a);
    let sy = sum(y, variant.mean_order, a);
    let xx = x
        .iter()
        .copied()
        .map(|value| a.mul(value, value))
        .collect::<Vec<_>>();
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
    Coefficients {
        slope,
        intercept: intercept_from(x, y, slope, mx, my, variant),
    }
}

fn determinant(x: &[f64], y: &[f64], variant: RegressionVariant) -> Coefficients {
    let a = variant.arith;
    let n = x.len() as f64;
    let sx = sum(x, variant.mean_order, a);
    let sy = sum(y, variant.mean_order, a);
    let xx = x
        .iter()
        .copied()
        .map(|value| a.mul(value, value))
        .collect::<Vec<_>>();
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
    Coefficients {
        slope,
        intercept: match variant.intercept {
            InterceptForm::RawDeterminant => raw_intercept,
            InterceptForm::SolverCoefficient => {
                panic!("solver coefficient is not a determinant intercept graph")
            }
            _ => intercept_from(x, y, slope, mx, my, variant),
        },
    }
}

#[derive(Clone, Copy)]
struct MomentState {
    count: f64,
    mx: f64,
    my: f64,
    xx: f64,
    xy: f64,
}

fn merge_moment_states(left: MomentState, right: MomentState, a: Arithmetic) -> MomentState {
    if left.count == 0.0 {
        return right;
    }
    if right.count == 0.0 {
        return left;
    }
    let count = a.add(left.count, right.count);
    let dx = a.sub(right.mx, left.mx);
    let dy = a.sub(right.my, left.my);
    let right_weight = a.div(right.count, count);
    let cross_weight = a.div(a.mul(left.count, right.count), count);
    MomentState {
        count,
        mx: a.add(left.mx, a.mul(dx, right_weight)),
        my: a.add(left.my, a.mul(dy, right_weight)),
        xx: a.add(a.add(left.xx, right.xx), a.mul(a.mul(dx, dx), cross_weight)),
        xy: a.add(a.add(left.xy, right.xy), a.mul(a.mul(dx, dy), cross_weight)),
    }
}

fn pairwise_moment_state(x: &[f64], y: &[f64], a: Arithmetic) -> MomentState {
    // Public pairwise Chan-Golub-LeVeque merge recurrence.
    assert_eq!(x.len(), y.len());
    match x.len() {
        0 => MomentState {
            count: 0.0,
            mx: 0.0,
            my: 0.0,
            xx: 0.0,
            xy: 0.0,
        },
        1 => MomentState {
            count: 1.0,
            mx: x[0],
            my: y[0],
            xx: 0.0,
            xy: 0.0,
        },
        _ => {
            let middle = x.len() / 2;
            merge_moment_states(
                pairwise_moment_state(&x[..middle], &y[..middle], a),
                pairwise_moment_state(&x[middle..], &y[middle..], a),
                a,
            )
        }
    }
}

fn welford(x: &[f64], y: &[f64], variant: RegressionVariant) -> Coefficients {
    let a = variant.arith;
    if variant.intercept == InterceptForm::MeanMinusSlopeMean {
        assert_eq!(variant.mean_order, Order::Forward);
    }
    let state = if variant.moment_order == Order::Pairwise {
        pairwise_moment_state(x, y, a)
    } else {
        let indices: Box<dyn Iterator<Item = usize>> = match variant.moment_order {
            Order::Forward => Box::new(0..x.len()),
            Order::Reverse => Box::new((0..x.len()).rev()),
            Order::Pairwise => unreachable!(),
        };
        let mut state = MomentState {
            count: 0.0,
            mx: 0.0,
            my: 0.0,
            xx: 0.0,
            xy: 0.0,
        };
        for index in indices {
            state.count = a.add(state.count, 1.0);
            let dx = a.sub(x[index], state.mx);
            let dy = a.sub(y[index], state.my);
            state.mx = a.add(state.mx, a.div(dx, state.count));
            state.my = a.add(state.my, a.div(dy, state.count));
            state.xx = a.add(state.xx, a.mul(dx, a.sub(x[index], state.mx)));
            state.xy = a.add(state.xy, a.mul(dx, a.sub(y[index], state.my)));
        }
        state
    };
    let slope = a.div(state.xy, state.xx);
    Coefficients {
        slope,
        intercept: intercept_from(x, y, slope, state.mx, state.my, variant),
    }
}

fn mgs(x: &[f64], y: &[f64], variant: RegressionVariant, const_first: bool) -> Coefficients {
    let a = variant.arith;
    assert_eq!(variant.mean_order, Order::Forward);
    assert_eq!(variant.intercept, InterceptForm::SolverCoefficient);
    let (mut c0, mut c1) = if const_first {
        (vec![1.0; x.len()], x.to_vec())
    } else {
        (x.to_vec(), vec![1.0; x.len()])
    };
    let dot = |left: &[f64], right: &[f64]| {
        let products = left
            .iter()
            .copied()
            .zip(right.iter().copied())
            .map(|(left, right)| a.mul(left, right))
            .collect::<Vec<_>>();
        sum(&products, variant.moment_order, a)
    };
    let r00 = dot(&c0, &c0).sqrt();
    for value in &mut c0 {
        *value = a.div(*value, r00);
    }
    let r01 = dot(&c0, &c1);
    for index in 0..c1.len() {
        c1[index] = a.sub(c1[index], a.mul(r01, c0[index]));
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
        Coefficients {
            slope: beta1,
            intercept: beta0,
        }
    } else {
        Coefficients {
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
) -> Coefficients {
    let a = variant.arith;
    assert_eq!(variant.mean_order, Order::Forward);
    assert_eq!(variant.intercept, InterceptForm::SolverCoefficient);
    let mut matrix = if const_first {
        x.iter()
            .copied()
            .map(|value| [1.0, value])
            .collect::<Vec<_>>()
    } else {
        x.iter()
            .copied()
            .map(|value| [value, 1.0])
            .collect::<Vec<_>>()
    };
    let mut rhs = y.to_vec();
    for col in 0..2 {
        let norm2 = (col..matrix.len())
            .map(|row| a.mul(matrix[row][col], matrix[row][col]))
            .collect::<Vec<_>>();
        let norm = sum(&norm2, variant.moment_order, a).sqrt();
        if norm == 0.0 {
            return Coefficients {
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
        Coefficients {
            slope: beta1,
            intercept: beta0,
        }
    } else {
        Coefficients {
            slope: beta0,
            intercept: beta1,
        }
    }
}

pub fn regress(x: &[f64], y: &[f64], variant: RegressionVariant) -> Coefficients {
    match variant.family {
        RegressionFamily::Centered => centered(x, y, variant),
        RegressionFamily::RawNormal => raw_normal(x, y, variant),
        RegressionFamily::Welford => welford(x, y, variant),
        RegressionFamily::Determinant => determinant(x, y, variant),
        RegressionFamily::MgsXConst => mgs(x, y, variant, false),
        RegressionFamily::MgsConstX => mgs(x, y, variant, true),
        RegressionFamily::HouseholderXConst => householder(x, y, variant, false),
        RegressionFamily::HouseholderConstX => householder(x, y, variant, true),
    }
}

pub fn regression_variants() -> Vec<RegressionVariant> {
    let mut variants = Vec::new();
    let arithmetic = [Arithmetic::F64, Arithmetic::X87Stored];
    let orders = [Order::Forward, Order::Reverse, Order::Pairwise];
    let post_slope_intercepts = [
        InterceptForm::MeanMinusSlopeMean,
        InterceptForm::SumResidualOverN,
        InterceptForm::MeanResidual,
    ];

    for family in [RegressionFamily::Centered, RegressionFamily::RawNormal] {
        for arith in arithmetic {
            for mean_order in orders {
                for moment_order in orders {
                    for intercept in post_slope_intercepts {
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

    for arith in arithmetic {
        for mean_order in orders {
            for moment_order in orders {
                for intercept in [
                    InterceptForm::MeanMinusSlopeMean,
                    InterceptForm::SumResidualOverN,
                    InterceptForm::MeanResidual,
                    InterceptForm::RawDeterminant,
                ] {
                    variants.push(RegressionVariant {
                        family: RegressionFamily::Determinant,
                        arith,
                        mean_order,
                        moment_order,
                        intercept,
                    });
                }
            }
        }
    }

    for arith in arithmetic {
        for moment_order in orders {
            variants.push(RegressionVariant {
                family: RegressionFamily::Welford,
                arith,
                mean_order: Order::Forward,
                moment_order,
                intercept: InterceptForm::MeanMinusSlopeMean,
            });
            for mean_order in orders {
                for intercept in [InterceptForm::SumResidualOverN, InterceptForm::MeanResidual] {
                    variants.push(RegressionVariant {
                        family: RegressionFamily::Welford,
                        arith,
                        mean_order,
                        moment_order,
                        intercept,
                    });
                }
            }
        }
    }

    for family in [
        RegressionFamily::MgsXConst,
        RegressionFamily::MgsConstX,
        RegressionFamily::HouseholderXConst,
        RegressionFamily::HouseholderConstX,
    ] {
        for arith in arithmetic {
            for moment_order in orders {
                variants.push(RegressionVariant {
                    family,
                    arith,
                    mean_order: Order::Forward,
                    moment_order,
                    intercept: InterceptForm::SolverCoefficient,
                });
            }
        }
    }

    assert_eq!(variants.len(), 246, "coefficient graph-space drift");
    variants
}
