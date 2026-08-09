//! W109 G6-06 research-only race for the missing IRR evaluator family.
//!
//! The existing IRR racers cover Horner multiplication in discount-factor
//! space and forward per-term discounting, but not the reverse-Horner
//! division graph identified independently for worksheet NPV:
//!
//!     total = 0
//!     for cashflow in reverse(values):
//!         total = total / (1 + rate)
//!         total = total + cashflow
//!
//! This tool races that graph (including two ways to reconstruct `1+rate`
//! from the v-space iterate) against the answer-blind W109 discovery capture.
//! It is an offline measurement tool only and does not change production code.

use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use rx::Ext80;

const CW: u16 = rx::CW_PC64_RN;

#[derive(Clone)]
struct Obs {
    id: String,
    cashflows: Vec<f64>,
    guess: f64,
    want: u64,
}

#[derive(Clone, Copy, Debug)]
enum FGraph {
    HornerMul,
    ReverseDivRecipV,
    ReverseTailAddThenDivide,
    ReverseDivRoundTripRate,
    ReverseDivRoundTripRateFused,
    ForwardDiscount,
}

#[derive(Clone, Copy, Debug)]
enum HRule {
    Positive,
    Negative,
    WithFSign,
    AgainstFSign,
}

#[derive(Clone, Copy, Debug)]
enum UpdateAssoc {
    FTimesHOverDen,
    FTimesHOverDenGrouped,
    FOverSlope,
}

#[derive(Clone, Copy, Debug)]
enum Publish {
    ReciprocalMinusOne,
    OneMinusVOverV,
}

#[derive(Clone, Copy, Debug)]
struct Config {
    graph: FGraph,
    h_rule: HRule,
    h_abs: f64,
    assoc: UpdateAssoc,
    publish: Publish,
    tolerance: f64,
    min_steps_before_stop: usize,
    cap: usize,
    relative_tolerance: bool,
    stop_on_fzero: bool,
}

fn f_eval(cashflows: &[f64], v: f64, graph: FGraph) -> f64 {
    match graph {
        FGraph::HornerMul => {
            let mut total = 0.0;
            for &cashflow in cashflows.iter().rev() {
                total = total * v;
                total += cashflow;
            }
            total
        }
        FGraph::ReverseDivRecipV => {
            let div_rate = 1.0 / v;
            let mut total = 0.0;
            for &cashflow in cashflows.iter().rev() {
                total /= div_rate;
                total += cashflow;
            }
            total
        }
        FGraph::ReverseTailAddThenDivide => {
            let div_rate = 1.0 / v;
            let mut tail = 0.0;
            for &cashflow in cashflows[1..].iter().rev() {
                tail = (tail + cashflow) / div_rate;
            }
            cashflows[0] + tail
        }
        FGraph::ReverseDivRoundTripRate => {
            let rate = 1.0 / v - 1.0;
            let div_rate = 1.0 + rate;
            let mut total = 0.0;
            for &cashflow in cashflows.iter().rev() {
                total /= div_rate;
                total += cashflow;
            }
            total
        }
        FGraph::ReverseDivRoundTripRateFused => {
            let rate = (1.0 - v) / v;
            let div_rate = 1.0 + rate;
            let mut total = 0.0;
            for &cashflow in cashflows.iter().rev() {
                total /= div_rate;
                total += cashflow;
            }
            total
        }
        FGraph::ForwardDiscount => {
            let div_rate = 1.0 / v;
            let mut discount = 1.0;
            let mut total = cashflows[0];
            for &cashflow in &cashflows[1..] {
                discount *= div_rate;
                total += cashflow / discount;
            }
            total
        }
    }
}

fn h_for(f0: f64, rule: HRule, magnitude: f64) -> f64 {
    match rule {
        HRule::Positive => magnitude,
        HRule::Negative => -magnitude,
        HRule::WithFSign => magnitude.copysign(f0),
        HRule::AgainstFSign => -magnitude.copysign(f0),
    }
}

fn publish(v: f64, rule: Publish) -> f64 {
    match rule {
        Publish::ReciprocalMinusOne => 1.0 / v - 1.0,
        Publish::OneMinusVOverV => (1.0 - v) / v,
    }
}

fn simulate(obs: &Obs, config: Config) -> Option<f64> {
    let mut v = 1.0 / (1.0 + obs.guess);
    let mut f0 = f_eval(&obs.cashflows, v, config.graph);
    if f0 == 0.0 {
        return Some(obs.guess);
    }

    for step in 0..config.cap {
        let h = h_for(f0, config.h_rule, config.h_abs);
        let f1 = f_eval(&obs.cashflows, v + h, config.graph);
        let den = f1 - f0;
        if !den.is_finite() || den == 0.0 {
            return None;
        }
        let dv = match config.assoc {
            UpdateAssoc::FTimesHOverDen => f0 * h / den,
            UpdateAssoc::FTimesHOverDenGrouped => f0 * (h / den),
            UpdateAssoc::FOverSlope => f0 / (den / h),
        };
        if !dv.is_finite() {
            return None;
        }

        let next = v - dv;
        let limit = if config.relative_tolerance {
            config.tolerance * v.abs()
        } else {
            config.tolerance
        };
        if step + 1 >= config.min_steps_before_stop && dv.abs() < limit {
            v = next; // apply-last
            break;
        }

        v = next;
        f0 = f_eval(&obs.cashflows, v, config.graph);
        if config.stop_on_fzero && f0 == 0.0 {
            break;
        }
    }
    Some(publish(v, config.publish))
}

fn load() -> Vec<Obs> {
    let mut observations = Vec::new();
    for path in ["../../work/w109/G6-solvers/answers-irr-exact-graph-discovery-20260809.json"] {
        let witnesses: WitnessSet =
            serde_json::from_str(&std::fs::read_to_string(path).expect("read frozen IRR answers"))
                .expect("parse frozen IRR answers");
        for witness in witnesses.witnesses {
            let cashflows = match &witness.args[0] {
                WitnessArg::Array(items) => items
                    .iter()
                    .map(|item| parse_bits_hex(item).expect("numeric cashflow"))
                    .collect(),
                _ => continue,
            };
            let guess = match &witness.args[1] {
                WitnessArg::Scalar(value) => parse_bits_hex(value).expect("numeric guess"),
                _ => continue,
            };
            let Some(want) = parse_bits_hex(&witness.expected_bits) else {
                continue;
            };
            observations.push(Obs {
                id: witness.id.unwrap_or_default(),
                cashflows,
                guess,
                want: want.to_bits(),
            });
        }
    }
    observations
}

#[derive(Clone, Copy)]
struct Ext(Ext80);

impl Ext {
    fn new(value: f64) -> Self {
        Self(rx::ext_from_f64(value))
    }

    fn to_f64(self) -> f64 {
        rx::ext_to_f64(&self.0, CW)
    }

    fn store(self, yes: bool) -> Self {
        if yes { Self::new(self.to_f64()) } else { self }
    }

    fn add(self, other: Self) -> Self {
        Self(rx::ext_add(&self.0, &other.0, CW))
    }

    fn sub(self, other: Self) -> Self {
        Self(rx::ext_sub(&self.0, &other.0, CW))
    }

