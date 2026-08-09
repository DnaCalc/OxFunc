//! W109 G6-03: offline YIELD solver/publication schedule race.
//!
//! This clean-room racer covers the historical 19-row witness corpus, the frozen
//! current-build 384-row near-seed discovery, the 120-row seed-family discovery,
//! and the 136-row PRICE companion. `price_kernel` is the companion-validated
//! controlled forward model for candidate races; it is not a claim that Excel's
//! exact YIELD residual association, iteration variable, or publication graph has
//! been identified.

use oxfunc_core::excel_numeric::research as rx;
use oxfunc_core::functions::bond_core_family::{price_kernel, yield_kernel};
use oxfunc_core::functions::coupon_family::{
    coupdaybs_kernel, coupdays_kernel, coupdaysnc_kernel, coupnum_kernel,
};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::cmp::Reverse;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

const CORPUS: &str = "../../work/w109/G6-solvers/yield_corpus_out.json";
const ARTIFACT_ROOT: &str = "../../work/w109/G6-solvers";
const FREEZE_ID: &str = "w109-g6-03-yield-near-seed-v1-20260809";
const SEED_FAMILY_FREEZE_ID: &str = "w109-g6-03-yield-seed-family-v2-20260809";
const CW: u16 = rx::CW_PC64_RN;

fn ext(value: f64) -> rx::Ext80 {
    rx::ext_from_f64(value)
}

fn excel_x87_add(a: f64, b: f64) -> f64 {
    rx::ext_to_f64(&rx::ext_add(&ext(a), &ext(b), CW), CW)
}

fn excel_x87_sub(a: f64, b: f64) -> f64 {
    rx::ext_to_f64(&rx::ext_sub(&ext(a), &ext(b), CW), CW)
}

fn excel_x87_mul(a: f64, b: f64) -> f64 {
    rx::ext_to_f64(&rx::ext_mul(&ext(a), &ext(b), CW), CW)
}

fn excel_x87_div(a: f64, b: f64) -> f64 {
    rx::ext_to_f64(&rx::ext_div(&ext(a), &ext(b), CW), CW)
}

#[derive(Deserialize)]
struct Witness {
    tag: String,
    kind: String,
    bits: String,
}

#[derive(Clone, Copy)]
struct Bond {
    settlement: f64,
    maturity: f64,
    coupon: f64,
    redemption: f64,
    frequency: f64,
    basis: f64,
}

#[derive(Clone)]
struct Row {
    tag: String,
    bond: Bond,
    price: f64,
    want: u64,
}

#[derive(Clone, Copy, Debug)]
enum Ops {
    Native,
    X87,
}

impl Ops {
    fn add(self, a: f64, b: f64) -> f64 {
        match self {
            Self::Native => a + b,
            Self::X87 => excel_x87_add(a, b),
        }
    }

    fn sub(self, a: f64, b: f64) -> f64 {
        match self {
            Self::Native => a - b,
            Self::X87 => excel_x87_sub(a, b),
        }
    }

    fn mul(self, a: f64, b: f64) -> f64 {
        match self {
            Self::Native => a * b,
            Self::X87 => excel_x87_mul(a, b),
        }
    }

    fn div(self, a: f64, b: f64) -> f64 {
        match self {
            Self::Native => a / b,
            Self::X87 => excel_x87_div(a, b),
        }
    }
}

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
        if store {
            Self::new(self.f())
        } else {
            self
        }
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
enum UpdateForm {
    DerivativeThenDivide,
    MultiplyThenDivide,
    DivideThenMultiply,
    HOverDifferenceThenMultiply,
    MultiplyReciprocalDifference,
}

#[derive(Clone, Copy, Debug)]
enum ObjectiveGraph {
    PriceMinusTarget,
    TargetMinusPrice,
    DifferenceOverTarget,
    NegativeDifferenceOverTarget,
    PriceOverTargetMinusOne,
    OneMinusPriceOverTarget,
    ScaledDifference01,
    ScaledDifference100,
    DirtyMinusDirtyTarget,
    DirtyOverDirtyTargetMinusOne,
    HornerReciprocalPowDirty,
    HornerBasePowDirty,
}

#[derive(Clone, Copy, Debug)]
struct FirstStepGraph {
    step: Step,
    difference: Difference,
    form: UpdateForm,
    spill_mask: u16,
}

#[derive(Clone, Copy, Debug)]
enum Step {
    Absolute(f64),
    Relative(f64),
    RawRelative(f64),
}

