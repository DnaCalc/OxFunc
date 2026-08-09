//! Discovery-only FV spill isolation and RATE inline-helper boundary race.
//!
//! This extends the historical `fit_fv_stores` grammar.  First it identifies
//! the worksheet-published FV graph on the paired 512-row companion.  It then
//! substitutes RATE's established raw x87 power route and races whether the
//! final FV sum is kept in PC64 through `calcFv(...) - requested_fv` or spilled
//! to binary64 at a helper-call / worksheet-publication boundary.

use oxfunc_core::excel_numeric::research as rx;
use oxfunc_core::functions::power_fn::power_kernel;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::BTreeSet;
use std::path::PathBuf;

const ROOT: &str = "../../work/w109/G6-rate";
const RATE_ROWS: usize = 256;
const FV_ROWS: usize = 512;
const CW: u16 = rx::CW_PC64_RN;

#[derive(Clone, Copy)]
struct V(rx::Ext80);

impl V {
    fn new(value: f64) -> Self {
        Self(rx::ext_from_f64(value))
    }

    fn f(self) -> f64 {
        rx::ext_to_f64(&self.0, CW)
    }

    fn st(self, store: bool) -> Self {
        if store { Self::new(self.f()) } else { self }
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

    fn neg(self) -> Self {
        V::new(0.0).sub(self)
    }
}

#[derive(Deserialize)]
struct Probe {
    id: String,
    args: Vec<String>,
}

#[derive(Deserialize)]
struct RankedProbe {
    probe: Probe,
}

#[derive(Deserialize)]
struct Batch {
    function: String,
    probes: Vec<RankedProbe>,
}

#[derive(Deserialize)]
struct Witness {
    id: String,
    args: Vec<String>,
    expected_bits: Option<String>,
    expected_error: Option<String>,
}

#[derive(Deserialize)]
struct Environment {
    excel_version: String,
    excel_build: String,
    excel_bitness: String,
    workbook_compatibility: String,
    excel_input_plumbing: String,
}

#[derive(Deserialize)]
struct OracleCache {
    mode: String,
    hits: u64,
    misses: u64,
}

#[derive(Deserialize)]
struct Runner {
    name: String,
    version: String,
}

#[derive(Deserialize)]
struct Provenance {
    schema_version: String,
    environment: Environment,
    oracle_cache: OracleCache,
    runner: Runner,
}

#[derive(Deserialize)]
struct WitnessSet {
    function: String,
    witnesses: Vec<Witness>,
    capture_provenance: Provenance,
}

#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
enum Factor {
    PowerKernel,
    RawChain,
    RawDirect,
    NativePowf,
}

impl Factor {
    const PUBLIC: [Self; 4] = [
        Self::PowerKernel,
        Self::RawChain,
        Self::RawDirect,
        Self::NativePowf,
    ];

    fn tag(self) -> &'static str {
        match self {
            Self::PowerKernel => "worksheet-power-kernel",
            Self::RawChain => "raw-x87-ln-product-exp",
            Self::RawDirect => "continuous-x87-power",
            Self::NativePowf => "native-powf-control",
        }
    }