    fn mul(self, other: Self) -> Self {
        Self(rx::ext_mul(&self.0, &other.0, CW))
    }

    fn div(self, other: Self) -> Self {
        Self(rx::ext_div(&self.0, &other.0, CW))
    }
}

#[derive(Clone, Copy, Debug)]
enum ExtFGraph {
    ForwardRepeatedDivision,
    ForwardRepeatedDivisionTailFinal,
    ForwardRepeatedMultiplyTailFinal,
    ReverseHornerDivision,
    ReverseTailAddThenDivide,
    ReverseTailAddThenMultiply,
    ReverseWorksheetNpvMulW,
    ReverseWorksheetNpvDivV,
    ReverseWorksheetTailCompose,
    ReverseWorksheetTailComposeSnap,
    ReverseHornerMultiply,
    ForwardGrowingDiscount,
    ForwardGrowingDiscountDivideV,
    ForwardGrowingDiscountTailFinal,
    ForwardGrowingDiscountDivideVTailFinal,
    ForwardRecomputedPowerW,
    ForwardRecomputedPowerDivideV,
    ForwardTermRepeatedDivision,
    ForwardTermRepeatedMultiply,
    ForwardAddProductDiscount,
    ForwardAddProductDiscountCommuted,
    ForwardAddProductDiscountTailFinal,
    ForwardAddProductDiscountTailFinalCommuted,
    ForwardAddProductWorksheetNpvMulW,
    ForwardAddProductWorksheetNpvDivV,
}

#[derive(Clone, Copy, Debug)]
enum WGraph {
    ReciprocalV,
    ReciprocalMinusOnePlusOne,
    OneMinusVOverVPlusOne,
}

fn ext_w(v: Ext, mask: u16, graph: WGraph) -> Ext {
    let bit = |index: u16| mask & (1 << index) != 0;
    match graph {
        WGraph::ReciprocalV => Ext::new(1.0).div(v).store(bit(0)),
        WGraph::ReciprocalMinusOnePlusOne => {
            let reciprocal = Ext::new(1.0).div(v).store(bit(0));
            let rate = reciprocal.sub(Ext::new(1.0)).store(bit(1));
            Ext::new(1.0).add(rate).store(bit(2))
        }
        WGraph::OneMinusVOverVPlusOne => {
            let rate = Ext::new(1.0).sub(v).store(bit(0)).div(v).store(bit(1));
            Ext::new(1.0).add(rate).store(bit(2))
        }
    }
}

