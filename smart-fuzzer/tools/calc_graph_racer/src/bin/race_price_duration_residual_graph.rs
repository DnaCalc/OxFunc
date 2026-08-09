//! Clean-room PRICE/DURATION residual calculation-graph racer.
//!
//! This lane deliberately excludes YIELD and ODDFYIELD.  It races only the
//! forward PRICE sum and the DURATION numerator/denominator accumulators over
//! Actual/360 and Actual/365 rows, where schedule quantities are unambiguous.

use oxfunc_core::excel_numeric::research as rx;
use oxfunc_core::functions::bond_core_family::{duration_kernel, price_kernel};
use oxfunc_core::locale_format::{
    WorkbookDateSystem, excel_serial_from_ymd, ymd_from_excel_serial,
};
use serde::Deserialize;
use serde_json::Value;
use std::cmp::Reverse;
use std::collections::BTreeMap;
use std::path::Path;

const CW: u16 = rx::CW_PC64_RN;
const OLD_PRICE: &str = "../../work/w109/G6-b2b3/answers-b37-price.json";
const OLD_DURATION: &str = "../../work/w109/G6-b2b3/answers-b44-duration.json";

#[derive(Deserialize)]
struct AnswerSet {
    function: String,
    witnesses: Vec<Witness>,
    #[serde(default)]
    capture_provenance: Option<Value>,
}

#[derive(Deserialize)]
struct Witness {
    id: String,
    args: Vec<String>,
    expected_bits: String,
}

#[derive(Clone)]
struct PriceRow {
    id: String,
    settlement: i64,
    maturity: i64,
    rate: f64,
    yld: f64,
    redemption: f64,
    frequency: i64,
    basis: i64,
    want: u64,
}

#[derive(Clone)]
struct DurationRow {
    id: String,
    settlement: i64,
    maturity: i64,
    coupon: f64,
    yld: f64,
    frequency: i64,
    basis: i64,
    want: u64,
}

#[derive(Clone, Default)]
struct Score {
    exact: usize,
    total: usize,
    max_ulp: u64,
    sum_ulp: u128,
    signed: BTreeMap<i64, usize>,
}

impl Score {
    fn add(&mut self, got: u64, want: u64) {
        let signed = signed_ulp(got, want);
        let distance = signed.unsigned_abs();
        self.exact += usize::from(distance == 0);
        self.total += 1;
        self.max_ulp = self.max_ulp.max(distance);
        self.sum_ulp += u128::from(distance);
        *self.signed.entry(signed.clamp(-16, 16)).or_default() += 1;
    }

    fn rank(&self) -> (Reverse<usize>, u64, u128) {
        (Reverse(self.exact), self.max_ulp, self.sum_ulp)
    }
}

#[derive(Clone, Copy)]
struct X(rx::Ext80);

impl X {
    fn f(value: f64) -> Self {
        Self(rx::ext_from_f64(value))
    }

    fn one() -> Self {
        Self(rx::ext_one())
    }

    fn add(self, rhs: Self) -> Self {
        Self(rx::ext_add(&self.0, &rhs.0, CW))
    }

    fn sub(self, rhs: Self) -> Self {
        Self(rx::ext_sub(&self.0, &rhs.0, CW))
    }

    fn mul(self, rhs: Self) -> Self {
        Self(rx::ext_mul(&self.0, &rhs.0, CW))
    }

    fn div(self, rhs: Self) -> Self {
        Self(rx::ext_div(&self.0, &rhs.0, CW))
    }

    fn store(self) -> f64 {
        rx::ext_to_f64(&self.0, CW)
    }
}

fn exp2_from_x(t: X) -> X {
    let k = X(rx::ext_rndint(&t.0, CW));
    let f = t.sub(k);
    let negative = f.store() < 0.0;
    let w = X(rx::ext_f2xm1(&rx::ext_abs(&f.0, CW), CW));
    let mut m = w.add(X::one());
    if negative {
        m = X::one().div(m);
    }
    X(rx::ext_scale(&m.0, &k.0, CW))
}

fn exp_from_x(argument: X) -> X {
    exp2_from_x(X(rx::ext_mul(&argument.0, &rx::ext_l2e(), CW)))
}

fn bits(text: &str) -> u64 {
    u64::from_str_radix(text.strip_prefix("0x").expect("0x bits"), 16).expect("hex bits")
}

fn ordered(value: u64) -> u64 {
    if value >> 63 == 0 {
        value | (1_u64 << 63)
    } else {
        !value
    }
}

fn signed_ulp(got: u64, want: u64) -> i64 {
    let got = i128::from(ordered(got));
    let want = i128::from(ordered(want));
    (got - want).clamp(i128::from(i64::MIN), i128::from(i64::MAX)) as i64
}

fn load(path: &Path, function: &str, require_current_provenance: bool) -> AnswerSet {
    let text = std::fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    let set: AnswerSet = serde_json::from_str(&text).expect("parse answer set");
    assert_eq!(set.function, function);
    if require_current_provenance {
        let provenance = set
            .capture_provenance
            .as_ref()
            .expect("fresh discovery capture provenance");
        assert_eq!(
            provenance
                .pointer("/environment/excel_version")
                .and_then(Value::as_str),
            Some("16.0")
        );
        assert_eq!(
            provenance
                .pointer("/environment/excel_build")
                .and_then(Value::as_str),
            Some("20228")
        );
        assert_eq!(
            provenance
                .pointer("/environment/excel_bitness")
                .and_then(Value::as_str),
            Some("64-bit")
        );
        assert_eq!(
            provenance
                .pointer("/environment/workbook_compatibility")
                .and_then(Value::as_str),
            Some("2")
        );
        assert_eq!(
            provenance
                .pointer("/environment/excel_input_plumbing")
                .and_then(Value::as_str),
            Some("cell_value2_bulk")
        );
        assert_eq!(
            provenance
                .pointer("/oracle_cache/mode")
                .and_then(Value::as_str),
            Some("no_cache")
        );
    }
    set
}

fn price_rows(set: AnswerSet) -> Vec<PriceRow> {
    set.witnesses
        .into_iter()
        .map(|witness| {
            assert_eq!(witness.args.len(), 7);
            let args = witness
                .args
                .iter()
                .map(|arg| f64::from_bits(bits(arg)))
                .collect::<Vec<_>>();
            PriceRow {
                id: witness.id,
                settlement: args[0] as i64,
                maturity: args[1] as i64,
                rate: args[2],
                yld: args[3],
                redemption: args[4],
                frequency: args[5] as i64,
                basis: args[6] as i64,
                want: bits(&witness.expected_bits),
            }
        })
        .filter(|row| matches!(row.basis, 2 | 3))
        .collect()
}

fn duration_rows(set: AnswerSet) -> Vec<DurationRow> {
    set.witnesses
        .into_iter()
        .map(|witness| {
            assert_eq!(witness.args.len(), 6);
            let args = witness
                .args
                .iter()
                .map(|arg| f64::from_bits(bits(arg)))
                .collect::<Vec<_>>();
            DurationRow {
                id: witness.id,
                settlement: args[0] as i64,
                maturity: args[1] as i64,
                coupon: args[2],
                yld: args[3],
                frequency: args[4] as i64,
                basis: args[5] as i64,
                want: bits(&witness.expected_bits),
            }
        })
        .filter(|row| matches!(row.basis, 2 | 3))
        .collect()
}

fn leap(year: i64) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

fn days_in_month(year: i64, month: i64) -> i64 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap(year) => 29,
        2 => 28,
        _ => panic!("month"),
    }
}

fn add_months(serial: i64, months: i64) -> i64 {
    let (year, month, day) =
        ymd_from_excel_serial(WorkbookDateSystem::System1900, serial as f64).expect("serial date");
    let index = year * 12 + month - 1 + months;
    let target_year = index.div_euclid(12);
    let target_month = index.rem_euclid(12) + 1;
    let target_day = day.min(days_in_month(target_year, target_month));
    excel_serial_from_ymd(
        WorkbookDateSystem::System1900,
        target_year,
        target_month,
        target_day,
    )
    .expect("shifted serial") as i64
}

