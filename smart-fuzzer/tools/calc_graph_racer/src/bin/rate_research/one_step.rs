//! Clean-room RATE one-step candidate grammar.
//!
//! The power node is fixed to the already established raw worksheet x87
//! LN/product/EXP route.  The remaining axes model balance association and
//! storage, forward-difference publication, and the final Newton update.

#![allow(dead_code)]

use oxfunc_core::excel_numeric::research as rx;
use serde::{Deserialize, Serialize};

const CW: u16 = rx::CW_PC64_RN;

fn e(value: f64) -> rx::Ext80 {
    rx::ext_from_f64(value)
}

fn x87_add(a: f64, b: f64) -> f64 {
    rx::ext_to_f64(&rx::ext_add(&e(a), &e(b), CW), CW)
}

fn x87_sub(a: f64, b: f64) -> f64 {
    rx::ext_to_f64(&rx::ext_sub(&e(a), &e(b), CW), CW)
}

fn x87_mul(a: f64, b: f64) -> f64 {
    rx::ext_to_f64(&rx::ext_mul(&e(a), &e(b), CW), CW)
}

fn x87_div(a: f64, b: f64) -> f64 {
    rx::ext_to_f64(&rx::ext_div(&e(a), &e(b), CW), CW)
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum StoredOp {
    F64,
    X87,
}

impl StoredOp {
    pub const ALL: [Self; 2] = [Self::F64, Self::X87];

    pub fn tag(self) -> &'static str {
        match self {
            Self::F64 => "f64",
            Self::X87 => "x87dr",
        }
    }

    fn add(self, a: f64, b: f64) -> f64 {
        match self {
            Self::F64 => a + b,
            Self::X87 => x87_add(a, b),
        }
    }

    fn sub(self, a: f64, b: f64) -> f64 {
        match self {
            Self::F64 => a - b,
            Self::X87 => x87_sub(a, b),
        }
    }

    fn mul(self, a: f64, b: f64) -> f64 {
        match self {
            Self::F64 => a * b,
            Self::X87 => x87_mul(a, b),
        }
    }

    fn div(self, a: f64, b: f64) -> f64 {
        match self {
            Self::F64 => a / b,
            Self::X87 => x87_div(a, b),
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Annuity {
    InvRatePlusType,
    TimingThenDivide,
}

impl Annuity {
    pub const ALL: [Self; 2] = [Self::InvRatePlusType, Self::TimingThenDivide];

    pub fn tag(self) -> &'static str {
        match self {
            Self::InvRatePlusType => "pmt*(1/r+ty)*(p-1)",
            Self::TimingThenDivide => "pmt*(1+r*ty)*(p-1)/r",
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum TermAssoc {
    Left,
    Right,
}

impl TermAssoc {
    pub const ALL: [Self; 2] = [Self::Left, Self::Right];

    pub fn tag(self) -> &'static str {
        match self {
            Self::Left => "left",
            Self::Right => "right",
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum SumPair {
    PresentPayment,
    PresentFuture,
    PaymentFuture,
}

impl SumPair {
    pub const ALL: [Self; 3] = [
        Self::PresentPayment,
        Self::PresentFuture,
        Self::PaymentFuture,
    ];

    pub fn tag(self) -> &'static str {
        match self {
            Self::PresentPayment => "(pv+pay)+fv",
            Self::PresentFuture => "(pv+fv)+pay",
            Self::PaymentFuture => "(pay+fv)+pv",
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum BalanceSchedule {
    F64PerOp,
    X87PerOpStored,
    X87ContinuousAll,
    X87TermsStoredF64Sum,
    F64TermsX87ContinuousSum,
    X87FirstSumStoredThenF64,
}

impl BalanceSchedule {
    pub const ALL: [Self; 6] = [
        Self::F64PerOp,
        Self::X87PerOpStored,
        Self::X87ContinuousAll,
        Self::X87TermsStoredF64Sum,
        Self::F64TermsX87ContinuousSum,
        Self::X87FirstSumStoredThenF64,
    ];

    pub fn tag(self) -> &'static str {
        match self {
            Self::F64PerOp => "balance-f64",
            Self::X87PerOpStored => "balance-x87dr",
            Self::X87ContinuousAll => "balance-x87-cont",
            Self::X87TermsStoredF64Sum => "balance-x87-terms-f64sum",
            Self::F64TermsX87ContinuousSum => "balance-f64terms-x87sum",
            Self::X87FirstSumStoredThenF64 => "balance-x87-firstsum-spill",
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct BalanceModel {
    pub schedule: BalanceSchedule,
    pub annuity: Annuity,
    pub term_assoc: TermAssoc,
    pub sum_pair: SumPair,
}

impl BalanceModel {
    pub fn id(self) -> String {
        format!(
            "{}|{}|term-{}|sum-{}",
            self.schedule.tag(),
            self.annuity.tag(),
            self.term_assoc.tag(),
            self.sum_pair.tag()
        )
    }
}

fn sum_stored(present: f64, payment: f64, future: f64, pair: SumPair, op: StoredOp) -> f64 {
    match pair {
        SumPair::PresentPayment => op.add(op.add(present, payment), future),
        SumPair::PresentFuture => op.add(op.add(present, future), payment),
        SumPair::PaymentFuture => op.add(op.add(payment, future), present),
    }
}

fn sum_ext(
    present: &rx::Ext80,
    payment: &rx::Ext80,
    future: &rx::Ext80,
    pair: SumPair,
) -> rx::Ext80 {
    match pair {
        SumPair::PresentPayment => rx::ext_add(&rx::ext_add(present, payment, CW), future, CW),
        SumPair::PresentFuture => rx::ext_add(&rx::ext_add(present, future, CW), payment, CW),
        SumPair::PaymentFuture => rx::ext_add(&rx::ext_add(payment, future, CW), present, CW),
    }
}

fn terms_stored(
    rate: f64,
    periods: f64,
    payment: f64,
    present: f64,
    timing: f64,
    annuity: Annuity,
    assoc: TermAssoc,
    op: StoredOp,
) -> (f64, f64) {
    let base = op.add(1.0, rate);
    let powered = rx::excel_pow_chain(base, periods);
    let pm1 = op.sub(powered, 1.0);
    let present_term = op.mul(present, powered);
    let payment_term = match annuity {
        Annuity::InvRatePlusType => {
            let coefficient = op.add(op.div(1.0, rate), timing);
            match assoc {
                TermAssoc::Left => op.mul(op.mul(payment, coefficient), pm1),
                TermAssoc::Right => op.mul(payment, op.mul(coefficient, pm1)),
            }
        }
        Annuity::TimingThenDivide => {
            let timing_factor = op.add(1.0, op.mul(rate, timing));
            let numerator = match assoc {
                TermAssoc::Left => op.mul(op.mul(payment, timing_factor), pm1),
                TermAssoc::Right => op.mul(payment, op.mul(timing_factor, pm1)),
            };
            op.div(numerator, rate)
        }
    };
    (present_term, payment_term)
}

fn terms_ext(
    rate: f64,
    periods: f64,
    payment: f64,
    present: f64,
    timing: f64,
    annuity: Annuity,
    assoc: TermAssoc,
) -> (rx::Ext80, rx::Ext80) {
    // The public power call consumes and returns binary64.  Continuous x87
    // candidates begin after that established call boundary.
    let base = x87_add(1.0, rate);
    let powered = rx::excel_pow_chain(base, periods);
    let powered_ext = e(powered);
    let pm1 = rx::ext_sub(&powered_ext, &rx::ext_one(), CW);
    let payment_ext = e(payment);
    let rate_ext = e(rate);
    let timing_ext = e(timing);
    let payment_term = match annuity {
        Annuity::InvRatePlusType => {
            let coefficient =
                rx::ext_add(&rx::ext_div(&rx::ext_one(), &rate_ext, CW), &timing_ext, CW);
            match assoc {
                TermAssoc::Left => {
                    rx::ext_mul(&rx::ext_mul(&payment_ext, &coefficient, CW), &pm1, CW)
                }
                TermAssoc::Right => {
                    rx::ext_mul(&payment_ext, &rx::ext_mul(&coefficient, &pm1, CW), CW)
                }
            }
        }
        Annuity::TimingThenDivide => {
            let timing_factor =
                rx::ext_add(&rx::ext_one(), &rx::ext_mul(&rate_ext, &timing_ext, CW), CW);
            let numerator = match assoc {
                TermAssoc::Left => {
                    rx::ext_mul(&rx::ext_mul(&payment_ext, &timing_factor, CW), &pm1, CW)
                }
                TermAssoc::Right => {
                    rx::ext_mul(&payment_ext, &rx::ext_mul(&timing_factor, &pm1, CW), CW)
                }
            };
            rx::ext_div(&numerator, &rate_ext, CW)
        }
    };
    (rx::ext_mul(&e(present), &powered_ext, CW), payment_term)
}

pub fn balance(args: [f64; 6], rate: f64, model: BalanceModel) -> f64 {
    let [periods, payment, present, future, timing, _guess] = args;
    if rate == 0.0 {
        return match model.schedule {
            BalanceSchedule::F64PerOp | BalanceSchedule::F64TermsX87ContinuousSum => sum_stored(
                present,
                payment * periods,
                future,
                model.sum_pair,
                StoredOp::F64,
            ),
            BalanceSchedule::X87PerOpStored => sum_stored(
                present,
                x87_mul(payment, periods),
                future,
                model.sum_pair,
                StoredOp::X87,
            ),
            _ => {
                let result = sum_ext(
                    &e(present),
                    &rx::ext_mul(&e(payment), &e(periods), CW),
                    &e(future),
                    model.sum_pair,
                );
                rx::ext_to_f64(&result, CW)
            }
        };
    }
    match model.schedule {
        BalanceSchedule::F64PerOp => {
            let (present, payment) = terms_stored(
                rate,
                periods,
                payment,
                present,
                timing,
                model.annuity,
                model.term_assoc,
                StoredOp::F64,
            );
            sum_stored(present, payment, future, model.sum_pair, StoredOp::F64)
        }
        BalanceSchedule::X87PerOpStored => {
            let (present, payment) = terms_stored(
                rate,
                periods,
                payment,
                present,
                timing,
                model.annuity,
                model.term_assoc,
                StoredOp::X87,
            );
            sum_stored(present, payment, future, model.sum_pair, StoredOp::X87)
        }
        BalanceSchedule::X87ContinuousAll => {
            let (present, payment) = terms_ext(
                rate,
                periods,
                payment,
                present,
                timing,
                model.annuity,
                model.term_assoc,
            );
            rx::ext_to_f64(&sum_ext(&present, &payment, &e(future), model.sum_pair), CW)
        }
        BalanceSchedule::X87TermsStoredF64Sum => {
            let (present, payment) = terms_ext(
                rate,
                periods,
                payment,
                present,
                timing,
                model.annuity,
                model.term_assoc,
            );
            sum_stored(
                rx::ext_to_f64(&present, CW),
                rx::ext_to_f64(&payment, CW),
                future,
                model.sum_pair,
                StoredOp::F64,
            )
        }
        BalanceSchedule::F64TermsX87ContinuousSum => {
            let (present, payment) = terms_stored(
                rate,
                periods,
                payment,
                present,
                timing,
                model.annuity,
                model.term_assoc,
                StoredOp::F64,
            );
            rx::ext_to_f64(
                &sum_ext(&e(present), &e(payment), &e(future), model.sum_pair),
                CW,
            )
        }
        BalanceSchedule::X87FirstSumStoredThenF64 => {
            let (present, payment) = terms_ext(
                rate,
                periods,
                payment,
                present,
                timing,
                model.annuity,
                model.term_assoc,
            );
            let future_ext = e(future);
            let (first, last) = match model.sum_pair {
                SumPair::PresentPayment => (rx::ext_add(&present, &payment, CW), future),
                SumPair::PresentFuture => (
                    rx::ext_add(&present, &future_ext, CW),
                    rx::ext_to_f64(&payment, CW),
                ),
                SumPair::PaymentFuture => (
                    rx::ext_add(&payment, &future_ext, CW),
                    rx::ext_to_f64(&present, CW),
                ),
            };
            rx::ext_to_f64(&first, CW) + last
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum DerivativeGraph {
    F64Divide,
    F64Reciprocal,
    X87DifferenceStoredDivide,
    X87DifferenceStoredReciprocal,
    F64DifferenceX87Divide,
    F64DifferenceX87Reciprocal,
    X87ContinuousDivide,
    X87ContinuousReciprocal,
}

impl DerivativeGraph {
    pub const ALL: [Self; 8] = [
        Self::F64Divide,
        Self::F64Reciprocal,
        Self::X87DifferenceStoredDivide,
        Self::X87DifferenceStoredReciprocal,
        Self::F64DifferenceX87Divide,
        Self::F64DifferenceX87Reciprocal,
        Self::X87ContinuousDivide,
        Self::X87ContinuousReciprocal,
    ];

    pub fn tag(self) -> &'static str {
        match self {
            Self::F64Divide => "d=(fn-f)/h-f64",
            Self::F64Reciprocal => "d=(fn-f)*(1/h)-f64",
            Self::X87DifferenceStoredDivide => "d=x87diff/h-x87dr",
            Self::X87DifferenceStoredReciprocal => "d=x87diff*(1/h)-x87dr",
            Self::F64DifferenceX87Divide => "d=f64diff/h-x87dr",
            Self::F64DifferenceX87Reciprocal => "d=f64diff*(1/h)-x87dr",
            Self::X87ContinuousDivide => "d=(fn-f)/h-x87cont",
            Self::X87ContinuousReciprocal => "d=(fn-f)*(1/h)-x87cont",
        }
    }

    fn eval(self, next: f64, current: f64, h: f64) -> f64 {
        match self {
            Self::F64Divide => (next - current) / h,
            Self::F64Reciprocal => (next - current) * (1.0 / h),
            Self::X87DifferenceStoredDivide => x87_div(x87_sub(next, current), h),
            Self::X87DifferenceStoredReciprocal => x87_mul(x87_sub(next, current), x87_div(1.0, h)),
            Self::F64DifferenceX87Divide => x87_div(next - current, h),
            Self::F64DifferenceX87Reciprocal => x87_mul(next - current, x87_div(1.0, h)),
            Self::X87ContinuousDivide => rx::ext_to_f64(
                &rx::ext_div(&rx::ext_sub(&e(next), &e(current), CW), &e(h), CW),
                CW,
            ),
            Self::X87ContinuousReciprocal => rx::ext_to_f64(
                &rx::ext_mul(
                    &rx::ext_sub(&e(next), &e(current), CW),
                    &rx::ext_div(&rx::ext_one(), &e(h), CW),
                    CW,
                ),
                CW,
            ),
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum UpdateGraph {
    F64DivideSubtract,
    F64ReciprocalSubtract,
    X87StoredDivideSubtract,
    X87StoredReciprocalSubtract,
    X87ContinuousDivideSubtract,
    X87ContinuousReciprocalSubtract,
}

impl UpdateGraph {
    pub const ALL: [Self; 6] = [
        Self::F64DivideSubtract,
        Self::F64ReciprocalSubtract,
        Self::X87StoredDivideSubtract,
        Self::X87StoredReciprocalSubtract,
        Self::X87ContinuousDivideSubtract,
        Self::X87ContinuousReciprocalSubtract,
    ];

    pub fn tag(self) -> &'static str {
        match self {
            Self::F64DivideSubtract => "x-f/d-f64",
            Self::F64ReciprocalSubtract => "x-f*(1/d)-f64",
            Self::X87StoredDivideSubtract => "x-(f/d)-x87dr",
            Self::X87StoredReciprocalSubtract => "x-f*(1/d)-x87dr",
            Self::X87ContinuousDivideSubtract => "x-f/d-x87cont",
            Self::X87ContinuousReciprocalSubtract => "x-f*(1/d)-x87cont",
        }
    }

    fn eval(self, x: f64, f: f64, derivative: f64) -> f64 {
        match self {
            Self::F64DivideSubtract => x - f / derivative,
            Self::F64ReciprocalSubtract => x - f * (1.0 / derivative),
            Self::X87StoredDivideSubtract => x87_sub(x, x87_div(f, derivative)),
            Self::X87StoredReciprocalSubtract => x87_sub(x, x87_mul(f, x87_div(1.0, derivative))),
            Self::X87ContinuousDivideSubtract => rx::ext_to_f64(
                &rx::ext_sub(&e(x), &rx::ext_div(&e(f), &e(derivative), CW), CW),
                CW,
            ),
            Self::X87ContinuousReciprocalSubtract => rx::ext_to_f64(
                &rx::ext_sub(
                    &e(x),
                    &rx::ext_mul(&e(f), &rx::ext_div(&rx::ext_one(), &e(derivative), CW), CW),
                    CW,
                ),
                CW,
            ),
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Model {
    pub balance: BalanceModel,
    pub h_op: StoredOp,
    pub input_op: StoredOp,
    pub derivative: DerivativeGraph,
    pub update: UpdateGraph,
}

impl Model {
    pub fn id(self) -> String {
        format!(
            "{}|h-{}|xh-{}|{}|{}",
            self.balance.id(),
            self.h_op.tag(),
            self.input_op.tag(),
            self.derivative.tag(),
            self.update.tag()
        )
    }
}

pub fn models() -> Vec<Model> {
    let mut out = Vec::new();
    for schedule in BalanceSchedule::ALL {
        for annuity in Annuity::ALL {
            for term_assoc in TermAssoc::ALL {
                for sum_pair in SumPair::ALL {
                    let balance = BalanceModel {
                        schedule,
                        annuity,
                        term_assoc,
                        sum_pair,
                    };
                    for h_op in StoredOp::ALL {
                        for input_op in StoredOp::ALL {
                            for derivative in DerivativeGraph::ALL {
                                for update in UpdateGraph::ALL {
                                    out.push(Model {
                                        balance,
                                        h_op,
                                        input_op,
                                        derivative,
                                        update,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    out
}

#[derive(Clone, Copy, Debug)]
pub struct StepTrace {
    pub residual: f64,
    pub h: f64,
    pub next_input: f64,
    pub next_residual: f64,
    pub derivative: f64,
    pub published: f64,
}

pub fn one_step(args: [f64; 6], model: Model) -> Option<StepTrace> {
    let x = args[5];
    let residual = balance(args, x, model.balance);
    let h = model.h_op.mul(1.0e-6, x);
    let next_input = model.input_op.add(x, h);
    let next_residual = balance(args, next_input, model.balance);
    let derivative = model.derivative.eval(next_residual, residual, h);
    let published = model.update.eval(x, residual, derivative);
    (residual.is_finite()
        && residual.abs() < 1.0e-7
        && h.is_finite()
        && h != 0.0
        && next_input.is_finite()
        && next_residual.is_finite()
        && derivative.is_finite()
        && derivative != 0.0
        && published.is_finite())
    .then_some(StepTrace {
        residual,
        h,
        next_input,
        next_residual,
        derivative,
        published,
    })
}

pub fn canonical_balance() -> BalanceModel {
    BalanceModel {
        schedule: BalanceSchedule::F64PerOp,
        annuity: Annuity::InvRatePlusType,
        term_assoc: TermAssoc::Left,
        sum_pair: SumPair::PresentPayment,
    }
}