fn ext_f_eval(cashflows: &[f64], v: Ext, mask: u16, graph: ExtFGraph, w_graph: WGraph) -> Ext {
    let bit = |index: u16| mask & (1 << index) != 0;
    let w = ext_w(v, mask, w_graph);
    let value = match graph {
        ExtFGraph::ForwardRepeatedDivision => {
            let mut factor = Ext::new(1.0);
            let mut total = Ext::new(cashflows[0]);
            for &cashflow in &cashflows[1..] {
                factor = factor.div(w).store(bit(3));
                let term = Ext::new(cashflow).mul(factor).store(bit(4));
                total = total.add(term).store(bit(5));
            }
            total
        }
        ExtFGraph::ForwardRepeatedDivisionTailFinal
        | ExtFGraph::ForwardRepeatedMultiplyTailFinal => {
            let mut factor = Ext::new(1.0);
            let mut tail = Ext::new(0.0);
            for &cashflow in &cashflows[1..] {
                factor = match graph {
                    ExtFGraph::ForwardRepeatedDivisionTailFinal => factor.div(w),
                    ExtFGraph::ForwardRepeatedMultiplyTailFinal => factor.mul(v),
                    _ => unreachable!(),
                }
                .store(bit(3));
                let term = Ext::new(cashflow).mul(factor).store(bit(4));
                tail = tail.add(term).store(bit(5));
            }
            Ext::new(cashflows[0]).add(tail.store(bit(6))).store(bit(7))
        }
        ExtFGraph::ReverseHornerDivision => {
            let mut total = Ext::new(0.0);
            for &cashflow in cashflows.iter().rev() {
                total = total.div(w).store(bit(3));
                total = total.add(Ext::new(cashflow)).store(bit(5));
            }
            total
        }
        ExtFGraph::ReverseTailAddThenDivide => {
            let mut tail = Ext::new(0.0);
            for &cashflow in cashflows[1..].iter().rev() {
                tail = tail.add(Ext::new(cashflow)).store(bit(3));
                tail = tail.div(w).store(bit(4));
            }
            Ext::new(cashflows[0]).add(tail.store(bit(5))).store(bit(6))
        }
        ExtFGraph::ReverseTailAddThenMultiply => {
            let mut tail = Ext::new(0.0);
            for &cashflow in cashflows[1..].iter().rev() {
                tail = tail.add(Ext::new(cashflow)).store(bit(3));
                tail = tail.mul(v).store(bit(4));
            }
            Ext::new(cashflows[0]).add(tail.store(bit(5))).store(bit(6))
        }
        ExtFGraph::ReverseWorksheetNpvMulW | ExtFGraph::ReverseWorksheetNpvDivV => {
            let mut npv = Ext::new(0.0);
            for &cashflow in cashflows.iter().rev() {
                npv = npv.add(Ext::new(cashflow)).store(bit(3));
                npv = npv.div(w).store(bit(4));
            }
            let npv = npv.store(bit(5));
            match graph {
                ExtFGraph::ReverseWorksheetNpvMulW => npv.mul(w),
                ExtFGraph::ReverseWorksheetNpvDivV => npv.div(v),
                _ => unreachable!(),
            }
            .store(bit(6))
        }
        ExtFGraph::ReverseWorksheetTailCompose | ExtFGraph::ReverseWorksheetTailComposeSnap => {
            let mut npv = Ext::new(0.0);
            for &cashflow in cashflows[1..].iter().rev() {
                npv = npv.add(Ext::new(cashflow)).store(bit(3));
                npv = npv.div(w).store(bit(4));
            }
            // Worksheet NPV publishes a binary64 result before the evaluator
            // composes c0.  The companion capture brackets the evaluator's
            // cancellation correction between 3 and 64 binary64 epsilons;
            // use the smallest exact-classifying structural threshold.
            let raw = npv.store(true).to_f64();
            let composed = raw + cashflows[0];
            let result = if matches!(graph, ExtFGraph::ReverseWorksheetTailComposeSnap)
                && composed.abs() <= 3.0 * f64::EPSILON * raw.abs().max(cashflows[0].abs())
            {
                0.0
            } else {
                composed
            };
            Ext::new(result).store(bit(6))
        }
        ExtFGraph::ReverseHornerMultiply => {
            let mut total = Ext::new(0.0);
            for &cashflow in cashflows.iter().rev() {
                total = total.mul(v).store(bit(3));
                total = total.add(Ext::new(cashflow)).store(bit(5));
            }
            total
        }
        ExtFGraph::ForwardGrowingDiscount => {
            let mut discount = Ext::new(1.0);
            let mut total = Ext::new(cashflows[0]);
            for &cashflow in &cashflows[1..] {
                discount = discount.mul(w).store(bit(3));
                let term = Ext::new(cashflow).div(discount).store(bit(4));
                total = total.add(term).store(bit(5));
            }
            total
        }
        ExtFGraph::ForwardGrowingDiscountDivideV => {
            let mut discount = Ext::new(1.0);
            let mut total = Ext::new(cashflows[0]);
            for &cashflow in &cashflows[1..] {
                discount = discount.div(v).store(bit(3));
                let term = Ext::new(cashflow).div(discount).store(bit(4));
                total = total.add(term).store(bit(5));
            }
            total
        }
        ExtFGraph::ForwardGrowingDiscountTailFinal
        | ExtFGraph::ForwardGrowingDiscountDivideVTailFinal => {
            let mut discount = Ext::new(1.0);
            let mut tail = Ext::new(0.0);
            for &cashflow in &cashflows[1..] {
                discount = match graph {
                    ExtFGraph::ForwardGrowingDiscountTailFinal => discount.mul(w),
                    ExtFGraph::ForwardGrowingDiscountDivideVTailFinal => discount.div(v),
                    _ => unreachable!(),
                }
                .store(bit(3));
                let term = Ext::new(cashflow).div(discount).store(bit(4));
                tail = tail.add(term).store(bit(5));
            }
            Ext::new(cashflows[0]).add(tail.store(bit(6))).store(bit(7))
        }
        ExtFGraph::ForwardRecomputedPowerW | ExtFGraph::ForwardRecomputedPowerDivideV => {
            let mut total = Ext::new(cashflows[0]);
            for (index, &cashflow) in cashflows.iter().enumerate().skip(1) {
                let mut discount = Ext::new(1.0);
                for _ in 0..index {
                    discount = match graph {
                        ExtFGraph::ForwardRecomputedPowerW => discount.mul(w),
                        ExtFGraph::ForwardRecomputedPowerDivideV => discount.div(v),
                        _ => unreachable!(),
                    }
                    .store(bit(3));
                }
                let term = Ext::new(cashflow).div(discount).store(bit(4));
                total = total.add(term).store(bit(5));
            }
            total
        }
        ExtFGraph::ForwardTermRepeatedDivision | ExtFGraph::ForwardTermRepeatedMultiply => {
            let mut total = Ext::new(cashflows[0]);
            for (index, &cashflow) in cashflows.iter().enumerate().skip(1) {
                let mut term = Ext::new(cashflow);
                for _ in 0..index {
                    term = match graph {
                        ExtFGraph::ForwardTermRepeatedDivision => term.div(w),
                        ExtFGraph::ForwardTermRepeatedMultiply => term.mul(v),
                        _ => unreachable!(),
                    }
                    .store(bit(4));
                }
                total = total.add(term).store(bit(5));
            }
            total
        }
        ExtFGraph::ForwardAddProductDiscount | ExtFGraph::ForwardAddProductDiscountCommuted => {
            let rate = w.sub(Ext::new(1.0)).store(bit(7));
            let mut discount = Ext::new(1.0);
            let mut total = Ext::new(cashflows[0]);
            for &cashflow in &cashflows[1..] {
                let increment = match graph {
                    ExtFGraph::ForwardAddProductDiscount => discount.mul(rate),
                    ExtFGraph::ForwardAddProductDiscountCommuted => rate.mul(discount),
                    _ => unreachable!(),
                }
                .store(bit(4));
                discount = discount.add(increment).store(bit(3));
                let term = Ext::new(cashflow).div(discount).store(bit(5));
                total = total.add(term).store(bit(6));
            }
            total
        }
        ExtFGraph::ForwardAddProductDiscountTailFinal
        | ExtFGraph::ForwardAddProductDiscountTailFinalCommuted => {
            let rate = w.sub(Ext::new(1.0)).store(bit(7));
            let mut discount = Ext::new(1.0);
            let mut tail = Ext::new(0.0);
            for &cashflow in &cashflows[1..] {
                let increment = match graph {
                    ExtFGraph::ForwardAddProductDiscountTailFinal => discount.mul(rate),
                    ExtFGraph::ForwardAddProductDiscountTailFinalCommuted => rate.mul(discount),
                    _ => unreachable!(),
                }
                .store(bit(4));
                discount = discount.add(increment).store(bit(3));
                let term = Ext::new(cashflow).div(discount).store(bit(5));
                tail = tail.add(term).store(bit(6));
            }
            Ext::new(cashflows[0]).add(tail).store(bit(8))
        }
        ExtFGraph::ForwardAddProductWorksheetNpvMulW
        | ExtFGraph::ForwardAddProductWorksheetNpvDivV => {
            let rate = w.sub(Ext::new(1.0)).store(bit(7));
            let mut discount = Ext::new(1.0);
            let mut npv = Ext::new(0.0);
            for &cashflow in cashflows {
                let increment = discount.mul(rate).store(bit(4));
                discount = discount.add(increment).store(bit(3));
                let term = Ext::new(cashflow).div(discount).store(bit(5));
                npv = npv.add(term).store(bit(6));
            }
            let npv = npv.store(bit(8));
            match graph {
                ExtFGraph::ForwardAddProductWorksheetNpvMulW => npv.mul(w),
                ExtFGraph::ForwardAddProductWorksheetNpvDivV => npv.div(v),
                _ => unreachable!(),
            }
            .store(bit(9))
        }
    };
    match graph {
        ExtFGraph::ForwardAddProductDiscount
        | ExtFGraph::ForwardAddProductDiscountCommuted
        | ExtFGraph::ForwardAddProductDiscountTailFinal
        | ExtFGraph::ForwardAddProductDiscountTailFinalCommuted => value.store(bit(8)),
        ExtFGraph::ForwardRepeatedDivisionTailFinal
        | ExtFGraph::ForwardRepeatedMultiplyTailFinal
        | ExtFGraph::ForwardGrowingDiscountTailFinal
        | ExtFGraph::ForwardGrowingDiscountDivideVTailFinal => value.store(bit(8)),
        ExtFGraph::ReverseTailAddThenDivide | ExtFGraph::ReverseTailAddThenMultiply => {
            value.store(bit(7))
        }
        ExtFGraph::ReverseWorksheetNpvMulW | ExtFGraph::ReverseWorksheetNpvDivV => {
            value.store(bit(7))
        }
        ExtFGraph::ReverseWorksheetTailCompose | ExtFGraph::ReverseWorksheetTailComposeSnap => {
            value.store(bit(7))
        }
        ExtFGraph::ForwardAddProductWorksheetNpvMulW
        | ExtFGraph::ForwardAddProductWorksheetNpvDivV => value.store(bit(10)),
        _ => value.store(bit(6)),
    }
}

fn f_mask_limit(graph: ExtFGraph) -> u16 {
    match graph {
        ExtFGraph::ForwardAddProductDiscount
        | ExtFGraph::ForwardAddProductDiscountCommuted
        | ExtFGraph::ForwardAddProductDiscountTailFinal
        | ExtFGraph::ForwardAddProductDiscountTailFinalCommuted
        | ExtFGraph::ForwardRepeatedDivisionTailFinal
        | ExtFGraph::ForwardRepeatedMultiplyTailFinal
        | ExtFGraph::ForwardGrowingDiscountTailFinal
        | ExtFGraph::ForwardGrowingDiscountDivideVTailFinal => 1 << 9,
        ExtFGraph::ForwardAddProductWorksheetNpvMulW
        | ExtFGraph::ForwardAddProductWorksheetNpvDivV => 1 << 11,
        _ => 1 << 8,
    }
}