#[derive(Clone, Copy)]
struct Period {
    prev: i64,
    n: i64,
}

fn period(settlement: i64, maturity: i64, frequency: i64) -> Period {
    let months = 12 / frequency;
    let mut next = maturity;
    let mut n = 1_i64;
    loop {
        let prev = add_months(next, -months);
        if prev <= settlement {
            return Period { prev, n };
        }
        next = prev;
        n += 1;
    }
}

fn schedule(settlement: i64, maturity: i64, frequency: i64, basis: i64) -> (Period, f64, f64, f64) {
    assert!(matches!(basis, 2 | 3));
    let p = period(settlement, maturity, frequency);
    let e = if basis == 2 { 360.0 } else { 365.0 } / frequency as f64;
    let a = (settlement - p.prev) as f64;
    let off = (e - a) / e;
    (p, e, a, off)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Power {
    Chain,
    DirectX87,
    Powf,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Store {
    Strict,
    X87Stored,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Body {
    Strict,
    X87Stored,
    ExtFoldStoredTerm,
    X87Continuous,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Term {
    Divide,
    ReciprocalMultiply,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Order {
    Forward,
    Reverse,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Redemption {
    SeparateAfter,
    FoldLast,
    RedemptionFirst,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Accrued {
    CoupMulADivE,
    CoupMulRatio,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Weight {
    DiffCashDivDisc,
    DiffTimesCashDivDisc,
    DiffDivDiscTimesCash,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FinalRatio {
    NumDenThenF,
    NumOverDenF,
    NumOverFThenDen,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BaseInput {
    Stored,
    QuotientStored,
    RawPc64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ExponentInput {
    Stored,
    OffsetStored,
    RawPc64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RetainedPower {
    ChainStored,
    ChainResultPc64,
    ChainAllPc64,
    DirectPc64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PublishAt {
    Discount,
    Term,
    Accumulator,
    Final,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AccruedInput {
    Stored,
    RawPc64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DurationDiff {
    Stored,
    SamePc64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PriceFactorGraph {
    CouponPvRedemptionAccrued,
    CouponPvAccruedRedemption,
    RedemptionCouponPvAccrued,
    CouponNetThenRedemption,
    RedemptionThenCouponNet,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Association {
    Left,
    Right,
    Balanced,
    AdjacentRounds,
    Blocks2,
    Blocks3,
    Blocks4,
    Blocks5,
    Blocks6,
    Lanes2,
    Lanes3,
    Lanes4,
}

#[derive(Clone, Copy, Debug)]
struct Common {
    power: Power,
    base: Store,
    exponent: Store,
    body: Body,
}

#[derive(Clone, Copy, Debug)]
struct PriceCandidate {
    common: Common,
    term: Term,
    order: Order,
    redemption: Redemption,
    accrued: Accrued,
}

#[derive(Clone, Copy, Debug)]
struct DurationCandidate {
    common: Common,
    weight: Weight,
    order: Order,
    redemption: Redemption,
    final_ratio: FinalRatio,
}

#[derive(Clone, Copy, Debug)]
struct RetainedPriceCandidate {
    base: BaseInput,
    exponent: ExponentInput,
    power: RetainedPower,
    publish: PublishAt,
    accrued: AccruedInput,
}

#[derive(Clone, Copy, Debug)]
struct RetainedDurationCandidate {
    base: BaseInput,
    exponent: ExponentInput,
    power: RetainedPower,
    publish: PublishAt,
    diff: DurationDiff,
}

#[derive(Clone, Copy, Debug)]
struct FactorPriceCandidate {
    body: Body,
    term: Term,
    order: Order,
    graph: PriceFactorGraph,
}

#[derive(Clone, Copy, Debug)]
struct FactorDurationCandidate {
    body: Body,
    weight: Weight,
    order: Order,
    final_ratio: FinalRatio,
}

#[derive(Clone, Copy, Debug)]
struct AssociationPriceCandidate {
    body: Body,
    association: Association,
    redemption_in_tree: bool,
}

fn add(store: Store, lhs: f64, rhs: f64) -> f64 {
    match store {
        Store::Strict => lhs + rhs,
        Store::X87Stored => X::f(lhs).add(X::f(rhs)).store(),
    }
}

fn sub(store: Store, lhs: f64, rhs: f64) -> f64 {
    match store {
        Store::Strict => lhs - rhs,
        Store::X87Stored => X::f(lhs).sub(X::f(rhs)).store(),
    }
}

fn mul(store: Store, lhs: f64, rhs: f64) -> f64 {
    match store {
        Store::Strict => lhs * rhs,
        Store::X87Stored => X::f(lhs).mul(X::f(rhs)).store(),
    }
}

fn div(store: Store, lhs: f64, rhs: f64) -> f64 {
    match store {
        Store::Strict => lhs / rhs,
        Store::X87Stored => X::f(lhs).div(X::f(rhs)).store(),
    }
}

fn binexp(base: f64, exponent: f64) -> f64 {
    let mut n = exponent as u64;
    let mut result = 1.0;
    let mut factor = base;
    while n > 0 {
        if n & 1 == 1 {
            result *= factor;
        }
        n >>= 1;
        if n > 0 {
            factor *= factor;
        }
    }
    result
}

fn discount(power: Power, base: f64, exponent: f64) -> f64 {
    if exponent >= 0.0 && exponent < 1024.0 && exponent.fract() == 0.0 {
        return binexp(base, exponent);
    }
    match power {
        Power::Chain => rx::excel_pow_chain(base, exponent),
        Power::DirectX87 => rx::excel_pow_x87_direct(base, exponent),
        Power::Powf => base.powf(exponent),
    }
}

fn common_values(common: Common, yld: f64, frequency: f64, off: f64, k: i64) -> (f64, f64) {
    let base = add(common.base, 1.0, div(common.base, yld, frequency));
    let exponent = add(common.exponent, off, k as f64);
    (exponent, discount(common.power, base, exponent))
}

fn retained_inputs(
    base_input: BaseInput,
    exponent_input: ExponentInput,
    yld: f64,
    frequency: f64,
    e: f64,
    a: f64,
    off: f64,
    k: i64,
) -> (X, X) {
    let base = match base_input {
        BaseInput::Stored => X::f(1.0 + yld / frequency),
        BaseInput::QuotientStored => X::one().add(X::f(yld / frequency)),
        BaseInput::RawPc64 => X::one().add(X::f(yld).div(X::f(frequency))),
    };
    let exponent = match exponent_input {
        ExponentInput::Stored => X::f(off + k as f64),
        ExponentInput::OffsetStored => X::f(off).add(X::f(k as f64)),
        ExponentInput::RawPc64 => X::f(e).sub(X::f(a)).div(X::f(e)).add(X::f(k as f64)),
    };
    (base, exponent)
}

fn retained_discount(power: RetainedPower, base: X, exponent: X) -> X {
    match power {
        RetainedPower::ChainStored => X::f(rx::excel_pow_chain(base.store(), exponent.store())),
        RetainedPower::ChainResultPc64 => {
            let ln_base = rx::excel_ln(base.store());
            let product = rx::x87_mul(exponent.store(), ln_base);
            exp_from_x(X::f(product))
        }
        RetainedPower::ChainAllPc64 => {
            let ln_base = X(rx::ext_fyl2x(&rx::ext_ln2(), &base.0, CW));
            exp_from_x(exponent.mul(ln_base))
        }
        RetainedPower::DirectPc64 => exp2_from_x(X(rx::ext_fyl2x(&exponent.0, &base.0, CW))),
    }
}

fn retained_price_model(candidate: RetainedPriceCandidate, row: &PriceRow) -> f64 {
    let (p, e, a, off) = schedule(row.settlement, row.maturity, row.frequency, row.basis);
    let f = row.frequency as f64;
    assert!(p.n > 1);
    let coup = (100.0 * row.rate) / f;
    let accrued = match candidate.accrued {
        AccruedInput::Stored => X::f(coup * (a / e)),
        AccruedInput::RawPc64 => X::f(coup).mul(X::f(a).div(X::f(e))),
    };
    let mut sum_f64 = 0.0;
    let mut sum_pc64 = X::f(0.0);
    let mut add_cash = |k: i64, cash: f64| {
        let (base, exponent) =
            retained_inputs(candidate.base, candidate.exponent, row.yld, f, e, a, off, k);
        let disc = retained_discount(candidate.power, base, exponent);
        match candidate.publish {
            PublishAt::Discount => sum_f64 += cash / disc.store(),
            PublishAt::Term => sum_f64 += X::f(cash).div(disc).store(),
            PublishAt::Accumulator => {
                let term = X::f(cash).div(disc);
                sum_f64 = X::f(sum_f64).add(term).store();
            }
            PublishAt::Final => {
                sum_pc64 = sum_pc64.add(X::f(cash).div(disc));
            }
        }
    };
    for k in 0..p.n {
        add_cash(k, coup);
    }
    add_cash(p.n - 1, row.redemption);
    match candidate.publish {
        PublishAt::Discount | PublishAt::Term => sum_f64 - accrued.store(),
        PublishAt::Accumulator => X::f(sum_f64).sub(accrued).store(),
        PublishAt::Final => sum_pc64.sub(accrued).store(),
    }
}

fn retained_duration_model(candidate: RetainedDurationCandidate, row: &DurationRow) -> f64 {
    let (p, e, a, off) = schedule(row.settlement, row.maturity, row.frequency, row.basis);
    let f = row.frequency as f64;
    if p.n == 1 {
        return (off / f).max(0.0);
    }
    let coup = (100.0 * row.coupon) / f;
    let mut num_f64 = 0.0;
    let mut den_f64 = 0.0;
    let mut num_pc64 = X::f(0.0);
    let mut den_pc64 = X::f(0.0);
    let mut add_cash = |k: i64, cash: f64| {
        let (base, exponent) =
            retained_inputs(candidate.base, candidate.exponent, row.yld, f, e, a, off, k);
        let diff = match candidate.diff {
            DurationDiff::Stored => X::f(exponent.store()),
            DurationDiff::SamePc64 => exponent,
        };
        let disc = retained_discount(candidate.power, base, exponent);
        match candidate.publish {
            PublishAt::Discount => {
                let diff = diff.store();
                let disc = disc.store();
                num_f64 += (diff * cash) / disc;
                den_f64 += cash / disc;
            }
            PublishAt::Term => {
                num_f64 += diff.mul(X::f(cash)).div(disc).store();
                den_f64 += X::f(cash).div(disc).store();
            }
            PublishAt::Accumulator => {
                num_f64 = X::f(num_f64).add(diff.mul(X::f(cash)).div(disc)).store();
                den_f64 = X::f(den_f64).add(X::f(cash).div(disc)).store();
            }
            PublishAt::Final => {
                num_pc64 = num_pc64.add(diff.mul(X::f(cash)).div(disc));
                den_pc64 = den_pc64.add(X::f(cash).div(disc));
            }
        }
    };
    for k in 0..p.n {
        add_cash(k, coup);
    }
    add_cash(p.n - 1, 100.0);
    match candidate.publish {
        PublishAt::Final => num_pc64.div(den_pc64).div(X::f(f)).store(),
        PublishAt::Discount | PublishAt::Term | PublishAt::Accumulator => (num_f64 / den_f64) / f,
    }
}

fn factor_price_model(candidate: FactorPriceCandidate, row: &PriceRow) -> f64 {
    let (p, e, a, off) = schedule(row.settlement, row.maturity, row.frequency, row.basis);
    let f = row.frequency as f64;
    assert!(p.n > 1);
    let common = Common {
        power: Power::Chain,
        base: Store::Strict,
        exponent: Store::Strict,
        body: candidate.body,
    };
    let store = body_store(candidate.body);
    let coup = div(store, mul(store, 100.0, row.rate), f);
    let indices = match candidate.order {
        Order::Forward => (0..p.n).collect::<Vec<_>>(),
        Order::Reverse => (0..p.n).rev().collect::<Vec<_>>(),
    };
    let discount_sum = match candidate.body {
        Body::Strict | Body::X87Stored => {
            let mut sum = 0.0;
            for k in indices {
                let (_, disc) = common_values(common, row.yld, f, off, k);
                sum = add(
                    store,
                    sum,
                    term_f64(candidate.body, candidate.term, 1.0, disc),
                );
            }
            sum
        }
        Body::ExtFoldStoredTerm | Body::X87Continuous => {
            let continuous_term = candidate.body == Body::X87Continuous;
            let mut sum = X::f(0.0);
            for k in indices {
                let (_, disc) = common_values(common, row.yld, f, off, k);
                let term = if continuous_term {
                    term_x(candidate.term, 1.0, disc)
                } else {
                    X::f(term_f64(Body::Strict, candidate.term, 1.0, disc))
                };
                sum = sum.add(term);
            }
            sum.store()
        }
    };
    let (_, last_disc) = common_values(common, row.yld, f, off, p.n - 1);
    let redemption_pv = term_f64(candidate.body, candidate.term, row.redemption, last_disc);
    let coupon_pv = mul(store, coup, discount_sum);
    let accrued_ratio = div(store, a, e);
    let accrued = mul(store, coup, accrued_ratio);
    match candidate.graph {
        PriceFactorGraph::CouponPvRedemptionAccrued => {
            sub(store, add(store, coupon_pv, redemption_pv), accrued)
        }
        PriceFactorGraph::CouponPvAccruedRedemption => {
            add(store, sub(store, coupon_pv, accrued), redemption_pv)
        }
        PriceFactorGraph::RedemptionCouponPvAccrued => {
            sub(store, add(store, redemption_pv, coupon_pv), accrued)
        }
        PriceFactorGraph::CouponNetThenRedemption => add(
            store,
            mul(store, coup, sub(store, discount_sum, accrued_ratio)),
            redemption_pv,
        ),
        PriceFactorGraph::RedemptionThenCouponNet => add(
            store,
            redemption_pv,
            mul(store, coup, sub(store, discount_sum, accrued_ratio)),
        ),
    }
}

fn balanced_sum(store: Store, terms: &[f64]) -> f64 {
    match terms {
        [] => 0.0,
        [term] => *term,
        _ => {
            let middle = terms.len() / 2;
            add(
                store,
                balanced_sum(store, &terms[..middle]),
                balanced_sum(store, &terms[middle..]),
            )
        }
    }
}

fn association_sum(store: Store, association: Association, terms: &[f64]) -> f64 {
    match association {
        Association::Left => terms.iter().fold(0.0, |sum, term| add(store, sum, *term)),
        Association::Right => terms
            .iter()
            .rev()
            .fold(0.0, |sum, term| add(store, sum, *term)),
        Association::Balanced => balanced_sum(store, terms),
        Association::AdjacentRounds => {
            let mut level = terms.to_vec();
            while level.len() > 1 {
                let mut next = Vec::with_capacity(level.len().div_ceil(2));
                let mut pairs = level.chunks_exact(2);
                for pair in &mut pairs {
                    next.push(add(store, pair[0], pair[1]));
                }
                if let [tail] = pairs.remainder() {
                    next.push(*tail);
                }
                level = next;
            }
            level.first().copied().unwrap_or(0.0)
        }
        Association::Blocks2
        | Association::Blocks3
        | Association::Blocks4
        | Association::Blocks5
        | Association::Blocks6 => {
            let width = match association {
                Association::Blocks2 => 2,
                Association::Blocks3 => 3,
                Association::Blocks4 => 4,
                Association::Blocks5 => 5,
                Association::Blocks6 => 6,
                _ => unreachable!(),
            };
            terms.chunks(width).fold(0.0, |total, chunk| {
                add(
                    store,
                    total,
                    chunk.iter().fold(0.0, |sum, term| add(store, sum, *term)),
                )
            })
        }
        Association::Lanes2 | Association::Lanes3 | Association::Lanes4 => {
            let width = match association {
                Association::Lanes2 => 2,
                Association::Lanes3 => 3,
                Association::Lanes4 => 4,
                _ => unreachable!(),
            };
            let mut lanes = vec![0.0; width];
            for (index, term) in terms.iter().enumerate() {
                lanes[index % width] = add(store, lanes[index % width], *term);
            }
            lanes.iter().fold(0.0, |sum, lane| add(store, sum, *lane))
        }
    }
}

fn association_price_model(candidate: AssociationPriceCandidate, row: &PriceRow) -> f64 {
    let (p, e, a, off) = schedule(row.settlement, row.maturity, row.frequency, row.basis);
    let f = row.frequency as f64;
    assert!(p.n > 1);
    let store = body_store(candidate.body);
    let common = Common {
        power: Power::Chain,
        base: Store::Strict,
        exponent: Store::Strict,
        body: candidate.body,
    };
    let coup = div(store, mul(store, 100.0, row.rate), f);
    let mut terms = Vec::with_capacity(p.n as usize + usize::from(candidate.redemption_in_tree));
    for k in 0..p.n {
        let (_, disc) = common_values(common, row.yld, f, off, k);
        terms.push(div(store, coup, disc));
    }
    let (_, last_disc) = common_values(common, row.yld, f, off, p.n - 1);
    let redemption = div(store, row.redemption, last_disc);
    if candidate.redemption_in_tree {
        terms.push(redemption);
    }
    let coupon_or_dirty = association_sum(store, candidate.association, &terms);
    let dirty = if candidate.redemption_in_tree {
        coupon_or_dirty
    } else {
        add(store, coupon_or_dirty, redemption)
    };
    sub(store, dirty, mul(store, coup, div(store, a, e)))
}

fn factor_duration_model(candidate: FactorDurationCandidate, row: &DurationRow) -> f64 {
    let (p, _, _, off) = schedule(row.settlement, row.maturity, row.frequency, row.basis);
    let f = row.frequency as f64;
    if p.n == 1 {
        return (off / f).max(0.0);
    }
    let common = Common {
        power: Power::Chain,
        base: Store::Strict,
        exponent: Store::Strict,
        body: candidate.body,
    };
    let store = body_store(candidate.body);
    let coup = div(store, mul(store, 100.0, row.coupon), f);
    let indices = match candidate.order {
        Order::Forward => (0..p.n).collect::<Vec<_>>(),
        Order::Reverse => (0..p.n).rev().collect::<Vec<_>>(),
    };
    let (coupon_num_factor, coupon_den_factor) = match candidate.body {
        Body::Strict | Body::X87Stored => {
            let mut num = 0.0;
            let mut den = 0.0;
            for k in indices {
                let (diff, disc) = common_values(common, row.yld, f, off, k);
                num = add(
                    store,
                    num,
                    weight_f64(candidate.body, candidate.weight, diff, 1.0, disc),
                );
                den = add(store, den, div(store, 1.0, disc));
            }
            (num, den)
        }
        Body::ExtFoldStoredTerm | Body::X87Continuous => {
            let continuous_term = candidate.body == Body::X87Continuous;
            let mut num = X::f(0.0);
            let mut den = X::f(0.0);
            for k in indices {
                let (diff, disc) = common_values(common, row.yld, f, off, k);
                if continuous_term {
                    num = num.add(weight_x(candidate.weight, diff, 1.0, disc));
                    den = den.add(X::one().div(X::f(disc)));
                } else {
                    num = num.add(X::f(weight_f64(
                        Body::Strict,
                        candidate.weight,
                        diff,
                        1.0,
                        disc,
                    )));
                    den = den.add(X::f(1.0 / disc));
                }
            }
            (num.store(), den.store())
        }
    };
    let (last_diff, last_disc) = common_values(common, row.yld, f, off, p.n - 1);
    let coupon_num = mul(store, coup, coupon_num_factor);
    let coupon_den = mul(store, coup, coupon_den_factor);
    let redemption_num = weight_f64(
        candidate.body,
        candidate.weight,
        last_diff,
        100.0,
        last_disc,
    );
    let redemption_den = div(store, 100.0, last_disc);
    let num = add(store, coupon_num, redemption_num);
    let den = add(store, coupon_den, redemption_den);
    final_f64(store, candidate.final_ratio, num, den, f)
}

fn body_store(body: Body) -> Store {
    match body {
        Body::X87Stored => Store::X87Stored,
        Body::Strict | Body::ExtFoldStoredTerm | Body::X87Continuous => Store::Strict,
    }
}

fn term_f64(body: Body, graph: Term, cash: f64, disc: f64) -> f64 {
    let store = body_store(body);
    match graph {
        Term::Divide => div(store, cash, disc),
        Term::ReciprocalMultiply => mul(store, cash, div(store, 1.0, disc)),
    }
}

fn term_x(graph: Term, cash: f64, disc: f64) -> X {
    match graph {
        Term::Divide => X::f(cash).div(X::f(disc)),
        Term::ReciprocalMultiply => X::f(cash).mul(X::one().div(X::f(disc))),
    }
}

fn price_model(candidate: PriceCandidate, row: &PriceRow) -> f64 {
    let (p, e, a, off) = schedule(row.settlement, row.maturity, row.frequency, row.basis);
    let f = row.frequency as f64;
    let store = body_store(candidate.common.body);
    let coup = div(store, mul(store, 100.0, row.rate), f);
    if p.n == 1 {
        let den = add(
            store,
            1.0,
            mul(store, div(store, row.yld, f), div(store, e - a, e)),
        );
        let dirty = div(store, add(store, row.redemption, coup), den);
        let accr = div(store, mul(store, coup, a), e);
        return sub(store, dirty, accr);
    }

    let indices = match candidate.order {
        Order::Forward => (0..p.n).collect::<Vec<_>>(),
        Order::Reverse => (0..p.n).rev().collect::<Vec<_>>(),
    };
    let last = p.n - 1;

    let accrued_f64 = match candidate.accrued {
        Accrued::CoupMulADivE => div(store, mul(store, coup, a), e),
        Accrued::CoupMulRatio => mul(store, coup, div(store, a, e)),
    };

    match candidate.common.body {
        Body::Strict | Body::X87Stored => {
            let mut sum = 0.0;
            if candidate.redemption == Redemption::RedemptionFirst {
                let (_, disc) = common_values(candidate.common, row.yld, f, off, last);
                sum = add(
                    store,
                    sum,
                    term_f64(candidate.common.body, candidate.term, row.redemption, disc),
                );
            }
            for k in indices {
                let (_, disc) = common_values(candidate.common, row.yld, f, off, k);
                let cash = if candidate.redemption == Redemption::FoldLast && k == last {
                    add(store, coup, row.redemption)
                } else {
                    coup
                };
                sum = add(
                    store,
                    sum,
                    term_f64(candidate.common.body, candidate.term, cash, disc),
                );
            }
            if candidate.redemption == Redemption::SeparateAfter {
                let (_, disc) = common_values(candidate.common, row.yld, f, off, last);
                sum = add(
                    store,
                    sum,
                    term_f64(candidate.common.body, candidate.term, row.redemption, disc),
                );
            }
            sub(store, sum, accrued_f64)
        }
        Body::ExtFoldStoredTerm | Body::X87Continuous => {
            let extended_term = candidate.common.body == Body::X87Continuous;
            let mut sum = X::f(0.0);
            let make_term = |cash: f64, disc: f64| {
                if extended_term {
                    term_x(candidate.term, cash, disc)
                } else {
                    X::f(term_f64(Body::Strict, candidate.term, cash, disc))
                }
            };
            if candidate.redemption == Redemption::RedemptionFirst {
                let (_, disc) = common_values(candidate.common, row.yld, f, off, last);
                sum = sum.add(make_term(row.redemption, disc));
            }
            for k in indices {
                let (_, disc) = common_values(candidate.common, row.yld, f, off, k);
                let cash = if candidate.redemption == Redemption::FoldLast && k == last {
                    if extended_term {
                        X::f(coup).add(X::f(row.redemption)).store()
                    } else {
                        coup + row.redemption
                    }
                } else {
                    coup
                };
                sum = sum.add(make_term(cash, disc));
            }
            if candidate.redemption == Redemption::SeparateAfter {
                let (_, disc) = common_values(candidate.common, row.yld, f, off, last);
                sum = sum.add(make_term(row.redemption, disc));
            }
            if extended_term {
                let accr = match candidate.accrued {
                    Accrued::CoupMulADivE => X::f(coup).mul(X::f(a)).div(X::f(e)),
                    Accrued::CoupMulRatio => X::f(coup).mul(X::f(a).div(X::f(e))),
                };
                sum.sub(accr).store()
            } else {
                sum.store() - accrued_f64
            }
        }
    }
}

fn weight_f64(body: Body, graph: Weight, diff: f64, cash: f64, disc: f64) -> f64 {
    let store = body_store(body);
    match graph {
        Weight::DiffCashDivDisc => div(store, mul(store, diff, cash), disc),
        Weight::DiffTimesCashDivDisc => mul(store, diff, div(store, cash, disc)),
        Weight::DiffDivDiscTimesCash => mul(store, div(store, diff, disc), cash),
    }
}

fn weight_x(graph: Weight, diff: f64, cash: f64, disc: f64) -> X {
    match graph {
        Weight::DiffCashDivDisc => X::f(diff).mul(X::f(cash)).div(X::f(disc)),
        Weight::DiffTimesCashDivDisc => X::f(diff).mul(X::f(cash).div(X::f(disc))),
        Weight::DiffDivDiscTimesCash => X::f(diff).div(X::f(disc)).mul(X::f(cash)),
    }
}

fn final_f64(store: Store, graph: FinalRatio, num: f64, den: f64, f: f64) -> f64 {
    match graph {
        FinalRatio::NumDenThenF => div(store, div(store, num, den), f),
        FinalRatio::NumOverDenF => div(store, num, mul(store, den, f)),
        FinalRatio::NumOverFThenDen => div(store, div(store, num, f), den),
    }
}

fn final_x(graph: FinalRatio, num: X, den: X, f: f64) -> f64 {
    match graph {
        FinalRatio::NumDenThenF => num.div(den).div(X::f(f)).store(),
        FinalRatio::NumOverDenF => num.div(den.mul(X::f(f))).store(),
        FinalRatio::NumOverFThenDen => num.div(X::f(f)).div(den).store(),
    }
}

fn duration_model(candidate: DurationCandidate, row: &DurationRow) -> f64 {
    let (p, _, _, off) = schedule(row.settlement, row.maturity, row.frequency, row.basis);
    let f = row.frequency as f64;
    if p.n == 1 {
        return (off / f).max(0.0);
    }
    let store = body_store(candidate.common.body);
    let coup = div(store, mul(store, 100.0, row.coupon), f);
    let indices = match candidate.order {
        Order::Forward => (0..p.n).collect::<Vec<_>>(),
        Order::Reverse => (0..p.n).rev().collect::<Vec<_>>(),
    };
    let last = p.n - 1;

    match candidate.common.body {
        Body::Strict | Body::X87Stored => {
            let mut num = 0.0;
            let mut den = 0.0;
            let mut add_cash = |k: i64, cash: f64| {
                let (diff, disc) = common_values(candidate.common, row.yld, f, off, k);
                num = add(
                    store,
                    num,
                    weight_f64(candidate.common.body, candidate.weight, diff, cash, disc),
                );
                den = add(store, den, div(store, cash, disc));
            };
            if candidate.redemption == Redemption::RedemptionFirst {
                add_cash(last, 100.0);
            }
            for k in indices {
                let cash = if candidate.redemption == Redemption::FoldLast && k == last {
                    add(store, coup, 100.0)
                } else {
                    coup
                };
                add_cash(k, cash);
            }
            if candidate.redemption == Redemption::SeparateAfter {
                add_cash(last, 100.0);
            }
            final_f64(store, candidate.final_ratio, num, den, f)
        }
        Body::ExtFoldStoredTerm | Body::X87Continuous => {
            let extended_term = candidate.common.body == Body::X87Continuous;
            let mut num = X::f(0.0);
            let mut den = X::f(0.0);
            let mut add_cash = |k: i64, cash: f64| {
                let (diff, disc) = common_values(candidate.common, row.yld, f, off, k);
                if extended_term {
                    num = num.add(weight_x(candidate.weight, diff, cash, disc));
                    den = den.add(X::f(cash).div(X::f(disc)));
                } else {
                    num = num.add(X::f(weight_f64(
                        Body::Strict,
                        candidate.weight,
                        diff,
                        cash,
                        disc,
                    )));
                    den = den.add(X::f(cash / disc));
                }
            };
            if candidate.redemption == Redemption::RedemptionFirst {
                add_cash(last, 100.0);
            }
            for k in indices {
                let cash = if candidate.redemption == Redemption::FoldLast && k == last {
                    coup + 100.0
                } else {
                    coup
                };
                add_cash(k, cash);
            }
            if candidate.redemption == Redemption::SeparateAfter {
                add_cash(last, 100.0);
            }
            if extended_term {
                final_x(candidate.final_ratio, num, den, f)
            } else {
                final_f64(
                    Store::Strict,
                    candidate.final_ratio,
                    num.store(),
                    den.store(),
                    f,
                )
            }
        }
    }
}

fn score_price(candidate: PriceCandidate, rows: &[PriceRow]) -> Score {
    let mut score = Score::default();
    for row in rows {
        score.add(price_model(candidate, row).to_bits(), row.want);
    }
    score
}

fn score_duration(candidate: DurationCandidate, rows: &[DurationRow]) -> Score {
    let mut score = Score::default();
    for row in rows {
        score.add(duration_model(candidate, row).to_bits(), row.want);
    }
    score
}

fn score_retained_price(candidate: RetainedPriceCandidate, rows: &[PriceRow]) -> Score {
    let mut score = Score::default();
    for row in rows {
        score.add(retained_price_model(candidate, row).to_bits(), row.want);
    }
    score
}

fn score_retained_duration(candidate: RetainedDurationCandidate, rows: &[DurationRow]) -> Score {
    let mut score = Score::default();
    for row in rows {
        score.add(retained_duration_model(candidate, row).to_bits(), row.want);
    }
    score
}

fn score_factor_price(candidate: FactorPriceCandidate, rows: &[PriceRow]) -> Score {
    let mut score = Score::default();
    for row in rows {
        score.add(factor_price_model(candidate, row).to_bits(), row.want);
    }
    score
}

fn score_factor_duration(candidate: FactorDurationCandidate, rows: &[DurationRow]) -> Score {
    let mut score = Score::default();
    for row in rows {
        score.add(factor_duration_model(candidate, row).to_bits(), row.want);
    }
    score
}

fn score_association_price(candidate: AssociationPriceCandidate, rows: &[PriceRow]) -> Score {
    let mut score = Score::default();
    for row in rows {
        score.add(association_price_model(candidate, row).to_bits(), row.want);
    }
    score
}

fn print_price_misses(candidate: RetainedPriceCandidate, rows: &[PriceRow]) {
    println!("PRICE retained-best mismatches:");
    for row in rows {
        let got = retained_price_model(candidate, row).to_bits();
        if got == row.want {
            continue;
        }
        let (p, e, a, off) = schedule(row.settlement, row.maturity, row.frequency, row.basis);
        let f = row.frequency as f64;
        let k = p.n - 1;
        let (base, exponent) =
            retained_inputs(candidate.base, candidate.exponent, row.yld, f, e, a, off, k);
        println!(
            "  {} got=0x{got:016x} want=0x{:016x} signed={} last-base64=0x{:016x} last-exp64=0x{:016x} base-pc64={:?} exp-pc64={:?}",
            row.id,
            row.want,
            signed_ulp(got, row.want),
            base.store().to_bits(),
            exponent.store().to_bits(),
            base.0,
            exponent.0,
        );
    }
}

fn print_fixed_price_misses(candidate: PriceCandidate, rows: &[PriceRow]) {
    println!("PRICE fixed-best mismatches:");
    for row in rows {
        let got = price_model(candidate, row).to_bits();
        if got == row.want {
            continue;
        }
        let (p, _, _, off) = schedule(row.settlement, row.maturity, row.frequency, row.basis);
        let (_, disc) = common_values(
            candidate.common,
            row.yld,
            row.frequency as f64,
            off,
            p.n - 1,
        );
        let base = 1.0 + row.yld / row.frequency as f64;
        let exponent = off + (p.n - 1) as f64;
        println!(
            "  {} got=0x{got:016x} want=0x{:016x} signed={} last-base=0x{:016x} last-exp=0x{:016x} last-disc=0x{:016x}",
            row.id,
            row.want,
            signed_ulp(got, row.want),
            base.to_bits(),
            exponent.to_bits(),
            disc.to_bits(),
        );
    }
}

fn race_retained(price: &[PriceRow], duration: &[DurationRow]) {
    let mut price_ranked = Vec::new();
    for base in [
        BaseInput::Stored,
        BaseInput::QuotientStored,
        BaseInput::RawPc64,
    ] {
        for exponent in [
            ExponentInput::Stored,
            ExponentInput::OffsetStored,
            ExponentInput::RawPc64,
        ] {
            for power in [
                RetainedPower::ChainStored,
                RetainedPower::ChainResultPc64,
                RetainedPower::ChainAllPc64,
                RetainedPower::DirectPc64,
            ] {
                for publish in [
                    PublishAt::Discount,
                    PublishAt::Term,
                    PublishAt::Accumulator,
                    PublishAt::Final,
                ] {
                    for accrued in [AccruedInput::Stored, AccruedInput::RawPc64] {
                        let candidate = RetainedPriceCandidate {
                            base,
                            exponent,
                            power,
                            publish,
                            accrued,
                        };
                        price_ranked.push((score_retained_price(candidate, price), candidate));
                    }
                }
            }
        }
    }
    price_ranked.sort_by_key(|(score, _)| score.rank());
    let price_survivors = price_ranked
        .iter()
        .filter(|(score, _)| score.exact == score.total)
        .count();
    let price_best_rank = price_ranked[0].0.rank();
    let price_best_ties = price_ranked
        .iter()
        .filter(|(score, _)| score.rank() == price_best_rank)
        .count();
    println!(
        "\nPRICE retained-PC64 top 20 ({} fixed candidates; exact survivors={price_survivors}; best-rank ties={price_best_ties}):",
        price_ranked.len()
    );
    for (score, candidate) in price_ranked.iter().take(20) {
        print_score(&format!("{candidate:?}"), score);
    }
    print_price_misses(price_ranked[0].1, price);

    let mut duration_ranked = Vec::new();
    for base in [
        BaseInput::Stored,
        BaseInput::QuotientStored,
        BaseInput::RawPc64,
    ] {
        for exponent in [
            ExponentInput::Stored,
            ExponentInput::OffsetStored,
            ExponentInput::RawPc64,
        ] {
            for power in [
                RetainedPower::ChainStored,
                RetainedPower::ChainResultPc64,
                RetainedPower::ChainAllPc64,
                RetainedPower::DirectPc64,
            ] {
                for publish in [
                    PublishAt::Discount,
                    PublishAt::Term,
                    PublishAt::Accumulator,
                    PublishAt::Final,
                ] {
                    for diff in [DurationDiff::Stored, DurationDiff::SamePc64] {
                        let candidate = RetainedDurationCandidate {
                            base,
                            exponent,
                            power,
                            publish,
                            diff,
                        };
                        duration_ranked
                            .push((score_retained_duration(candidate, duration), candidate));
                    }
                }
            }
        }
    }
    duration_ranked.sort_by_key(|(score, _)| score.rank());
    let duration_survivors = duration_ranked
        .iter()
        .filter(|(score, _)| score.exact == score.total)
        .count();
    let duration_best_rank = duration_ranked[0].0.rank();
    let duration_best_ties = duration_ranked
        .iter()
        .filter(|(score, _)| score.rank() == duration_best_rank)
        .count();
    println!(
        "\nDURATION retained-PC64 top 20 ({} fixed candidates; exact survivors={duration_survivors}; best-rank ties={duration_best_ties}):",
        duration_ranked.len()
    );
    for (score, candidate) in duration_ranked.iter().take(20) {
        print_score(&format!("{candidate:?}"), score);
    }
}

fn race_factorized(price: &[PriceRow], duration: &[DurationRow]) {
    let mut price_ranked = Vec::new();
    for body in [
        Body::Strict,
        Body::X87Stored,
        Body::ExtFoldStoredTerm,
        Body::X87Continuous,
    ] {
        for term in [Term::Divide, Term::ReciprocalMultiply] {
            for order in [Order::Forward, Order::Reverse] {
                for graph in [
                    PriceFactorGraph::CouponPvRedemptionAccrued,
                    PriceFactorGraph::CouponPvAccruedRedemption,
                    PriceFactorGraph::RedemptionCouponPvAccrued,
                    PriceFactorGraph::CouponNetThenRedemption,
                    PriceFactorGraph::RedemptionThenCouponNet,
                ] {
                    let candidate = FactorPriceCandidate {
                        body,
                        term,
                        order,
                        graph,
                    };
                    price_ranked.push((score_factor_price(candidate, price), candidate));
                }
            }
        }
    }
    price_ranked.sort_by_key(|(score, _)| score.rank());
    let price_survivors = price_ranked
        .iter()
        .filter(|(score, _)| score.exact == score.total)
        .count();
    let price_best_rank = price_ranked[0].0.rank();
    let price_best_ties = price_ranked
        .iter()
        .filter(|(score, _)| score.rank() == price_best_rank)
        .count();
    println!(
        "\nPRICE factorized-coupon top 20 ({} fixed candidates; exact survivors={price_survivors}; best-rank ties={price_best_ties}):",
        price_ranked.len()
    );
    for (score, candidate) in price_ranked.iter().take(20) {
        print_score(&format!("{candidate:?}"), score);
    }
    println!("PRICE factorized-best mismatches:");
    for row in price {
        let got = factor_price_model(price_ranked[0].1, row).to_bits();
        if got != row.want {
            println!(
                "  {} got=0x{got:016x} want=0x{:016x} signed={}",
                row.id,
                row.want,
                signed_ulp(got, row.want)
            );
        }
    }

    let mut duration_ranked = Vec::new();
    for body in [
        Body::Strict,
        Body::X87Stored,
        Body::ExtFoldStoredTerm,
        Body::X87Continuous,
    ] {
        for weight in [
            Weight::DiffCashDivDisc,
            Weight::DiffTimesCashDivDisc,
            Weight::DiffDivDiscTimesCash,
        ] {
            for order in [Order::Forward, Order::Reverse] {
                for final_ratio in [
                    FinalRatio::NumDenThenF,
                    FinalRatio::NumOverDenF,
                    FinalRatio::NumOverFThenDen,
                ] {
                    let candidate = FactorDurationCandidate {
                        body,
                        weight,
                        order,
                        final_ratio,
                    };
                    duration_ranked.push((score_factor_duration(candidate, duration), candidate));
                }
            }
        }
    }
    duration_ranked.sort_by_key(|(score, _)| score.rank());
    let duration_survivors = duration_ranked
        .iter()
        .filter(|(score, _)| score.exact == score.total)
        .count();
    let duration_best_rank = duration_ranked[0].0.rank();
    let duration_best_ties = duration_ranked
        .iter()
        .filter(|(score, _)| score.rank() == duration_best_rank)
        .count();
    println!(
        "\nDURATION factorized-coupon top 20 ({} fixed candidates; exact survivors={duration_survivors}; best-rank ties={duration_best_ties}):",
        duration_ranked.len()
    );
    for (score, candidate) in duration_ranked.iter().take(20) {
        print_score(&format!("{candidate:?}"), score);
    }
}

fn race_associations(price: &[PriceRow]) {
    let mut ranked = Vec::new();
    for body in [Body::Strict, Body::X87Stored] {
        for association in [
            Association::Left,
            Association::Right,
            Association::Balanced,
            Association::AdjacentRounds,
            Association::Blocks2,
            Association::Blocks3,
            Association::Blocks4,
            Association::Blocks5,
            Association::Blocks6,
            Association::Lanes2,
            Association::Lanes3,
            Association::Lanes4,
        ] {
            for redemption_in_tree in [false, true] {
                let candidate = AssociationPriceCandidate {
                    body,
                    association,
                    redemption_in_tree,
                };
                ranked.push((score_association_price(candidate, price), candidate));
            }
        }
    }
    ranked.sort_by_key(|(score, _)| score.rank());
    let survivors = ranked
        .iter()
        .filter(|(score, _)| score.exact == score.total)
        .count();
    let best_rank = ranked[0].0.rank();
    let best_ties = ranked
        .iter()
        .filter(|(score, _)| score.rank() == best_rank)
        .count();
    println!(
        "\nPRICE association top 20 ({} fixed candidates; exact survivors={survivors}; best-rank ties={best_ties}):",
        ranked.len()
    );
    for (score, candidate) in ranked.iter().take(20) {
        print_score(&format!("{candidate:?}"), score);
    }
    println!("PRICE association-best mismatches:");
    for row in price {
        let got = association_price_model(ranked[0].1, row).to_bits();
        if got != row.want {
            println!(
                "  {} got=0x{got:016x} want=0x{:016x} signed={}",
                row.id,
                row.want,
                signed_ulp(got, row.want)
            );
        }
    }
}

fn print_ranked<T: std::fmt::Debug>(label: &str, ranked: &mut [(Score, T)]) {
    ranked.sort_by_key(|(score, _)| score.rank());
    let survivors = ranked
        .iter()
        .filter(|(score, _)| score.exact == score.total)
        .count();
    let best_rank = ranked[0].0.rank();
    let best_ties = ranked
        .iter()
        .filter(|(score, _)| score.rank() == best_rank)
        .count();
    println!(
        "\n{label} ({} fixed candidates; exact survivors={survivors}; best-rank ties={best_ties}):",
        ranked.len()
    );
    for (score, candidate) in ranked.iter().take(10) {
        print_score(&format!("{candidate:?}"), score);
    }
}

fn race_price_companion(price: &[PriceRow]) {
    println!(
        "\n=== fresh build-20228 PRICE discovery + frozen companion: {} rows ===",
        price.len()
    );
    let (production, reference_equal) = production_price(price);
    print_score("production PRICE", &production);
    println!(
        "production-reference PRICE graph equality: {reference_equal}/{}",
        price.len()
    );

    let commons = [Power::Chain, Power::DirectX87, Power::Powf]
        .into_iter()
        .flat_map(|power| {
            [Store::Strict, Store::X87Stored]
                .into_iter()
                .flat_map(move |base| {
                    [Store::Strict, Store::X87Stored]
                        .into_iter()
                        .flat_map(move |exponent| {
                            [
                                Body::Strict,
                                Body::X87Stored,
                                Body::ExtFoldStoredTerm,
                                Body::X87Continuous,
                            ]
                            .into_iter()
                            .map(move |body| Common {
                                power,
                                base,
                                exponent,
                                body,
                            })
                        })
                })
        })
        .collect::<Vec<_>>();
    let mut fixed = Vec::new();
    for common in &commons {
        for term in [Term::Divide, Term::ReciprocalMultiply] {
            for order in [Order::Forward, Order::Reverse] {
                for redemption in [
                    Redemption::SeparateAfter,
                    Redemption::FoldLast,
                    Redemption::RedemptionFirst,
                ] {
                    for accrued in [Accrued::CoupMulADivE, Accrued::CoupMulRatio] {
                        let candidate = PriceCandidate {
                            common: *common,
                            term,
                            order,
                            redemption,
                            accrued,
                        };
                        fixed.push((score_price(candidate, price), candidate));
                    }
                }
            }
        }
    }
    print_ranked("PRICE combined original graph family", &mut fixed);

    let mut retained = Vec::new();
    for base in [
        BaseInput::Stored,
        BaseInput::QuotientStored,
        BaseInput::RawPc64,
    ] {
        for exponent in [
            ExponentInput::Stored,
            ExponentInput::OffsetStored,
            ExponentInput::RawPc64,
        ] {
            for power in [
                RetainedPower::ChainStored,
                RetainedPower::ChainResultPc64,
                RetainedPower::ChainAllPc64,
                RetainedPower::DirectPc64,
            ] {
                for publish in [
                    PublishAt::Discount,
                    PublishAt::Term,
                    PublishAt::Accumulator,
                    PublishAt::Final,
                ] {
                    for accrued in [AccruedInput::Stored, AccruedInput::RawPc64] {
                        let candidate = RetainedPriceCandidate {
                            base,
                            exponent,
                            power,
                            publish,
                            accrued,
                        };
                        retained.push((score_retained_price(candidate, price), candidate));
                    }
                }
            }
        }
    }
    print_ranked("PRICE combined retained-PC64 family", &mut retained);

    let mut factorized = Vec::new();
    for body in [
        Body::Strict,
        Body::X87Stored,
        Body::ExtFoldStoredTerm,
        Body::X87Continuous,
    ] {
        for term in [Term::Divide, Term::ReciprocalMultiply] {
            for order in [Order::Forward, Order::Reverse] {
                for graph in [
                    PriceFactorGraph::CouponPvRedemptionAccrued,
                    PriceFactorGraph::CouponPvAccruedRedemption,
                    PriceFactorGraph::RedemptionCouponPvAccrued,
                    PriceFactorGraph::CouponNetThenRedemption,
                    PriceFactorGraph::RedemptionThenCouponNet,
                ] {
                    let candidate = FactorPriceCandidate {
                        body,
                        term,
                        order,
                        graph,
                    };
                    factorized.push((score_factor_price(candidate, price), candidate));
                }
            }
        }
    }
    print_ranked("PRICE combined factorized-coupon family", &mut factorized);

    let mut associations = Vec::new();
    for body in [Body::Strict, Body::X87Stored] {
        for association in [
            Association::Left,
            Association::Right,
            Association::Balanced,
            Association::AdjacentRounds,
            Association::Blocks2,
            Association::Blocks3,
            Association::Blocks4,
            Association::Blocks5,
            Association::Blocks6,
            Association::Lanes2,
            Association::Lanes3,
            Association::Lanes4,
        ] {
            for redemption_in_tree in [false, true] {
                let candidate = AssociationPriceCandidate {
                    body,
                    association,
                    redemption_in_tree,
                };
                associations.push((score_association_price(candidate, price), candidate));
            }
        }
    }
    print_ranked("PRICE combined association family", &mut associations);
}

fn production_price(rows: &[PriceRow]) -> (Score, usize) {
    let reference = PriceCandidate {
        common: Common {
            power: Power::Chain,
            base: Store::Strict,
            exponent: Store::Strict,
            body: Body::Strict,
        },
        term: Term::Divide,
        order: Order::Forward,
        redemption: Redemption::SeparateAfter,
        accrued: Accrued::CoupMulADivE,
    };
    let mut score = Score::default();
    let mut model_equal = 0;
    for row in rows {
        let got = price_kernel(
            row.settlement as f64,
            row.maturity as f64,
            row.rate,
            row.yld,
            row.redemption,
            row.frequency as f64,
            Some(row.basis as f64),
        )
        .unwrap_or_else(|error| panic!("production PRICE {}: {error:?}", row.id));
        score.add(got.to_bits(), row.want);
        model_equal += usize::from(got.to_bits() == price_model(reference, row).to_bits());
    }
    (score, model_equal)
}

fn production_duration(rows: &[DurationRow]) -> (Score, usize) {
    let reference = DurationCandidate {
        common: Common {
            power: Power::Chain,
            base: Store::Strict,
            exponent: Store::Strict,
            body: Body::Strict,
        },
        weight: Weight::DiffCashDivDisc,
        order: Order::Forward,
        redemption: Redemption::SeparateAfter,
        final_ratio: FinalRatio::NumDenThenF,
    };
    let mut score = Score::default();
    let mut model_equal = 0;
    for row in rows {
        let got = duration_kernel(
            row.settlement as f64,
            row.maturity as f64,
            row.coupon,
            row.yld,
            row.frequency as f64,
            Some(row.basis as f64),
        )
        .unwrap_or_else(|error| panic!("production DURATION {}: {error:?}", row.id));
        score.add(got.to_bits(), row.want);
        model_equal += usize::from(got.to_bits() == duration_model(reference, row).to_bits());
    }
    (score, model_equal)
}

fn print_score(label: &str, score: &Score) {
    println!(
        "{label}: {}/{} max={} sum={} signed={:?}",
        score.exact, score.total, score.max_ulp, score.sum_ulp, score.signed
    );
}

fn race(label: &str, price: &[PriceRow], duration: &[DurationRow]) {
    println!(
        "\n=== {label}: PRICE={} DURATION={} (basis 2/3 only) ===",
        price.len(),
        duration.len()
    );
    let (price_production, price_equal) = production_price(price);
    let (duration_production, duration_equal) = production_duration(duration);
    print_score("production PRICE", &price_production);
    println!(
        "production-reference PRICE graph equality: {price_equal}/{}",
        price.len()
    );
    print_score("production DURATION", &duration_production);
    println!(
        "production-reference DURATION graph equality: {duration_equal}/{}",
        duration.len()
    );

    let commons = [Power::Chain, Power::DirectX87, Power::Powf]
        .into_iter()
        .flat_map(|power| {
            [Store::Strict, Store::X87Stored]
                .into_iter()
                .flat_map(move |base| {
                    [Store::Strict, Store::X87Stored]
                        .into_iter()
                        .flat_map(move |exponent| {
                            [
                                Body::Strict,
                                Body::X87Stored,
                                Body::ExtFoldStoredTerm,
                                Body::X87Continuous,
                            ]
                            .into_iter()
                            .map(move |body| Common {
                                power,
                                base,
                                exponent,
                                body,
                            })
                        })
                })
        })
        .collect::<Vec<_>>();

    let mut price_ranked = Vec::new();
    for common in &commons {
        for term in [Term::Divide, Term::ReciprocalMultiply] {
            for order in [Order::Forward, Order::Reverse] {
                for redemption in [
                    Redemption::SeparateAfter,
                    Redemption::FoldLast,
                    Redemption::RedemptionFirst,
                ] {
                    for accrued in [Accrued::CoupMulADivE, Accrued::CoupMulRatio] {
                        let candidate = PriceCandidate {
                            common: *common,
                            term,
                            order,
                            redemption,
                            accrued,
                        };
                        price_ranked.push((score_price(candidate, price), candidate));
                    }
                }
            }
        }
    }
    price_ranked.sort_by_key(|(score, _)| score.rank());
    let price_survivors = price_ranked
        .iter()
        .filter(|(score, _)| score.exact == score.total)
        .count();
    let price_best_rank = price_ranked[0].0.rank();
    let price_best_ties = price_ranked
        .iter()
        .filter(|(score, _)| score.rank() == price_best_rank)
        .count();
    println!(
        "\nPRICE top 20 ({} fixed candidates; exact survivors={price_survivors}; best-rank ties={price_best_ties}):",
        price_ranked.len()
    );
    for (score, candidate) in price_ranked.iter().take(20) {
        print_score(&format!("{candidate:?}"), score);
    }
    if label.contains("fresh") {
        print_fixed_price_misses(price_ranked[0].1, price);
    }

    let mut duration_ranked = Vec::new();
    for common in &commons {
        for weight in [
            Weight::DiffCashDivDisc,
            Weight::DiffTimesCashDivDisc,
            Weight::DiffDivDiscTimesCash,
        ] {
            for order in [Order::Forward, Order::Reverse] {
                for redemption in [
                    Redemption::SeparateAfter,
                    Redemption::FoldLast,
                    Redemption::RedemptionFirst,
                ] {
                    for final_ratio in [
                        FinalRatio::NumDenThenF,
                        FinalRatio::NumOverDenF,
                        FinalRatio::NumOverFThenDen,
                    ] {
                        let candidate = DurationCandidate {
                            common: *common,
                            weight,
                            order,
                            redemption,
                            final_ratio,
                        };
                        duration_ranked.push((score_duration(candidate, duration), candidate));
                    }
                }
            }
        }
    }
    duration_ranked.sort_by_key(|(score, _)| score.rank());
    let duration_survivors = duration_ranked
        .iter()
        .filter(|(score, _)| score.exact == score.total)
        .count();
    let duration_best_rank = duration_ranked[0].0.rank();
    let duration_best_ties = duration_ranked
        .iter()
        .filter(|(score, _)| score.rank() == duration_best_rank)
        .count();
    println!(
        "\nDURATION top 20 ({} fixed candidates; exact survivors={duration_survivors}; best-rank ties={duration_best_ties}):",
        duration_ranked.len()
    );
    for (score, candidate) in duration_ranked.iter().take(20) {
        print_score(&format!("{candidate:?}"), score);
    }
}

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    if args.get(3).map(String::as_str) == Some("--price-companion") {
        let discovery_path = args.get(1).expect("PRICE discovery answers");
        let companion_path = args.get(2).expect("PRICE companion answers");
        let mut discovery = price_rows(load(Path::new(discovery_path), "PRICE", true));
        let companion = price_rows(load(Path::new(companion_path), "PRICE", true));
        assert_eq!(discovery.len(), 528);
        assert_eq!(companion.len(), 72);
        discovery.extend(companion);
        race_price_companion(&discovery);
        return;
    }
    if matches!(
        args.get(3).map(String::as_str),
        Some("--focused" | "--associations")
    ) {
        let price_path = args.get(1).expect("focused PRICE answers");
        let duration_path = args.get(2).expect("focused DURATION answers");
        let discovery_price = price_rows(load(Path::new(price_path), "PRICE", true));
        let discovery_duration = duration_rows(load(Path::new(duration_path), "DURATION", true));
        assert_eq!(discovery_price.len(), 528);
        assert_eq!(discovery_duration.len(), 264);
        if args.get(3).map(String::as_str) == Some("--focused") {
            race_factorized(&discovery_price, &discovery_duration);
        }
        race_associations(&discovery_price);
        return;
    }
    let old_price = price_rows(load(Path::new(OLD_PRICE), "PRICE", false));
    let old_duration = duration_rows(load(Path::new(OLD_DURATION), "DURATION", false));
    assert_eq!(old_price.len(), 7328);
    assert_eq!(old_duration.len(), 2544);
    race(
        "historical build-20131 identification bank",
        &old_price,
        &old_duration,
    );

    match (args.get(1), args.get(2)) {
        (Some(price_path), Some(duration_path)) => {
            let discovery_price = price_rows(load(Path::new(price_path), "PRICE", true));
            let discovery_duration =
                duration_rows(load(Path::new(duration_path), "DURATION", true));
            assert_eq!(discovery_price.len(), 528);
            assert_eq!(discovery_duration.len(), 264);
            race(
                "fresh build-20228 discovery",
                &discovery_price,
                &discovery_duration,
            );
            race_retained(&discovery_price, &discovery_duration);
            race_factorized(&discovery_price, &discovery_duration);
            race_associations(&discovery_price);
        }
        (None, None) => {}
        _ => panic!("pass both discovery answer files: <PRICE> <DURATION>"),
    }
}