impl Step {
    fn at(self, x: f64, ops: Ops) -> f64 {
        match self {
            Self::Absolute(h) => h,
            Self::Relative(h) => ops.mul(h, x.abs().max(1.0)),
            Self::RawRelative(h) => ops.mul(h, x.abs()),
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum Difference {
    Forward,
    Backward,
    Central,
}

#[derive(Clone, Copy, Debug)]
enum Stop {
    Step(f64),
    Residual(f64),
    Either(f64),
    Fixed,
}

#[derive(Clone, Copy, Debug)]
enum Publish {
    Old,
    New,
    Previous,
}

#[derive(Clone, Copy, Debug)]
struct NewtonCfg {
    ops: Ops,
    step: Step,
    difference: Difference,
    stop: Stop,
    publish: Publish,
    cap: usize,
    seed: f64,
}

#[derive(Clone, Copy, Debug)]
struct SecantCfg {
    ops: Ops,
    stop: Stop,
    publish: Publish,
    cap: usize,
    seed0: f64,
    seed1: f64,
}

#[derive(Clone, Copy, Debug)]
struct FalsePositionCfg {
    ops: Ops,
    stop: Stop,
    publish: Publish,
    cap: usize,
    low: f64,
    high: f64,
}

#[derive(Default, Clone, Copy)]
struct Score {
    exact: usize,
    sum: u128,
    max: u64,
}

impl Score {
    fn add(&mut self, got: f64, want: u64) {
        let d = ulp(got.to_bits(), want);
        self.exact += usize::from(d == 0);
        self.sum += u128::from(d);
        self.max = self.max.max(d);
    }

    fn rank(self) -> (Reverse<usize>, u64, u128) {
        (Reverse(self.exact), self.max, self.sum)
    }
}

fn ordered(bits: u64) -> u64 {
    if bits >> 63 == 0 {
        bits | (1 << 63)
    } else {
        !bits
    }
}

fn ulp(a: u64, b: u64) -> u64 {
    ordered(a).abs_diff(ordered(b))
}

fn signed_ulp(a: u64, b: u64) -> i128 {
    i128::from(ordered(a)) - i128::from(ordered(b))
}

fn parse_bits(s: &str) -> u64 {
    u64::from_str_radix(s.trim_start_matches("0x"), 16).expect("valid bits")
}

fn bond(tag: &str) -> Bond {
    let key = tag.as_bytes()[0] as char;
    match key {
        'A' => Bond {
            settlement: 44013.0,
            maturity: 44562.0,
            coupon: 0.05,
            redemption: 100.0,
            frequency: 2.0,
            basis: 0.0,
        },
        'B' => Bond {
            settlement: 44058.0,
            maturity: 44562.0,
            coupon: 0.05,
            redemption: 100.0,
            frequency: 2.0,
            basis: 0.0,
        },
        'C' => Bond {
            settlement: 44013.0,
            maturity: 46753.0,
            coupon: 0.05,
            redemption: 100.0,
            frequency: 2.0,
            basis: 0.0,
        },
        'D' => Bond {
            settlement: 44013.0,
            maturity: 47119.0,
            coupon: 0.075,
            redemption: 102.0,
            frequency: 2.0,
            basis: 0.0,
        },
        'E' => Bond {
            settlement: 44094.0,
            maturity: 45658.0,
            coupon: 0.06,
            redemption: 103.0,
            frequency: 2.0,
            basis: 0.0,
        },
        _ => panic!("unknown bond tag {tag}"),
    }
}

fn load() -> Vec<Row> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(CORPUS);
    let witnesses: Vec<Witness> = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    witnesses
        .into_iter()
        .map(|w| {
            assert_eq!(w.kind, "number");
            let price = w.tag.split('@').nth(1).unwrap().parse::<f64>().unwrap();
            Row {
                bond: bond(&w.tag),
                price,
                want: parse_bits(&w.bits),
                tag: w.tag,
            }
        })
        .collect()
}

fn objective(row: &Row, y: f64) -> Option<f64> {
    price_kernel(
        row.bond.settlement,
        row.bond.maturity,
        row.bond.coupon,
        y,
        row.bond.redemption,
        row.bond.frequency,
        Some(row.bond.basis),
    )
    .ok()
    .map(|p| p - row.price)
}

fn schedule_factors(row: &Row) -> (i64, f64, f64, f64) {
    // Reconstruct the same schedule quantities used by the corrected PRICE
    // objective.  The earlier 19-row prototype had date-only hard-coded
    // factors; that aliases identical dates across bases and is invalid for the
    // basis-2/3 discovery shapes.
    let basis = Some(row.bond.basis);
    let n = coupnum_kernel(
        row.bond.settlement,
        row.bond.maturity,
        row.bond.frequency,
        basis,
    )
    .expect("valid discovery coupon count") as i64;
    let a = coupdaybs_kernel(
        row.bond.settlement,
        row.bond.maturity,
        row.bond.frequency,
        basis,
    )
    .expect("valid discovery accrued days");
    let e = coupdays_kernel(
        row.bond.settlement,
        row.bond.maturity,
        row.bond.frequency,
        basis,
    )
    .expect("valid discovery coupon days");
    let coupon = 100.0 * row.bond.coupon / row.bond.frequency;
    let accrual = coupon * a / e;
    (n, (e - a) / e, coupon, accrual)
}

#[derive(Clone, Copy, Debug)]
enum PricePower {
    Corrected,
    PlatformPowf,
    X87Chain,
    RepeatedMultiply,
}

fn graph_pow(base: f64, exponent: f64, power: PricePower) -> f64 {
    match power {
        PricePower::Corrected => schedule_pow(base, exponent),
        PricePower::PlatformPowf => base.powf(exponent),
        PricePower::X87Chain => rx::excel_pow_chain(base, exponent),
        PricePower::RepeatedMultiply if exponent >= 0.0 && exponent.fract() == 0.0 => {
            let mut result = 1.0;
            for _ in 0..exponent as usize {
                result *= base;
            }
            result
        }
        PricePower::RepeatedMultiply => rx::excel_pow_chain(base, exponent),
    }
}

fn local_price(row: &Row, y: f64, ops: Ops, power: PricePower) -> Option<f64> {
    let (n, off, coupon, accrual) = schedule_factors(row);
    let base = ops.add(1.0, ops.div(y, row.bond.frequency));
    if base <= 0.0 {
        return None;
    }
    let mut price = 0.0;
    for k in 0..n {
        let exponent = ops.add(off, k as f64);
        price = ops.add(price, ops.div(coupon, graph_pow(base, exponent, power)));
    }
    let redemption_exponent = ops.add(off, (n - 1) as f64);
    price = ops.add(
        price,
        ops.div(
            row.bond.redemption,
            graph_pow(base, redemption_exponent, power),
        ),
    );
    Some(ops.sub(price, accrual))
}

fn newton_local(row: &Row, cfg: NewtonCfg, power: PricePower) -> Option<f64> {
    let seed = if cfg.seed.is_nan() {
        row.bond.coupon
    } else {
        cfg.seed
    };
    let objective =
        |x: f64| local_price(row, x, cfg.ops, power).map(|price| cfg.ops.sub(price, row.price));
    let mut previous = seed;
    let mut x = seed;
    for _ in 0..cfg.cap {
        let fx = objective(x)?;
        let h = cfg.step.at(x, cfg.ops);
        let derivative = match cfg.difference {
            Difference::Forward => {
                let xp = cfg.ops.add(x, h);
                cfg.ops.div(cfg.ops.sub(objective(xp)?, fx), h)
            }
            Difference::Backward => {
                let xm = cfg.ops.sub(x, h);
                cfg.ops.div(cfg.ops.sub(fx, objective(xm)?), h)
            }
            Difference::Central => {
                let xp = cfg.ops.add(x, h);
                let xm = cfg.ops.sub(x, h);
                cfg.ops.div(
                    cfg.ops.sub(objective(xp)?, objective(xm)?),
                    cfg.ops.mul(2.0, h),
                )
            }
        };
        if derivative == 0.0 || !derivative.is_finite() {
            return None;
        }
        let dx = cfg.ops.div(fx, derivative);
        let next = cfg.ops.sub(x, dx);
        if !next.is_finite() || next <= -row.bond.frequency {
            return None;
        }
        if stopped(cfg.stop, dx, fx) {
            return Some(publish(cfg.publish, previous, x, next));
        }
        previous = x;
        x = next;
    }
    Some(publish(cfg.publish, previous, x, x))
}

fn schedule_pow(base: f64, exponent: f64) -> f64 {
    if exponent >= 0.0 && exponent < 1024.0 && exponent.fract() == 0.0 {
        let mut n = exponent as u64;
        let mut result = 1.0;
        let mut factor = base;
        while n != 0 {
            if n & 1 != 0 {
                result *= factor;
            }
            n >>= 1;
            if n != 0 {
                factor *= factor;
            }
        }
        result
    } else {
        rx::excel_pow_chain(base, exponent)
    }
}

fn analytic_price_and_derivative(row: &Row, y: f64, ops: Ops) -> Option<(f64, f64)> {
    let (n, off, coupon, accrual) = schedule_factors(row);
    let frequency = row.bond.frequency;
    let base = ops.add(1.0, ops.div(y, frequency));
    if base <= 0.0 {
        return None;
    }
    let mut price = 0.0;
    let mut magnitude = 0.0;
    for k in 0..n {
        let exponent = ops.add(off, k as f64);
        let discount = schedule_pow(base, exponent);
        price = ops.add(price, ops.div(coupon, discount));
        let derivative_discount = schedule_pow(base, ops.add(exponent, 1.0));
        let weighted = ops.mul(coupon, exponent);
        magnitude = ops.add(magnitude, ops.div(weighted, derivative_discount));
    }
    let redemption_exponent = ops.add(off, (n - 1) as f64);
    price = ops.add(
        price,
        ops.div(row.bond.redemption, schedule_pow(base, redemption_exponent)),
    );
    price = ops.sub(price, accrual);
    let redemption_weighted = ops.mul(row.bond.redemption, redemption_exponent);
    magnitude = ops.add(
        magnitude,
        ops.div(
            redemption_weighted,
            schedule_pow(base, ops.add(redemption_exponent, 1.0)),
        ),
    );
    Some((price, -ops.div(magnitude, frequency)))
}

fn analytic_newton(row: &Row, cfg: NewtonCfg) -> Option<f64> {
    let seed = if cfg.seed.is_nan() {
        row.bond.coupon
    } else {
        cfg.seed
    };
    let mut previous = seed;
    let mut x = seed;
    for _ in 0..cfg.cap {
        let (price, derivative) = analytic_price_and_derivative(row, x, cfg.ops)?;
        let fx = cfg.ops.sub(price, row.price);
        if derivative == 0.0 || !derivative.is_finite() {
            return None;
        }
        let dx = cfg.ops.div(fx, derivative);
        let next = cfg.ops.sub(x, dx);
        if !next.is_finite() || next <= -row.bond.frequency {
            return None;
        }
        if stopped(cfg.stop, dx, fx) {
            return Some(publish(cfg.publish, previous, x, next));
        }
        previous = x;
        x = next;
    }
    Some(publish(cfg.publish, previous, x, x))
}

fn stopped(stop: Stop, dx: f64, fx: f64) -> bool {
    match stop {
        Stop::Step(t) => dx.abs() < t,
        Stop::Residual(t) => fx.abs() < t,
        Stop::Either(t) => dx.abs() < t || fx.abs() < t,
        Stop::Fixed => false,
    }
}

fn publish(which: Publish, previous: f64, old: f64, new: f64) -> f64 {
    match which {
        Publish::Old => old,
        Publish::New => new,
        Publish::Previous => previous,
    }
}

fn newton(row: &Row, cfg: NewtonCfg) -> Option<f64> {
    let seed = if cfg.seed.is_nan() {
        row.bond.coupon
    } else {
        cfg.seed
    };
    let mut previous = seed;
    let mut x = seed;
    for _ in 0..cfg.cap {
        let fx = objective(row, x)?;
        let h = cfg.step.at(x, cfg.ops);
        let derivative = match cfg.difference {
            Difference::Forward => {
                let xp = cfg.ops.add(x, h);
                cfg.ops.div(cfg.ops.sub(objective(row, xp)?, fx), h)
            }
            Difference::Backward => {
                let xm = cfg.ops.sub(x, h);
                cfg.ops.div(cfg.ops.sub(fx, objective(row, xm)?), h)
            }
            Difference::Central => {
                let xp = cfg.ops.add(x, h);
                let xm = cfg.ops.sub(x, h);
                cfg.ops.div(
                    cfg.ops.sub(objective(row, xp)?, objective(row, xm)?),
                    cfg.ops.mul(2.0, h),
                )
            }
        };
        if derivative == 0.0 || !derivative.is_finite() {
            return None;
        }
        let dx = cfg.ops.div(fx, derivative);
        let next = cfg.ops.sub(x, dx);
        if !next.is_finite() || next <= -row.bond.frequency {
            return None;
        }
        if stopped(cfg.stop, dx, fx) {
            return Some(publish(cfg.publish, previous, x, next));
        }
        previous = x;
        x = next;
    }
    Some(publish(cfg.publish, previous, x, x))
}

fn secant(row: &Row, cfg: SecantCfg) -> Option<f64> {
    let mut previous = cfg.seed0;
    let mut x0 = cfg.seed0;
    let mut f0 = objective(row, x0)?;
    let mut x1 = cfg.seed1;
    let mut f1 = objective(row, x1)?;
    for _ in 0..cfg.cap {
        let den = cfg.ops.sub(f1, f0);
        if den == 0.0 || !den.is_finite() {
            return Some(x1);
        }
        let dx = cfg.ops.div(cfg.ops.mul(f1, cfg.ops.sub(x1, x0)), den);
        let next = cfg.ops.sub(x1, dx);
        if !next.is_finite() || next <= -row.bond.frequency {
            return None;
        }
        if stopped(cfg.stop, dx, f1) {
            return Some(publish(cfg.publish, previous, x1, next));
        }
        previous = x1;
        x0 = x1;
        f0 = f1;
        x1 = next;
        f1 = objective(row, x1)?;
    }
    Some(publish(cfg.publish, previous, x1, x1))
}

fn false_position(row: &Row, cfg: FalsePositionCfg) -> Option<f64> {
    let mut a = cfg.low;
    let mut b = cfg.high;
    let mut fa = objective(row, a)?;
    let mut fb = objective(row, b)?;
    if fa.signum() == fb.signum() {
        return None;
    }
    let mut previous = a;
    let mut x = a;
    for _ in 0..cfg.cap {
        let den = cfg.ops.sub(fb, fa);
        if den == 0.0 {
            return Some(x);
        }
        let weighted = cfg.ops.div(cfg.ops.mul(fa, cfg.ops.sub(b, a)), den);
        let next = cfg.ops.sub(a, weighted);
        let fx = objective(row, next)?;
        let dx = cfg.ops.sub(next, x);
        if stopped(cfg.stop, dx, fx) {
            return Some(publish(cfg.publish, previous, x, next));
        }
        previous = x;
        x = next;
        if fa.signum() == fx.signum() {
            a = next;
            fa = fx;
        } else {
            b = next;
            fb = fx;
        }
    }
    Some(publish(cfg.publish, previous, x, x))
}

fn score(rows: &[Row], mut f: impl FnMut(&Row) -> Option<f64>) -> Score {
    let mut score = Score::default();
    for row in rows {
        score.add(f(row).unwrap_or(f64::NAN), row.want);
    }
    score
}

fn next_down_positive(value: f64) -> f64 {
    f64::from_bits(value.to_bits() - 1)
}

fn next_up_positive(value: f64) -> f64 {
    f64::from_bits(value.to_bits() + 1)
}

fn print_excel_output_plateaus(rows: &[Row]) {
    println!("excel-output corrected-PRICE plateaus:");
    for row in rows {
        let want = f64::from_bits(row.want);
        let center = price_kernel(
            row.bond.settlement,
            row.bond.maturity,
            row.bond.coupon,
            want,
            row.bond.redemption,
            row.bond.frequency,
            Some(row.bond.basis),
        )
        .unwrap();
        let mut lo = want;
        let mut hi = want;
        if center.to_bits() == row.price.to_bits() {
            for _ in 0..1_000_000 {
                let candidate = next_down_positive(lo);
                let price = price_kernel(
                    row.bond.settlement,
                    row.bond.maturity,
                    row.bond.coupon,
                    candidate,
                    row.bond.redemption,
                    row.bond.frequency,
                    Some(row.bond.basis),
                )
                .unwrap();
                if price.to_bits() != row.price.to_bits() {
                    break;
                }
                lo = candidate;
            }
            for _ in 0..1_000_000 {
                let candidate = next_up_positive(hi);
                let price = price_kernel(
                    row.bond.settlement,
                    row.bond.maturity,
                    row.bond.coupon,
                    candidate,
                    row.bond.redemption,
                    row.bond.frequency,
                    Some(row.bond.basis),
                )
                .unwrap();
                if price.to_bits() != row.price.to_bits() {
                    break;
                }
                hi = candidate;
            }
        }
        println!(
            "  {} y=0x{:016x} price=0x{:016x} target=0x{:016x} exact_price={} plateau=[{:+},{:+}] width={}",
            row.tag,
            row.want,
            center.to_bits(),
            row.price.to_bits(),
            center.to_bits() == row.price.to_bits(),
            ordered(lo.to_bits()) as i128 - ordered(row.want) as i128,
            ordered(hi.to_bits()) as i128 - ordered(row.want) as i128,
            ulp(lo.to_bits(), hi.to_bits()) + 1,
        );
    }
}

#[derive(Clone, Copy)]
struct Shape {
    id: &'static str,
    bond: Bond,
}

#[derive(Clone)]
struct NearSeedProbe {
    class: &'static str,
    target: f64,
    delta_from_anchor: f64,
    distinct_predictions: usize,
    prediction_span_ulp: i128,
}

#[derive(Serialize)]
struct FrozenProbe {
    id: String,
    args: [String; 7],
}

#[derive(Serialize)]
struct RankedFrozenProbe {
    probe: FrozenProbe,
    distinct_outputs: usize,
    prediction_span_ulp: i128,
}

#[derive(Serialize)]
struct FrozenBatch {
    function: &'static str,
    row_id: String,
    probes: Vec<RankedFrozenProbe>,
}

#[derive(Serialize)]
struct NearSeedRecord {
    id: String,
    split: String,
    shape: String,
    class: String,
    args_bits: [String; 7],
    seed: String,
    worksheet_price_anchor_model_bits: String,
    target_delta_from_anchor_bits: String,
    distinct_one_step_predictions: usize,
    prediction_span_ulp: i128,
}

#[derive(Serialize)]
struct NearSeedBank {
    schema_version: &'static str,
    freeze_id: &'static str,
    answer_blind: bool,
    split: String,
    records: Vec<NearSeedRecord>,
}

#[derive(Serialize)]
struct CompanionRecord {
    id: String,
    shape: String,
    role: String,
    args_bits: [String; 7],
}

#[derive(Serialize)]
struct CompanionBank {
    schema_version: &'static str,
    freeze_id: &'static str,
    answer_blind: bool,
    records: Vec<CompanionRecord>,
}

#[derive(Deserialize, Serialize)]
struct SeedFamilyRecord {
    id: String,
    shape: String,
    source_roles: Vec<String>,
    args_bits: [String; 7],
    distinct_candidate_outputs: usize,
    prediction_span_ulp: i128,
}

#[derive(Deserialize, Serialize)]
struct SeedFamilyBank {
    schema_version: String,
    freeze_id: String,
    answer_blind: bool,
    candidate_families: Vec<String>,
    records: Vec<SeedFamilyRecord>,
}

#[derive(Deserialize)]
struct AnsweredBatch {
    function: String,
    witnesses: Vec<AnsweredWitness>,
}

#[derive(Deserialize)]
struct AnsweredWitness {
    id: String,
    args: Vec<String>,
    expected_bits: String,
}

fn hex(value: f64) -> String {
    format!("0x{:016x}", value.to_bits())
}

fn discovery_shapes() -> Vec<Shape> {
    vec![
        Shape {
            id: "d-oncoupon-short-b0-f2",
            bond: Bond {
                settlement: 44013.0,
                maturity: 44562.0,
                coupon: 0.05,
                redemption: 100.0,
                frequency: 2.0,
                basis: 0.0,
            },
        },
        Shape {
            id: "d-offcoupon-short-b0-f2",
            bond: Bond {
                settlement: 44058.0,
                maturity: 44562.0,
                coupon: 0.05,
                redemption: 100.0,
                frequency: 2.0,
                basis: 0.0,
            },
        },
        Shape {
            id: "d-oncoupon-long-b0-f2",
            bond: Bond {
                settlement: 44013.0,
                maturity: 46753.0,
                coupon: 0.05,
                redemption: 100.0,
                frequency: 2.0,
                basis: 0.0,
            },
        },
        Shape {
            id: "d-offcoupon-b2-f2",
            bond: Bond {
                settlement: 44094.0,
                maturity: 45658.0,
                coupon: 0.06,
                redemption: 103.0,
                frequency: 2.0,
                basis: 2.0,
            },
        },
        Shape {
            id: "d-offcoupon-b3-f2",
            bond: Bond {
                settlement: 44094.0,
                maturity: 45658.0,
                coupon: 0.06,
                redemption: 103.0,
                frequency: 2.0,
                basis: 3.0,
            },
        },
        Shape {
            id: "d-oncoupon-b4-f1",
            bond: Bond {
                settlement: 44197.0,
                maturity: 45658.0,
                coupon: 0.0325,
                redemption: 97.0,
                frequency: 1.0,
                basis: 4.0,
            },
        },
        Shape {
            id: "d-offcoupon-b0-f4",
            bond: Bond {
                settlement: 44242.0,
                maturity: 45658.0,
                coupon: 0.0875,
                redemption: 105.0,
                frequency: 4.0,
                basis: 0.0,
            },
        },
        Shape {
            id: "d-leap-b1-f2",
            bond: Bond {
                settlement: 43890.0,
                maturity: 45658.0,
                coupon: 0.04125,
                redemption: 101.5,
                frequency: 2.0,
                basis: 1.0,
            },
        },
    ]
}

fn heldout_shapes() -> Vec<Shape> {
    vec![
        Shape {
            id: "h-oncoupon-b0-f2",
            bond: Bond {
                settlement: 44378.0,
                maturity: 47293.0,
                coupon: 0.0275,
                redemption: 99.0,
                frequency: 2.0,
                basis: 0.0,
            },
        },
        Shape {
            id: "h-offcoupon-b4-f2",
            bond: Bond {
                settlement: 44423.0,
                maturity: 47293.0,
                coupon: 0.07125,
                redemption: 104.0,
                frequency: 2.0,
                basis: 4.0,
            },
        },
        Shape {
            id: "h-offcoupon-b1-f4",
            bond: Bond {
                settlement: 44713.0,
                maturity: 47849.0,
                coupon: 0.01375,
                redemption: 96.5,
                frequency: 4.0,
                basis: 1.0,
            },
        },
        Shape {
            id: "h-offcoupon-b2-f1",
            bond: Bond {
                settlement: 44908.0,
                maturity: 48214.0,
                coupon: 0.105,
                redemption: 107.0,
                frequency: 1.0,
                basis: 2.0,
            },
        },
        Shape {
            id: "h-offcoupon-b3-f2",
            bond: Bond {
                settlement: 45124.0,
                maturity: 48675.0,
                coupon: 0.0525,
                redemption: 100.5,
                frequency: 2.0,
                basis: 3.0,
            },
        },
        Shape {
            id: "h-long-b0-f4",
            bond: Bond {
                settlement: 45292.0,
                maturity: 49310.0,
                coupon: 0.09375,
                redemption: 102.25,
                frequency: 4.0,
                basis: 0.0,
            },
        },
    ]
}

fn price_for_bond(bond: Bond, y: f64) -> f64 {
    price_kernel(
        bond.settlement,
        bond.maturity,
        bond.coupon,
        y,
        bond.redemption,
        bond.frequency,
        Some(bond.basis),
    )
    .expect("valid frozen bond shape")
}

#[derive(Clone, Copy, Debug)]
enum SeedFormula {
    /// Approximate the capital-gain amortization over the fractional number of
    /// remaining coupon periods and divide by the discrete average book value.
    /// The latter gives the purchase price `(m + 1) / (2m)` weight and the
    /// redemption value `(m - 1) / (2m)` weight.
    DiscreteBookDirect,
    /// Algebraically identical to `DiscreteBookDirect`, but exposes the
    /// weighted-average association as a distinct binary64/x87 graph.
    DiscreteBookWeighted,
    /// The common continuous-time textbook approximation, retained as a
    /// negative control for the discrete endpoint weighting.
    TextbookAverage,
}

#[derive(Clone, Copy, Debug)]
enum PeriodPolicy {
    IntegerCouponCount,
    FractionalRemaining,
    DirectRemaining,
}

#[derive(Clone, Copy, Debug)]
enum AccrualPolicy {
    Clean,
    DerivedElapsed,
    DirectElapsed,
}

fn remaining_coupon_periods(row: &Row, ops: Ops) -> Option<f64> {
    let basis = Some(row.bond.basis);
    let n = coupnum_kernel(
        row.bond.settlement,
        row.bond.maturity,
        row.bond.frequency,
        basis,
    )
    .ok()?;
    let a = coupdaybs_kernel(
        row.bond.settlement,
        row.bond.maturity,
        row.bond.frequency,
        basis,
    )
    .ok()?;
    let e = coupdays_kernel(
        row.bond.settlement,
        row.bond.maturity,
        row.bond.frequency,
        basis,
    )
    .ok()?;
    let m = ops.sub(n, ops.div(a, e));
    (m > 0.0 && m.is_finite()).then_some(m)
}

fn coupon_periods(row: &Row, ops: Ops) -> Option<(f64, f64, f64)> {
    let basis = Some(row.bond.basis);
    let n = coupnum_kernel(
        row.bond.settlement,
        row.bond.maturity,
        row.bond.frequency,
        basis,
    )
    .ok()?;
    let fractional = remaining_coupon_periods(row, ops)?;
    let e = coupdays_kernel(
        row.bond.settlement,
        row.bond.maturity,
        row.bond.frequency,
        basis,
    )
    .ok()?;
    let direct_days = coupdaysnc_kernel(
        row.bond.settlement,
        row.bond.maturity,
        row.bond.frequency,
        basis,
    )
    .ok()?;
    let direct = ops.add(ops.sub(n, 1.0), ops.div(direct_days, e));
    Some((n, fractional, direct))
}

fn approximate_yield_seed_periods(
    row: &Row,
    ops: Ops,
    gain_policy: PeriodPolicy,
    weight_policy: PeriodPolicy,
) -> Option<f64> {
    let (n, fractional, direct) = coupon_periods(row, ops)?;
    let choose = |policy| match policy {
        PeriodPolicy::IntegerCouponCount => n,
        PeriodPolicy::FractionalRemaining => fractional,
        PeriodPolicy::DirectRemaining => direct,
    };
    let gain_periods = choose(gain_policy);
    let weight_periods = choose(weight_policy);
    approximate_yield_seed_explicit_periods(row, ops, gain_periods, weight_periods)
}

fn approximate_yield_seed_explicit_periods(
    row: &Row,
    ops: Ops,
    gain_periods: f64,
    weight_periods: f64,
) -> Option<f64> {
    if !(gain_periods > 0.0 && weight_periods > 0.0) {
        return None;
    }
    let spread = ops.sub(row.bond.redemption, row.price);
    let coupon = ops.mul(100.0, row.bond.coupon);
    let annualized_gain = ops.div(ops.mul(spread, row.bond.frequency), gain_periods);
    let numerator = ops.add(coupon, annualized_gain);
    let price_term = ops.mul(row.price, ops.add(weight_periods, 1.0));
    let redemption_term = ops.mul(row.bond.redemption, ops.sub(weight_periods, 1.0));
    let denominator = ops.div(
        ops.add(price_term, redemption_term),
        ops.mul(2.0, weight_periods),
    );
    let seed = ops.div(numerator, denominator);
    (seed.is_finite() && seed > -row.bond.frequency).then_some(seed)
}

fn accrued_interest(row: &Row, ops: Ops, policy: AccrualPolicy) -> Option<f64> {
    if matches!(policy, AccrualPolicy::Clean) {
        return Some(0.0);
    }
    let basis = Some(row.bond.basis);
    let e = coupdays_kernel(
        row.bond.settlement,
        row.bond.maturity,
        row.bond.frequency,
        basis,
    )
    .ok()?;
    let elapsed = match policy {
        AccrualPolicy::Clean => unreachable!(),
        AccrualPolicy::DerivedElapsed => coupdaybs_kernel(
            row.bond.settlement,
            row.bond.maturity,
            row.bond.frequency,
            basis,
        )
        .ok()?,
        AccrualPolicy::DirectElapsed => {
            let remaining = coupdaysnc_kernel(
                row.bond.settlement,
                row.bond.maturity,
                row.bond.frequency,
                basis,
            )
            .ok()?;
            ops.sub(e, remaining)
        }
    };
    let coupon_payment = ops.div(ops.mul(100.0, row.bond.coupon), row.bond.frequency);
    Some(ops.mul(coupon_payment, ops.div(elapsed, e)))
}

fn approximate_yield_seed_inputs(
    row: &Row,
    ops: Ops,
    gain_policy: PeriodPolicy,
    weight_policy: PeriodPolicy,
    spread_accrual: AccrualPolicy,
    weight_accrual: AccrualPolicy,
) -> Option<f64> {
    let (n, fractional, direct) = coupon_periods(row, ops)?;
    let choose = |policy| match policy {
        PeriodPolicy::IntegerCouponCount => n,
        PeriodPolicy::FractionalRemaining => fractional,
        PeriodPolicy::DirectRemaining => direct,
    };
    let gain_periods = choose(gain_policy);
    let weight_periods = choose(weight_policy);
    let spread_price = ops.add(row.price, accrued_interest(row, ops, spread_accrual)?);
    let weight_price = ops.add(row.price, accrued_interest(row, ops, weight_accrual)?);
    let spread = ops.sub(row.bond.redemption, spread_price);
    let coupon = ops.mul(100.0, row.bond.coupon);
    let annualized_gain = ops.div(ops.mul(spread, row.bond.frequency), gain_periods);
    let numerator = ops.add(coupon, annualized_gain);
    let price_term = ops.mul(weight_price, ops.add(weight_periods, 1.0));
    let redemption_term = ops.mul(row.bond.redemption, ops.sub(weight_periods, 1.0));
    let denominator = ops.div(
        ops.add(price_term, redemption_term),
        ops.mul(2.0, weight_periods),
    );
    let seed = ops.div(numerator, denominator);
    (seed.is_finite() && seed > -row.bond.frequency).then_some(seed)
}

fn approximate_yield_seed(row: &Row, ops: Ops, formula: SeedFormula) -> Option<f64> {
    let m = remaining_coupon_periods(row, ops)?;
    let spread = ops.sub(row.bond.redemption, row.price);
    let coupon = ops.mul(100.0, row.bond.coupon);
    let annualized_gain = ops.div(ops.mul(spread, row.bond.frequency), m);
    let numerator = ops.add(coupon, annualized_gain);
    let denominator = match formula {
        SeedFormula::DiscreteBookDirect => {
            let tail_weight = ops.div(ops.sub(m, 1.0), ops.mul(2.0, m));
            ops.add(row.price, ops.mul(spread, tail_weight))
        }
        SeedFormula::DiscreteBookWeighted => {
            let price_term = ops.mul(row.price, ops.add(m, 1.0));
            let redemption_term = ops.mul(row.bond.redemption, ops.sub(m, 1.0));
            ops.div(ops.add(price_term, redemption_term), ops.mul(2.0, m))
        }
        SeedFormula::TextbookAverage => ops.div(ops.add(row.price, row.bond.redemption), 2.0),
    };
    let seed = ops.div(numerator, denominator);
    (seed.is_finite() && seed > -row.bond.frequency).then_some(seed)
}

fn approximate_yield_seed_ext(row: &Row, spill_mask: u16) -> Option<f64> {
    let basis = Some(row.bond.basis);
    let n = coupnum_kernel(
        row.bond.settlement,
        row.bond.maturity,
        row.bond.frequency,
        basis,
    )
    .ok()?;
    let a = coupdaybs_kernel(
        row.bond.settlement,
        row.bond.maturity,
        row.bond.frequency,
        basis,
    )
    .ok()?;
    let e = coupdays_kernel(
        row.bond.settlement,
        row.bond.maturity,
        row.bond.frequency,
        basis,
    )
    .ok()?;
    let bit = |index: u32| spill_mask & (1_u16 << index) != 0;
    let fraction = V::new(a).div(V::new(e)).st(bit(0));
    let m = V::new(n).sub(fraction).st(bit(1));
    if m.f() <= 0.0 {
        return None;
    }
    let spread = V::new(row.bond.redemption)
        .sub(V::new(row.price))
        .st(bit(2));
    let coupon = V::new(100.0).mul(V::new(row.bond.coupon)).st(bit(3));
    let gain_numerator = spread.mul(V::new(row.bond.frequency)).st(bit(4));
    let annualized_gain = gain_numerator.div(m).st(bit(5));
    let numerator = coupon.add(annualized_gain).st(bit(6));
    let m_plus = m.add(V::new(1.0)).st(bit(7));
    let price_term = V::new(row.price).mul(m_plus).st(bit(8));
    let m_minus = m.sub(V::new(1.0)).st(bit(9));
    let redemption_term = V::new(row.bond.redemption).mul(m_minus).st(bit(10));
    let weighted_sum = price_term.add(redemption_term).st(bit(11));
    let scale = V::new(2.0).mul(m).st(bit(12));
    let denominator = weighted_sum.div(scale).st(bit(13));
    let seed = numerator.div(denominator).st(bit(14)).f();
    (seed.is_finite() && seed > -row.bond.frequency).then_some(seed)
}

fn initial_correction(row: &Row, x: f64) -> Option<f64> {
    let h = 1e-6_f64;
    let fx = objective(row, x)?;
    let plus = objective(row, x + h)?;
    let minus = objective(row, x - h)?;
    let derivative = (plus - minus) / (2.0 * h);
    (derivative != 0.0 && derivative.is_finite()).then_some(fx / derivative)
}

fn print_newton_trace(row: &Row, seed: f64) {
    let mut x = seed;
    println!(
        "trace {} want=0x{:016x} seed=0x{:016x}",
        row.tag,
        row.want,
        seed.to_bits()
    );
    for iteration in 0..12 {
        let fx = objective(row, x).unwrap();
        let correction = initial_correction(row, x).unwrap();
        println!(
            "  i={iteration} x=0x{:016x} want_ulp={} fx={:+.17e} correction={:+.17e} stop={}",
            x.to_bits(),
            ulp(x.to_bits(), row.want),
            fx,
            correction,
            correction.abs() < 1e-10
        );
        if correction.abs() < 1e-10 {
            break;
        }
        x -= correction;
    }
}

fn seeded_graph_newton(row: &Row, seed: f64, ops: Ops, form: UpdateForm) -> Option<f64> {
    let h = 1e-6_f64;
    let scaled_h = ops.mul(2.0, h);
    let mut x = seed;
    for _ in 0..100 {
        let fx = objective(row, x)?;
        let plus = objective(row, ops.add(x, h))?;
        let minus = objective(row, ops.sub(x, h))?;
        let difference = ops.sub(plus, minus);
        if difference == 0.0 || !difference.is_finite() {
            return None;
        }
        let correction = match form {
            UpdateForm::DerivativeThenDivide => {
                let derivative = ops.div(difference, scaled_h);
                ops.div(fx, derivative)
            }
            UpdateForm::MultiplyThenDivide => ops.div(ops.mul(fx, scaled_h), difference),
            UpdateForm::DivideThenMultiply => ops.mul(ops.div(fx, difference), scaled_h),
            UpdateForm::HOverDifferenceThenMultiply => ops.mul(ops.div(scaled_h, difference), fx),
            UpdateForm::MultiplyReciprocalDifference => {
                ops.mul(ops.mul(fx, scaled_h), ops.div(1.0, difference))
            }
        };
        if !correction.is_finite() {
            return None;
        }
        if correction.abs() < 1e-10 {
            return Some(x);
        }
        x = ops.sub(x, correction);
        if !x.is_finite() || x <= -row.bond.frequency {
            return None;
        }
    }
    Some(x)
}

fn configured_graph_newton(row: &Row, seed: f64, cfg: NewtonCfg, form: UpdateForm) -> Option<f64> {
    configured_graph_newton_objective(row, seed, cfg, form, ObjectiveGraph::PriceMinusTarget)
}

fn configured_objective(row: &Row, y: f64, ops: Ops, graph: ObjectiveGraph) -> Option<f64> {
    if matches!(
        graph,
        ObjectiveGraph::DirtyMinusDirtyTarget
            | ObjectiveGraph::DirtyOverDirtyTargetMinusOne
            | ObjectiveGraph::HornerReciprocalPowDirty
            | ObjectiveGraph::HornerBasePowDirty
    ) {
        let (n, off, coupon, accrual) = schedule_factors(row);
        let base = ops.add(1.0, ops.div(y, row.bond.frequency));
        if base <= 0.0 {
            return None;
        }
        let dirty = if matches!(
            graph,
            ObjectiveGraph::HornerReciprocalPowDirty | ObjectiveGraph::HornerBasePowDirty
        ) {
            let v = ops.div(1.0, base);
            let mut polynomial = ops.add(coupon, row.bond.redemption);
            for _ in 1..n {
                polynomial = ops.add(coupon, ops.mul(v, polynomial));
            }
            let fractional_discount = match graph {
                ObjectiveGraph::HornerReciprocalPowDirty => schedule_pow(v, off),
                ObjectiveGraph::HornerBasePowDirty => ops.div(1.0, schedule_pow(base, off)),
                _ => unreachable!(),
            };
            ops.mul(fractional_discount, polynomial)
        } else {
            let mut dirty = 0.0;
            for k in 0..n {
                let exponent = ops.add(off, k as f64);
                dirty = ops.add(dirty, ops.div(coupon, schedule_pow(base, exponent)));
            }
            let redemption_exponent = ops.add(off, (n - 1) as f64);
            ops.add(
                dirty,
                ops.div(row.bond.redemption, schedule_pow(base, redemption_exponent)),
            )
        };
        let dirty_target = ops.add(row.price, accrual);
        return Some(match graph {
            ObjectiveGraph::DirtyMinusDirtyTarget => ops.sub(dirty, dirty_target),
            ObjectiveGraph::DirtyOverDirtyTargetMinusOne => {
                ops.sub(ops.div(dirty, dirty_target), 1.0)
            }
            ObjectiveGraph::HornerReciprocalPowDirty | ObjectiveGraph::HornerBasePowDirty => {
                ops.sub(dirty, dirty_target)
            }
            _ => unreachable!(),
        });
    }
    let price = price_kernel(
        row.bond.settlement,
        row.bond.maturity,
        row.bond.coupon,
        y,
        row.bond.redemption,
        row.bond.frequency,
        Some(row.bond.basis),
    )
    .ok()?;
    Some(match graph {
        ObjectiveGraph::PriceMinusTarget => ops.sub(price, row.price),
        ObjectiveGraph::TargetMinusPrice => ops.sub(row.price, price),
        ObjectiveGraph::DifferenceOverTarget => ops.div(ops.sub(price, row.price), row.price),
        ObjectiveGraph::NegativeDifferenceOverTarget => {
            ops.div(ops.sub(row.price, price), row.price)
        }
        ObjectiveGraph::PriceOverTargetMinusOne => ops.sub(ops.div(price, row.price), 1.0),
        ObjectiveGraph::OneMinusPriceOverTarget => ops.sub(1.0, ops.div(price, row.price)),
        ObjectiveGraph::ScaledDifference01 => {
            ops.sub(ops.mul(price, 0.01), ops.mul(row.price, 0.01))
        }
        ObjectiveGraph::ScaledDifference100 => {
            ops.sub(ops.mul(price, 100.0), ops.mul(row.price, 100.0))
        }
        ObjectiveGraph::DirtyMinusDirtyTarget
        | ObjectiveGraph::DirtyOverDirtyTargetMinusOne
        | ObjectiveGraph::HornerReciprocalPowDirty
        | ObjectiveGraph::HornerBasePowDirty => unreachable!(),
    })
}

fn configured_graph_newton_objective(
    row: &Row,
    seed: f64,
    cfg: NewtonCfg,
    form: UpdateForm,
    objective_graph: ObjectiveGraph,
) -> Option<f64> {
    let mut previous = seed;
    let mut x = seed;
    for _ in 0..cfg.cap {
        let fx = configured_objective(row, x, cfg.ops, objective_graph)?;
        let h = cfg.step.at(x, cfg.ops);
        if h == 0.0 || !h.is_finite() {
            return None;
        }
        let (difference, scale) = match cfg.difference {
            Difference::Forward => {
                let xp = cfg.ops.add(x, h);
                (
                    cfg.ops
                        .sub(configured_objective(row, xp, cfg.ops, objective_graph)?, fx),
                    h,
                )
            }
            Difference::Backward => {
                let xm = cfg.ops.sub(x, h);
                (
                    cfg.ops
                        .sub(fx, configured_objective(row, xm, cfg.ops, objective_graph)?),
                    h,
                )
            }
            Difference::Central => {
                let xp = cfg.ops.add(x, h);
                let xm = cfg.ops.sub(x, h);
                (
                    cfg.ops.sub(
                        configured_objective(row, xp, cfg.ops, objective_graph)?,
                        configured_objective(row, xm, cfg.ops, objective_graph)?,
                    ),
                    cfg.ops.mul(2.0, h),
                )
            }
        };
        if difference == 0.0 || !difference.is_finite() {
            return None;
        }
        let correction = match form {
            UpdateForm::DerivativeThenDivide => cfg.ops.div(fx, cfg.ops.div(difference, scale)),
            UpdateForm::MultiplyThenDivide => cfg.ops.div(cfg.ops.mul(fx, scale), difference),
            UpdateForm::DivideThenMultiply => cfg.ops.mul(cfg.ops.div(fx, difference), scale),
            UpdateForm::HOverDifferenceThenMultiply => {
                cfg.ops.mul(cfg.ops.div(scale, difference), fx)
            }
            UpdateForm::MultiplyReciprocalDifference => cfg
                .ops
                .mul(cfg.ops.mul(fx, scale), cfg.ops.div(1.0, difference)),
        };
        if !correction.is_finite() {
            return None;
        }
        let next = cfg.ops.sub(x, correction);
        if !next.is_finite() || next <= -row.bond.frequency {
            return None;
        }
        if stopped(cfg.stop, correction, fx) {
            return Some(publish(cfg.publish, previous, x, next));
        }
        previous = x;
        x = next;
    }
    Some(publish(cfg.publish, previous, x, x))
}

fn fd_bootstrap_secant(
    row: &Row,
    seed: f64,
    ops: Ops,
    objective_graph: ObjectiveGraph,
    threshold: f64,
    publication: Publish,
    form: UpdateForm,
) -> Option<f64> {
    let h = 1e-6;
    let mut x0 = seed;
    let mut f0 = configured_objective(row, x0, ops, objective_graph)?;
    let plus = configured_objective(row, ops.add(x0, h), ops, objective_graph)?;
    let minus = configured_objective(row, ops.sub(x0, h), ops, objective_graph)?;
    let difference = ops.sub(plus, minus);
    if difference == 0.0 || !difference.is_finite() {
        return None;
    }
    let derivative = ops.div(difference, ops.mul(2.0, h));
    let first_correction = ops.div(f0, derivative);
    let mut x1 = ops.sub(x0, first_correction);
    if !x1.is_finite() || x1 <= -row.bond.frequency {
        return None;
    }
    let mut previous = x0;
    for _ in 0..100 {
        let f1 = configured_objective(row, x1, ops, objective_graph)?;
        let denominator = ops.sub(f1, f0);
        if denominator == 0.0 || !denominator.is_finite() {
            return Some(x1);
        }
        let interval = ops.sub(x1, x0);
        let correction = match form {
            UpdateForm::DerivativeThenDivide | UpdateForm::MultiplyThenDivide => {
                ops.div(ops.mul(f1, interval), denominator)
            }
            UpdateForm::DivideThenMultiply => ops.mul(ops.div(f1, denominator), interval),
            UpdateForm::HOverDifferenceThenMultiply => ops.mul(ops.div(interval, denominator), f1),
            UpdateForm::MultiplyReciprocalDifference => {
                ops.mul(ops.mul(f1, interval), ops.div(1.0, denominator))
            }
        };
        if !correction.is_finite() {
            return None;
        }
        let next = ops.sub(x1, correction);
        if !next.is_finite() || next <= -row.bond.frequency {
            return None;
        }
        if correction.abs() < threshold {
            return Some(publish(publication, previous, x1, next));
        }
        previous = x1;
        x0 = x1;
        f0 = f1;
        x1 = next;
    }
    Some(publish(publication, previous, x1, x1))
}

fn seeded_ext_graph_newton(row: &Row, seed: f64, graph: FirstStepGraph) -> Option<f64> {
    let h = match graph.step {
        Step::Absolute(value) => value,
        Step::Relative(value) => value * seed.abs().max(1.0),
        Step::RawRelative(value) => value * seed.abs(),
    };
    let bit = |index: u32| graph.spill_mask & (1_u16 << index) != 0;
    let mut x = seed;
    for _ in 0..100 {
        let f0 = V::new(objective(row, x)?).st(bit(0));
        let plus = V::new(objective(row, x + h)?).st(bit(1));
        let minus = V::new(objective(row, x - h)?).st(bit(1));
        let difference = plus.sub(minus).st(bit(2));
        if difference.f() == 0.0 {
            return None;
        }
        let scaled_h = V::new(h).mul(V::new(2.0)).st(bit(3));
        let correction = match graph.form {
            UpdateForm::DerivativeThenDivide => {
                let derivative = difference.div(scaled_h).st(bit(4));
                f0.div(derivative)
            }
            UpdateForm::MultiplyThenDivide => f0.mul(scaled_h).st(bit(4)).div(difference),
            UpdateForm::DivideThenMultiply => f0.div(difference).st(bit(4)).mul(scaled_h),
            UpdateForm::HOverDifferenceThenMultiply => scaled_h.div(difference).st(bit(4)).mul(f0),
            UpdateForm::MultiplyReciprocalDifference => f0
                .mul(scaled_h)
                .st(bit(4))
                .mul(V::new(1.0).div(difference).st(bit(5))),
        }
        .st(bit(6));
        let correction_f64 = correction.f();
        if !correction_f64.is_finite() {
            return None;
        }
        if correction_f64.abs() < 1e-10 {
            return Some(x);
        }
        x = V::new(x).sub(correction).st(bit(7)).f();
        if !x.is_finite() || x <= -row.bond.frequency {
            return None;
        }
    }
    Some(x)
}

fn score_seed_formula_discovery(run_stopped: bool) {
    let path = PathBuf::from(ARTIFACT_ROOT).join("answers-yield-near-seed-discovery-20260809.json");
    let answered: AnsweredBatch = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    assert_eq!(answered.function, "YIELD");
    assert_eq!(answered.witnesses.len(), 384);
    let rows = rows_from_answered(&answered);

    let formulas = [
        SeedFormula::DiscreteBookDirect,
        SeedFormula::DiscreteBookWeighted,
        SeedFormula::TextbookAverage,
    ];
    println!("approximate starting-estimate formulas:");
    for formula in formulas {
        for ops in [Ops::Native, Ops::X87] {
            let total = score(&rows, |row| approximate_yield_seed(row, ops, formula));
            println!(
                "  seed {}/{} max={} sum={} {formula:?} {ops:?}",
                total.exact,
                rows.len(),
                total.max,
                total.sum
            );
            for shape in discovery_shapes() {
                let shape_rows: Vec<Row> = rows
                    .iter()
                    .filter(|row| row.tag.contains(shape.id))
                    .cloned()
                    .collect();
                let s = score(&shape_rows, |row| approximate_yield_seed(row, ops, formula));
                println!(
                    "    {:<28} {}/{} max={} sum={}",
                    shape.id,
                    s.exact,
                    shape_rows.len(),
                    s.max,
                    s.sum
                );
            }
        }
    }

    let mut fixed = Vec::new();
    for formula in formulas {
        for seed_ops in [Ops::Native, Ops::X87] {
            for ops in [Ops::Native, Ops::X87] {
                for step in [
                    Step::Absolute(1e-3),
                    Step::Absolute(1e-4),
                    Step::Absolute(1e-5),
                    Step::Absolute(1e-6),
                    Step::Absolute(1e-7),
                    Step::Absolute(1e-8),
                    Step::Relative(1e-3),
                    Step::Relative(1e-4),
                    Step::Relative(1e-5),
                    Step::Relative(1e-6),
                ] {
                    for difference in [
                        Difference::Forward,
                        Difference::Backward,
                        Difference::Central,
                    ] {
                        for cap in 1..=6 {
                            let base = NewtonCfg {
                                ops,
                                step,
                                difference,
                                stop: Stop::Fixed,
                                publish: Publish::New,
                                cap,
                                seed: 0.0,
                            };
                            let s = score(&rows, |row| {
                                let seed = approximate_yield_seed(row, seed_ops, formula)?;
                                newton(row, NewtonCfg { seed, ..base })
                            });
                            fixed.push((s, formula, seed_ops, base));
                        }
                    }
                }
            }
        }
    }
    fixed.sort_by_key(|item| item.0.rank());
    println!("seeded fixed-iteration candidates={}:", fixed.len());
    for (s, formula, seed_ops, cfg) in fixed.iter().take(30) {
        println!(
            "  {}/{} max={} sum={} formula={formula:?} seed_ops={seed_ops:?} {cfg:?}",
            s.exact,
            rows.len(),
            s.max,
            s.sum
        );
    }

    if !run_stopped {
        return;
    }

    let thresholds: Vec<f64> = (4..=16)
        .map(|power| 10.0_f64.powi(-power))
        .chain((18..=56).map(|power| 2.0_f64.powi(-power)))
        .collect();
    let mut stopped_candidates = Vec::new();
    for formula in [
        SeedFormula::DiscreteBookDirect,
        SeedFormula::DiscreteBookWeighted,
    ] {
        for seed_ops in [Ops::Native] {
            for ops in [Ops::Native] {
                for step in [Step::Absolute(1e-6)] {
                    for difference in [Difference::Central] {
                        for &threshold in &thresholds {
                            for stop_kind in 0..3 {
                                let stop = match stop_kind {
                                    0 => Stop::Residual(threshold),
                                    1 => Stop::Step(threshold),
                                    _ => Stop::Either(threshold),
                                };
                                for publish in [Publish::Old, Publish::New, Publish::Previous] {
                                    let base = NewtonCfg {
                                        ops,
                                        step,
                                        difference,
                                        stop,
                                        publish,
                                        cap: 100,
                                        seed: 0.0,
                                    };
                                    let s = score(&rows, |row| {
                                        let seed = approximate_yield_seed(row, seed_ops, formula)?;
                                        newton(row, NewtonCfg { seed, ..base })
                                    });
                                    stopped_candidates.push((s, formula, seed_ops, base));
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    stopped_candidates.sort_by_key(|item| item.0.rank());
    println!("seeded stopped candidates={}:", stopped_candidates.len());
    for (s, formula, seed_ops, cfg) in stopped_candidates.iter().take(50) {
        println!(
            "  {}/{} max={} sum={} formula={formula:?} seed_ops={seed_ops:?} {cfg:?}",
            s.exact,
            rows.len(),
            s.max,
            s.sum
        );
    }

    let (_, best_formula, best_seed_ops, best_cfg) = stopped_candidates[0];
    println!(
        "best seeded-stop breakdown formula={best_formula:?} seed_ops={best_seed_ops:?} {best_cfg:?}:"
    );
    let mut worst = Vec::new();
    for shape in discovery_shapes() {
        let shape_rows: Vec<Row> = rows
            .iter()
            .filter(|row| row.tag.contains(shape.id))
            .cloned()
            .collect();
        let s = score(&shape_rows, |row| {
            let seed = approximate_yield_seed(row, best_seed_ops, best_formula)?;
            newton(row, NewtonCfg { seed, ..best_cfg })
        });
        let mut buckets = [0_usize; 8];
        for row in &shape_rows {
            let seed = approximate_yield_seed(row, best_seed_ops, best_formula).unwrap();
            let got = newton(row, NewtonCfg { seed, ..best_cfg }).unwrap();
            let distance = ulp(got.to_bits(), row.want);
            let bucket = match distance {
                0 => 0,
                1 => 1,
                2..=4 => 2,
                5..=16 => 3,
                17..=64 => 4,
                65..=256 => 5,
                257..=1024 => 6,
                _ => 7,
            };
            buckets[bucket] += 1;
            worst.push((distance, row.tag.clone(), got.to_bits(), row.want));
        }
        println!(
            "  {:<28} exact={}/{} max={} sum={} buckets[0,1,2-4,5-16,17-64,65-256,257-1024,>1024]={buckets:?}",
            shape.id,
            s.exact,
            shape_rows.len(),
            s.max,
            s.sum
        );
    }
    worst.sort_by_key(|item| Reverse(item.0));
    for (distance, tag, got, want) in worst.into_iter().take(30) {
        println!("  worst {tag} got=0x{got:016x} want=0x{want:016x} ulp={distance}");
    }

    let mut update_graphs = Vec::new();
    for formula in [
        SeedFormula::DiscreteBookDirect,
        SeedFormula::DiscreteBookWeighted,
    ] {
        for seed_ops in [Ops::Native, Ops::X87] {
            for ops in [Ops::Native, Ops::X87] {
                for form in [
                    UpdateForm::DerivativeThenDivide,
                    UpdateForm::MultiplyThenDivide,
                    UpdateForm::DivideThenMultiply,
                    UpdateForm::HOverDifferenceThenMultiply,
                    UpdateForm::MultiplyReciprocalDifference,
                ] {
                    let s = score(&rows, |row| {
                        let seed = approximate_yield_seed(row, seed_ops, formula)?;
                        seeded_graph_newton(row, seed, ops, form)
                    });
                    update_graphs.push((s, formula, seed_ops, ops, form));
                }
            }
        }
    }
    update_graphs.sort_by_key(|item| item.0.rank());
    println!(
        "seeded outer-update graph candidates={}:",
        update_graphs.len()
    );
    for (s, formula, seed_ops, ops, form) in update_graphs.iter().take(40) {
        println!(
            "  {}/{} max={} sum={} formula={formula:?} seed_ops={seed_ops:?} ops={ops:?} form={form:?}",
            s.exact,
            rows.len(),
            s.max,
            s.sum
        );
    }

    let mut core_schedules = Vec::new();
    for formula in [
        SeedFormula::DiscreteBookDirect,
        SeedFormula::DiscreteBookWeighted,
    ] {
        for step in [
            Step::Absolute(1e-3),
            Step::Absolute(1e-4),
            Step::Absolute(1e-5),
            Step::Absolute(1e-6),
            Step::Absolute(1e-7),
            Step::Absolute(1e-8),
            Step::Absolute(1e-9),
            Step::Absolute(2.0_f64.powi(-20)),
            Step::Absolute(2.0_f64.powi(-24)),
            Step::Absolute(2.0_f64.powi(-28)),
        ] {
            for difference in [
                Difference::Forward,
                Difference::Backward,
                Difference::Central,
            ] {
                let cfg = NewtonCfg {
                    ops: Ops::Native,
                    step,
                    difference,
                    stop: Stop::Step(1e-10),
                    publish: Publish::Old,
                    cap: 100,
                    seed: 0.0,
                };
                let s = score(&rows, |row| {
                    let seed = approximate_yield_seed(row, Ops::Native, formula)?;
                    newton(row, NewtonCfg { seed, ..cfg })
                });
                core_schedules.push((s, formula, cfg));
            }
        }
    }
    core_schedules.sort_by_key(|item| item.0.rank());
    println!("seeded core schedules={}:", core_schedules.len());
    for (s, formula, cfg) in core_schedules.iter().take(40) {
        println!(
            "  {}/{} max={} sum={} formula={formula:?} {cfg:?}",
            s.exact,
            rows.len(),
            s.max,
            s.sum
        );
    }

    let seed_stop_rows: Vec<Row> = rows
        .iter()
        .filter(|row| {
            approximate_yield_seed(row, Ops::Native, SeedFormula::DiscreteBookWeighted)
                .and_then(|seed| initial_correction(row, seed))
                .is_some_and(|correction| correction.abs() < 1e-10)
        })
        .cloned()
        .collect();
    println!("candidate initial-step stop rows={}", seed_stop_rows.len());
    for shape in discovery_shapes() {
        println!(
            "  {:<28} {}",
            shape.id,
            seed_stop_rows
                .iter()
                .filter(|row| row.tag.contains(shape.id))
                .count()
        );
    }
    let mut seed_spills = Vec::new();
    for spill_mask in 0_u16..0x8000 {
        seed_spills.push((
            score(&seed_stop_rows, |row| {
                approximate_yield_seed_ext(row, spill_mask)
            }),
            spill_mask,
        ));
    }
    seed_spills.sort_by_key(|item| item.0.rank());
    println!("extended seed spill candidates={}:", seed_spills.len());
    for (s, spill_mask) in seed_spills.iter().take(30) {
        println!(
            "  {}/{} max={} sum={} spill=0x{spill_mask:04x}",
            s.exact,
            seed_stop_rows.len(),
            s.max,
            s.sum
        );
    }
    let best_cfg = NewtonCfg {
        ops: Ops::Native,
        step: Step::Absolute(1e-6),
        difference: Difference::Central,
        stop: Stop::Step(1e-10),
        publish: Publish::Old,
        cap: 100,
        seed: 0.0,
    };
    let mut full_seed_spills = Vec::new();
    for (_, spill_mask) in seed_spills.iter().take(512).copied() {
        let s = score(&rows, |row| {
            let seed = approximate_yield_seed_ext(row, spill_mask)?;
            newton(row, NewtonCfg { seed, ..best_cfg })
        });
        full_seed_spills.push((s, spill_mask));
    }
    full_seed_spills.sort_by_key(|item| item.0.rank());
    println!(
        "top-seed-spill end-to-end candidates={}:",
        full_seed_spills.len()
    );
    for (s, spill_mask) in full_seed_spills.iter().take(30) {
        println!(
            "  {}/{} max={} sum={} spill=0x{spill_mask:04x}",
            s.exact,
            rows.len(),
            s.max,
            s.sum
        );
    }

    let seed_spill = full_seed_spills[0].1;
    let mut outer_spills = Vec::new();
    for form in [
        UpdateForm::DerivativeThenDivide,
        UpdateForm::MultiplyThenDivide,
        UpdateForm::DivideThenMultiply,
        UpdateForm::HOverDifferenceThenMultiply,
        UpdateForm::MultiplyReciprocalDifference,
    ] {
        for spill_mask in 0_u16..=0x00ff {
            let graph = FirstStepGraph {
                step: Step::Absolute(1e-6),
                difference: Difference::Central,
                form,
                spill_mask,
            };
            let s = score(&rows, |row| {
                let seed = approximate_yield_seed_ext(row, seed_spill)?;
                seeded_ext_graph_newton(row, seed, graph)
            });
            outer_spills.push((s, graph));
        }
    }
    outer_spills.sort_by_key(|item| item.0.rank());
    println!(
        "extended outer graph candidates={} seed_spill=0x{seed_spill:04x}:",
        outer_spills.len()
    );
    for (s, graph) in outer_spills.iter().take(40) {
        println!(
            "  {}/{} max={} sum={} {graph:?}",
            s.exact,
            rows.len(),
            s.max,
            s.sum
        );
    }

    let mut combined_spills = Vec::new();
    for (_, seed_mask) in full_seed_spills.iter().take(64).copied() {
        for (_, graph) in outer_spills.iter().take(64).copied() {
            let s = score(&rows, |row| {
                let seed = approximate_yield_seed_ext(row, seed_mask)?;
                seeded_ext_graph_newton(row, seed, graph)
            });
            combined_spills.push((s, seed_mask, graph));
        }
    }
    combined_spills.sort_by_key(|item| item.0.rank());
    println!(
        "combined seed/outer spill candidates={}:",
        combined_spills.len()
    );
    for (s, seed_mask, graph) in combined_spills.iter().take(40) {
        println!(
            "  {}/{} max={} sum={} seed_spill=0x{seed_mask:04x} {graph:?}",
            s.exact,
            rows.len(),
            s.max,
            s.sum
        );
    }

    let mut analytic_seeded = Vec::new();
    for seed_formula in [
        SeedFormula::DiscreteBookDirect,
        SeedFormula::DiscreteBookWeighted,
    ] {
        for ops in [Ops::Native, Ops::X87] {
            for threshold in [
                1e-8,
                1e-9,
                1e-10,
                1e-11,
                2.0_f64.powi(-33),
                2.0_f64.powi(-34),
            ] {
                let cfg = NewtonCfg {
                    ops,
                    step: Step::Absolute(0.0),
                    difference: Difference::Central,
                    stop: Stop::Step(threshold),
                    publish: Publish::Old,
                    cap: 100,
                    seed: 0.0,
                };
                let s = score(&rows, |row| {
                    let seed = approximate_yield_seed(row, Ops::Native, seed_formula)?;
                    analytic_newton(row, NewtonCfg { seed, ..cfg })
                });
                analytic_seeded.push((s, seed_formula, cfg));
            }
        }
    }
    for (_, seed_mask) in full_seed_spills.iter().take(32).copied() {
        for ops in [Ops::Native, Ops::X87] {
            let cfg = NewtonCfg {
                ops,
                step: Step::Absolute(0.0),
                difference: Difference::Central,
                stop: Stop::Step(1e-10),
                publish: Publish::Old,
                cap: 100,
                seed: 0.0,
            };
            let s = score(&rows, |row| {
                let seed = approximate_yield_seed_ext(row, seed_mask)?;
                analytic_newton(row, NewtonCfg { seed, ..cfg })
            });
            analytic_seeded.push((s, SeedFormula::DiscreteBookWeighted, cfg));
        }
    }
    analytic_seeded.sort_by_key(|item| item.0.rank());
    println!("analytic seeded candidates={}:", analytic_seeded.len());
    for (s, formula, cfg) in analytic_seeded.iter().take(30) {
        println!(
            "  {}/{} max={} sum={} formula={formula:?} {cfg:?}",
            s.exact,
            rows.len(),
            s.max,
            s.sum
        );
    }
    let mut local_seeded = Vec::new();
    for power in [
        PricePower::Corrected,
        PricePower::PlatformPowf,
        PricePower::X87Chain,
        PricePower::RepeatedMultiply,
    ] {
        for ops in [Ops::Native, Ops::X87] {
            for threshold in [1e-9, 1e-10, 1e-11, 2.0_f64.powi(-33)] {
                let cfg = NewtonCfg {
                    ops,
                    step: Step::Absolute(1e-6),
                    difference: Difference::Central,
                    stop: Stop::Step(threshold),
                    publish: Publish::Old,
                    cap: 100,
                    seed: 0.0,
                };
                for seed_kind in 0..2 {
                    let s = score(&rows, |row| {
                        let seed = if seed_kind == 0 {
                            approximate_yield_seed(
                                row,
                                Ops::Native,
                                SeedFormula::DiscreteBookWeighted,
                            )?
                        } else {
                            approximate_yield_seed_ext(row, seed_spill)?
                        };
                        newton_local(row, NewtonCfg { seed, ..cfg }, power)
                    });
                    local_seeded.push((s, power, cfg, seed_kind));
                }
            }
        }
    }
    local_seeded.sort_by_key(|item| item.0.rank());
    println!(
        "local forward-power seeded candidates={}:",
        local_seeded.len()
    );
    for (s, power, cfg, seed_kind) in local_seeded.iter().take(40) {
        println!(
            "  {}/{} max={} sum={} power={power:?} seed_kind={seed_kind} {cfg:?}",
            s.exact,
            rows.len(),
            s.max,
            s.sum
        );
    }
    println!("local forward-power shape breakdown at step=1e-10:");
    for power in [
        PricePower::Corrected,
        PricePower::PlatformPowf,
        PricePower::X87Chain,
        PricePower::RepeatedMultiply,
    ] {
        let cfg = NewtonCfg {
            ops: Ops::Native,
            step: Step::Absolute(1e-6),
            difference: Difference::Central,
            stop: Stop::Step(1e-10),
            publish: Publish::Old,
            cap: 100,
            seed: 0.0,
        };
        println!("  power={power:?}");
        for shape in discovery_shapes() {
            let shape_rows: Vec<Row> = rows
                .iter()
                .filter(|row| row.tag.contains(shape.id))
                .cloned()
                .collect();
            let s = score(&shape_rows, |row| {
                let seed = approximate_yield_seed_ext(row, seed_spill)?;
                newton_local(row, NewtonCfg { seed, ..cfg }, power)
            });
            println!(
                "    {:<28} {}/{} max={} sum={}",
                shape.id,
                s.exact,
                shape_rows.len(),
                s.max,
                s.sum
            );
        }
    }
    let mut period_policy_candidates = Vec::new();
    for gain_policy in [
        PeriodPolicy::IntegerCouponCount,
        PeriodPolicy::FractionalRemaining,
        PeriodPolicy::DirectRemaining,
    ] {
        for weight_policy in [
            PeriodPolicy::IntegerCouponCount,
            PeriodPolicy::FractionalRemaining,
            PeriodPolicy::DirectRemaining,
        ] {
            for ops in [Ops::Native, Ops::X87] {
                let cfg = NewtonCfg {
                    ops,
                    step: Step::Absolute(1e-6),
                    difference: Difference::Central,
                    stop: Stop::Step(1e-10),
                    publish: Publish::Old,
                    cap: 100,
                    seed: 0.0,
                };
                let s = score(&rows, |row| {
                    let seed =
                        approximate_yield_seed_periods(row, ops, gain_policy, weight_policy)?;
                    newton(row, NewtonCfg { seed, ..cfg })
                });
                period_policy_candidates.push((s, gain_policy, weight_policy, cfg));
            }
        }
    }
    period_policy_candidates.sort_by_key(|item| item.0.rank());
    println!(
        "off-coupon seed-period policy candidates={}:",
        period_policy_candidates.len()
    );
    for (s, gain_policy, weight_policy, cfg) in &period_policy_candidates {
        println!(
            "  {}/{} max={} sum={} gain={gain_policy:?} weight={weight_policy:?} {cfg:?}",
            s.exact,
            rows.len(),
            s.max,
            s.sum
        );
        for shape in discovery_shapes() {
            let shape_rows: Vec<Row> = rows
                .iter()
                .filter(|row| row.tag.contains(shape.id))
                .cloned()
                .collect();
            let ss = score(&shape_rows, |row| {
                let seed =
                    approximate_yield_seed_periods(row, cfg.ops, *gain_policy, *weight_policy)?;
                newton(row, NewtonCfg { seed, ..*cfg })
            });
            println!(
                "    {:<28} {}/{} max={} sum={}",
                shape.id,
                ss.exact,
                shape_rows.len(),
                ss.max,
                ss.sum
            );
        }
    }
    let mut frequency_scaled_steps = Vec::new();
    for scale_policy in 0..6 {
        for difference in [
            Difference::Forward,
            Difference::Backward,
            Difference::Central,
        ] {
            let s = score(&rows, |row| {
                let h = match scale_policy {
                    0 => 1e-6,
                    1 => 1e-6 * row.bond.frequency,
                    2 => 1e-6 / row.bond.frequency,
                    3 => 2e-6 / row.bond.frequency,
                    4 => 5e-7 * row.bond.frequency,
                    _ => 1e-6 * row.bond.frequency.sqrt(),
                };
                let seed = approximate_yield_seed_periods(
                    row,
                    Ops::Native,
                    PeriodPolicy::DirectRemaining,
                    PeriodPolicy::IntegerCouponCount,
                )?;
                newton(
                    row,
                    NewtonCfg {
                        ops: Ops::Native,
                        step: Step::Absolute(h),
                        difference,
                        stop: Stop::Step(1e-10),
                        publish: Publish::Old,
                        cap: 100,
                        seed,
                    },
                )
            });
            frequency_scaled_steps.push((s, scale_policy, difference));
        }
    }
    frequency_scaled_steps.sort_by_key(|item| item.0.rank());
    println!("frequency-scaled derivative-step candidates:");
    for (s, scale_policy, difference) in &frequency_scaled_steps {
        println!(
            "  {}/{} max={} sum={} scale_policy={} difference={difference:?}",
            s.exact,
            rows.len(),
            s.max,
            s.sum,
            scale_policy
        );
    }
    println!("representative seeded Newton traces:");
    for shape in discovery_shapes() {
        for index in [6_usize, 25_usize] {
            let suffix = format!("{}-{index:03}", shape.id);
            let row = rows
                .iter()
                .find(|row| row.tag.ends_with(&suffix))
                .expect("representative discovery row");
            let seed = approximate_yield_seed_ext(row, seed_spill).unwrap();
            print_newton_trace(row, seed);
        }
    }
}

fn score_seed_input_refinements() {
    let path = PathBuf::from(ARTIFACT_ROOT).join("answers-yield-near-seed-discovery-20260809.json");
    let answered: AnsweredBatch = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    assert_eq!(answered.function, "YIELD");
    assert_eq!(answered.witnesses.len(), 384);
    let rows = rows_from_answered(&answered);
    let base = NewtonCfg {
        ops: Ops::Native,
        step: Step::Absolute(1e-6),
        difference: Difference::Central,
        stop: Stop::Step(1e-10),
        publish: Publish::Old,
        cap: 100,
        seed: 0.0,
    };
    let mut candidates = Vec::new();
    for ops in [Ops::Native, Ops::X87] {
        for gain_policy in [
            PeriodPolicy::IntegerCouponCount,
            PeriodPolicy::FractionalRemaining,
            PeriodPolicy::DirectRemaining,
        ] {
            for weight_policy in [
                PeriodPolicy::IntegerCouponCount,
                PeriodPolicy::FractionalRemaining,
                PeriodPolicy::DirectRemaining,
            ] {
                for spread_accrual in [
                    AccrualPolicy::Clean,
                    AccrualPolicy::DerivedElapsed,
                    AccrualPolicy::DirectElapsed,
                ] {
                    for weight_accrual in [
                        AccrualPolicy::Clean,
                        AccrualPolicy::DerivedElapsed,
                        AccrualPolicy::DirectElapsed,
                    ] {
                        let cfg = NewtonCfg { ops, ..base };
                        let s = score(&rows, |row| {
                            let seed = approximate_yield_seed_inputs(
                                row,
                                ops,
                                gain_policy,
                                weight_policy,
                                spread_accrual,
                                weight_accrual,
                            )?;
                            newton(row, NewtonCfg { seed, ..cfg })
                        });
                        candidates.push((
                            s,
                            ops,
                            gain_policy,
                            weight_policy,
                            spread_accrual,
                            weight_accrual,
                        ));
                    }
                }
            }
        }
    }
    candidates.sort_by_key(|item| item.0.rank());
    println!("seed input-refinement candidates={}:", candidates.len());
    for (s, ops, gain, weight, spread_accrual, weight_accrual) in candidates.iter().take(40) {
        println!(
            "  {}/{} max={} sum={} ops={ops:?} gain={gain:?} weight={weight:?} spread={spread_accrual:?} book={weight_accrual:?}",
            s.exact,
            rows.len(),
            s.max,
            s.sum
        );
    }
    let (_, ops, gain, weight, spread_accrual, weight_accrual) = candidates[0];
    println!(
        "best seed input-refinement breakdown ops={ops:?} gain={gain:?} weight={weight:?} spread={spread_accrual:?} book={weight_accrual:?}:"
    );
    for shape in discovery_shapes() {
        let shape_rows: Vec<Row> = rows
            .iter()
            .filter(|row| row.tag.contains(shape.id))
            .cloned()
            .collect();
        let s = score(&shape_rows, |row| {
            let seed = approximate_yield_seed_inputs(
                row,
                ops,
                gain,
                weight,
                spread_accrual,
                weight_accrual,
            )?;
            newton(row, NewtonCfg { seed, ops, ..base })
        });
        println!(
            "  {:<28} {}/{} max={} sum={}",
            shape.id,
            s.exact,
            shape_rows.len(),
            s.max,
            s.sum
        );
    }

    println!("per-shape seed input-refinement leaders:");
    for shape in discovery_shapes() {
        let shape_rows: Vec<Row> = rows
            .iter()
            .filter(|row| row.tag.contains(shape.id))
            .cloned()
            .collect();
        let mut shape_candidates = Vec::new();
        for (_, ops, gain, weight, spread_accrual, weight_accrual) in &candidates {
            let cfg = NewtonCfg { ops: *ops, ..base };
            let s = score(&shape_rows, |row| {
                let seed = approximate_yield_seed_inputs(
                    row,
                    *ops,
                    *gain,
                    *weight,
                    *spread_accrual,
                    *weight_accrual,
                )?;
                newton(row, NewtonCfg { seed, ..cfg })
            });
            shape_candidates.push((s, *ops, *gain, *weight, *spread_accrual, *weight_accrual));
        }
        shape_candidates.sort_by_key(|item| item.0.rank());
        println!("  {}:", shape.id);
        for (s, ops, gain, weight, spread_accrual, weight_accrual) in
            shape_candidates.iter().take(8)
        {
            println!(
                "    {}/{} max={} sum={} ops={ops:?} gain={gain:?} weight={weight:?} spread={spread_accrual:?} book={weight_accrual:?}",
                s.exact,
                shape_rows.len(),
                s.max,
                s.sum
            );
        }
    }
}

fn score_local_forward_refinements() {
    let path = PathBuf::from(ARTIFACT_ROOT).join("answers-yield-near-seed-discovery-20260809.json");
    let answered: AnsweredBatch = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    assert_eq!(answered.function, "YIELD");
    assert_eq!(answered.witnesses.len(), 384);
    let rows = rows_from_answered(&answered);

    println!("local forward-kernel replay:");
    for power in [
        PricePower::Corrected,
        PricePower::PlatformPowf,
        PricePower::X87Chain,
        PricePower::RepeatedMultiply,
    ] {
        let mut exact = 0_usize;
        let mut max = 0_u64;
        let mut sum = 0_u128;
        for row in &rows {
            for y in [0.05, 0.05 - 1e-6, 0.05 + 1e-6] {
                let want = price_for_bond(row.bond, y);
                let got = local_price(row, y, Ops::Native, power).unwrap();
                let distance = ulp(got.to_bits(), want.to_bits());
                exact += usize::from(distance == 0);
                max = max.max(distance);
                sum += u128::from(distance);
            }
        }
        println!(
            "  {exact}/{} max={max} sum={sum} power={power:?}",
            rows.len() * 3
        );
    }

    let mut numeric = Vec::new();
    for power in [
        PricePower::Corrected,
        PricePower::PlatformPowf,
        PricePower::X87Chain,
        PricePower::RepeatedMultiply,
    ] {
        for ops in [Ops::Native, Ops::X87] {
            for step in [
                Step::Absolute(1e-4),
                Step::Absolute(1e-5),
                Step::Absolute(1e-6),
                Step::Absolute(1e-7),
                Step::Absolute(1e-8),
            ] {
                for difference in [
                    Difference::Forward,
                    Difference::Backward,
                    Difference::Central,
                ] {
                    let cfg = NewtonCfg {
                        ops,
                        step,
                        difference,
                        stop: Stop::Step(1e-10),
                        publish: Publish::Old,
                        cap: 100,
                        seed: 0.0,
                    };
                    let s = score(&rows, |row| {
                        let seed = approximate_yield_seed_periods(
                            row,
                            Ops::Native,
                            PeriodPolicy::DirectRemaining,
                            PeriodPolicy::IntegerCouponCount,
                        )?;
                        newton_local(row, NewtonCfg { seed, ..cfg }, power)
                    });
                    numeric.push((s, power, cfg));
                }
            }
        }
    }
    numeric.sort_by_key(|item| item.0.rank());
    println!("local numerical-derivative candidates={}:", numeric.len());
    for (s, power, cfg) in numeric.iter().take(30) {
        println!(
            "  {}/{} max={} sum={} power={power:?} {cfg:?}",
            s.exact,
            rows.len(),
            s.max,
            s.sum
        );
    }

    let mut analytic = Vec::new();
    for ops in [Ops::Native, Ops::X87] {
        for threshold in [
            1e-8,
            1e-9,
            1e-10,
            1e-11,
            1e-12,
            2.0_f64.powi(-33),
            2.0_f64.powi(-34),
        ] {
            for publish in [Publish::Old, Publish::New, Publish::Previous] {
                let cfg = NewtonCfg {
                    ops,
                    step: Step::Absolute(0.0),
                    difference: Difference::Central,
                    stop: Stop::Step(threshold),
                    publish,
                    cap: 100,
                    seed: 0.0,
                };
                let s = score(&rows, |row| {
                    let seed = approximate_yield_seed_periods(
                        row,
                        Ops::Native,
                        PeriodPolicy::DirectRemaining,
                        PeriodPolicy::IntegerCouponCount,
                    )?;
                    analytic_newton(row, NewtonCfg { seed, ..cfg })
                });
                analytic.push((s, cfg));
            }
        }
    }
    analytic.sort_by_key(|item| item.0.rank());
    println!("local analytic-derivative candidates={}:", analytic.len());
    for (s, cfg) in analytic.iter().take(30) {
        println!(
            "  {}/{} max={} sum={} {cfg:?}",
            s.exact,
            rows.len(),
            s.max,
            s.sum
        );
    }
}

fn print_candidate_diagnostics(shape_filter: Option<&str>) {
    let path = PathBuf::from(ARTIFACT_ROOT).join("answers-yield-near-seed-discovery-20260809.json");
    let answered: AnsweredBatch = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    assert_eq!(answered.function, "YIELD");
    assert_eq!(answered.witnesses.len(), 384);
    let rows = rows_from_answered(&answered);
    let cfg = NewtonCfg {
        ops: Ops::Native,
        step: Step::Absolute(1e-6),
        difference: Difference::Central,
        stop: Stop::Step(1e-10),
        publish: Publish::Old,
        cap: 100,
        seed: 0.0,
    };
    for shape in discovery_shapes() {
        if shape_filter.is_some_and(|filter| !shape.id.contains(filter)) {
            continue;
        }
        println!("diagnostics {}:", shape.id);
        let schedule_row = rows
            .iter()
            .find(|row| row.tag.contains(shape.id))
            .expect("shape row");
        let (n, fractional, direct) = coupon_periods(schedule_row, Ops::Native).unwrap();
        let basis = Some(schedule_row.bond.basis);
        let a = coupdaybs_kernel(
            schedule_row.bond.settlement,
            schedule_row.bond.maturity,
            schedule_row.bond.frequency,
            basis,
        )
        .unwrap();
        let e = coupdays_kernel(
            schedule_row.bond.settlement,
            schedule_row.bond.maturity,
            schedule_row.bond.frequency,
            basis,
        )
        .unwrap();
        let nc = coupdaysnc_kernel(
            schedule_row.bond.settlement,
            schedule_row.bond.maturity,
            schedule_row.bond.frequency,
            basis,
        )
        .unwrap();
        println!(
            "  schedule n={n} a={a} e={e} nc={nc} fractional={fractional:.17e} direct={direct:.17e}"
        );
        let anchor = price_for_bond(shape.bond, 0.05);
        for row in rows.iter().filter(|row| row.tag.contains(shape.id)) {
            let seed = approximate_yield_seed_periods(
                row,
                Ops::Native,
                PeriodPolicy::DirectRemaining,
                PeriodPolicy::IntegerCouponCount,
            )
            .unwrap();
            let got = newton(row, NewtonCfg { seed, ..cfg }).unwrap();
            let want = f64::from_bits(row.want);
            let signed_ulp = i128::from(ordered(got.to_bits())) - i128::from(ordered(row.want));
            let got_fx = objective(row, got).unwrap();
            let want_fx = objective(row, want).unwrap();
            let got_dx = initial_correction(row, got).unwrap();
            let want_dx = initial_correction(row, want).unwrap();
            println!(
                "  {} dp={:+.17e} got-want={signed_ulp:+} got_fx={got_fx:+.17e} want_fx={want_fx:+.17e} got_dx={got_dx:+.17e} want_dx={want_dx:+.17e}",
                row.tag,
                row.price - anchor,
            );
            if row.tag.ends_with("-006") {
                print_newton_trace(row, seed);
                let seed_variants = [
                    ("direct/n/clean", Some(seed)),
                    (
                        "direct/m/clean",
                        approximate_yield_seed_inputs(
                            row,
                            Ops::Native,
                            PeriodPolicy::DirectRemaining,
                            PeriodPolicy::DirectRemaining,
                            AccrualPolicy::Clean,
                            AccrualPolicy::Clean,
                        ),
                    ),
                    (
                        "direct/m/dirty-book",
                        approximate_yield_seed_inputs(
                            row,
                            Ops::Native,
                            PeriodPolicy::DirectRemaining,
                            PeriodPolicy::DirectRemaining,
                            AccrualPolicy::Clean,
                            AccrualPolicy::DirectElapsed,
                        ),
                    ),
                    (
                        "direct/n/dirty-book",
                        approximate_yield_seed_inputs(
                            row,
                            Ops::Native,
                            PeriodPolicy::DirectRemaining,
                            PeriodPolicy::IntegerCouponCount,
                            AccrualPolicy::Clean,
                            AccrualPolicy::DirectElapsed,
                        ),
                    ),
                    (
                        "textbook/clean",
                        approximate_yield_seed(row, Ops::Native, SeedFormula::TextbookAverage),
                    ),
                    ("fixed-0.05", Some(0.05)),
                    ("coupon-rate", Some(row.bond.coupon)),
                ];
                println!("  two-step seed variants:");
                for (label, seed_variant) in seed_variants {
                    let seed_variant = seed_variant.unwrap();
                    let x1 = one_numeric_step(
                        row,
                        NewtonCfg {
                            seed: seed_variant,
                            ..cfg
                        },
                    )
                    .unwrap();
                    let x2 = one_numeric_step(row, NewtonCfg { seed: x1, ..cfg }).unwrap();
                    let signed = i128::from(ordered(x2.to_bits())) - i128::from(ordered(row.want));
                    println!(
                        "    {label:<24} seed=0x{:016x} x1=0x{:016x} x2=0x{:016x} x2-want={signed:+} fx={:+.17e}",
                        seed_variant.to_bits(),
                        x1.to_bits(),
                        x2.to_bits(),
                        objective(row, x2).unwrap(),
                    );
                }
            }
        }
    }
}

fn score_derivative_step_refinements() {
    let path = PathBuf::from(ARTIFACT_ROOT).join("answers-yield-near-seed-discovery-20260809.json");
    let answered: AnsweredBatch = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    assert_eq!(answered.function, "YIELD");
    assert_eq!(answered.witnesses.len(), 384);
    let rows = rows_from_answered(&answered);
    let mut steps = vec![
        Step::Absolute(1e-2),
        Step::Absolute(1e-3),
        Step::Absolute(1e-4),
        Step::Absolute(1e-5),
        Step::Absolute(1e-6),
        Step::Absolute(1e-7),
        Step::Absolute(1e-8),
        Step::Absolute(1e-9),
        Step::Absolute(1e-10),
    ];
    for multiplier in [0.125, 0.25, 0.5, 0.75, 1.25, 1.5, 2.0, 3.0, 4.0, 5.0, 8.0] {
        steps.push(Step::Absolute(multiplier * 1e-6));
    }
    for power in 16..=32 {
        steps.push(Step::Absolute(2.0_f64.powi(-power)));
    }
    for scale in [1e-2, 1e-3, 1e-4, 1e-5, 1e-6, 1e-7] {
        steps.push(Step::RawRelative(scale));
    }

    let mut global_candidates = Vec::new();
    for &step in &steps {
        for difference in [
            Difference::Forward,
            Difference::Backward,
            Difference::Central,
        ] {
            let cfg = NewtonCfg {
                ops: Ops::Native,
                step,
                difference,
                stop: Stop::Step(1e-10),
                publish: Publish::Old,
                cap: 100,
                seed: 0.0,
            };
            for weight_policy in [
                PeriodPolicy::IntegerCouponCount,
                PeriodPolicy::FractionalRemaining,
                PeriodPolicy::DirectRemaining,
            ] {
                for book_accrual in [
                    AccrualPolicy::Clean,
                    AccrualPolicy::DerivedElapsed,
                    AccrualPolicy::DirectElapsed,
                ] {
                    let s = score(&rows, |row| {
                        let seed = approximate_yield_seed_inputs(
                            row,
                            Ops::Native,
                            PeriodPolicy::DirectRemaining,
                            weight_policy,
                            AccrualPolicy::Clean,
                            book_accrual,
                        )?;
                        newton(row, NewtonCfg { seed, ..cfg })
                    });
                    global_candidates.push((s, cfg, weight_policy, book_accrual));
                }
            }
        }
    }
    global_candidates.sort_by_key(|item| item.0.rank());
    println!(
        "global derivative-step leaders={}:",
        global_candidates.len()
    );
    for (s, cfg, weight_policy, book_accrual) in global_candidates.iter().take(40) {
        println!(
            "  {}/{} max={} sum={} weight={weight_policy:?} book={book_accrual:?} {cfg:?}",
            s.exact,
            rows.len(),
            s.max,
            s.sum
        );
    }
    let mut minimax_candidates = global_candidates.clone();
    minimax_candidates.sort_by_key(|item| (item.0.max, item.0.sum, Reverse(item.0.exact)));
    println!("global derivative-step minimax leaders:");
    for (s, cfg, weight_policy, book_accrual) in minimax_candidates.iter().take(40) {
        println!(
            "  {}/{} max={} sum={} weight={weight_policy:?} book={book_accrual:?} {cfg:?}",
            s.exact,
            rows.len(),
            s.max,
            s.sum
        );
    }

    println!("per-shape derivative-step leaders:");
    for shape in discovery_shapes() {
        let shape_rows: Vec<Row> = rows
            .iter()
            .filter(|row| row.tag.contains(shape.id))
            .cloned()
            .collect();
        let mut candidates = Vec::new();
        for &step in &steps {
            for difference in [
                Difference::Forward,
                Difference::Backward,
                Difference::Central,
            ] {
                let cfg = NewtonCfg {
                    ops: Ops::Native,
                    step,
                    difference,
                    stop: Stop::Step(1e-10),
                    publish: Publish::Old,
                    cap: 100,
                    seed: 0.0,
                };
                for weight_policy in [
                    PeriodPolicy::IntegerCouponCount,
                    PeriodPolicy::FractionalRemaining,
                    PeriodPolicy::DirectRemaining,
                ] {
                    for book_accrual in [
                        AccrualPolicy::Clean,
                        AccrualPolicy::DerivedElapsed,
                        AccrualPolicy::DirectElapsed,
                    ] {
                        let s = score(&shape_rows, |row| {
                            let seed = approximate_yield_seed_inputs(
                                row,
                                Ops::Native,
                                PeriodPolicy::DirectRemaining,
                                weight_policy,
                                AccrualPolicy::Clean,
                                book_accrual,
                            )?;
                            newton(row, NewtonCfg { seed, ..cfg })
                        });
                        candidates.push((s, cfg, weight_policy, book_accrual));
                    }
                }
            }
        }
        candidates.sort_by_key(|item| item.0.rank());
        println!("  {} candidates={}:", shape.id, candidates.len());
        for (s, cfg, weight_policy, book_accrual) in candidates.iter().take(16) {
            println!(
                "    {}/{} max={} sum={} weight={weight_policy:?} book={book_accrual:?} {cfg:?}",
                s.exact,
                shape_rows.len(),
                s.max,
                s.sum
            );
        }
    }
}

fn transformed_weight_periods(row: &Row, ops: Ops, policy: u8) -> Option<f64> {
    let (n, fractional, direct) = coupon_periods(row, ops)?;
    let basis = Some(row.bond.basis);
    let a = coupdaybs_kernel(
        row.bond.settlement,
        row.bond.maturity,
        row.bond.frequency,
        basis,
    )
    .ok()?;
    let e = coupdays_kernel(
        row.bond.settlement,
        row.bond.maturity,
        row.bond.frequency,
        basis,
    )
    .ok()?;
    let direct_days = coupdaysnc_kernel(
        row.bond.settlement,
        row.bond.maturity,
        row.bond.frequency,
        basis,
    )
    .ok()?;
    let elapsed = ops.div(a, e);
    let direct_elapsed = ops.div(ops.sub(e, direct_days), e);
    let derived_off = ops.sub(1.0, elapsed);
    let direct_off = ops.div(direct_days, e);
    let weight = match policy {
        0 => n,
        1 => fractional,
        2 => direct,
        3 => ops.add(n, elapsed),
        4 => ops.add(n, direct_elapsed),
        5 => ops.div(n, derived_off),
        6 => ops.div(n, direct_off),
        7 => ops.div(fractional, derived_off),
        8 => ops.div(direct, direct_off),
        9 => ops.mul(n, row.bond.frequency),
        10 => ops.add(n, ops.mul(2.0, elapsed)),
        11 => ops.add(n, ops.mul(4.0, elapsed)),
        12 => ops.add(n, ops.mul(8.0, elapsed)),
        13 => ops.add(n, ops.div(elapsed, derived_off)),
        14 => ops.add(n, ops.div(direct_elapsed, direct_off)),
        15 => ops.add(1.0, ops.div(ops.sub(n, 1.0), derived_off)),
        16 => ops.add(1.0, ops.div(ops.sub(n, 1.0), direct_off)),
        17 => ops.div(n, derived_off.sqrt()),
        18 => ops.div(n, direct_off.sqrt()),
        19 => 1e300,
        20 => ops.div(n, ops.mul(derived_off, derived_off)),
        21 => ops.div(n, ops.mul(direct_off, direct_off)),
        22 => ops.div(n, ops.mul(ops.mul(derived_off, derived_off), derived_off)),
        23 => ops.div(n, ops.mul(ops.mul(direct_off, direct_off), direct_off)),
        24 => ops.div(fractional, ops.mul(derived_off, derived_off)),
        25 => ops.div(direct, ops.mul(direct_off, direct_off)),
        26 => ops.div(n, ops.mul(derived_off, direct_off)),
        _ => ops.div(
            n,
            ops.mul(
                ops.mul(derived_off, derived_off),
                ops.mul(derived_off, derived_off),
            ),
        ),
    };
    (weight > 0.0 && weight.is_finite()).then_some(weight)
}

fn weight_policy_name(policy: u8) -> &'static str {
    match policy {
        0 => "n",
        1 => "fractional",
        2 => "direct",
        3 => "n+elapsed",
        4 => "n+direct-elapsed",
        5 => "n/derived-off",
        6 => "n/direct-off",
        7 => "fractional/derived-off",
        8 => "direct/direct-off",
        9 => "n*frequency",
        10 => "n+2elapsed",
        11 => "n+4elapsed",
        12 => "n+8elapsed",
        13 => "n+elapsed/off",
        14 => "n+direct-elapsed/direct-off",
        15 => "1+(n-1)/derived-off",
        16 => "1+(n-1)/direct-off",
        17 => "n/sqrt(derived-off)",
        18 => "n/sqrt(direct-off)",
        19 => "textbook-limit",
        20 => "n/derived-off^2",
        21 => "n/direct-off^2",
        22 => "n/derived-off^3",
        23 => "n/direct-off^3",
        24 => "fractional/derived-off^2",
        25 => "direct/direct-off^2",
        26 => "n/(derived-off*direct-off)",
        _ => "n/derived-off^4",
    }
}

const SEED_FAMILY_NAMES: [&str; 10] = [
    "direct-gain_n-weight",
    "fractional-gain_n-weight",
    "fractional-gain_n-over-derived-off-squared",
    "fractional-gain_n-over-direct-off-squared",
    "fractional-gain_fractional-over-derived-off-squared",
    "direct-gain_direct-over-direct-off-squared",
    "direct-gain_n-over-direct-off",
    "direct-gain_n-plus-elapsed",
    "fractional-gain_textbook-average",
    "direct-gain_n-weight_dirty-book",
];

fn seed_family_value_ops(row: &Row, family: u8, ops: Ops) -> Option<f64> {
    match family {
        0 => approximate_yield_seed_periods(
            row,
            ops,
            PeriodPolicy::DirectRemaining,
            PeriodPolicy::IntegerCouponCount,
        ),
        1 => approximate_yield_seed_periods(
            row,
            ops,
            PeriodPolicy::FractionalRemaining,
            PeriodPolicy::IntegerCouponCount,
        ),
        2 | 3 | 4 | 5 | 6 | 7 => {
            let (_, fractional, direct) = coupon_periods(row, ops)?;
            let gain = if family == 5 || family == 6 || family == 7 {
                direct
            } else {
                fractional
            };
            let weight_policy = match family {
                2 => 20,
                3 => 21,
                4 => 24,
                5 => 25,
                6 => 6,
                _ => 3,
            };
            let weight = transformed_weight_periods(row, ops, weight_policy)?;
            approximate_yield_seed_explicit_periods(row, ops, gain, weight)
        }
        8 => approximate_yield_seed(row, ops, SeedFormula::TextbookAverage),
        _ => approximate_yield_seed_inputs(
            row,
            ops,
            PeriodPolicy::DirectRemaining,
            PeriodPolicy::IntegerCouponCount,
            AccrualPolicy::Clean,
            AccrualPolicy::DirectElapsed,
        ),
    }
}

fn seed_family_value(row: &Row, family: u8) -> Option<f64> {
    seed_family_value_ops(row, family, Ops::Native)
}

fn seed_family_fixed_point(bond: Bond, family: u8) -> Option<f64> {
    let value = |yield_value: f64| {
        let price = price_for_bond(bond, yield_value);
        let row = Row {
            tag: String::new(),
            bond,
            price,
            want: 0,
        };
        seed_family_value(&row, family).map(|seed| seed - yield_value)
    };
    let low_yield = 1e-8_f64;
    let high_yield = 0.5_f64;
    let steps = 20_000_usize;
    let mut previous_yield = low_yield;
    let mut previous_value = value(previous_yield)?;
    let mut brackets = Vec::new();
    for index in 1..=steps {
        let yield_value = low_yield + (high_yield - low_yield) * (index as f64) / (steps as f64);
        let current_value = value(yield_value)?;
        if current_value == 0.0 || previous_value.signum() != current_value.signum() {
            brackets.push((previous_yield, yield_value));
        }
        previous_yield = yield_value;
        previous_value = current_value;
    }
    let (mut low, mut high) = brackets.into_iter().min_by(|a, b| {
        let da = ((a.0 + a.1) * 0.5 - 0.05).abs();
        let db = ((b.0 + b.1) * 0.5 - 0.05).abs();
        da.total_cmp(&db)
    })?;
    let mut low_value = value(low)?;
    for _ in 0..160 {
        let mid = (low + high) * 0.5;
        if mid == low || mid == high {
            break;
        }
        let mid_value = value(mid)?;
        if mid_value == 0.0 {
            low = mid;
            high = mid;
            break;
        }
        if low_value.signum() == mid_value.signum() {
            low = mid;
            low_value = mid_value;
        } else {
            high = mid;
        }
    }
    [low, high, (low + high) * 0.5].into_iter().min_by(|a, b| {
        value(*a)
            .unwrap()
            .abs()
            .total_cmp(&value(*b).unwrap().abs())
    })
}

fn score_weight_period_refinements() {
    let path = PathBuf::from(ARTIFACT_ROOT).join("answers-yield-near-seed-discovery-20260809.json");
    let answered: AnsweredBatch = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    assert_eq!(answered.function, "YIELD");
    assert_eq!(answered.witnesses.len(), 384);
    let rows = rows_from_answered(&answered);
    let cfg = NewtonCfg {
        ops: Ops::Native,
        step: Step::Absolute(1e-6),
        difference: Difference::Central,
        stop: Stop::Step(1e-10),
        publish: Publish::Old,
        cap: 100,
        seed: 0.0,
    };
    let mut global = Vec::new();
    for policy in 0_u8..=27 {
        for gain_policy in [
            PeriodPolicy::FractionalRemaining,
            PeriodPolicy::DirectRemaining,
        ] {
            let s = score(&rows, |row| {
                let (_, fractional, direct) = coupon_periods(row, Ops::Native)?;
                let gain = match gain_policy {
                    PeriodPolicy::FractionalRemaining => fractional,
                    PeriodPolicy::DirectRemaining => direct,
                    PeriodPolicy::IntegerCouponCount => unreachable!(),
                };
                let weight = transformed_weight_periods(row, Ops::Native, policy)?;
                let seed = approximate_yield_seed_explicit_periods(row, Ops::Native, gain, weight)?;
                newton(row, NewtonCfg { seed, ..cfg })
            });
            global.push((s, policy, gain_policy));
        }
    }
    global.sort_by_key(|item| item.0.rank());
    println!("global transformed-weight candidates={}:", global.len());
    for (s, policy, gain_policy) in &global {
        println!(
            "  {}/{} max={} sum={} gain={gain_policy:?} weight={}",
            s.exact,
            rows.len(),
            s.max,
            s.sum,
            weight_policy_name(*policy),
        );
    }
    println!("per-shape transformed-weight leaders:");
    for shape in discovery_shapes() {
        let shape_rows: Vec<Row> = rows
            .iter()
            .filter(|row| row.tag.contains(shape.id))
            .cloned()
            .collect();
        let mut candidates = Vec::new();
        for policy in 0_u8..=27 {
            for gain_policy in [
                PeriodPolicy::FractionalRemaining,
                PeriodPolicy::DirectRemaining,
            ] {
                let s = score(&shape_rows, |row| {
                    let (_, fractional, direct) = coupon_periods(row, Ops::Native)?;
                    let gain = match gain_policy {
                        PeriodPolicy::FractionalRemaining => fractional,
                        PeriodPolicy::DirectRemaining => direct,
                        PeriodPolicy::IntegerCouponCount => unreachable!(),
                    };
                    let weight = transformed_weight_periods(row, Ops::Native, policy)?;
                    let seed =
                        approximate_yield_seed_explicit_periods(row, Ops::Native, gain, weight)?;
                    newton(row, NewtonCfg { seed, ..cfg })
                });
                candidates.push((s, policy, gain_policy));
            }
        }
        candidates.sort_by_key(|item| item.0.rank());
        println!("  {}:", shape.id);
        for (s, policy, gain_policy) in candidates.iter().take(12) {
            println!(
                "    {}/{} max={} sum={} gain={gain_policy:?} weight={}",
                s.exact,
                shape_rows.len(),
                s.max,
                s.sum,
                weight_policy_name(*policy),
            );
        }
    }
}

fn one_numeric_step(row: &Row, cfg: NewtonCfg) -> Option<f64> {
    let x = if cfg.seed.is_nan() {
        row.bond.coupon
    } else {
        cfg.seed
    };
    let fx = objective(row, x)?;
    let h = cfg.step.at(x, cfg.ops);
    let derivative = match cfg.difference {
        Difference::Forward => {
            let xp = cfg.ops.add(x, h);
            cfg.ops.div(cfg.ops.sub(objective(row, xp)?, fx), h)
        }
        Difference::Backward => {
            let xm = cfg.ops.sub(x, h);
            cfg.ops.div(cfg.ops.sub(fx, objective(row, xm)?), h)
        }
        Difference::Central => {
            let xp = cfg.ops.add(x, h);
            let xm = cfg.ops.sub(x, h);
            cfg.ops.div(
                cfg.ops.sub(objective(row, xp)?, objective(row, xm)?),
                cfg.ops.mul(2.0, h),
            )
        }
    };
    if derivative == 0.0 || !derivative.is_finite() {
        return None;
    }
    let dx = cfg.ops.div(fx, derivative);
    let next = cfg.ops.sub(x, dx);
    next.is_finite().then_some(next)
}

fn one_step_graph(row: &Row, graph: FirstStepGraph) -> Option<f64> {
    let x = 0.05_f64;
    let h = graph.step.at(x, Ops::Native);
    let f0 = objective(row, x)?;
    let (probe, f1, denominator, central_scale) = match graph.difference {
        Difference::Forward => {
            let probe = x + h;
            (probe, objective(row, probe)?, None, 1.0)
        }
        Difference::Backward => {
            let probe = x - h;
            (probe, objective(row, probe)?, None, -1.0)
        }
        Difference::Central => {
            let plus = objective(row, x + h)?;
            let minus = objective(row, x - h)?;
            (x + h, plus, Some(minus), 2.0)
        }
    };
    if !probe.is_finite() || h == 0.0 {
        return None;
    }
    let bit = |index: u32| graph.spill_mask & (1_u16 << index) != 0;
    let f0 = V::new(f0).st(bit(0));
    let f1 = V::new(f1).st(bit(1));
    let h_v = V::new(h);
    let difference = match graph.difference {
        Difference::Forward => f1.sub(f0),
        Difference::Backward => f0.sub(f1),
        Difference::Central => f1.sub(V::new(denominator.unwrap()).st(bit(1))),
    }
    .st(bit(2));
    let scaled_h = h_v.mul(V::new(central_scale)).st(bit(3));
    let correction = match graph.form {
        UpdateForm::DerivativeThenDivide => {
            let derivative = difference.div(scaled_h).st(bit(4));
            f0.div(derivative)
        }
        UpdateForm::MultiplyThenDivide => f0.mul(scaled_h).st(bit(4)).div(difference),
        UpdateForm::DivideThenMultiply => f0.div(difference).st(bit(4)).mul(scaled_h),
        UpdateForm::HOverDifferenceThenMultiply => scaled_h.div(difference).st(bit(4)).mul(f0),
        UpdateForm::MultiplyReciprocalDifference => f0
            .mul(scaled_h)
            .st(bit(4))
            .mul(V::new(1.0).div(difference).st(bit(5))),
    }
    .st(bit(6));
    let next = V::new(x).sub(correction).st(bit(7)).f();
    next.is_finite().then_some(next)
}

fn discriminator_models() -> Vec<NewtonCfg> {
    let mut models = Vec::new();
    for ops in [Ops::Native, Ops::X87] {
        for step in [
            Step::Absolute(1e-3),
            Step::Absolute(1e-4),
            Step::Absolute(1e-5),
            Step::Absolute(1e-6),
            Step::Absolute(1e-7),
            Step::Absolute(1e-8),
        ] {
            for difference in [
                Difference::Forward,
                Difference::Backward,
                Difference::Central,
            ] {
                for seed in [0.05, f64::NAN] {
                    models.push(NewtonCfg {
                        ops,
                        step,
                        difference,
                        stop: Stop::Fixed,
                        publish: Publish::New,
                        cap: 1,
                        seed,
                    });
                }
            }
        }
    }
    models
}

fn score_near_seed_target(
    shape: Shape,
    anchor: f64,
    class: &'static str,
    target: f64,
) -> NearSeedProbe {
    let row = Row {
        tag: shape.id.to_owned(),
        bond: shape.bond,
        price: target,
        want: 0,
    };
    let mut predictions = BTreeSet::new();
    for cfg in discriminator_models() {
        if let Some(value) = one_numeric_step(&row, cfg) {
            predictions.insert(value.to_bits());
        }
    }
    let low = predictions.iter().map(|bits| ordered(*bits) as i128).min();
    let high = predictions.iter().map(|bits| ordered(*bits) as i128).max();
    NearSeedProbe {
        class,
        target,
        delta_from_anchor: target - anchor,
        distinct_predictions: predictions.len(),
        prediction_span_ulp: match (low, high) {
            (Some(low), Some(high)) => high - low,
            _ => 0,
        },
    }
}

fn ranked_pool(shape: Shape) -> Vec<NearSeedProbe> {
    let seed = 0.05;
    let anchor = price_for_bond(shape.bond, seed);
    let mut seen = BTreeSet::new();
    let mut ulp_ladder = Vec::new();
    for offset in [
        -1024_i64, -256, -64, -16, -4, -1, 0, 1, 4, 16, 64, 256, 1024,
    ] {
        let bits = (anchor.to_bits() as i128 + offset as i128) as u64;
        let target = f64::from_bits(bits);
        if target > 0.0 && seen.insert(target.to_bits()) {
            ulp_ladder.push(score_near_seed_target(shape, anchor, "ulp-ladder", target));
        }
    }

    let mut tiny = Vec::new();
    let mut threshold = Vec::new();
    let mut far = Vec::new();
    for exponent in -42..=-12 {
        for sign in [-1.0, 1.0] {
            for mantissa in [1.0, 3.0] {
                let target = anchor + sign * mantissa * 2.0_f64.powi(exponent);
                if target <= 0.0 || !seen.insert(target.to_bits()) {
                    continue;
                }
                let (class, lane) = if exponent <= -31 {
                    ("delta-tiny", &mut tiny)
                } else if exponent <= -16 {
                    ("delta-threshold", &mut threshold)
                } else {
                    ("delta-far", &mut far)
                };
                lane.push(score_near_seed_target(shape, anchor, class, target));
            }
        }
    }
    let rank = |items: &mut Vec<NearSeedProbe>| {
        items.sort_by(|a, b| {
            b.distinct_predictions
                .cmp(&a.distinct_predictions)
                .then_with(|| b.prediction_span_ulp.cmp(&a.prediction_span_ulp))
                .then_with(|| a.target.to_bits().cmp(&b.target.to_bits()))
        });
    };
    rank(&mut tiny);
    rank(&mut threshold);
    rank(&mut far);
    ulp_ladder.extend(tiny.into_iter().take(12));
    ulp_ladder.extend(threshold.into_iter().take(16));
    ulp_ladder.extend(far.into_iter().take(7));
    assert_eq!(ulp_ladder.len(), 48);
    ulp_ladder
}

fn yield_args(bond: Bond, target: f64) -> [String; 7] {
    [
        hex(bond.settlement),
        hex(bond.maturity),
        hex(bond.coupon),
        hex(target),
        hex(bond.redemption),
        hex(bond.frequency),
        hex(bond.basis),
    ]
}

fn price_args(bond: Bond, y: f64) -> [String; 7] {
    [
        hex(bond.settlement),
        hex(bond.maturity),
        hex(bond.coupon),
        hex(y),
        hex(bond.redemption),
        hex(bond.frequency),
        hex(bond.basis),
    ]
}

fn build_yield_split(split: &str, shapes: Vec<Shape>) -> (FrozenBatch, NearSeedBank) {
    let mut probes = Vec::new();
    let mut records = Vec::new();
    for shape in shapes {
        let anchor = price_for_bond(shape.bond, 0.05);
        for (index, selected) in ranked_pool(shape).into_iter().enumerate() {
            let id = format!("yield-near-seed-{split}-{}-{index:03}", shape.id);
            let args = yield_args(shape.bond, selected.target);
            probes.push(RankedFrozenProbe {
                probe: FrozenProbe {
                    id: id.clone(),
                    args: args.clone(),
                },
                distinct_outputs: selected.distinct_predictions,
                prediction_span_ulp: selected.prediction_span_ulp,
            });
            records.push(NearSeedRecord {
                id,
                split: split.to_owned(),
                shape: shape.id.to_owned(),
                class: selected.class.to_owned(),
                args_bits: args,
                seed: hex(0.05),
                worksheet_price_anchor_model_bits: hex(anchor),
                target_delta_from_anchor_bits: hex(selected.delta_from_anchor),
                distinct_one_step_predictions: selected.distinct_predictions,
                prediction_span_ulp: selected.prediction_span_ulp,
            });
        }
    }
    (
        FrozenBatch {
            function: "YIELD",
            row_id: format!("G6-03-yield-near-seed-{split}-v1-20260809"),
            probes,
        },
        NearSeedBank {
            schema_version: "oxfunc.w109.yield_near_seed_dataset_bank.v1",
            freeze_id: FREEZE_ID,
            answer_blind: true,
            split: split.to_owned(),
            records,
        },
    )
}

fn build_price_companion(shapes: Vec<Shape>) -> (FrozenBatch, CompanionBank) {
    let mut probes = Vec::new();
    let mut records = Vec::new();
    let seed = 0.05;
    let steps = [
        1e-3,
        1e-4,
        1e-5,
        1e-6,
        1e-7,
        1e-8,
        2.0_f64.powi(-20),
        2.0_f64.powi(-24),
    ];
    for shape in shapes {
        let mut points = vec![("seed".to_owned(), seed)];
        for h in steps {
            points.push((format!("plus-{}", hex(h)), seed + h));
            points.push((format!("minus-{}", hex(h)), seed - h));
        }
        for (index, (role, y)) in points.into_iter().enumerate() {
            let id = format!("price-yield-companion-{}-{index:03}", shape.id);
            let args = price_args(shape.bond, y);
            probes.push(RankedFrozenProbe {
                probe: FrozenProbe {
                    id: id.clone(),
                    args: args.clone(),
                },
                distinct_outputs: 0,
                prediction_span_ulp: 0,
            });
            records.push(CompanionRecord {
                id,
                shape: shape.id.to_owned(),
                role,
                args_bits: args,
            });
        }
    }
    (
        FrozenBatch {
            function: "PRICE",
            row_id: "G6-03-yield-price-companion-discovery-v1-20260809".to_owned(),
            probes,
        },
        CompanionBank {
            schema_version: "oxfunc.w109.yield_price_companion_bank.v1",
            freeze_id: FREEZE_ID,
            answer_blind: true,
            records,
        },
    )
}

fn pretty<T: Serialize>(value: &T) -> Vec<u8> {
    let mut bytes = serde_json::to_vec_pretty(value).unwrap();
    bytes.push(b'\n');
    bytes
}

fn write_frozen(path: &Path, bytes: &[u8]) {
    if let Ok(existing) = std::fs::read(path) {
        assert_eq!(
            existing,
            bytes,
            "refusing to overwrite changed frozen artifact {}",
            path.display()
        );
        println!("verified frozen {}", path.display());
        return;
    }
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, bytes).unwrap();
    println!("wrote frozen {}", path.display());
}

fn generate_frozen_near_seed_battery() {
    let models = discriminator_models();
    assert_eq!(models.len(), 72);
    let discovery_shapes = discovery_shapes();
    let (discovery, discovery_meta) = build_yield_split("discovery", discovery_shapes.clone());
    let (heldout, heldout_meta) = build_yield_split("heldout", heldout_shapes());
    let (companion, companion_meta) = build_price_companion(discovery_shapes);
    assert_eq!(discovery.probes.len(), 384);
    assert_eq!(heldout.probes.len(), 288);
    assert_eq!(companion.probes.len(), 136);
    let manifest = json!({
        "schema_version": "oxfunc.w109.yield_near_seed_candidate_manifest.v1",
        "freeze_id": FREEZE_ID,
        "answer_blind": true,
        "clean_room": true,
        "public_basis": {
            "excel_documentation": "YIELD uses Newton iteration based on PRICE, up to 100 iterations",
            "candidate_implementation_reference": "public ExcelFinancialFunctions findRoot family only; empirical Excel behavior remains authoritative"
        },
        "fixed_nodes": {
            "candidate_objective": "current corrected PRICE forward kernel",
            "near_seed_anchor": hex(0.05),
            "numeric_input_plumbing": "binary64 bits -> Range.Value2 cells -> relative Formula2R1C1",
        },
        "candidate_space": {
            "method": "one numeric Newton update",
            "outer_arithmetic": ["binary64", "x87 RN53(RN64(op)) spill"],
            "difference": ["forward", "backward", "central"],
            "absolute_h": [1e-3, 1e-4, 1e-5, 1e-6, 1e-7, 1e-8],
            "seed_policy": ["fixed 0.05", "coupon rate"],
            "candidate_count": models.len(),
        },
        "selection": {
            "method": "rank a dense adjacent-target pool by distinct one-step outputs and ULP span, then retain balanced tiny/threshold/far strata per bond shape",
            "discovery_shapes": 8,
            "discovery_rows": discovery.probes.len(),
            "sealed_heldout_shapes": 6,
            "sealed_heldout_rows": heldout.probes.len(),
            "price_companion_rows": companion.probes.len(),
        },
        "scope": {
            "discovery": "seed policy, derivative direction/h, first-update arithmetic, and stop-boundary fingerprints",
            "heldout": "sealed until one coherent exact discovery survivor is frozen",
            "excluded": [
                "production changes",
                "ODDFYIELD forward-kernel identification",
                "coercion/error boundaries",
                "cross-build and compatibility-version claims"
            ]
        }
    });
    let root = PathBuf::from(ARTIFACT_ROOT);
    for (name, bytes) in [
        (
            "candidate-manifest-yield-near-seed-v1.json",
            pretty(&manifest),
        ),
        (
            "meta-yield-near-seed-discovery-v1.json",
            pretty(&discovery_meta),
        ),
        (
            "batch-yield-near-seed-discovery-v1.json",
            pretty(&discovery),
        ),
        (
            "meta-yield-near-seed-heldout-v1.json",
            pretty(&heldout_meta),
        ),
        ("batch-yield-near-seed-heldout-v1.json", pretty(&heldout)),
        (
            "meta-price-yield-companion-discovery-v1.json",
            pretty(&companion_meta),
        ),
        (
            "batch-price-yield-companion-discovery-v1.json",
            pretty(&companion),
        ),
    ] {
        write_frozen(&root.join(name), &bytes);
    }
    println!(
        "freeze_id={FREEZE_ID} candidate_models={} discovery_calls={} sealed_heldout_calls={} companion_calls={}",
        models.len(),
        discovery.probes.len(),
        heldout.probes.len(),
        companion.probes.len()
    );
}

fn generate_seed_family_discovery() {
    let cfg = NewtonCfg {
        ops: Ops::Native,
        step: Step::Absolute(1e-6),
        difference: Difference::Central,
        stop: Stop::Step(1e-10),
        publish: Publish::Old,
        cap: 100,
        seed: 0.0,
    };
    let mut probes = Vec::new();
    let mut records = Vec::new();
    let shapes: Vec<Shape> = discovery_shapes()
        .into_iter()
        .filter(|shape| !matches!(shape.id, "d-oncoupon-short-b0-f2" | "d-oncoupon-long-b0-f2"))
        .collect();
    for shape in shapes {
        let mut targets: BTreeMap<u64, BTreeSet<String>> = BTreeMap::new();
        for family in 0_u8..SEED_FAMILY_NAMES.len() as u8 {
            let Some(fixed_yield) = seed_family_fixed_point(shape.bond, family) else {
                println!(
                    "no positive fixed point shape={} family={}",
                    shape.id, SEED_FAMILY_NAMES[family as usize]
                );
                continue;
            };
            let anchor_price = price_for_bond(shape.bond, fixed_yield);
            for offset in [-4_i64, 0, 4] {
                let target_bits = (anchor_price.to_bits() as i128 + i128::from(offset)) as u64;
                let target = f64::from_bits(target_bits);
                assert!(target > 0.0 && target.is_finite());
                targets.entry(target_bits).or_default().insert(format!(
                    "{}:yield={}:price-ulp={offset:+}",
                    SEED_FAMILY_NAMES[family as usize],
                    hex(fixed_yield),
                ));
            }
        }
        for (index, (target_bits, source_roles)) in targets.into_iter().enumerate() {
            let target = f64::from_bits(target_bits);
            let row = Row {
                tag: shape.id.to_owned(),
                bond: shape.bond,
                price: target,
                want: 0,
            };
            let mut predictions = BTreeSet::new();
            for family in 0_u8..SEED_FAMILY_NAMES.len() as u8 {
                if let Some(seed) = seed_family_value(&row, family) {
                    if let Some(output) = newton(&row, NewtonCfg { seed, ..cfg }) {
                        predictions.insert(output.to_bits());
                    }
                }
            }
            let low = predictions
                .iter()
                .map(|bits| ordered(*bits) as i128)
                .min()
                .unwrap();
            let high = predictions
                .iter()
                .map(|bits| ordered(*bits) as i128)
                .max()
                .unwrap();
            let id = format!("yield-seed-family-discovery-{}-{index:03}", shape.id);
            let args = yield_args(shape.bond, target);
            probes.push(RankedFrozenProbe {
                probe: FrozenProbe {
                    id: id.clone(),
                    args: args.clone(),
                },
                distinct_outputs: predictions.len(),
                prediction_span_ulp: high - low,
            });
            records.push(SeedFamilyRecord {
                id,
                shape: shape.id.to_owned(),
                source_roles: source_roles.into_iter().collect(),
                args_bits: args,
                distinct_candidate_outputs: predictions.len(),
                prediction_span_ulp: high - low,
            });
        }
    }
    let batch = FrozenBatch {
        function: "YIELD",
        row_id: "G6-03-yield-seed-family-discovery-v2-20260809".to_owned(),
        probes,
    };
    let meta = SeedFamilyBank {
        schema_version: "oxfunc.w109.yield_seed_family_discovery_bank.v2".to_owned(),
        freeze_id: SEED_FAMILY_FREEZE_ID.to_owned(),
        answer_blind: true,
        candidate_families: SEED_FAMILY_NAMES
            .iter()
            .map(|name| (*name).to_owned())
            .collect(),
        records,
    };
    assert_eq!(batch.probes.len(), meta.records.len());
    let root = PathBuf::from(ARTIFACT_ROOT);
    write_frozen(
        &root.join("batch-yield-seed-family-discovery-v2.json"),
        &pretty(&batch),
    );
    write_frozen(
        &root.join("meta-yield-seed-family-discovery-v2.json"),
        &pretty(&meta),
    );
    println!(
        "freeze_id={SEED_FAMILY_FREEZE_ID} shapes=6 candidate_families={} discovery_calls={}",
        SEED_FAMILY_NAMES.len(),
        batch.probes.len(),
    );
}

fn score_seed_family_discovery() {
    let path =
        PathBuf::from(ARTIFACT_ROOT).join("answers-yield-seed-family-discovery-20260809.json");
    let answered: AnsweredBatch = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    assert_eq!(answered.function, "YIELD");
    assert_eq!(answered.witnesses.len(), 120);
    let rows = rows_from_answered(&answered);
    let meta_path = PathBuf::from(ARTIFACT_ROOT).join("meta-yield-seed-family-discovery-v2.json");
    let meta: SeedFamilyBank = serde_json::from_slice(&std::fs::read(meta_path).unwrap()).unwrap();
    assert_eq!(meta.freeze_id, SEED_FAMILY_FREEZE_ID);
    assert_eq!(meta.records.len(), rows.len());
    let cfg = NewtonCfg {
        ops: Ops::Native,
        step: Step::Absolute(1e-6),
        difference: Difference::Central,
        stop: Stop::Step(1e-10),
        publish: Publish::Old,
        cap: 100,
        seed: 0.0,
    };
    let mut candidates = Vec::new();
    for family in 0_u8..SEED_FAMILY_NAMES.len() as u8 {
        let seed_score = score(&rows, |row| seed_family_value(row, family));
        let solver_score = score(&rows, |row| {
            let seed = seed_family_value(row, family)?;
            newton(row, NewtonCfg { seed, ..cfg })
        });
        candidates.push((solver_score, seed_score, family));
    }
    candidates.sort_by_key(|item| item.0.rank());
    println!("seed-family fixed-point discovery candidates:");
    for (solver_score, seed_score, family) in &candidates {
        println!(
            "  solver={}/{} max={} sum={} seed-only={}/{} max={} sum={} family={}",
            solver_score.exact,
            rows.len(),
            solver_score.max,
            solver_score.sum,
            seed_score.exact,
            rows.len(),
            seed_score.max,
            seed_score.sum,
            SEED_FAMILY_NAMES[*family as usize],
        );
    }
    println!("source-conditioned fixed-point classification:");
    for source_family in SEED_FAMILY_NAMES {
        let source_ids: BTreeSet<&str> = meta
            .records
            .iter()
            .filter(|record| {
                record
                    .source_roles
                    .iter()
                    .any(|role| role.starts_with(source_family))
            })
            .map(|record| record.id.as_str())
            .collect();
        let source_rows: Vec<Row> = rows
            .iter()
            .filter(|row| source_ids.contains(row.tag.as_str()))
            .cloned()
            .collect();
        let mut source_candidates = Vec::new();
        for family in 0_u8..SEED_FAMILY_NAMES.len() as u8 {
            let seed_score = score(&source_rows, |row| seed_family_value(row, family));
            let solver_score = score(&source_rows, |row| {
                let seed = seed_family_value(row, family)?;
                newton(row, NewtonCfg { seed, ..cfg })
            });
            source_candidates.push((solver_score, seed_score, family));
        }
        source_candidates.sort_by_key(|item| item.0.rank());
        println!("  source={source_family} rows={}:", source_rows.len());
        for (solver_score, seed_score, family) in source_candidates.iter().take(5) {
            println!(
                "    solver={}/{} max={} sum={} seed={}/{} max={} sum={} candidate={}",
                solver_score.exact,
                source_rows.len(),
                solver_score.max,
                solver_score.sum,
                seed_score.exact,
                source_rows.len(),
                seed_score.max,
                seed_score.sum,
                SEED_FAMILY_NAMES[*family as usize],
            );
        }
    }
    println!("per-shape seed-family leaders:");
    for shape in discovery_shapes()
        .into_iter()
        .filter(|shape| !matches!(shape.id, "d-oncoupon-short-b0-f2" | "d-oncoupon-long-b0-f2"))
    {
        let shape_rows: Vec<Row> = rows
            .iter()
            .filter(|row| row.tag.contains(shape.id))
            .cloned()
            .collect();
        let mut shape_candidates = Vec::new();
        for family in 0_u8..SEED_FAMILY_NAMES.len() as u8 {
            let s = score(&shape_rows, |row| {
                let seed = seed_family_value(row, family)?;
                newton(row, NewtonCfg { seed, ..cfg })
            });
            shape_candidates.push((s, family));
        }
        shape_candidates.sort_by_key(|item| item.0.rank());
        println!("  {} rows={}:", shape.id, shape_rows.len());
        for (s, family) in shape_candidates.iter().take(6) {
            println!(
                "    {}/{} max={} sum={} family={}",
                s.exact,
                shape_rows.len(),
                s.max,
                s.sum,
                SEED_FAMILY_NAMES[*family as usize],
            );
        }
    }
}

fn diagnose_seed_family_fixed_points() {
    let answer_path =
        PathBuf::from(ARTIFACT_ROOT).join("answers-yield-seed-family-discovery-20260809.json");
    let answered: AnsweredBatch =
        serde_json::from_slice(&std::fs::read(answer_path).unwrap()).unwrap();
    assert_eq!(answered.function, "YIELD");
    assert_eq!(answered.witnesses.len(), 120);
    let rows = rows_from_answered(&answered);
    let rows_by_id: BTreeMap<&str, &Row> = rows.iter().map(|row| (row.tag.as_str(), row)).collect();

    let meta_path = PathBuf::from(ARTIFACT_ROOT).join("meta-yield-seed-family-discovery-v2.json");
    let meta: SeedFamilyBank = serde_json::from_slice(&std::fs::read(meta_path).unwrap()).unwrap();
    assert_eq!(meta.freeze_id, SEED_FAMILY_FREEZE_ID);
    assert_eq!(meta.records.len(), rows.len());
    assert_eq!(rows_by_id.len(), rows.len());

    #[derive(Default)]
    struct Summary {
        count: usize,
        exact: usize,
        signed_sum: i128,
        abs_sum: u128,
        abs_max: u64,
        signed_min: i128,
        signed_max: i128,
    }

    impl Summary {
        fn add(&mut self, delta: i128) {
            if self.count == 0 {
                self.signed_min = delta;
                self.signed_max = delta;
            } else {
                self.signed_min = self.signed_min.min(delta);
                self.signed_max = self.signed_max.max(delta);
            }
            let magnitude = delta.unsigned_abs();
            self.count += 1;
            self.exact += usize::from(delta == 0);
            self.signed_sum += delta;
            self.abs_sum += magnitude;
            self.abs_max = self.abs_max.max(magnitude as u64);
        }
    }

    let cfg = NewtonCfg {
        ops: Ops::Native,
        step: Step::Absolute(1e-6),
        difference: Difference::Central,
        stop: Stop::Step(1e-10),
        publish: Publish::Old,
        cap: 100,
        seed: 0.0,
    };
    let mut by_source: BTreeMap<&str, Summary> = BTreeMap::new();
    let mut by_source_shape: BTreeMap<(&str, &str), Summary> = BTreeMap::new();
    println!("seed-family exact-center diagnostics (signed ULP = Excel - encoded root):");
    for record in &meta.records {
        let row = *rows_by_id
            .get(record.id.as_str())
            .unwrap_or_else(|| panic!("missing answer for {}", record.id));
        for role in &record.source_roles {
            let Some((family_name, suffix)) = role.split_once(":yield=") else {
                panic!("malformed source role {role}");
            };
            let Some((yield_bits, price_offset)) = suffix.split_once(":price-ulp=") else {
                panic!("malformed source role {role}");
            };
            if price_offset != "+0" {
                continue;
            }
            let family = SEED_FAMILY_NAMES
                .iter()
                .position(|name| *name == family_name)
                .unwrap_or_else(|| panic!("unknown seed family {family_name}"))
                as u8;
            let fixed_bits = parse_bits(yield_bits);
            let fixed_yield = f64::from_bits(fixed_bits);
            let excel_yield = f64::from_bits(row.want);
            let seed = seed_family_value(row, family).expect("source seed is defined");
            let solver = newton(row, NewtonCfg { seed, ..cfg }).expect("source solver converges");
            let excel_delta = signed_ulp(row.want, fixed_bits);
            let seed_delta = signed_ulp(seed.to_bits(), fixed_bits);
            let solver_delta = signed_ulp(solver.to_bits(), fixed_bits);
            let fixed_residual = objective(row, fixed_yield).unwrap();
            let excel_residual = objective(row, excel_yield).unwrap();
            let powf_residual =
                local_price(row, excel_yield, Ops::Native, PricePower::PlatformPowf).unwrap()
                    - row.price;
            let chain_residual = local_price(row, excel_yield, Ops::Native, PricePower::X87Chain)
                .unwrap()
                - row.price;
            let repeated_residual =
                local_price(row, excel_yield, Ops::Native, PricePower::RepeatedMultiply).unwrap()
                    - row.price;
            println!(
                "  source={family_name} shape={} fixed={} excel={} excel-root={excel_delta:+} seed-root={seed_delta:+} solver-root={solver_delta:+} residual-fixed={fixed_residual:.17e} residual-excel={excel_residual:.17e} residual-powf={powf_residual:.17e} residual-chain={chain_residual:.17e} residual-repeat={repeated_residual:.17e}",
                record.shape,
                hex(fixed_yield),
                hex(excel_yield),
            );
            by_source.entry(family_name).or_default().add(excel_delta);
            by_source_shape
                .entry((family_name, record.shape.as_str()))
                .or_default()
                .add(excel_delta);
        }
    }
    println!("exact-center aggregate by source family:");
    for family_name in SEED_FAMILY_NAMES {
        let Some(summary) = by_source.get(family_name) else {
            println!("  source={family_name} centers=0");
            continue;
        };
        println!(
            "  source={family_name} centers={} exact={} signed=[{:+},{:+}] signed-sum={:+} abs-max={} abs-sum={}",
            summary.count,
            summary.exact,
            summary.signed_min,
            summary.signed_max,
            summary.signed_sum,
            summary.abs_max,
            summary.abs_sum,
        );
    }
    println!("exact-center aggregate by source and shape:");
    for ((family_name, shape), summary) in by_source_shape {
        println!(
            "  source={family_name} shape={shape} centers={} exact={} signed=[{:+},{:+}] signed-sum={:+} abs-max={} abs-sum={}",
            summary.count,
            summary.exact,
            summary.signed_min,
            summary.signed_max,
            summary.signed_sum,
            summary.abs_max,
            summary.abs_sum,
        );
    }
}

#[derive(Clone, Copy, Debug)]
struct SeedSolverCandidate {
    seed_kind: u8,
    cfg: NewtonCfg,
    form: UpdateForm,
}

const SEED_SOLVER_KIND_COUNT: u8 = 29;

fn one_coupon_extension_seed(row: &Row, ops: Ops, direct_off: bool) -> Option<f64> {
    let (_, derived_off, coupon, accrual) = schedule_factors(row);
    let off = if direct_off {
        let basis = Some(row.bond.basis);
        let direct_days = coupdaysnc_kernel(
            row.bond.settlement,
            row.bond.maturity,
            row.bond.frequency,
            basis,
        )
        .ok()?;
        let e = coupdays_kernel(
            row.bond.settlement,
            row.bond.maturity,
            row.bond.frequency,
            basis,
        )
        .ok()?;
        ops.div(direct_days, e)
    } else {
        derived_off
    };
    let dirty_target = ops.add(row.price, accrual);
    let future = ops.add(row.bond.redemption, coupon);
    let ratio = ops.sub(ops.div(future, dirty_target), 1.0);
    let seed = ops.div(ops.mul(ratio, row.bond.frequency), off);
    (seed.is_finite() && seed > -row.bond.frequency).then_some(seed)
}

fn simple_price_denominator_seed(row: &Row, ops: Ops, direct_periods: bool) -> Option<f64> {
    let (_, fractional, direct) = coupon_periods(row, ops)?;
    let periods = if direct_periods { direct } else { fractional };
    let coupon = ops.mul(100.0, row.bond.coupon);
    let spread = ops.sub(row.bond.redemption, row.price);
    let gain = ops.div(ops.mul(spread, row.bond.frequency), periods);
    let seed = ops.div(ops.add(coupon, gain), row.price);
    (seed.is_finite() && seed > -row.bond.frequency).then_some(seed)
}

fn seed_candidate_value(row: &Row, seed_kind: u8) -> Option<f64> {
    match seed_kind {
        0..=9 => seed_family_value(row, seed_kind),
        10 => Some(0.05),
        11 => Some(row.bond.coupon),
        12..=21 => seed_family_value_ops(row, seed_kind - 12, Ops::X87),
        22 => Some(0.1),
        23 => Some(0.0),
        24 => Some(100.0 * row.bond.coupon / row.price),
        25 => one_coupon_extension_seed(row, Ops::Native, false),
        26 => one_coupon_extension_seed(row, Ops::Native, true),
        27 => simple_price_denominator_seed(row, Ops::Native, false),
        _ => simple_price_denominator_seed(row, Ops::Native, true),
    }
}

fn seed_candidate_name(seed_kind: u8) -> &'static str {
    match seed_kind {
        0..=9 => SEED_FAMILY_NAMES[seed_kind as usize],
        10 => "fixed-0.05",
        11 => "coupon-rate",
        12 => "x87-direct-gain_n-weight",
        13 => "x87-fractional-gain_n-weight",
        14 => "x87-fractional-gain_n-over-derived-off-squared",
        15 => "x87-fractional-gain_n-over-direct-off-squared",
        16 => "x87-fractional-gain_fractional-over-derived-off-squared",
        17 => "x87-direct-gain_direct-over-direct-off-squared",
        18 => "x87-direct-gain_n-over-direct-off",
        19 => "x87-direct-gain_n-plus-elapsed",
        20 => "x87-fractional-gain_textbook-average",
        21 => "x87-direct-gain_n-weight_dirty-book",
        22 => "fixed-0.1",
        23 => "fixed-0.0",
        24 => "current-yield",
        25 => "one-coupon-extension-derived-off",
        26 => "one-coupon-extension-direct-off",
        27 => "simple-price-denominator-fractional-periods",
        _ => "simple-price-denominator-direct-periods",
    }
}

fn seed_solver_candidates() -> Vec<SeedSolverCandidate> {
    let steps = [
        Step::Absolute(1e-3),
        Step::Absolute(1e-4),
        Step::Absolute(1e-5),
        Step::Absolute(1e-6),
        Step::Absolute(1e-7),
        Step::Absolute(1e-8),
        Step::Absolute(1e-9),
        Step::RawRelative(1e-3),
        Step::RawRelative(1e-4),
        Step::RawRelative(1e-5),
        Step::RawRelative(1e-6),
        Step::RawRelative(1e-7),
    ];
    let thresholds = [
        1e-7,
        1e-8,
        1e-9,
        1e-10,
        1e-11,
        1e-12,
        2.0_f64.powi(-30),
        2.0_f64.powi(-31),
        2.0_f64.powi(-32),
        2.0_f64.powi(-33),
        2.0_f64.powi(-34),
        2.0_f64.powi(-35),
        2.0_f64.powi(-36),
    ];
    let differences = [
        Difference::Forward,
        Difference::Backward,
        Difference::Central,
    ];
    let publications = [Publish::Old, Publish::New, Publish::Previous];
    let mut candidates = Vec::new();
    for seed_kind in 0_u8..SEED_SOLVER_KIND_COUNT {
        for step in steps {
            for difference in differences {
                for threshold in thresholds {
                    for publication in publications {
                        candidates.push(SeedSolverCandidate {
                            seed_kind,
                            cfg: NewtonCfg {
                                ops: Ops::Native,
                                step,
                                difference,
                                stop: Stop::Step(threshold),
                                publish: publication,
                                cap: 100,
                                seed: 0.0,
                            },
                            form: UpdateForm::DerivativeThenDivide,
                        });
                    }
                }
            }
        }
        for form in [
            UpdateForm::MultiplyThenDivide,
            UpdateForm::DivideThenMultiply,
            UpdateForm::HOverDifferenceThenMultiply,
            UpdateForm::MultiplyReciprocalDifference,
        ] {
            for difference in differences {
                for threshold in [1e-9, 1e-10, 1e-11, 2.0_f64.powi(-33), 2.0_f64.powi(-34)] {
                    for publication in publications {
                        candidates.push(SeedSolverCandidate {
                            seed_kind,
                            cfg: NewtonCfg {
                                ops: Ops::Native,
                                step: Step::Absolute(1e-6),
                                difference,
                                stop: Stop::Step(threshold),
                                publish: publication,
                                cap: 100,
                                seed: 0.0,
                            },
                            form,
                        });
                    }
                }
            }
        }
    }
    candidates
}

fn race_seed_family_solver_vm() {
    let answer_path =
        PathBuf::from(ARTIFACT_ROOT).join("answers-yield-seed-family-discovery-20260809.json");
    let answered: AnsweredBatch =
        serde_json::from_slice(&std::fs::read(answer_path).unwrap()).unwrap();
    assert_eq!(answered.function, "YIELD");
    assert_eq!(answered.witnesses.len(), 120);
    let rows = rows_from_answered(&answered);

    let candidates = seed_solver_candidates();
    let mut scored: Vec<(Score, SeedSolverCandidate)> = candidates
        .par_iter()
        .map(|candidate| {
            let result = score(&rows, |row| {
                let seed = seed_candidate_value(row, candidate.seed_kind)?;
                configured_graph_newton(row, seed, candidate.cfg, candidate.form)
            });
            (result, *candidate)
        })
        .collect();
    scored.sort_by_key(|item| item.0.rank());
    println!("seed-family solver-VM candidates={}:", scored.len());
    for (result, candidate) in scored.iter().take(60) {
        println!(
            "  {}/{} max={} sum={} seed={} form={:?} {:?}",
            result.exact,
            rows.len(),
            result.max,
            result.sum,
            seed_candidate_name(candidate.seed_kind),
            candidate.form,
            candidate.cfg,
        );
    }
    println!("per-shape seed-family solver-VM leaders:");
    for shape in discovery_shapes()
        .into_iter()
        .filter(|shape| !matches!(shape.id, "d-oncoupon-short-b0-f2" | "d-oncoupon-long-b0-f2"))
    {
        let shape_rows: Vec<Row> = rows
            .iter()
            .filter(|row| row.tag.contains(shape.id))
            .cloned()
            .collect();
        let mut shape_scored: Vec<(Score, SeedSolverCandidate)> = scored
            .par_iter()
            .map(|(_, candidate)| {
                let result = score(&shape_rows, |row| {
                    let seed = seed_candidate_value(row, candidate.seed_kind)?;
                    configured_graph_newton(row, seed, candidate.cfg, candidate.form)
                });
                (result, *candidate)
            })
            .collect();
        shape_scored.sort_by_key(|item| item.0.rank());
        println!("  {} rows={}:", shape.id, shape_rows.len());
        for (result, candidate) in shape_scored.iter().take(12) {
            println!(
                "    {}/{} max={} sum={} seed={} form={:?} {:?}",
                result.exact,
                shape_rows.len(),
                result.max,
                result.sum,
                seed_candidate_name(candidate.seed_kind),
                candidate.form,
                candidate.cfg,
            );
        }
    }
}

fn race_original_seed_solver_vm() {
    let answer_path =
        PathBuf::from(ARTIFACT_ROOT).join("answers-yield-near-seed-discovery-20260809.json");
    let answered: AnsweredBatch =
        serde_json::from_slice(&std::fs::read(answer_path).unwrap()).unwrap();
    assert_eq!(answered.function, "YIELD");
    assert_eq!(answered.witnesses.len(), 384);
    let rows = rows_from_answered(&answered);

    let candidates = seed_solver_candidates();
    let mut scored: Vec<(Score, SeedSolverCandidate)> = candidates
        .par_iter()
        .map(|candidate| {
            let result = score(&rows, |row| {
                let seed = seed_candidate_value(row, candidate.seed_kind)?;
                configured_graph_newton(row, seed, candidate.cfg, candidate.form)
            });
            (result, *candidate)
        })
        .collect();
    scored.sort_by_key(|item| item.0.rank());
    println!("original-discovery solver-VM candidates={}:", scored.len());
    for (result, candidate) in scored.iter().take(60) {
        println!(
            "  {}/{} max={} sum={} seed={} form={:?} {:?}",
            result.exact,
            rows.len(),
            result.max,
            result.sum,
            seed_candidate_name(candidate.seed_kind),
            candidate.form,
            candidate.cfg,
        );
    }

    println!("per-shape original-discovery scores for aggregate leaders:");
    for (_, candidate) in scored.iter().take(12) {
        println!(
            "  seed={} form={:?} {:?}",
            seed_candidate_name(candidate.seed_kind),
            candidate.form,
            candidate.cfg,
        );
        for shape in discovery_shapes() {
            let shape_rows: Vec<Row> = rows
                .iter()
                .filter(|row| row.tag.contains(shape.id))
                .cloned()
                .collect();
            let result = score(&shape_rows, |row| {
                let seed = seed_candidate_value(row, candidate.seed_kind)?;
                configured_graph_newton(row, seed, candidate.cfg, candidate.form)
            });
            println!(
                "    shape={} {}/{} max={} sum={}",
                shape.id,
                result.exact,
                shape_rows.len(),
                result.max,
                result.sum,
            );
        }
    }
}

fn race_objective_graphs() {
    let answer_path =
        PathBuf::from(ARTIFACT_ROOT).join("answers-yield-near-seed-discovery-20260809.json");
    let answered: AnsweredBatch =
        serde_json::from_slice(&std::fs::read(answer_path).unwrap()).unwrap();
    assert_eq!(answered.function, "YIELD");
    assert_eq!(answered.witnesses.len(), 384);
    let rows = rows_from_answered(&answered);
    let graphs = [
        ObjectiveGraph::PriceMinusTarget,
        ObjectiveGraph::TargetMinusPrice,
        ObjectiveGraph::DifferenceOverTarget,
        ObjectiveGraph::NegativeDifferenceOverTarget,
        ObjectiveGraph::PriceOverTargetMinusOne,
        ObjectiveGraph::OneMinusPriceOverTarget,
        ObjectiveGraph::ScaledDifference01,
        ObjectiveGraph::ScaledDifference100,
        ObjectiveGraph::DirtyMinusDirtyTarget,
        ObjectiveGraph::DirtyOverDirtyTargetMinusOne,
        ObjectiveGraph::HornerReciprocalPowDirty,
        ObjectiveGraph::HornerBasePowDirty,
    ];
    let candidates: Vec<SeedSolverCandidate> = seed_solver_candidates()
        .into_iter()
        .filter(|candidate| matches!(candidate.seed_kind, 0 | 10 | 22 | 25 | 27))
        .collect();
    let work: Vec<(ObjectiveGraph, SeedSolverCandidate)> = graphs
        .iter()
        .flat_map(|graph| candidates.iter().map(move |candidate| (*graph, *candidate)))
        .collect();
    let mut scored: Vec<(Score, ObjectiveGraph, SeedSolverCandidate)> = work
        .par_iter()
        .map(|(graph, candidate)| {
            let result = score(&rows, |row| {
                let seed = seed_candidate_value(row, candidate.seed_kind)?;
                configured_graph_newton_objective(row, seed, candidate.cfg, candidate.form, *graph)
            });
            (result, *graph, *candidate)
        })
        .collect();
    scored.sort_by_key(|item| item.0.rank());
    println!(
        "objective-graph solver-VM candidates={} seeds={} graphs={}:",
        scored.len(),
        candidates.len(),
        graphs.len(),
    );
    for (result, graph, candidate) in scored.iter().take(80) {
        println!(
            "  {}/{} max={} sum={} objective={graph:?} seed={} form={:?} {:?}",
            result.exact,
            rows.len(),
            result.max,
            result.sum,
            seed_candidate_name(candidate.seed_kind),
            candidate.form,
            candidate.cfg,
        );
    }
    println!("best by objective graph:");
    for graph in graphs {
        let (result, _, candidate) = scored
            .iter()
            .filter(|(_, candidate_graph, _)| {
                std::mem::discriminant(candidate_graph) == std::mem::discriminant(&graph)
            })
            .min_by_key(|item| item.0.rank())
            .unwrap();
        println!(
            "  objective={graph:?} {}/{} max={} sum={} seed={} form={:?} {:?}",
            result.exact,
            rows.len(),
            result.max,
            result.sum,
            seed_candidate_name(candidate.seed_kind),
            candidate.form,
            candidate.cfg,
        );
    }
}

fn race_fd_bootstrap_secant() {
    let answer_path =
        PathBuf::from(ARTIFACT_ROOT).join("answers-yield-near-seed-discovery-20260809.json");
    let answered: AnsweredBatch =
        serde_json::from_slice(&std::fs::read(answer_path).unwrap()).unwrap();
    assert_eq!(answered.function, "YIELD");
    assert_eq!(answered.witnesses.len(), 384);
    let rows = rows_from_answered(&answered);
    let graphs = [
        ObjectiveGraph::PriceMinusTarget,
        ObjectiveGraph::DifferenceOverTarget,
        ObjectiveGraph::DirtyMinusDirtyTarget,
        ObjectiveGraph::DirtyOverDirtyTargetMinusOne,
    ];
    let seed_kinds = [0_u8, 10, 22, 25, 27];
    let thresholds = [
        1e-8,
        1e-9,
        1e-10,
        1e-11,
        1e-12,
        2.0_f64.powi(-33),
        2.0_f64.powi(-34),
    ];
    let publications = [Publish::Old, Publish::New, Publish::Previous];
    let forms = [
        UpdateForm::MultiplyThenDivide,
        UpdateForm::DivideThenMultiply,
        UpdateForm::HOverDifferenceThenMultiply,
        UpdateForm::MultiplyReciprocalDifference,
    ];
    let mut scored = Vec::new();
    for graph in graphs {
        for seed_kind in seed_kinds {
            for ops in [Ops::Native, Ops::X87] {
                for threshold in thresholds {
                    for publication in publications {
                        for form in forms {
                            let result = score(&rows, |row| {
                                let seed = seed_candidate_value(row, seed_kind)?;
                                fd_bootstrap_secant(
                                    row,
                                    seed,
                                    ops,
                                    graph,
                                    threshold,
                                    publication,
                                    form,
                                )
                            });
                            scored.push((
                                result,
                                graph,
                                seed_kind,
                                ops,
                                threshold,
                                publication,
                                form,
                            ));
                        }
                    }
                }
            }
        }
    }
    scored.sort_by_key(|item| item.0.rank());
    println!("FD-bootstrap secant candidates={}:", scored.len());
    for (result, graph, seed_kind, ops, threshold, publication, form) in scored.iter().take(80) {
        println!(
            "  {}/{} max={} sum={} objective={graph:?} seed={} ops={ops:?} stop={threshold:?} publish={publication:?} form={form:?}",
            result.exact,
            rows.len(),
            result.max,
            result.sum,
            seed_candidate_name(*seed_kind),
        );
    }
}

fn score_price_companion() {
    let path =
        PathBuf::from(ARTIFACT_ROOT).join("answers-price-yield-companion-discovery-20260809.json");
    let answered: AnsweredBatch = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    assert_eq!(answered.function, "PRICE");
    assert_eq!(answered.witnesses.len(), 136);
    let mut total = Score::default();
    let mut misses = Vec::new();
    for witness in answered.witnesses {
        assert_eq!(witness.args.len(), 7);
        let values: Vec<f64> = witness
            .args
            .iter()
            .map(|arg| f64::from_bits(parse_bits(arg)))
            .collect();
        let want = parse_bits(&witness.expected_bits);
        let got = price_kernel(
            values[0],
            values[1],
            values[2],
            values[3],
            values[4],
            values[5],
            Some(values[6]),
        )
        .unwrap();
        total.add(got, want);
        if got.to_bits() != want {
            misses.push((witness.id, got.to_bits(), want, ulp(got.to_bits(), want)));
        }
    }
    println!(
        "PRICE companion production {}/136 max={} sum={}",
        total.exact, total.max, total.sum
    );
    for (id, got, want, distance) in misses {
        println!("  miss {id} got=0x{got:016x} want=0x{want:016x} ulp={distance}");
    }
}

fn rows_from_answered(answered: &AnsweredBatch) -> Vec<Row> {
    answered
        .witnesses
        .iter()
        .map(|witness| {
            assert_eq!(witness.args.len(), 7);
            let values: Vec<f64> = witness
                .args
                .iter()
                .map(|arg| f64::from_bits(parse_bits(arg)))
                .collect();
            Row {
                tag: witness.id.clone(),
                bond: Bond {
                    settlement: values[0],
                    maturity: values[1],
                    coupon: values[2],
                    redemption: values[4],
                    frequency: values[5],
                    basis: values[6],
                },
                price: values[3],
                want: parse_bits(&witness.expected_bits),
            }
        })
        .collect()
}

fn score_yield_discovery() {
    let path = PathBuf::from(ARTIFACT_ROOT).join("answers-yield-near-seed-discovery-20260809.json");
    let answered: AnsweredBatch = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    assert_eq!(answered.function, "YIELD");
    assert_eq!(answered.witnesses.len(), 384);
    let rows = rows_from_answered(&answered);

    println!("published corrected-PRICE residual fingerprint:");
    for (class, lo, hi) in [
        ("ulp-ladder", 0_usize, 13_usize),
        ("delta-tiny", 13, 25),
        ("delta-threshold", 25, 41),
        ("delta-far", 41, 48),
    ] {
        let mut residuals = Vec::new();
        for row in &rows {
            let index = row
                .tag
                .rsplit('-')
                .next()
                .unwrap()
                .parse::<usize>()
                .unwrap();
            if (lo..hi).contains(&index) {
                residuals.push(objective(row, f64::from_bits(row.want)).unwrap().abs());
            }
        }
        residuals.sort_by(|a, b| a.total_cmp(b));
        let count_below = |threshold: f64| residuals.iter().filter(|r| **r < threshold).count();
        println!(
            "  {class}: n={} min={:.17e} median={:.17e} max={:.17e} below1e-7={} below1e-8={} below1e-9={}",
            residuals.len(),
            residuals[0],
            residuals[residuals.len() / 2],
            residuals[residuals.len() - 1],
            count_below(1e-7),
            count_below(1e-8),
            count_below(1e-9),
        );
    }

    let production = score(&rows, |r| {
        yield_kernel(
            r.bond.settlement,
            r.bond.maturity,
            r.bond.coupon,
            r.price,
            r.bond.redemption,
            r.bond.frequency,
            Some(r.bond.basis),
        )
        .ok()
    });
    println!(
        "YIELD discovery production {}/{} max={} sum={}",
        production.exact,
        rows.len(),
        production.max,
        production.sum
    );

    let seed = 0.05_f64;
    let mut shapes: BTreeMap<String, (usize, usize, f64, f64)> = BTreeMap::new();
    for row in &rows {
        let shape = row
            .tag
            .split('-')
            .skip(5)
            .take_while(|part| !part.chars().all(|c| c.is_ascii_digit()))
            .collect::<Vec<_>>()
            .join("-");
        let output = f64::from_bits(row.want);
        let entry = shapes
            .entry(shape)
            .or_insert((0, 0, f64::INFINITY, f64::NEG_INFINITY));
        entry.0 += 1;
        entry.1 += usize::from(row.want == seed.to_bits());
        entry.2 = entry.2.min(output);
        entry.3 = entry.3.max(output);
    }
    println!("published-seed fingerprint:");
    for (shape, (count, seed_exact, low, high)) in shapes {
        println!("  {shape}: seed-exact={seed_exact}/{count} range=[{low:.17e},{high:.17e}]");
    }

    let models = discriminator_models();
    let mut one_step_all = Vec::new();
    for cfg in models.iter().copied() {
        one_step_all.push((score(&rows, |row| one_numeric_step(row, cfg)), cfg));
    }
    one_step_all.sort_by_key(|item| item.0.rank());
    println!("one-step all-row candidates={}:", one_step_all.len());
    for (s, cfg) in one_step_all.iter().take(20) {
        println!(
            "  {}/{} max={} sum={} {cfg:?}",
            s.exact,
            rows.len(),
            s.max,
            s.sum
        );
    }

    let residual_gate_rows: Vec<Row> = rows
        .iter()
        .filter(|row| objective(row, seed).unwrap().abs() < 1e-7)
        .cloned()
        .collect();
    let mut one_step_gate = Vec::new();
    for cfg in models.iter().copied().filter(|cfg| !cfg.seed.is_nan()) {
        one_step_gate.push((
            score(&residual_gate_rows, |row| one_numeric_step(row, cfg)),
            cfg,
        ));
    }
    one_step_gate.sort_by_key(|item| item.0.rank());
    println!(
        "pre-step |PRICE(seed)-target|<1e-7 rows={} one-step candidates:",
        residual_gate_rows.len()
    );
    for (s, cfg) in one_step_gate.iter().take(20) {
        println!(
            "  {}/{} max={} sum={} {cfg:?}",
            s.exact,
            residual_gate_rows.len(),
            s.max,
            s.sum
        );
    }

    let mut graph_race = Vec::new();
    for step in [
        Step::Absolute(1e-3),
        Step::Absolute(1e-4),
        Step::Absolute(1e-5),
        Step::Absolute(1e-6),
        Step::Absolute(1e-7),
        Step::Absolute(1e-8),
    ] {
        for difference in [
            Difference::Forward,
            Difference::Backward,
            Difference::Central,
        ] {
            for form in [
                UpdateForm::DerivativeThenDivide,
                UpdateForm::MultiplyThenDivide,
                UpdateForm::DivideThenMultiply,
                UpdateForm::HOverDifferenceThenMultiply,
                UpdateForm::MultiplyReciprocalDifference,
            ] {
                for compact_mask in 0_u16..64 {
                    let graph = FirstStepGraph {
                        step,
                        difference,
                        form,
                        spill_mask: compact_mask << 2,
                    };
                    graph_race.push((
                        score(&residual_gate_rows, |row| one_step_graph(row, graph)),
                        graph,
                    ));
                }
            }
        }
    }
    graph_race.sort_by_key(|item| item.0.rank());
    println!("first-step outer graph candidates={}:", graph_race.len());
    for (s, graph) in graph_race.iter().take(30) {
        println!(
            "  {}/{} max={} sum={} {graph:?}",
            s.exact,
            residual_gate_rows.len(),
            s.max,
            s.sum
        );
    }

    let mut residual_schedules = Vec::new();
    for ops in [Ops::Native, Ops::X87] {
        for step in [
            Step::Absolute(1e-3),
            Step::Absolute(1e-4),
            Step::Absolute(1e-5),
            Step::Absolute(1e-6),
            Step::Absolute(1e-7),
            Step::Absolute(1e-8),
            Step::Relative(1e-3),
            Step::Relative(1e-4),
            Step::Relative(1e-5),
            Step::Relative(1e-6),
        ] {
            for difference in [
                Difference::Forward,
                Difference::Backward,
                Difference::Central,
            ] {
                for publish in [Publish::Old, Publish::New, Publish::Previous] {
                    for seed_policy in [0.0, 0.05, 0.1, f64::NAN] {
                        let cfg = NewtonCfg {
                            ops,
                            step,
                            difference,
                            stop: Stop::Residual(1e-7),
                            publish,
                            cap: 100,
                            seed: seed_policy,
                        };
                        residual_schedules.push((score(&rows, |row| newton(row, cfg)), cfg));
                    }
                }
            }
        }
    }
    residual_schedules.sort_by_key(|item| item.0.rank());
    println!("residual-1e-7 schedules={}:", residual_schedules.len());
    for (s, cfg) in residual_schedules.iter().take(30) {
        println!(
            "  {}/{} max={} sum={} {cfg:?}",
            s.exact,
            rows.len(),
            s.max,
            s.sum
        );
    }

    let near_fixed: Vec<Row> = rows
        .iter()
        .filter(|row| {
            models
                .iter()
                .filter(|cfg| !cfg.seed.is_nan())
                .filter_map(|cfg| one_numeric_step(row, *cfg))
                .all(|next| (next - seed).abs() < 1e-7)
        })
        .cloned()
        .collect();
    let stepped_near_fixed: Vec<Row> = near_fixed
        .iter()
        .filter(|row| row.want != seed.to_bits())
        .cloned()
        .collect();
    println!(
        "answer-blind all-fixed-seed-models-one-step-below-1e-7 rows={} (Excel moved on {})",
        near_fixed.len(),
        stepped_near_fixed.len()
    );
    let mut one_step_near = Vec::new();
    for cfg in models.iter().copied().filter(|cfg| !cfg.seed.is_nan()) {
        one_step_near.push((
            score(&stepped_near_fixed, |row| one_numeric_step(row, cfg)),
            cfg,
        ));
    }
    one_step_near.sort_by_key(|item| item.0.rank());
    for (s, cfg) in one_step_near.iter().take(20) {
        println!(
            "  near-moved {}/{} max={} sum={} {cfg:?}",
            s.exact,
            stepped_near_fixed.len(),
            s.max,
            s.sum
        );
    }

    let coupon_seed_rows: Vec<Row> = rows
        .iter()
        .filter(|row| row.bond.coupon.to_bits() == seed.to_bits())
        .cloned()
        .collect();
    let mut fixed_iterations = Vec::new();
    for base in models.iter().copied() {
        for cap in 1..=8 {
            let cfg = NewtonCfg {
                cap,
                stop: Stop::Fixed,
                publish: Publish::New,
                ..base
            };
            fixed_iterations.push((score(&rows, |row| newton(row, cfg)), cfg));
        }
    }
    fixed_iterations.sort_by_key(|item| item.0.rank());
    println!(
        "fixed-iteration full-discovery candidates={}:",
        fixed_iterations.len()
    );
    for (s, cfg) in fixed_iterations.iter().take(30) {
        let coupon_score = score(&coupon_seed_rows, |row| newton(row, *cfg));
        println!(
            "  all={}/{} max={} sum={} coupon=.05={}/{} max={} sum={} {cfg:?}",
            s.exact,
            rows.len(),
            s.max,
            s.sum,
            coupon_score.exact,
            coupon_seed_rows.len(),
            coupon_score.max,
            coupon_score.sum
        );
    }

    let historical_rows: Vec<Row> = rows
        .iter()
        .filter(|row| {
            let pair = (row.bond.settlement.to_bits(), row.bond.maturity.to_bits());
            pair == (44013.0_f64.to_bits(), 44562.0_f64.to_bits())
                || pair == (44058.0_f64.to_bits(), 44562.0_f64.to_bits())
                || pair == (44013.0_f64.to_bits(), 46753.0_f64.to_bits())
        })
        .cloned()
        .collect();
    println!(
        "historical A/B/C objective-at-published-output rows={}",
        historical_rows.len()
    );
    for power in [
        PricePower::Corrected,
        PricePower::PlatformPowf,
        PricePower::X87Chain,
        PricePower::RepeatedMultiply,
    ] {
        for ops in [Ops::Native, Ops::X87] {
            let mut s = Score::default();
            for row in &historical_rows {
                s.add(
                    local_price(row, f64::from_bits(row.want), ops, power).unwrap(),
                    row.price.to_bits(),
                );
            }
            println!(
                "  price residual {}/{} max={} sum={} power={power:?} ops={ops:?}",
                s.exact,
                historical_rows.len(),
                s.max,
                s.sum
            );
        }
    }
    println!("adjacent-target maps (first 13 rows per historical shape):");
    for row in historical_rows.iter().filter(|row| {
        row.tag
            .rsplit('-')
            .next()
            .unwrap()
            .parse::<usize>()
            .unwrap()
            < 13
    }) {
        let anchor = price_for_bond(row.bond, seed);
        let output = f64::from_bits(row.want);
        let corrected = local_price(row, output, Ops::Native, PricePower::Corrected).unwrap();
        let powf = local_price(row, output, Ops::Native, PricePower::PlatformPowf).unwrap();
        println!(
            "  {} target_dulp={:+} output_dulp={:+} corrected_price_dulp={:+} powf_price_dulp={:+} output=0x{:016x}",
            row.tag,
            ordered(row.price.to_bits()) as i128 - ordered(anchor.to_bits()) as i128,
            ordered(row.want) as i128 - ordered(seed.to_bits()) as i128,
            ordered(corrected.to_bits()) as i128 - ordered(row.price.to_bits()) as i128,
            ordered(powf.to_bits()) as i128 - ordered(row.price.to_bits()) as i128,
            row.want,
        );
    }

    let mut local_fixed = Vec::new();
    for power in [
        PricePower::Corrected,
        PricePower::PlatformPowf,
        PricePower::X87Chain,
        PricePower::RepeatedMultiply,
    ] {
        for base in models.iter().copied() {
            for cap in 1..=8 {
                let cfg = NewtonCfg {
                    cap,
                    stop: Stop::Fixed,
                    publish: Publish::New,
                    ..base
                };
                local_fixed.push((
                    score(&historical_rows, |row| newton_local(row, cfg, power)),
                    cfg,
                    power,
                ));
            }
        }
    }
    local_fixed.sort_by_key(|item| item.0.rank());
    println!(
        "local-power fixed-iteration candidates={}:",
        local_fixed.len()
    );
    for (s, cfg, power) in local_fixed.iter().take(30) {
        println!(
            "  {}/{} max={} sum={} power={power:?} {cfg:?}",
            s.exact,
            historical_rows.len(),
            s.max,
            s.sum
        );
    }

    let mut analytic_fixed = Vec::new();
    for ops in [Ops::Native, Ops::X87] {
        for seed in [0.05, f64::NAN] {
            for cap in 1..=12 {
                let cfg = NewtonCfg {
                    ops,
                    step: Step::Absolute(0.0),
                    difference: Difference::Forward,
                    stop: Stop::Fixed,
                    publish: Publish::New,
                    cap,
                    seed,
                };
                analytic_fixed.push((
                    score(&historical_rows, |row| analytic_newton(row, cfg)),
                    cfg,
                ));
            }
        }
    }
    analytic_fixed.sort_by_key(|item| item.0.rank());
    println!(
        "analytic fixed-iteration candidates={}:",
        analytic_fixed.len()
    );
    for (s, cfg) in analytic_fixed.iter().take(20) {
        println!(
            "  {}/{} max={} sum={} {cfg:?}",
            s.exact,
            historical_rows.len(),
            s.max,
            s.sum
        );
    }
}

fn main() {
    if std::env::args().nth(1).as_deref() == Some("generate") {
        generate_frozen_near_seed_battery();
        return;
    }
    if std::env::args().nth(1).as_deref() == Some("generate-seed-family-discovery") {
        generate_seed_family_discovery();
        return;
    }
    if std::env::args().nth(1).as_deref() == Some("score-companion") {
        score_price_companion();
        return;
    }
    if std::env::args().nth(1).as_deref() == Some("score-seed-family-discovery") {
        score_seed_family_discovery();
        return;
    }
    if std::env::args().nth(1).as_deref() == Some("diagnose-seed-family-fixed-points") {
        diagnose_seed_family_fixed_points();
        return;
    }
    if std::env::args().nth(1).as_deref() == Some("race-seed-family-solver-vm") {
        race_seed_family_solver_vm();
        return;
    }
    if std::env::args().nth(1).as_deref() == Some("race-original-seed-solver-vm") {
        race_original_seed_solver_vm();
        return;
    }
    if std::env::args().nth(1).as_deref() == Some("race-objective-graphs") {
        race_objective_graphs();
        return;
    }
    if std::env::args().nth(1).as_deref() == Some("race-fd-bootstrap-secant") {
        race_fd_bootstrap_secant();
        return;
    }
    if std::env::args().nth(1).as_deref() == Some("score-discovery") {
        score_yield_discovery();
        return;
    }
    if std::env::args().nth(1).as_deref() == Some("score-seed-formula") {
        score_seed_formula_discovery(false);
        return;
    }
    if std::env::args().nth(1).as_deref() == Some("score-seed-formula-full") {
        score_seed_formula_discovery(true);
        return;
    }
    if std::env::args().nth(1).as_deref() == Some("score-seed-inputs") {
        score_seed_input_refinements();
        return;
    }
    if std::env::args().nth(1).as_deref() == Some("score-local-refinements") {
        score_local_forward_refinements();
        return;
    }
    if std::env::args().nth(1).as_deref() == Some("diagnose-candidate") {
        let filter = std::env::args().nth(2);
        print_candidate_diagnostics(filter.as_deref());
        return;
    }
    if std::env::args().nth(1).as_deref() == Some("score-derivative-steps") {
        score_derivative_step_refinements();
        return;
    }
    if std::env::args().nth(1).as_deref() == Some("score-weight-periods") {
        score_weight_period_refinements();
        return;
    }
    let rows = load();
    assert_eq!(rows.len(), 19);

    print_excel_output_plateaus(&rows);

    let production = score(&rows, |r| {
        yield_kernel(
            r.bond.settlement,
            r.bond.maturity,
            r.bond.coupon,
            r.price,
            r.bond.redemption,
            r.bond.frequency,
            Some(r.bond.basis),
        )
        .ok()
    });
    println!(
        "production {}/{} max={} sum={}",
        production.exact,
        rows.len(),
        production.max,
        production.sum
    );

    println!("shared-solver skeleton spot checks:");
    for cfg in [
        NewtonCfg {
            ops: Ops::Native,
            step: Step::Absolute(1e-3),
            difference: Difference::Forward,
            stop: Stop::Step(1e-7),
            publish: Publish::New,
            cap: 100,
            seed: 0.05,
        },
        NewtonCfg {
            ops: Ops::X87,
            step: Step::Absolute(1e-3),
            difference: Difference::Forward,
            stop: Stop::Step(1e-7),
            publish: Publish::New,
            cap: 100,
            seed: 0.05,
        },
        NewtonCfg {
            ops: Ops::Native,
            step: Step::Absolute(1e-3),
            difference: Difference::Forward,
            stop: Stop::Residual(1e-7),
            publish: Publish::New,
            cap: 100,
            seed: 0.05,
        },
        NewtonCfg {
            ops: Ops::Native,
            step: Step::Absolute(1e-3),
            difference: Difference::Forward,
            stop: Stop::Step(1e-7),
            publish: Publish::Old,
            cap: 100,
            seed: 0.05,
        },
    ] {
        let s = score(&rows, |r| newton(r, cfg));
        println!(
            "  {}/{} max={} sum={} {cfg:?}",
            s.exact,
            rows.len(),
            s.max,
            s.sum
        );
    }

    let local_price_matches = rows
        .iter()
        .filter(|row| {
            let y = f64::from_bits(row.want);
            analytic_price_and_derivative(row, y, Ops::Native)
                .map(|(price, _)| {
                    price.to_bits()
                        == price_kernel(
                            row.bond.settlement,
                            row.bond.maturity,
                            row.bond.coupon,
                            y,
                            row.bond.redemption,
                            row.bond.frequency,
                            Some(row.bond.basis),
                        )
                        .unwrap()
                        .to_bits()
                })
                .unwrap_or(false)
        })
        .count();
    println!(
        "local analytic PRICE graph matches production {local_price_matches}/{}",
        rows.len()
    );

    let mut analytics = Vec::new();
    for ops in [Ops::Native, Ops::X87] {
        for stop in [
            Stop::Step(1e-4),
            Stop::Step(1e-5),
            Stop::Step(1e-6),
            Stop::Step(1e-7),
            Stop::Step(1e-8),
            Stop::Step(1e-9),
            Stop::Residual(1e-4),
            Stop::Residual(1e-5),
            Stop::Residual(1e-6),
            Stop::Residual(1e-7),
            Stop::Residual(1e-8),
            Stop::Residual(1e-9),
            Stop::Fixed,
        ] {
            for publish in [Publish::Old, Publish::New, Publish::Previous] {
                for cap in [2, 3, 4, 5, 10, 20, 100] {
                    for seed in [0.0, 0.05, 0.1, f64::NAN] {
                        let cfg = NewtonCfg {
                            ops,
                            step: Step::Absolute(0.0),
                            difference: Difference::Forward,
                            stop,
                            publish,
                            cap,
                            seed,
                        };
                        analytics.push((score(&rows, |r| analytic_newton(r, cfg)), cfg));
                    }
                }
            }
        }
    }
    analytics.sort_by_key(|item| item.0.rank());
    println!("analytic-newton candidates={}", analytics.len());
    for (s, cfg) in analytics.iter().take(20) {
        println!(
            "  {}/{} max={} sum={} {cfg:?}",
            s.exact,
            rows.len(),
            s.max,
            s.sum
        );
    }

    let mut newtons = Vec::new();
    for ops in [Ops::Native, Ops::X87] {
        for step in [
            Step::Absolute(1e-3),
            Step::Absolute(1e-4),
            Step::Absolute(1e-5),
            Step::Absolute(1e-6),
            Step::Absolute(1e-7),
            Step::Absolute(1e-8),
            Step::Relative(1e-3),
            Step::Relative(1e-4),
            Step::Relative(1e-5),
            Step::Relative(1e-6),
            Step::Relative(1e-7),
            Step::Relative(1e-8),
        ] {
            for difference in [
                Difference::Forward,
                Difference::Backward,
                Difference::Central,
            ] {
                for stop in [
                    Stop::Step(1e-6),
                    Stop::Step(1e-7),
                    Stop::Step(1e-8),
                    Stop::Residual(1e-6),
                    Stop::Residual(1e-7),
                    Stop::Residual(1e-8),
                    Stop::Either(1e-6),
                    Stop::Either(1e-7),
                    Stop::Either(1e-8),
                    Stop::Fixed,
                ] {
                    for publish in [Publish::Old, Publish::New, Publish::Previous] {
                        for cap in [20, 50, 100] {
                            // `NaN` is the row's coupon rate.  It is kept as a compact
                            // sentinel in the candidate record so this exploratory tool
                            // does not need a second config enum merely for seed policy.
                            for seed in [0.0, 0.05, 0.1, f64::NAN] {
                                let cfg = NewtonCfg {
                                    ops,
                                    step,
                                    difference,
                                    stop,
                                    publish,
                                    cap,
                                    seed,
                                };
                                let s = score(&rows, |r| newton(r, cfg));
                                newtons.push((s, cfg));
                            }
                        }
                    }
                }
            }
        }
    }
    newtons.sort_by_key(|x| x.0.rank());
    println!("newton candidates={}", newtons.len());
    for (s, cfg) in newtons.iter().take(20) {
        println!(
            "  {}/{} max={} sum={} {cfg:?}",
            s.exact,
            rows.len(),
            s.max,
            s.sum
        );
    }

    let mut secants = Vec::new();
    for ops in [Ops::Native, Ops::X87] {
        for stop in [
            Stop::Step(1e-6),
            Stop::Step(1e-7),
            Stop::Step(1e-8),
            Stop::Residual(1e-6),
            Stop::Residual(1e-7),
            Stop::Residual(1e-8),
            Stop::Either(1e-6),
            Stop::Either(1e-7),
            Stop::Either(1e-8),
            Stop::Fixed,
        ] {
            for publish in [Publish::Old, Publish::New, Publish::Previous] {
                for cap in [20, 50, 100] {
                    for (seed0, seed1) in [
                        (0.0, 0.05),
                        (0.0, 0.1),
                        (0.05, 0.1),
                        (0.05, 0.050001),
                        (0.05, 0.049999),
                        (0.1, 0.100001),
                    ] {
                        let cfg = SecantCfg {
                            ops,
                            stop,
                            publish,
                            cap,
                            seed0,
                            seed1,
                        };
                        let s = score(&rows, |r| secant(r, cfg));
                        secants.push((s, cfg));
                    }
                }
            }
        }
    }
    secants.sort_by_key(|x| x.0.rank());
    println!("secant candidates={}", secants.len());
    for (s, cfg) in secants.iter().take(20) {
        println!(
            "  {}/{} max={} sum={} {cfg:?}",
            s.exact,
            rows.len(),
            s.max,
            s.sum
        );
    }

    let mut false_positions = Vec::new();
    for ops in [Ops::Native, Ops::X87] {
        for stop in [
            Stop::Step(1e-6),
            Stop::Step(1e-7),
            Stop::Step(1e-8),
            Stop::Residual(1e-6),
            Stop::Residual(1e-7),
            Stop::Residual(1e-8),
            Stop::Either(1e-6),
            Stop::Either(1e-7),
            Stop::Either(1e-8),
            Stop::Fixed,
        ] {
            for publish in [Publish::Old, Publish::New, Publish::Previous] {
                for cap in [20, 50, 100] {
                    for (low, high) in [(-0.9, 1.0), (-0.5, 1.0), (0.0, 1.0), (0.0, 0.1)] {
                        let cfg = FalsePositionCfg {
                            ops,
                            stop,
                            publish,
                            cap,
                            low,
                            high,
                        };
                        let s = score(&rows, |r| false_position(r, cfg));
                        false_positions.push((s, cfg));
                    }
                }
            }
        }
    }
    false_positions.sort_by_key(|x| x.0.rank());
    println!("false-position candidates={}", false_positions.len());
    for (s, cfg) in false_positions.iter().take(20) {
        println!(
            "  {}/{} max={} sum={} {cfg:?}",
            s.exact,
            rows.len(),
            s.max,
            s.sum
        );
    }

    let winner = newtons.first().map(|x| x.1).unwrap();
    println!("best-newton residuals:");
    for row in &rows {
        let got = newton(row, winner).unwrap_or(f64::NAN);
        println!(
            "  {} got=0x{:016x} want=0x{:016x} signed={}",
            row.tag,
            got.to_bits(),
            row.want,
            ordered(got.to_bits()) as i128 - ordered(row.want) as i128
        );
    }
}