fn simulate_ext(obs: &Obs, mask: u16, graph: ExtFGraph, w_graph: WGraph) -> Option<f64> {
    // Schedule staging fixed to the 132/229 champion from fit_irr_stores:
    // stored v0 input sum and quotient; h=-1e-3; unspilled v+h and f1-f0;
    // stored h/den; unspilled f0*(h/den); stored v update; stored reciprocal
    // at publication. At least two steps, apply-last, |dv|<1e-7.
    let mut v = Ext::new(1.0)
        .div(Ext::new(1.0).add(Ext::new(obs.guess)).store(true))
        .store(true);
    let mut f0 = ext_f_eval(&obs.cashflows, v, mask, graph, w_graph).store(true);
    if f0.to_f64() == 0.0 {
        return Some(obs.guess);
    }
    let h = Ext::new(-1e-3);
    for step in 0..20 {
        let f1 = ext_f_eval(&obs.cashflows, v.add(h), mask, graph, w_graph).store(true);
        let den = f1.sub(f0);
        if den.to_f64() == 0.0 {
            return None;
        }
        let slope_inverse = h.div(den).store(true);
        let dv = f0.mul(slope_inverse);
        if step >= 1 && dv.to_f64().abs() < 1e-7 {
            v = v.sub(dv).store(true);
            break;
        }
        v = v.sub(dv).store(true);
        f0 = ext_f_eval(&obs.cashflows, v, mask, graph, w_graph).store(true);
    }
    Some(Ext::new(1.0).div(v).store(true).sub(Ext::new(1.0)).to_f64())
}

fn race_extended_missing_graph(observations: &[Obs]) {
    let graphs = [
        ExtFGraph::ForwardRepeatedDivision,
        ExtFGraph::ForwardRepeatedDivisionTailFinal,
        ExtFGraph::ForwardRepeatedMultiplyTailFinal,
        ExtFGraph::ReverseHornerDivision,
        ExtFGraph::ReverseTailAddThenDivide,
        ExtFGraph::ReverseTailAddThenMultiply,
        ExtFGraph::ReverseWorksheetNpvMulW,
        ExtFGraph::ReverseWorksheetNpvDivV,
        ExtFGraph::ReverseWorksheetTailCompose,
        ExtFGraph::ReverseWorksheetTailComposeSnap,
        ExtFGraph::ReverseHornerMultiply,
        ExtFGraph::ForwardGrowingDiscount,
        ExtFGraph::ForwardGrowingDiscountDivideV,
        ExtFGraph::ForwardGrowingDiscountTailFinal,
        ExtFGraph::ForwardGrowingDiscountDivideVTailFinal,
        ExtFGraph::ForwardRecomputedPowerW,
        ExtFGraph::ForwardRecomputedPowerDivideV,
        ExtFGraph::ForwardTermRepeatedDivision,
        ExtFGraph::ForwardTermRepeatedMultiply,
        ExtFGraph::ForwardAddProductDiscount,
        ExtFGraph::ForwardAddProductDiscountCommuted,
        ExtFGraph::ForwardAddProductDiscountTailFinal,
        ExtFGraph::ForwardAddProductDiscountTailFinalCommuted,
        ExtFGraph::ForwardAddProductWorksheetNpvMulW,
        ExtFGraph::ForwardAddProductWorksheetNpvDivV,
    ];
    let w_graphs = [
        WGraph::ReciprocalV,
        WGraph::ReciprocalMinusOnePlusOne,
        WGraph::OneMinusVOverVPlusOne,
    ];
    let mut results = Vec::new();
    for graph in graphs {
        for w_graph in w_graphs {
            for mask in 0u16..(1 << 8) {
                let exact = observations
                    .iter()
                    .filter(|obs| {
                        simulate_ext(obs, mask, graph, w_graph)
                            .is_some_and(|got| got.to_bits() == obs.want)
                    })
                    .count();
                results.push((exact, graph, w_graph, mask));
            }
        }
    }
    results.sort_by(|left, right| right.0.cmp(&left.0));
    println!("-- extended missing-graph race --");
    for (exact, graph, w_graph, mask) in results.iter().take(24) {
        println!(
            "{exact:3}/{} {graph:?} {w_graph:?} mask={mask:08b}",
            observations.len()
        );
    }
    println!("-- best by extended f-graph --");
    for graph in graphs {
        let best = results
            .iter()
            .filter(|(_, candidate, _, _)| {
                std::mem::discriminant(candidate) == std::mem::discriminant(&graph)
            })
            .max_by_key(|entry| entry.0)
            .unwrap();
        println!(
            "{:3}/{} {:?} {:?} mask={:08b}",
            best.0,
            observations.len(),
            best.1,
            best.2,
            best.3
        );
    }
}

#[derive(Clone, Copy, Debug)]
struct ExtScheduleConfig {
    mask: u16,
    h_rule: HRule,
    h_abs: f64,
    assoc: UpdateAssoc,
    tolerance: f64,
    min_steps_before_stop: usize,
    cap: usize,
    relative_tolerance: bool,
    stop_on_fzero: bool,
}

fn ext_h_for(f0: Ext, rule: HRule, magnitude: f64) -> Ext {
    let signed = match rule {
        HRule::Positive => magnitude,
        HRule::Negative => -magnitude,
        HRule::WithFSign => magnitude.copysign(f0.to_f64()),
        HRule::AgainstFSign => -magnitude.copysign(f0.to_f64()),
    };
    Ext::new(signed)
}

fn simulate_ext_schedule(obs: &Obs, config: ExtScheduleConfig) -> Option<f64> {
    let bit = |index: u16| config.mask & (1 << index) != 0;
    let mut v = Ext::new(1.0)
        .div(Ext::new(1.0).add(Ext::new(obs.guess)).store(bit(0)))
        .store(bit(1));
    let f_mask = 0b0000_1000; // stored growing discount; extended term/add
    let mut f0 = ext_f_eval(
        &obs.cashflows,
        v,
        f_mask,
        ExtFGraph::ForwardGrowingDiscount,
        WGraph::ReciprocalV,
    )
    .store(bit(2));
    if f0.to_f64() == 0.0 {
        return Some(obs.guess);
    }

    for step in 0..config.cap {
        let h = ext_h_for(f0, config.h_rule, config.h_abs);
        let vh = v.add(h).store(bit(3));
        let f1 = ext_f_eval(
            &obs.cashflows,
            vh,
            f_mask,
            ExtFGraph::ForwardGrowingDiscount,
            WGraph::ReciprocalV,
        )
        .store(bit(2));
        let den = f1.sub(f0).store(bit(4));
        if den.to_f64() == 0.0 {
            return None;
        }
        let dv = match config.assoc {
            UpdateAssoc::FTimesHOverDen => f0.mul(h).store(bit(5)).div(den).store(bit(6)),
            UpdateAssoc::FTimesHOverDenGrouped => f0.mul(h.div(den).store(bit(5))).store(bit(6)),
            UpdateAssoc::FOverSlope => f0.div(den.div(h).store(bit(5))).store(bit(6)),
        };
        let limit = if config.relative_tolerance {
            config.tolerance * v.to_f64().abs()
        } else {
            config.tolerance
        };
        if step + 1 >= config.min_steps_before_stop && dv.to_f64().abs() < limit {
            v = v.sub(dv).store(bit(7));
            break;
        }
        v = v.sub(dv).store(bit(7));
        f0 = ext_f_eval(
            &obs.cashflows,
            v,
            f_mask,
            ExtFGraph::ForwardGrowingDiscount,
            WGraph::ReciprocalV,
        )
        .store(bit(2));
        if config.stop_on_fzero && f0.to_f64() == 0.0 {
            break;
        }
    }
    Some(
        Ext::new(1.0)
            .div(v)
            .store(bit(8))
            .sub(Ext::new(1.0))
            .store(bit(9))
            .to_f64(),
    )
}