    fn eval(self, base: f64, exponent: f64) -> f64 {
        match self {
            Self::PowerKernel => power_kernel(base, exponent).expect("positive finite FV power"),
            Self::RawChain => rx::excel_pow_chain(base, exponent),
            Self::RawDirect => rx::excel_pow_x87_direct(base, exponent),
            Self::NativePowf => base.powf(exponent),
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
struct HelperModel {
    factor: Factor,
    association: u8,
    spill_mask: u16,
}

impl HelperModel {
    fn id(self) -> String {
        format!(
            "factor-{}|assoc-{}|spill-{:09b}",
            self.factor.tag(),
            self.association,
            self.spill_mask
        )
    }
}

#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
enum Quotient {
    Divide,
    ReciprocalMultiply,
}

impl Quotient {
    const ALL: [Self; 2] = [Self::Divide, Self::ReciprocalMultiply];

    fn tag(self) -> &'static str {
        match self {
            Self::Divide => "divide",
            Self::ReciprocalMultiply => "reciprocal-multiply",
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
struct OuterModel {
    spill_mask: u16,
    derivative_quotient: Quotient,
    update_quotient: Quotient,
}

impl OuterModel {
    fn id(self) -> String {
        format!(
            "outer-spill-{:08b}|deriv-{}|update-{}",
            self.spill_mask,
            self.derivative_quotient.tag(),
            self.update_quotient.tag()
        )
    }
}

#[derive(Clone, Serialize)]
struct Score<M> {
    model: M,
    model_id: String,
    exact: usize,
    within_1_ulp: usize,
    within_4_ulp: usize,
    within_16_ulp: usize,
    max_ulp: u128,
    sum_ulp: u128,
}

#[derive(Clone, Serialize)]
struct InlineScore {
    helper: HelperModel,
    helper_id: String,
    outer: OuterModel,
    outer_id: String,
    exact: usize,
    within_1_ulp: usize,
    within_4_ulp: usize,
    within_16_ulp: usize,
    max_ulp: u128,
    sum_ulp: u128,
}

#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
enum Schedule {
    FirstStep,
    SecondStep,
    ResidualPrePublishCurrent,
    ResidualPrePublishNext,
    ResidualPreMinTwoPublishNext,
    DeltaPublishCurrent,
    DeltaPublishNext,
    NextResidualPublishNext,
    StableOrLast,
    FixedHundred,
    RootAdjacentMinResidual,
}

impl Schedule {
    const ALL: [Self; 11] = [
        Self::FirstStep,
        Self::SecondStep,
        Self::ResidualPrePublishCurrent,
        Self::ResidualPrePublishNext,
        Self::ResidualPreMinTwoPublishNext,
        Self::DeltaPublishCurrent,
        Self::DeltaPublishNext,
        Self::NextResidualPublishNext,
        Self::StableOrLast,
        Self::FixedHundred,
        Self::RootAdjacentMinResidual,
    ];

    fn tag(self) -> &'static str {
        match self {
            Self::FirstStep => "first-step",
            Self::SecondStep => "second-step",
            Self::ResidualPrePublishCurrent => "stop-abs-f-pre-publish-current",
            Self::ResidualPrePublishNext => "stop-abs-f-pre-publish-next",
            Self::ResidualPreMinTwoPublishNext => "stop-abs-f-pre-min2-publish-next",
            Self::DeltaPublishCurrent => "stop-abs-delta-publish-current",
            Self::DeltaPublishNext => "stop-abs-delta-publish-next",
            Self::NextResidualPublishNext => "stop-next-abs-f-publish-next",
            Self::StableOrLast => "iterate-until-stable-or-last",
            Self::FixedHundred => "fixed-100-steps",
            Self::RootAdjacentMinResidual => "root-adjacent-min-residual",
        }
    }
}

#[derive(Clone, Serialize)]
struct ScheduleScore {
    helper: HelperModel,
    helper_id: String,
    outer: OuterModel,
    outer_id: String,
    schedule: Schedule,
    schedule_id: String,
    exact: usize,
    within_1_ulp: usize,
    within_4_ulp: usize,
    within_16_ulp: usize,
    max_ulp: u128,
    sum_ulp: u128,
    no_result: usize,
}

#[derive(Clone, Copy)]
struct Iteration {
    current: f64,
    residual: f64,
    next: f64,
}

#[derive(Clone, Copy)]
struct FvRow {
    rate: f64,
    nper: f64,
    pmt: f64,
    pv: f64,
    ty: f64,
    want: f64,
}

#[derive(Clone, Copy)]
struct RateRow {
    x: f64,
    xh: f64,
    nper: f64,
    pmt: f64,
    pv: f64,
    requested: f64,
    ty: f64,
    want: f64,
    p0: f64,
    p1: f64,
}

fn parse_hex(value: &str) -> f64 {
    f64::from_bits(u64::from_str_radix(value.strip_prefix("0x").unwrap(), 16).unwrap())
}

fn ordered(bits: u64) -> i128 {
    if bits >> 63 == 0 {
        (bits | (1_u64 << 63)) as i128
    } else {
        (!bits) as i128
    }
}

fn bit(mask: u16, index: u32) -> bool {
    mask & (1 << index) != 0
}

fn helper_with_power(
    rate: f64,
    nper: f64,
    pmt: f64,
    pv: f64,
    ty: f64,
    power: f64,
    model: HelperModel,
) -> V {
    if rate == 0.0 {
        return V::new(pv).add(V::new(pmt).mul(V::new(nper))).neg();
    }
    let mask = model.spill_mask;
    let w = V::new(1.0).add(V::new(rate)).st(bit(mask, 1));
    // Every factor family publishes binary64 at the public/helper boundary.
    let p = V::new(power).st(bit(mask, 2));
    let pm1 = p.sub(V::new(1.0)).st(bit(mask, 3));
    let q = pm1.div(V::new(rate)).st(bit(mask, 4));
    let tf = V::new(1.0)
        .add(V::new(rate).mul(V::new(ty)))
        .st(bit(mask, 8));
    let payment = match model.association {
        0 => V::new(pmt).mul(tf).mul(q).st(bit(mask, 5)),
        1 => V::new(pmt).mul(tf.mul(q).st(bit(mask, 8))).st(bit(mask, 5)),
        2 => V::new(pmt).mul(q).st(bit(mask, 8)).mul(tf).st(bit(mask, 5)),
        3 => {
            let q2 = q.mul(tf).st(bit(mask, 8));
            V::new(pmt).mul(q2).st(bit(mask, 5))
        }
        4 => {
            let q2 = if ty != 0.0 {
                q.mul(w).st(bit(mask, 8))
            } else {
                q
            };
            V::new(pmt).mul(q2).st(bit(mask, 5))
        }
        5 => {
            let coefficient = V::new(1.0)
                .div(V::new(rate))
                .add(V::new(ty))
                .st(bit(mask, 8));
            V::new(pmt).mul(coefficient).mul(pm1).st(bit(mask, 5))
        }
        6 => {
            let coefficient = V::new(1.0)
                .div(V::new(rate))
                .add(V::new(ty))
                .st(bit(mask, 8));
            V::new(pmt)
                .mul(coefficient.mul(pm1).st(bit(mask, 8)))
                .st(bit(mask, 5))
        }
        7 => {
            let coefficient = V::new(1.0)
                .div(V::new(rate))
                .add(V::new(ty))
                .st(bit(mask, 8));
            V::new(pmt)
                .mul(pm1)
                .st(bit(mask, 8))
                .mul(coefficient)
                .st(bit(mask, 5))
        }
        8 => {
            let ordinary = pm1.div(V::new(rate)).st(bit(mask, 4));
            let due = V::new(ty).mul(pm1).st(bit(mask, 8));
            V::new(pmt)
                .mul(ordinary.add(due).st(bit(mask, 8)))
                .st(bit(mask, 5))
        }
        9 => {
            // Historical/public helper source form:
            // term = (type_factor * (P-1)) / rate; payment = pmt * term.
            let term = tf
                .mul(pm1)
                .st(bit(mask, 8))
                .div(V::new(rate))
                .st(bit(mask, 4));
            V::new(pmt).mul(term).st(bit(mask, 5))
        }
        10 => {
            let term = tf.mul(pm1).div(V::new(rate)).st(bit(mask, 4));
            V::new(pmt).mul(term).st(bit(mask, 5))
        }
        11 => V::new(pmt)
            .mul(tf)
            .mul(pm1)
            .div(V::new(rate))
            .st(bit(mask, 5)),
        12 => V::new(pmt)
            .mul(tf.mul(pm1).st(bit(mask, 8)))
            .div(V::new(rate))
            .st(bit(mask, 5)),
        13 => V::new(pmt)
            .mul(pm1)
            .mul(tf)
            .div(V::new(rate))
            .st(bit(mask, 5)),
        14 => V::new(pmt)
            .mul(pm1.mul(tf).st(bit(mask, 8)))
            .div(V::new(rate))
            .st(bit(mask, 5)),
        _ => unreachable!(),
    };
    let present = V::new(pv).mul(p).st(bit(mask, 6));
    present.add(payment).st(bit(mask, 7)).neg()
}

fn helper(rate: f64, nper: f64, pmt: f64, pv: f64, ty: f64, model: HelperModel) -> V {
    let base = 1.0 + rate;
    let power = model.factor.eval(base, nper);
    helper_with_power(rate, nper, pmt, pv, ty, power, model)
}

fn publish_rate(row: RateRow, helper0: V, helper1: V, outer: OuterModel) -> f64 {
    let mask = outer.spill_mask;
    // b0 is the calcFv call/publication boundary.  Clear means the final FV
    // sum remains PC64 through subtraction of the requested future value.
    let fv0 = helper0.st(bit(mask, 0));
    let fv1 = helper1.st(bit(mask, 0));
    let requested = V::new(row.requested);
    let f0 = fv0.sub(requested).st(bit(mask, 1));
    let f1 = fv1.sub(requested).st(bit(mask, 1));
    let h = V::new(1.0e-6).mul(V::new(row.x)).st(bit(mask, 2));
    let difference = f1.sub(f0).st(bit(mask, 3));
    let derivative = match outer.derivative_quotient {
        Quotient::Divide => difference.div(h),
        Quotient::ReciprocalMultiply => difference.mul(V::new(1.0).div(h).st(bit(mask, 4))),
    }
    .st(bit(mask, 5));
    let correction = match outer.update_quotient {
        Quotient::Divide => f0.div(derivative),
        Quotient::ReciprocalMultiply => f0.mul(V::new(1.0).div(derivative).st(bit(mask, 6))),
    }
    .st(bit(mask, 7));
    V::new(row.x).sub(correction).f()
}

fn objective_at(rate: f64, row: RateRow, helper_model: HelperModel, outer: OuterModel) -> f64 {
    let power = helper_model.factor.eval(1.0 + rate, row.nper);
    let helper_value =
        helper_with_power(rate, row.nper, row.pmt, row.pv, row.ty, power, helper_model)
            .st(bit(outer.spill_mask, 0));
    helper_value
        .sub(V::new(row.requested))
        .st(bit(outer.spill_mask, 1))
        .f()
}

fn iteration_at(
    rate: f64,
    row: RateRow,
    helper_model: HelperModel,
    outer: OuterModel,
) -> Option<Iteration> {
    if !rate.is_finite() || rate == 0.0 || rate <= -1.0 {
        return None;
    }
    let h = V::new(1.0e-6)
        .mul(V::new(rate))
        .st(bit(outer.spill_mask, 2))
        .f();
    let next_input = rate + h;
    if !h.is_finite() || h == 0.0 || !next_input.is_finite() || next_input <= -1.0 {
        return None;
    }
    let p0 = helper_model.factor.eval(1.0 + rate, row.nper);
    let p1 = helper_model.factor.eval(1.0 + next_input, row.nper);
    let helper0 = helper_with_power(rate, row.nper, row.pmt, row.pv, row.ty, p0, helper_model);
    let helper1 = helper_with_power(
        next_input,
        row.nper,
        row.pmt,
        row.pv,
        row.ty,
        p1,
        helper_model,
    );
    let dynamic_row = RateRow {
        x: rate,
        xh: next_input,
        p0,
        p1,
        ..row
    };
    let next = publish_rate(dynamic_row, helper0, helper1, outer);
    let residual = helper0
        .st(bit(outer.spill_mask, 0))
        .sub(V::new(row.requested))
        .st(bit(outer.spill_mask, 1))
        .f();
    (next.is_finite() && residual.is_finite()).then_some(Iteration {
        current: rate,
        residual,
        next,
    })
}

fn trajectory(row: RateRow, helper_model: HelperModel, outer: OuterModel) -> Vec<Iteration> {
    let mut states = Vec::with_capacity(100);
    let mut rate = row.x;
    for _ in 0..100 {
        let Some(state) = iteration_at(rate, row, helper_model, outer) else {
            break;
        };
        states.push(state);
        rate = state.next;
    }
    states
}

fn neighbor(value: f64, direction: i32) -> f64 {
    if direction > 0 {
        value.next_up()
    } else {
        value.next_down()
    }
}

fn root_adjacent(
    seed: f64,
    row: RateRow,
    helper_model: HelperModel,
    outer: OuterModel,
) -> Option<f64> {
    if !seed.is_finite() || seed <= -1.0 {
        return None;
    }
    let mut best = seed;
    let mut best_residual = objective_at(seed, row, helper_model, outer).abs();
    for direction in [-1, 1] {
        let mut candidate = seed;
        for _ in 0..256 {
            candidate = neighbor(candidate, direction);
            if candidate <= -1.0 || !candidate.is_finite() {
                break;
            }
            let residual = objective_at(candidate, row, helper_model, outer).abs();
            if residual < best_residual
                || (residual == best_residual
                    && (ordered(candidate.to_bits()) - ordered(row.want.to_bits())).unsigned_abs()
                        < (ordered(best.to_bits()) - ordered(row.want.to_bits())).unsigned_abs())
            {
                best = candidate;
                best_residual = residual;
            }
        }
    }
    Some(best)
}

fn schedule_value(
    schedule: Schedule,
    states: &[Iteration],
    row: RateRow,
    helper_model: HelperModel,
    outer: OuterModel,
) -> Option<f64> {
    let tolerance = 1.0e-7;
    match schedule {
        Schedule::FirstStep => states.first().map(|state| state.next),
        Schedule::SecondStep => states.get(1).map(|state| state.next),
        Schedule::ResidualPrePublishCurrent => states
            .iter()
            .find(|state| state.residual.abs() < tolerance)
            .map(|state| state.current),
        Schedule::ResidualPrePublishNext => states
            .iter()
            .find(|state| state.residual.abs() < tolerance)
            .map(|state| state.next),
        Schedule::ResidualPreMinTwoPublishNext => states
            .iter()
            .enumerate()
            .find(|(index, state)| *index >= 1 && state.residual.abs() < tolerance)
            .map(|(_, state)| state.next),
        Schedule::DeltaPublishCurrent => states
            .iter()
            .find(|state| (state.next - state.current).abs() < tolerance)
            .map(|state| state.current),
        Schedule::DeltaPublishNext => states
            .iter()
            .find(|state| (state.next - state.current).abs() < tolerance)
            .map(|state| state.next),
        Schedule::NextResidualPublishNext => states
            .windows(2)
            .find(|pair| pair[1].residual.abs() < tolerance)
            .map(|pair| pair[0].next),
        Schedule::StableOrLast => states
            .iter()
            .find(|state| state.current.to_bits() == state.next.to_bits())
            .or_else(|| states.last())
            .map(|state| state.next),
        Schedule::FixedHundred => states.last().map(|state| state.next),
        Schedule::RootAdjacentMinResidual => states
            .last()
            .and_then(|state| root_adjacent(state.next, row, helper_model, outer)),
    }
}

fn score_schedule(
    helper_model: HelperModel,
    outer: OuterModel,
    schedule: Schedule,
    rows: &[RateRow],
    trajectories: &[Vec<Iteration>],
) -> ScheduleScore {
    let mut exact = 0;
    let mut within_1 = 0;
    let mut within_4 = 0;
    let mut within_16 = 0;
    let mut max_ulp = 0_u128;
    let mut sum_ulp = 0_u128;
    let mut no_result = 0;
    for (&row, states) in rows.iter().zip(trajectories) {
        let got = schedule_value(schedule, states, row, helper_model, outer);
        let distance = match got {
            Some(got) if got.is_finite() => {
                (ordered(got.to_bits()) - ordered(row.want.to_bits())).unsigned_abs()
            }
            _ => {
                no_result += 1;
                // Keep the aggregate JSON-representable while making a
                // missing numeric result decisively worse than a local miss.
                (u64::MAX / 1_024) as u128
            }
        };
        exact += usize::from(distance == 0);
        within_1 += usize::from(distance <= 1);
        within_4 += usize::from(distance <= 4);
        within_16 += usize::from(distance <= 16);
        max_ulp = max_ulp.max(distance);
        sum_ulp += distance;
    }
    ScheduleScore {
        helper: helper_model,
        helper_id: helper_model.id(),
        outer,
        outer_id: outer.id(),
        schedule,
        schedule_id: schedule.tag().to_owned(),
        exact,
        within_1_ulp: within_1,
        within_4_ulp: within_4,
        within_16_ulp: within_16,
        max_ulp,
        sum_ulp,
        no_result,
    }
}

fn sort_schedule(scores: &mut [ScheduleScore]) {
    scores.sort_by(|a, b| {
        b.exact
            .cmp(&a.exact)
            .then_with(|| b.within_1_ulp.cmp(&a.within_1_ulp))
            .then_with(|| b.within_4_ulp.cmp(&a.within_4_ulp))
            .then_with(|| b.within_16_ulp.cmp(&a.within_16_ulp))
            .then_with(|| a.no_result.cmp(&b.no_result))
            .then_with(|| a.max_ulp.cmp(&b.max_ulp))
            .then_with(|| a.sum_ulp.cmp(&b.sum_ulp))
            .then_with(|| a.schedule_id.cmp(&b.schedule_id))
            .then_with(|| a.helper_id.cmp(&b.helper_id))
            .then_with(|| a.outer_id.cmp(&b.outer_id))
    });
}

fn score_values<M: Clone>(
    model: M,
    model_id: String,
    values: impl Iterator<Item = (f64, f64)>,
) -> Score<M> {
    let mut exact = 0;
    let mut within_1 = 0;
    let mut within_4 = 0;
    let mut within_16 = 0;
    let mut max_ulp = 0_u128;
    let mut sum_ulp = 0_u128;
    for (got, want) in values {
        let distance = (ordered(got.to_bits()) - ordered(want.to_bits())).unsigned_abs();
        exact += usize::from(distance == 0);
        within_1 += usize::from(distance <= 1);
        within_4 += usize::from(distance <= 4);
        within_16 += usize::from(distance <= 16);
        max_ulp = max_ulp.max(distance);
        sum_ulp += distance;
    }
    Score {
        model,
        model_id,
        exact,
        within_1_ulp: within_1,
        within_4_ulp: within_4,
        within_16_ulp: within_16,
        max_ulp,
        sum_ulp,
    }
}

fn score_inline(helper_model: HelperModel, outer: OuterModel, rows: &[RateRow]) -> InlineScore {
    let values = rows.iter().map(|&row| {
        let helper0 = helper_with_power(
            row.x,
            row.nper,
            row.pmt,
            row.pv,
            row.ty,
            row.p0,
            helper_model,
        );
        let helper1 = helper_with_power(
            row.xh,
            row.nper,
            row.pmt,
            row.pv,
            row.ty,
            row.p1,
            helper_model,
        );
        (publish_rate(row, helper0, helper1, outer), row.want)
    });
    let score = score_values(helper_model, helper_model.id(), values);
    InlineScore {
        helper: helper_model,
        helper_id: helper_model.id(),
        outer,
        outer_id: outer.id(),
        exact: score.exact,
        within_1_ulp: score.within_1_ulp,
        within_4_ulp: score.within_4_ulp,
        within_16_ulp: score.within_16_ulp,
        max_ulp: score.max_ulp,
        sum_ulp: score.sum_ulp,
    }
}

fn sort_scores<M>(scores: &mut [Score<M>]) {
    scores.sort_by(|a, b| {
        b.exact
            .cmp(&a.exact)
            .then_with(|| b.within_1_ulp.cmp(&a.within_1_ulp))
            .then_with(|| b.within_4_ulp.cmp(&a.within_4_ulp))
            .then_with(|| b.within_16_ulp.cmp(&a.within_16_ulp))
            .then_with(|| a.max_ulp.cmp(&b.max_ulp))
            .then_with(|| a.sum_ulp.cmp(&b.sum_ulp))
            .then_with(|| a.model_id.cmp(&b.model_id))
    });
}

fn sort_inline(scores: &mut [InlineScore]) {
    scores.sort_by(|a, b| {
        b.exact
            .cmp(&a.exact)
            .then_with(|| b.within_1_ulp.cmp(&a.within_1_ulp))
            .then_with(|| b.within_4_ulp.cmp(&a.within_4_ulp))
            .then_with(|| b.within_16_ulp.cmp(&a.within_16_ulp))
            .then_with(|| a.max_ulp.cmp(&b.max_ulp))
            .then_with(|| a.sum_ulp.cmp(&b.sum_ulp))
            .then_with(|| a.helper_id.cmp(&b.helper_id))
            .then_with(|| a.outer_id.cmp(&b.outer_id))
    });
}

fn validate(batch: &Batch, answers: &WitnessSet, function: &str, count: usize) {
    assert_eq!(batch.function, function);
    assert_eq!(answers.function, function);
    assert_eq!(batch.probes.len(), count);
    assert_eq!(answers.witnesses.len(), count);
    assert_eq!(
        answers
            .witnesses
            .iter()
            .map(|witness| witness.id.as_str())
            .collect::<BTreeSet<_>>()
            .len(),
        count
    );
    for (input, answer) in batch.probes.iter().zip(&answers.witnesses) {
        assert_eq!(input.probe.id, answer.id);
        assert_eq!(input.probe.args, answer.args);
        assert!(answer.expected_error.is_none());
        assert!(answer.expected_bits.is_some());
    }
    let provenance = &answers.capture_provenance;
    assert_eq!(provenance.schema_version, "w109-capture-provenance-v1");
    assert_eq!(provenance.environment.excel_version, "16.0");
    assert_eq!(provenance.environment.excel_build, "20228");
    assert_eq!(provenance.environment.excel_bitness, "64-bit");
    assert_eq!(provenance.environment.workbook_compatibility, "2");
    assert_eq!(
        provenance.environment.excel_input_plumbing,
        "cell_value2_bulk"
    );
    assert_eq!(provenance.oracle_cache.mode, "no_cache");
    assert_eq!(
        (provenance.oracle_cache.hits, provenance.oracle_cache.misses),
        (0, 0)
    );
    assert_eq!(provenance.runner.name, "Run-W109BulkBatch.ps1");
    assert_eq!(provenance.runner.version, "w109-bulk-batch-v2");
}

fn main() {
    let root = PathBuf::from(ROOT);
    let rate_batch: Batch = serde_json::from_str(
        &std::fs::read_to_string(root.join("batch-rate-one-step-discovery-v2.json")).unwrap(),
    )
    .unwrap();
    let rate_answers: WitnessSet = serde_json::from_str(
        &std::fs::read_to_string(root.join("answers-rate-one-step-discovery-v2.json")).unwrap(),
    )
    .unwrap();
    let fv_batch: Batch = serde_json::from_str(
        &std::fs::read_to_string(root.join("batch-rate-fv-companion-discovery-v1.json")).unwrap(),
    )
    .unwrap();
    let fv_answers: WitnessSet = serde_json::from_str(
        &std::fs::read_to_string(root.join("answers-rate-fv-companion-discovery-v1.json")).unwrap(),
    )
    .unwrap();
    validate(&rate_batch, &rate_answers, "RATE", RATE_ROWS);
    validate(&fv_batch, &fv_answers, "FV", FV_ROWS);

    let fv_rows = fv_batch
        .probes
        .iter()
        .zip(&fv_answers.witnesses)
        .map(|(input, answer)| FvRow {
            rate: parse_hex(&input.probe.args[0]),
            nper: parse_hex(&input.probe.args[1]),
            pmt: parse_hex(&input.probe.args[2]),
            pv: parse_hex(&input.probe.args[3]),
            ty: parse_hex(&input.probe.args[4]),
            want: parse_hex(answer.expected_bits.as_deref().unwrap()),
        })
        .collect::<Vec<_>>();

    let mut public_models = Vec::new();
    for factor in Factor::PUBLIC {
        for association in 0..15 {
            for spill_mask in 0..(1 << 9) {
                public_models.push(HelperModel {
                    factor,
                    association,
                    spill_mask,
                });
            }
        }
    }
    let mut public_scores = public_models
        .par_iter()
        .map(|&model| {
            score_values(
                model,
                model.id(),
                fv_rows.iter().map(|row| {
                    (
                        helper(row.rate, row.nper, row.pmt, row.pv, row.ty, model).f(),
                        row.want,
                    )
                }),
            )
        })
        .collect::<Vec<_>>();
    sort_scores(&mut public_scores);
    let public_exact = public_scores
        .iter()
        .filter(|score| score.exact == FV_ROWS)
        .cloned()
        .collect::<Vec<_>>();

    let rate_rows = (0..RATE_ROWS)
        .map(|row| {
            let input = &rate_batch.probes[row].probe;
            let x = parse_hex(&input.args[5]);
            let xh = parse_hex(&fv_batch.probes[2 * row + 1].probe.args[0]);
            assert_eq!(parse_hex(&fv_batch.probes[2 * row].probe.args[0]), x);
            let nper = parse_hex(&input.args[0]);
            let pmt = parse_hex(&input.args[1]);
            let pv = parse_hex(&input.args[2]);
            let requested = parse_hex(&input.args[3]);
            let ty = parse_hex(&input.args[4]);
            let want = parse_hex(
                rate_answers.witnesses[row]
                    .expected_bits
                    .as_deref()
                    .unwrap(),
            );
            let factor = Factor::RawChain;
            RateRow {
                x,
                xh,
                nper,
                pmt,
                pv,
                requested,
                ty,
                want,
                p0: factor.eval(1.0 + x, nper),
                p1: factor.eval(1.0 + xh, nper),
            }
        })
        .collect::<Vec<_>>();

    let mut helper_models = Vec::new();
    for association in 0..15 {
        for spill_mask in 0..(1 << 9) {
            helper_models.push(HelperModel {
                factor: Factor::RawChain,
                association,
                spill_mask,
            });
        }
    }
    // Conventional all-stored outer graph ranks the helper models before the
    // full outer spill expansion.  Its mask has all eight material sites set.
    let conventional_outer = OuterModel {
        spill_mask: 0xff,
        derivative_quotient: Quotient::Divide,
        update_quotient: Quotient::Divide,
    };
    let mut helper_scores = helper_models
        .par_iter()
        .map(|&helper_model| score_inline(helper_model, conventional_outer, &rate_rows))
        .collect::<Vec<_>>();
    sort_inline(&mut helper_scores);

    let mut outer_models = Vec::new();
    for spill_mask in 0..(1 << 8) {
        for derivative_quotient in Quotient::ALL {
            for update_quotient in Quotient::ALL {
                outer_models.push(OuterModel {
                    spill_mask,
                    derivative_quotient,
                    update_quotient,
                });
            }
        }
    }
    assert_eq!(outer_models.len(), 1_024);
    let rate_rows_ref = rate_rows.as_slice();
    let outer_models_ref = outer_models.as_slice();

    // Exhaustive exact-only pass over every helper and outer graph.  The loop
    // exits on the first mismatch, so known-wrong graphs remain inexpensive.
    let exact_inline =
        helper_models
            .par_iter()
            .flat_map_iter(|&helper_model| {
                let helper_values = rate_rows_ref
                    .iter()
                    .map(|&row| {
                        (
                            helper_with_power(
                                row.x,
                                row.nper,
                                row.pmt,
                                row.pv,
                                row.ty,
                                row.p0,
                                helper_model,
                            ),
                            helper_with_power(
                                row.xh,
                                row.nper,
                                row.pmt,
                                row.pv,
                                row.ty,
                                row.p1,
                                helper_model,
                            ),
                        )
                    })
                    .collect::<Vec<_>>();
                outer_models_ref.iter().copied().filter_map(move |outer| {
                    let exact = rate_rows_ref.iter().zip(&helper_values).all(
                        |(&row, &(helper0, helper1))| {
                            publish_rate(row, helper0, helper1, outer).to_bits()
                                == row.want.to_bits()
                        },
                    );
                    exact.then_some((helper_model, outer))
                })
            })
            .collect::<Vec<_>>();

    // Full score the 32 best helper graphs across every outer graph.  This is
    // the bounded near-miss classification if the exact-only pass is empty.
    let leading_helpers = helper_scores
        .iter()
        .take(32)
        .map(|score| score.helper)
        .collect::<Vec<_>>();
    let mut expanded_scores = leading_helpers
        .par_iter()
        .flat_map_iter(|&helper_model| {
            outer_models_ref
                .iter()
                .copied()
                .map(move |outer| score_inline(helper_model, outer, rate_rows_ref))
        })
        .collect::<Vec<_>>();
    sort_inline(&mut expanded_scores);

    // Diversify the schedule pass across the best outer graph for every one
    // of the 32 leading helper families, then add the global leading pairs.
    let mut schedule_pairs = Vec::<(HelperModel, OuterModel)>::new();
    for &helper_model in &leading_helpers {
        if let Some(score) = expanded_scores
            .iter()
            .find(|score| score.helper == helper_model)
        {
            let pair = (score.helper, score.outer);
            if !schedule_pairs.contains(&pair) {
                schedule_pairs.push(pair);
            }
        }
    }
    for score in expanded_scores.iter().take(64) {
        let pair = (score.helper, score.outer);
        if !schedule_pairs.contains(&pair) {
            schedule_pairs.push(pair);
        }
    }
    let mut schedule_scores = schedule_pairs
        .par_iter()
        .flat_map_iter(|&(helper_model, outer)| {
            let trajectories = rate_rows_ref
                .iter()
                .map(|&row| trajectory(row, helper_model, outer))
                .collect::<Vec<_>>();
            Schedule::ALL.into_iter().map(move |schedule| {
                score_schedule(helper_model, outer, schedule, rate_rows_ref, &trajectories)
            })
        })
        .collect::<Vec<_>>();
    sort_schedule(&mut schedule_scores);
    let schedule_best_by_mode = Schedule::ALL
        .into_iter()
        .map(|schedule| {
            schedule_scores
                .iter()
                .find(|score| score.schedule == schedule)
                .unwrap()
        })
        .collect::<Vec<_>>();

    let report = json!({
        "schema_version": "oxfunc.w109.rate_fv_inline_spill_race.v1",
        "scope_status": "discovery_only",
        "public_fv": {
            "rows": FV_ROWS,
            "candidate_count": public_scores.len(),
            "exact_survivor_count": public_exact.len(),
            "exact_survivors": public_exact,
            "top_scores": public_scores.iter().take(64).collect::<Vec<_>>(),
        },
        "rate_inline": {
            "rows": RATE_ROWS,
            "helper_candidate_count": helper_models.len(),
            "outer_candidate_count": outer_models.len(),
            "exhaustive_combination_count": helper_models.len() * outer_models.len(),
            "exact_survivor_count": exact_inline.len(),
            "exact_survivors": exact_inline.iter().map(|(helper, outer)| json!({
                "helper": helper,
                "helper_id": helper.id(),
                "outer": outer,
                "outer_id": outer.id(),
            })).collect::<Vec<_>>(),
            "conventional_outer_top_helpers": helper_scores.iter().take(64).collect::<Vec<_>>(),
            "expanded_top32_helper_scores": expanded_scores.iter().take(128).collect::<Vec<_>>(),
        },
        "schedule_classification": {
            "arithmetic_pair_count": schedule_pairs.len(),
            "schedule_count": Schedule::ALL.len(),
            "candidate_count": schedule_scores.len(),
            "best_by_schedule": schedule_best_by_mode,
            "top_scores": schedule_scores.iter().take(128).collect::<Vec<_>>(),
        },
        "heldout": "not opened or scored",
    });
    let mut bytes = serde_json::to_vec_pretty(&report).unwrap();
    bytes.push(b'\n');
    std::fs::write(root.join("report-rate-fv-inline-discovery-v1.json"), bytes).unwrap();

    println!(
        "public_fv rows={} candidates={} exact_survivors={}",
        FV_ROWS,
        public_scores.len(),
        public_exact.len()
    );
    for score in public_scores.iter().take(12) {
        println!(
            "  {:>3}/{} <=1 {:>3} <=4 {:>3} max {:>8} {}",
            score.exact,
            FV_ROWS,
            score.within_1_ulp,
            score.within_4_ulp,
            score.max_ulp,
            score.model_id
        );
    }
    println!("public_fv best by association:");
    for association in 0..15 {
        let score = public_scores
            .iter()
            .find(|score| score.model.association == association)
            .unwrap();
        println!(
            "  assoc={association:02} exact={}/{} <=1={} <=4={} max={} {}",
            score.exact,
            FV_ROWS,
            score.within_1_ulp,
            score.within_4_ulp,
            score.max_ulp,
            score.model_id
        );
    }
    let public_best = public_scores[0].model;
    println!("public_fv best misses:");
    for (index, row) in fv_rows.iter().enumerate() {
        let got = helper(row.rate, row.nper, row.pmt, row.pv, row.ty, public_best).f();
        if got.to_bits() != row.want.to_bits() {
            let delta = ordered(got.to_bits()) - ordered(row.want.to_bits());
            println!(
                "  row={index:03} rate={:.17e} nper={:.17e} ty={} delta_ulp={delta} got=0x{:016x} want=0x{:016x}",
                row.rate,
                row.nper,
                row.ty,
                got.to_bits(),
                row.want.to_bits()
            );
        }
    }
    println!(
        "rate_inline helpers={} outer={} combinations={} exact_survivors={}",
        helper_models.len(),
        outer_models.len(),
        helper_models.len() * outer_models.len(),
        exact_inline.len()
    );
    for score in expanded_scores.iter().take(20) {
        println!(
            "  exact {:>3}/{} <=1 {:>3} <=4 {:>3} <=16 {:>3} max {:>8} sum {:>12} {} || {}",
            score.exact,
            RATE_ROWS,
            score.within_1_ulp,
            score.within_4_ulp,
            score.within_16_ulp,
            score.max_ulp,
            score.sum_ulp,
            score.helper_id,
            score.outer_id,
        );
    }
    println!(
        "schedule_classification arithmetic_pairs={} schedules={} candidates={}",
        schedule_pairs.len(),
        Schedule::ALL.len(),
        schedule_scores.len()
    );
    for score in &schedule_best_by_mode {
        println!(
            "  {:36} exact {:>3}/{} <=1 {:>3} <=4 {:>3} <=16 {:>3} no_result {:>3} max {:>8} sum {:>12}",
            score.schedule_id,
            score.exact,
            RATE_ROWS,
            score.within_1_ulp,
            score.within_4_ulp,
            score.within_16_ulp,
            score.no_result,
            score.max_ulp,
            score.sum_ulp,
        );
    }
}