fn race_extended_schedule(observations: &[Obs]) {
    let baseline = ExtScheduleConfig {
        mask: 0,
        h_rule: HRule::Negative,
        h_abs: 1e-3,
        assoc: UpdateAssoc::FTimesHOverDenGrouped,
        tolerance: 1e-7,
        min_steps_before_stop: 2,
        cap: 20,
        relative_tolerance: false,
        stop_on_fzero: true,
    };
    let mut mask_results = Vec::new();
    for mask in 0u16..(1 << 10) {
        let config = ExtScheduleConfig { mask, ..baseline };
        let exact = observations
            .iter()
            .filter(|obs| {
                simulate_ext_schedule(obs, config).is_some_and(|got| got.to_bits() == obs.want)
            })
            .count();
        mask_results.push((exact, config));
    }
    mask_results.sort_by(|left, right| right.0.cmp(&left.0));
    println!("-- extended schedule-mask race --");
    for (exact, config) in mask_results.iter().take(16) {
        println!("{exact:3}/{} {config:?}", observations.len());
    }

    let champion_mask = mask_results[0].1.mask;
    let mut axis_results = Vec::new();
    for h_rule in [
        HRule::Positive,
        HRule::Negative,
        HRule::WithFSign,
        HRule::AgainstFSign,
    ] {
        for h_abs in [1e-2, 1e-3, 1e-4, 1e-5, 1e-6] {
            for assoc in [
                UpdateAssoc::FTimesHOverDen,
                UpdateAssoc::FTimesHOverDenGrouped,
                UpdateAssoc::FOverSlope,
            ] {
                for tolerance in [1e-5, 1e-6, 1e-7, 1e-8, 1e-9] {
                    for min_steps_before_stop in [1usize, 2, 3] {
                        for cap in [20usize, 40, 100] {
                            for relative_tolerance in [false, true] {
                                for stop_on_fzero in [false, true] {
                                    let config = ExtScheduleConfig {
                                        mask: champion_mask,
                                        h_rule,
                                        h_abs,
                                        assoc,
                                        tolerance,
                                        min_steps_before_stop,
                                        cap,
                                        relative_tolerance,
                                        stop_on_fzero,
                                    };
                                    let exact = observations
                                        .iter()
                                        .filter(|obs| {
                                            simulate_ext_schedule(obs, config)
                                                .is_some_and(|got| got.to_bits() == obs.want)
                                        })
                                        .count();
                                    axis_results.push((exact, config));
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    axis_results.sort_by(|left, right| right.0.cmp(&left.0));
    println!("-- extended schedule-axis race --");
    for (exact, config) in axis_results.iter().take(24) {
        println!("{exact:3}/{} {config:?}", observations.len());
    }

    let champion = axis_results[0].1;
    let mut categories = std::collections::BTreeMap::<String, (usize, usize)>::new();
    let mut misses = Vec::new();
    for obs in observations {
        let got = simulate_ext_schedule(obs, champion);
        let round = if obs.id.contains("-l3-") {
            "r3"
        } else if obs.id.contains("-l2-") {
            "r2"
        } else if obs.id.contains("lad") {
            "r1"
        } else {
            "r0"
        };
        let key = format!("{round}/n{}", obs.cashflows.len());
        let entry = categories.entry(key).or_default();
        entry.1 += 1;
        if got.is_some_and(|value| value.to_bits() == obs.want) {
            entry.0 += 1;
        } else if let Some(value) = got {
            misses.push((obs.id.clone(), value.to_bits() as i64 - obs.want as i64));
        }
    }
    println!("-- extended champion anatomy {champion:?} --");
    for (category, (exact, count)) in categories {
        println!("{category}: {exact}/{count}");
    }
    for (id, delta) in misses.iter().take(60) {
        println!("MISS {id:24} {delta:+} ULP");
    }
}

fn report_ladder_contractions(observations: &[Obs]) {
    use std::collections::BTreeMap;
    let mut pairs = BTreeMap::<(String, u32), (Option<&Obs>, Option<&Obs>)>::new();
    for obs in observations {
        let Some((prefix, suffix)) = obs.id.rsplit_once('-') else {
            continue;
        };
        let (side, power_text) = suffix.split_at(1);
        let Ok(power) = power_text.parse::<u32>() else {
            continue;
        };
        if !matches!(power, 28 | 34 | 40 | 46) {
            continue;
        }
        let shape = prefix.split('-').nth(2).unwrap_or("unknown").to_owned();
        let entry = pairs.entry((shape, power)).or_default();
        match side {
            "m" => entry.0 = Some(obs),
            "p" => entry.1 = Some(obs),
            _ => {}
        }
    }
    println!("-- symmetric local-ladder contractions --");
    for ((shape, power), pair) in pairs {
        let (Some(minus), Some(plus)) = pair else {
            continue;
        };
        let in_span = plus.guess - minus.guess;
        let out_span = f64::from_bits(plus.want) - f64::from_bits(minus.want);
        println!("{shape} j{power:02}: {:.12e}", out_span / in_span);
    }
}

fn report_curvature_scale_test(observations: &[Obs]) {
    use std::collections::BTreeMap;
    let mut shapes = BTreeMap::<String, (&Obs, Option<&Obs>, Option<&Obs>)>::new();
    for obs in observations {
        let shape = obs.id.split('-').nth(2).unwrap_or("unknown").to_owned();
        if obs.id.ends_with("-root") {
            shapes.insert(shape, (obs, None, None));
        }
    }
    for obs in observations {
        let shape = obs.id.split('-').nth(2).unwrap_or("unknown").to_owned();
        let Some(entry) = shapes.get_mut(&shape) else {
            continue;
        };
        if obs.id.ends_with("-m34") {
            entry.1 = Some(obs);
        } else if obs.id.ends_with("-p34") {
            entry.2 = Some(obs);
        }
    }
    println!("-- j34 curvature-implied h by objective scaling --");
    for (shape, (root, minus, plus)) in shapes {
        let (Some(minus), Some(plus)) = (minus, plus) else {
            continue;
        };
        let rate = root.guess;
        let v = 1.0 / (1.0 + rate);
        let mut fp = 0.0;
        let mut fpp = 0.0;
        for (index, cashflow) in root.cashflows.iter().copied().enumerate().skip(1) {
            let degree = index as i32;
            fp += f64::from(degree) * cashflow * v.powi(degree - 1);
            if degree >= 2 {
                fpp += f64::from(degree * (degree - 1)) * cashflow * v.powi(degree - 2);
            }
        }
        let contraction = ((f64::from_bits(plus.want) - f64::from_bits(minus.want))
            / (plus.guess - minus.guess))
            .abs();
        let step_lambda = contraction.sqrt();
        let base_curvature = fpp / (2.0 * fp);
        let implied = |power: f64| step_lambda / (base_curvature + power / v).abs();
        println!(
            "{shape}: h[f]={:.9e} h[v*f]={:.9e} h[f/v]={:.9e}",
            implied(0.0),
            implied(1.0),
            implied(-1.0)
        );
    }
}

#[derive(Clone, Copy, Debug)]
struct TwoStepFConfig {
    mask: u16,
    graph: ExtFGraph,
    w_graph: WGraph,
}

#[derive(Clone, Copy, Debug)]
enum HConstant {
    Binary64,
    Binary32,
    ExtendedRatio,
    StoredRatio,
}

#[derive(Clone, Copy, Debug)]
struct TwoStepSchedule {
    mask: u16,
    h_rule: HRule,
    h_constant: HConstant,
    update: TwoStepUpdate,
    publish: Publish,
}

#[derive(Clone, Copy, Debug)]
enum TwoStepUpdate {
    FromBaseProductQuotient,
    FromBaseGrouped,
    FromBaseRatioTimesH,
    FromBaseSlope,
    FromProbeProductQuotient,
    FromProbeGrouped,
    FromProbeRatioTimesH,
    CrossProducts,
    CrossBaseDen,
}

#[derive(Clone, Copy, Debug)]
struct CandidateScore {
    exact: usize,
    ulp_sum: u128,
    max_ulp: u128,
}

fn ordered_bits(bits: u64) -> u64 {
    if bits >> 63 == 0 {
        bits | (1_u64 << 63)
    } else {
        !bits
    }
}

fn score_better(left: CandidateScore, right: CandidateScore) -> bool {
    (
        left.exact,
        std::cmp::Reverse(left.max_ulp),
        std::cmp::Reverse(left.ulp_sum),
    ) > (
        right.exact,
        std::cmp::Reverse(right.max_ulp),
        std::cmp::Reverse(right.ulp_sum),
    )
}

fn two_step_h(f0: Ext, schedule: TwoStepSchedule) -> Ext {
    let magnitude = match schedule.h_constant {
        HConstant::Binary64 => Ext::new(1e-3),
        HConstant::Binary32 => Ext::new(f64::from(1e-3_f32)),
        HConstant::ExtendedRatio => Ext::new(1.0).div(Ext::new(1000.0)),
        HConstant::StoredRatio => Ext::new(1.0).div(Ext::new(1000.0)).store(true),
    };
    match schedule.h_rule {
        HRule::Positive => magnitude,
        HRule::Negative => Ext::new(0.0).sub(magnitude),
        HRule::WithFSign => {
            if f0.to_f64().is_sign_negative() {
                Ext::new(0.0).sub(magnitude)
            } else {
                magnitude
            }
        }
        HRule::AgainstFSign => {
            if f0.to_f64().is_sign_negative() {
                magnitude
            } else {
                Ext::new(0.0).sub(magnitude)
            }
        }
    }
}

fn simulate_two_step(
    obs: &Obs,
    f_config: TwoStepFConfig,
    schedule: TwoStepSchedule,
) -> Option<f64> {
    let bit = |index: u16| schedule.mask & (1 << index) != 0;
    let mut v = Ext::new(1.0)
        .div(Ext::new(1.0).add(Ext::new(obs.guess)).store(bit(0)))
        .store(bit(1));
    let mut f0 = ext_f_eval(
        &obs.cashflows,
        v,
        f_config.mask,
        f_config.graph,
        f_config.w_graph,
    )
    .store(bit(2));
    if f0.to_f64() == 0.0 {
        return Some(obs.guess);
    }

    for step in 0..2 {
        let h = two_step_h(f0, schedule);
        let vh = v.add(h).store(bit(3));
        let f1 = ext_f_eval(
            &obs.cashflows,
            vh,
            f_config.mask,
            f_config.graph,
            f_config.w_graph,
        )
        .store(bit(2));
        let den = f1.sub(f0).store(bit(4));
        if den.to_f64() == 0.0 || !den.to_f64().is_finite() {
            return None;
        }
        v = match schedule.update {
            TwoStepUpdate::FromBaseProductQuotient => {
                let dv = f0.mul(h).store(bit(5)).div(den).store(bit(6));
                v.sub(dv).store(bit(7))
            }
            TwoStepUpdate::FromBaseGrouped => {
                let dv = f0.mul(h.div(den).store(bit(5))).store(bit(6));
                v.sub(dv).store(bit(7))
            }
            TwoStepUpdate::FromBaseRatioTimesH => {
                let dv = f0.div(den).store(bit(5)).mul(h).store(bit(6));
                v.sub(dv).store(bit(7))
            }
            TwoStepUpdate::FromBaseSlope => {
                let dv = f0.div(den.div(h).store(bit(5))).store(bit(6));
                v.sub(dv).store(bit(7))
            }
            TwoStepUpdate::FromProbeProductQuotient => {
                let correction = f1.mul(h).store(bit(5)).div(den).store(bit(6));
                vh.sub(correction).store(bit(7))
            }
            TwoStepUpdate::FromProbeGrouped => {
                let correction = f1.mul(h.div(den).store(bit(5))).store(bit(6));
                vh.sub(correction).store(bit(7))
            }
            TwoStepUpdate::FromProbeRatioTimesH => {
                let correction = f1.div(den).store(bit(5)).mul(h).store(bit(6));
                vh.sub(correction).store(bit(7))
            }
            TwoStepUpdate::CrossProducts => {
                let left = v.mul(f1).store(bit(5));
                let right = vh.mul(f0).store(bit(5));
                left.sub(right).store(bit(6)).div(den).store(bit(7))
            }
            TwoStepUpdate::CrossBaseDen => {
                let left = v.mul(den).store(bit(5));
                let right = f0.mul(h).store(bit(5));
                left.sub(right).store(bit(6)).div(den).store(bit(7))
            }
        };
        if step == 0 {
            f0 = ext_f_eval(
                &obs.cashflows,
                v,
                f_config.mask,
                f_config.graph,
                f_config.w_graph,
            )
            .store(bit(2));
        }
    }

    let published = match schedule.publish {
        Publish::ReciprocalMinusOne => Ext::new(1.0)
            .div(v)
            .store(bit(8))
            .sub(Ext::new(1.0))
            .store(bit(9)),
        Publish::OneMinusVOverV => Ext::new(1.0).sub(v).store(bit(8)).div(v).store(bit(9)),
    };
    Some(published.to_f64())
}

fn score_two_step(
    observations: &[&Obs],
    f_config: TwoStepFConfig,
    schedule: TwoStepSchedule,
) -> CandidateScore {
    let mut score = CandidateScore {
        exact: 0,
        ulp_sum: 0,
        max_ulp: 0,
    };
    for obs in observations {
        let distance = if let Some(got) = simulate_two_step(obs, f_config, schedule) {
            let left = ordered_bits(got.to_bits()) as i128;
            let right = ordered_bits(obs.want) as i128;
            (left - right).unsigned_abs().min(1_000_000_000_000)
        } else {
            1_000_000_000_000
        };
        score.exact += usize::from(distance == 0);
        score.ulp_sum += distance;
        score.max_ulp = score.max_ulp.max(distance);
    }
    score
}

#[allow(dead_code)]
fn all_f_configs() -> Vec<TwoStepFConfig> {
    let graphs = [
        ExtFGraph::ForwardRepeatedDivision,
        ExtFGraph::ForwardRepeatedDivisionTailFinal,
        ExtFGraph::ForwardRepeatedMultiplyTailFinal,
        ExtFGraph::ReverseHornerDivision,
        ExtFGraph::ReverseTailAddThenDivide,
        ExtFGraph::ReverseTailAddThenMultiply,
        ExtFGraph::ReverseWorksheetNpvMulW,
        ExtFGraph::ReverseWorksheetNpvDivV,
        ExtFGraph::ReverseWorksheetTailCompose,
        ExtFGraph::ReverseWorksheetTailComposeSnap,
        ExtFGraph::ReverseHornerMultiply,
        ExtFGraph::ForwardGrowingDiscount,
        ExtFGraph::ForwardGrowingDiscountDivideV,
        ExtFGraph::ForwardGrowingDiscountTailFinal,
        ExtFGraph::ForwardGrowingDiscountDivideVTailFinal,
        ExtFGraph::ForwardRecomputedPowerW,
        ExtFGraph::ForwardRecomputedPowerDivideV,
        ExtFGraph::ForwardTermRepeatedDivision,
        ExtFGraph::ForwardTermRepeatedMultiply,
        ExtFGraph::ForwardAddProductDiscount,
        ExtFGraph::ForwardAddProductDiscountCommuted,
        ExtFGraph::ForwardAddProductDiscountTailFinal,
        ExtFGraph::ForwardAddProductDiscountTailFinalCommuted,
        ExtFGraph::ForwardAddProductWorksheetNpvMulW,
        ExtFGraph::ForwardAddProductWorksheetNpvDivV,
    ];
    let w_graphs = [
        WGraph::ReciprocalV,
        WGraph::ReciprocalMinusOnePlusOne,
        WGraph::OneMinusVOverVPlusOne,
    ];
    let mut configs = Vec::new();
    for graph in graphs {
        for w_graph in w_graphs {
            for mask in 0..f_mask_limit(graph) {
                configs.push(TwoStepFConfig {
                    mask,
                    graph,
                    w_graph,
                });
            }
        }
    }
    configs
}

fn race_two_step_coordinate(observations: &[Obs]) {
    let validation = observations
        .iter()
        .filter(|obs| {
            ["m28", "p28", "m34", "p34"]
                .iter()
                .any(|suffix| obs.id.ends_with(suffix))
        })
        .collect::<Vec<_>>();
    let local = validation.clone();
    println!(
        "-- two-step coordinate race: {} search / {} validation rows --",
        local.len(),
        validation.len()
    );
    let mut f_configs = vec![TwoStepFConfig {
        mask: 105,
        graph: ExtFGraph::ForwardAddProductDiscountTailFinal,
        w_graph: WGraph::ReciprocalV,
    }];
    for graph in [
        ExtFGraph::ReverseWorksheetNpvMulW,
        ExtFGraph::ReverseWorksheetNpvDivV,
        ExtFGraph::ReverseWorksheetTailCompose,
        ExtFGraph::ReverseWorksheetTailComposeSnap,
        ExtFGraph::ForwardAddProductWorksheetNpvMulW,
        ExtFGraph::ForwardAddProductWorksheetNpvDivV,
    ] {
        for w_graph in [
            WGraph::ReciprocalV,
            WGraph::ReciprocalMinusOnePlusOne,
            WGraph::OneMinusVOverVPlusOne,
        ] {
            for mask in 0..f_mask_limit(graph) {
                f_configs.push(TwoStepFConfig {
                    mask,
                    graph,
                    w_graph,
                });
            }
        }
    }
    let seed_schedules = [
        TwoStepSchedule {
            mask: 420,
            h_rule: HRule::Negative,
            h_constant: HConstant::Binary64,
            update: TwoStepUpdate::FromBaseGrouped,
            publish: Publish::ReciprocalMinusOne,
        },
        TwoStepSchedule {
            mask: 448,
            h_rule: HRule::Negative,
            h_constant: HConstant::Binary32,
            update: TwoStepUpdate::CrossProducts,
            publish: Publish::ReciprocalMinusOne,
        },
        TwoStepSchedule {
            mask: 265,
            h_rule: HRule::Positive,
            h_constant: HConstant::Binary64,
            update: TwoStepUpdate::FromBaseProductQuotient,
            publish: Publish::ReciprocalMinusOne,
        },
        TwoStepSchedule {
            mask: 1023,
            h_rule: HRule::Negative,
            h_constant: HConstant::Binary64,
            update: TwoStepUpdate::FromBaseSlope,
            publish: Publish::ReciprocalMinusOne,
        },
        TwoStepSchedule {
            mask: 0,
            h_rule: HRule::Positive,
            h_constant: HConstant::ExtendedRatio,
            update: TwoStepUpdate::FromProbeProductQuotient,
            publish: Publish::OneMinusVOverV,
        },
        TwoStepSchedule {
            mask: 420,
            h_rule: HRule::WithFSign,
            h_constant: HConstant::Binary64,
            update: TwoStepUpdate::FromBaseGrouped,
            publish: Publish::ReciprocalMinusOne,
        },
        TwoStepSchedule {
            mask: 420,
            h_rule: HRule::AgainstFSign,
            h_constant: HConstant::Binary64,
            update: TwoStepUpdate::FromBaseGrouped,
            publish: Publish::ReciprocalMinusOne,
        },
    ];

    let mut ranked_f = Vec::new();
    for f_config in f_configs.iter().copied() {
        let mut best = CandidateScore {
            exact: 0,
            ulp_sum: u128::MAX,
            max_ulp: u128::MAX,
        };
        let mut best_schedule = seed_schedules[0];
        for schedule in seed_schedules {
            let score = score_two_step(&local, f_config, schedule);
            if score_better(score, best) {
                best = score;
                best_schedule = schedule;
            }
        }
        ranked_f.push((best, f_config, best_schedule));
    }
    ranked_f.sort_by(|left, right| {
        right
            .0
            .exact
            .cmp(&left.0.exact)
            .then_with(|| left.0.max_ulp.cmp(&right.0.max_ulp))
            .then_with(|| left.0.ulp_sum.cmp(&right.0.ulp_sum))
    });
    println!("seeded f-graph leaders:");
    for (score, f_config, schedule) in ranked_f.iter().take(12) {
        println!("{score:?} f={f_config:?} seed_schedule={schedule:?}");
    }
    println!("reverse worksheet tail-compose comparison:");
    for graph in [
        ExtFGraph::ReverseWorksheetTailCompose,
        ExtFGraph::ReverseWorksheetTailComposeSnap,
    ] {
        let best = ranked_f
            .iter()
            .filter(|(_, config, _)| {
                std::mem::discriminant(&config.graph) == std::mem::discriminant(&graph)
            })
            .min_by(|left, right| {
                right
                    .0
                    .exact
                    .cmp(&left.0.exact)
                    .then_with(|| left.0.max_ulp.cmp(&right.0.max_ulp))
                    .then_with(|| left.0.ulp_sum.cmp(&right.0.ulp_sum))
            })
            .unwrap();
        println!("{:?} f={:?} schedule={:?}", best.0, best.1, best.2);
    }
    if std::env::var_os("W109_IRR_FULL_SCHEDULE_RACE").is_none() {
        return;
    }

    let shortlist = ranked_f
        .iter()
        .take(8)
        .map(|entry| entry.1)
        .collect::<Vec<_>>();
    let mut schedule_leaders = Vec::new();
    for f_config in shortlist {
        for mask in 0..(1 << 10) {
            for h_rule in [
                HRule::Positive,
                HRule::Negative,
                HRule::WithFSign,
                HRule::AgainstFSign,
            ] {
                for h_constant in [
                    HConstant::Binary64,
                    HConstant::Binary32,
                    HConstant::ExtendedRatio,
                    HConstant::StoredRatio,
                ] {
                    for update in [
                        TwoStepUpdate::FromBaseProductQuotient,
                        TwoStepUpdate::FromBaseGrouped,
                        TwoStepUpdate::FromBaseRatioTimesH,
                        TwoStepUpdate::FromBaseSlope,
                        TwoStepUpdate::FromProbeProductQuotient,
                        TwoStepUpdate::FromProbeGrouped,
                        TwoStepUpdate::FromProbeRatioTimesH,
                        TwoStepUpdate::CrossProducts,
                        TwoStepUpdate::CrossBaseDen,
                    ] {
                        for publish in [Publish::ReciprocalMinusOne] {
                            let schedule = TwoStepSchedule {
                                mask,
                                h_rule,
                                h_constant,
                                update,
                                publish,
                            };
                            let score = score_two_step(&local, f_config, schedule);
                            schedule_leaders.push((score, f_config, schedule));
                        }
                    }
                }
            }
        }
    }
    schedule_leaders.sort_by(|left, right| {
        right
            .0
            .exact
            .cmp(&left.0.exact)
            .then_with(|| left.0.max_ulp.cmp(&right.0.max_ulp))
            .then_with(|| left.0.ulp_sum.cmp(&right.0.ulp_sum))
    });
    println!("joint f/schedule leaders:");
    for (score, f_config, schedule) in schedule_leaders.iter().take(24) {
        let validation_score = score_two_step(&validation, *f_config, *schedule);
        println!(
            "search={score:?} validation={validation_score:?} f={f_config:?} schedule={schedule:?}"
        );
    }
    let (_, champion_f, champion_schedule) = schedule_leaders[0];
    let mut anatomy = std::collections::BTreeMap::<String, (usize, usize)>::new();
    println!("two-step champion misses:");
    for obs in &validation {
        let shape = obs.id.split('-').nth(2).unwrap_or("unknown").to_owned();
        let entry = anatomy.entry(shape).or_default();
        entry.1 += 1;
        let got = simulate_two_step(obs, champion_f, champion_schedule).unwrap();
        let delta = ordered_bits(got.to_bits()) as i128 - ordered_bits(obs.want) as i128;
        if delta == 0 {
            entry.0 += 1;
        } else {
            println!("{} {delta:+} ULP", obs.id);
        }
    }
    println!("two-step champion anatomy:");
    for (shape, (exact, count)) in anatomy {
        println!("{shape}: {exact}/{count}");
    }
}

fn main() {
    let all_observations = load();
    report_ladder_contractions(&all_observations);
    report_curvature_scale_test(&all_observations);
    race_two_step_coordinate(&all_observations);
    if std::env::var_os("W109_IRR_LEGACY_RACES").is_none() {
        return;
    }
    let observations = &all_observations;
    println!("{} frozen IRR observations", observations.len());
    let graphs = [
        FGraph::HornerMul,
        FGraph::ReverseDivRecipV,
        FGraph::ReverseTailAddThenDivide,
        FGraph::ReverseDivRoundTripRate,
        FGraph::ReverseDivRoundTripRateFused,
        FGraph::ForwardDiscount,
    ];
    let h_rules = [
        HRule::Positive,
        HRule::Negative,
        HRule::WithFSign,
        HRule::AgainstFSign,
    ];
    let assocs = [
        UpdateAssoc::FTimesHOverDen,
        UpdateAssoc::FTimesHOverDenGrouped,
        UpdateAssoc::FOverSlope,
    ];
    let publications = [Publish::ReciprocalMinusOne, Publish::OneMinusVOverV];
    let tolerances = [1e-7, 1e-8, 1e-9];
    let mut results = Vec::new();

    for graph in graphs {
        for h_rule in h_rules {
            for h_abs in [1e-3] {
                for assoc in assocs {
                    for publish in publications {
                        for tolerance in tolerances {
                            for min_steps_before_stop in [1usize, 2, 3] {
                                for cap in [20usize, 40, 100] {
                                    for relative_tolerance in [false, true] {
                                        for stop_on_fzero in [false, true] {
                                            let config = Config {
                                                graph,
                                                h_rule,
                                                h_abs,
                                                assoc,
                                                publish,
                                                tolerance,
                                                min_steps_before_stop,
                                                cap,
                                                relative_tolerance,
                                                stop_on_fzero,
                                            };
                                            let exact = observations
                                                .iter()
                                                .filter(|obs| {
                                                    simulate(obs, config).is_some_and(|got| {
                                                        got.to_bits() == obs.want
                                                    })
                                                })
                                                .count();
                                            results.push((exact, config));
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

    results.sort_by(|left, right| right.0.cmp(&left.0));
    for (exact, config) in results.iter().take(24) {
        println!("{exact:3}/{} {config:?}", observations.len());
    }

    let (exact, champion) = results[0];
    println!("-- champion {exact}/{} {champion:?} --", observations.len());
    let mut categories = std::collections::BTreeMap::<String, (usize, usize)>::new();
    let mut shown = 0usize;
    for obs in observations {
        let got = simulate(obs, champion);
        let category =
            if obs.id.contains("lad") || obs.id.contains("-l2-") || obs.id.contains("-l3-") {
                format!("ladder-n{}", obs.cashflows.len())
            } else {
                format!("sweep-n{}", obs.cashflows.len())
            };
        let entry = categories.entry(category).or_default();
        entry.1 += 1;
        if got.is_some_and(|value| value.to_bits() == obs.want) {
            entry.0 += 1;
        } else if shown < 40 {
            shown += 1;
            match got {
                Some(value) => println!(
                    "MISS {:24} got-want {:+} ULP",
                    obs.id,
                    value.to_bits() as i64 - obs.want as i64
                ),
                None => println!("MISS {:24} non-numeric", obs.id),
            }
        }
    }
    for (category, (hit, count)) in categories {
        println!("{category}: {hit}/{count}");
    }
    race_extended_missing_graph(&observations);
    race_extended_schedule(&observations);
}
