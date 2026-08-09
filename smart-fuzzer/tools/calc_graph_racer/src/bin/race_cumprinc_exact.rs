//! W109 G6-07 CUMPRINC clean-room graph racer.
//!
//! The primary corpus is the frozen W108 live-Excel capture.  It contains
//! forty-five CUMPRINC ranges, the matching published PMT for every loan, and
//! seventeen single-period CUMPRINC/PPMT/IPMT decompositions.  Using the
//! captured PMT as an input deliberately removes the still-open PMT helper
//! from the recurrence and fold search.

use oxfunc_core::excel_numeric::research as rx;
use oxfunc_core::functions::cumulative_finance_family::cumprinc_kernel;
use rx::Ext80;
use serde::Deserialize;
use serde_json::Value;
use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};

const CW: u16 = rx::CW_PC64_RN;

#[derive(Clone, Copy)]
struct X(Ext80);

impl X {
    fn f(v: f64) -> Self {
        Self(rx::ext_from_f64(v))
    }
    fn add(self, rhs: Self) -> Self {
        Self(rx::ext_add(&self.0, &rhs.0, CW))
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

#[derive(Clone, Debug)]
struct CumRow {
    id: String,
    rate: f64,
    nper: i32,
    pv: f64,
    start: i32,
    end: i32,
    timing: i32,
    want: u64,
}

#[derive(Clone, Debug, Deserialize)]
struct AmortRow {
    rate: f64,
    nper: f64,
    pv: f64,
    per: i32,
    pmt: String,
    cumprinc: String,
}

#[derive(Debug, Deserialize)]
struct AnswerSet {
    function: String,
    witnesses: Vec<AnswerWitness>,
}

#[derive(Debug, Deserialize)]
struct AnswerWitness {
    id: String,
    args: Vec<String>,
    expected_bits: String,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct PmtKey(u64, u64, u64, u64);

fn key(rate: f64, nper: f64, pv: f64, timing: f64) -> PmtKey {
    PmtKey(
        rate.to_bits(),
        nper.to_bits(),
        pv.to_bits(),
        timing.to_bits(),
    )
}

fn bits_hex(s: &str) -> u64 {
    u64::from_str_radix(s.trim_start_matches("0x"), 16).expect("hex bits")
}

fn ordered(v: u64) -> u64 {
    if v & (1 << 63) != 0 {
        !v
    } else {
        v | (1 << 63)
    }
}

fn ulp(got: u64, want: u64) -> u64 {
    ordered(got).abs_diff(ordered(want))
}

fn load_jsonl(path: &Path) -> Vec<Value> {
    std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str(line).expect("jsonl row"))
        .collect()
}

fn load_w108(path: &Path) -> (Vec<CumRow>, HashMap<PmtKey, f64>) {
    let mut rows = Vec::new();
    let mut payments = HashMap::new();
    for v in load_jsonl(path) {
        let function = v["fn"].as_str().expect("fn");
        let args: Vec<f64> = v["args"]
            .as_array()
            .expect("args")
            .iter()
            .map(|x| x.as_f64().expect("numeric arg"))
            .collect();
        let out = bits_hex(v["out_bits"].as_str().expect("out_bits"));
        match function {
            "PMT" if args.len() == 5 && args[3] == 0.0 => {
                payments.insert(key(args[0], args[1], args[2], args[4]), f64::from_bits(out));
            }
            "CUMPRINC" if args.len() == 6 => rows.push(CumRow {
                id: v["id"].as_str().expect("id").to_owned(),
                rate: args[0],
                nper: args[1] as i32,
                pv: args[2],
                start: args[3] as i32,
                end: args[4] as i32,
                timing: args[5] as i32,
                want: out,
            }),
            _ => {}
        }
    }
    (rows, payments)
}

fn scalar_answer_args(witness: &AnswerWitness, arity: usize) -> Vec<f64> {
    assert_eq!(witness.args.len(), arity, "{} arity", witness.id);
    witness
        .args
        .iter()
        .map(|arg| f64::from_bits(bits_hex(arg)))
        .collect()
}

fn load_cumprinc_answers(path: &Path) -> Vec<CumRow> {
    let set: AnswerSet = serde_json::from_str(
        &std::fs::read_to_string(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display())),
    )
    .expect("parse CUMPRINC answer set");
    assert_eq!(set.function, "CUMPRINC");
    set.witnesses
        .iter()
        .map(|witness| {
            let args = scalar_answer_args(witness, 6);
            assert_eq!(args[1].fract(), 0.0, "{} integral nper", witness.id);
            assert_eq!(args[3].fract(), 0.0, "{} integral start", witness.id);
            assert_eq!(args[4].fract(), 0.0, "{} integral end", witness.id);
            assert!(matches!(args[5], 0.0 | 1.0), "{} timing", witness.id);
            CumRow {
                id: witness.id.clone(),
                rate: args[0],
                nper: args[1] as i32,
                pv: args[2],
                start: args[3] as i32,
                end: args[4] as i32,
                timing: args[5] as i32,
                want: bits_hex(&witness.expected_bits),
            }
        })
        .collect()
}

fn load_pmt_answers(path: &Path) -> HashMap<PmtKey, f64> {
    let set: AnswerSet = serde_json::from_str(
        &std::fs::read_to_string(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display())),
    )
    .expect("parse PMT answer set");
    assert_eq!(set.function, "PMT");
    set.witnesses
        .iter()
        .map(|witness| {
            let args = scalar_answer_args(witness, 5);
            assert_eq!(args[3], 0.0, "{} zero FV", witness.id);
            (
                key(args[0], args[1], args[2], args[4]),
                f64::from_bits(bits_hex(&witness.expected_bits)),
            )
        })
        .collect()
}

#[derive(Clone, Copy, Debug)]
enum Arith {
    Strict,
    X87Period,
    X87StoredMul,
}

#[derive(Clone, Copy, Debug)]
enum State {
    BalanceAddPrincipal,
    BalanceMulAdd,
    PvPlusCumulative,
    GeometricPrincipal,
}

#[derive(Clone, Copy, Debug)]
enum Fold {
    Strict,
    X87Continuous,
}

#[derive(Clone, Copy, Debug)]
enum EmSource {
    Internal,
    Portable,
    ExpMinusOne,
}

#[derive(Clone, Copy, Debug)]
enum FirstGraph {
    PvDivEmMulVMulR,
    PvMulVDivEmMulR,
    PvMulRDivEmMulV,
    PvMulRMulVDivEm,
    PvDivEmMulVR,
    PvMulVMulRDivEm,
}

#[derive(Clone, Copy, Debug)]
enum GrowthGraph {
    RepeatedMultiply,
    PowChain,
    PowPositive,
    ExpLog1p,
    Expm1Log1p,
}

#[derive(Clone, Copy, Debug)]
struct DiscountCandidate {
    arith: Arith,
    em_source: EmSource,
    first: FirstGraph,
    growth: GrowthGraph,
    fold: Fold,
}

#[derive(Clone, Copy, Debug)]
enum PowerGraph {
    StrictPowi,
    X87StoredPowi,
    PowChain,
    PowPositive,
    ExpLog1p,
    Expm1Log1p,
}

#[derive(Clone, Copy, Debug)]
enum DirectCombine {
    RatioThenMul,
    MulThenDiv,
    DivThenMul,
}

#[derive(Clone, Copy, Debug)]
struct DirectCandidate {
    arith: Arith,
    power: PowerGraph,
    combine: DirectCombine,
}

#[derive(Clone, Copy, Debug)]
struct StableBoundaryCandidate {
    arith: Arith,
    delta: EmSource,
    growth: PowerGraph,
    combine: DirectCombine,
}

#[derive(Clone, Copy, Debug)]
enum ContinuousState {
    BalanceAddPrincipal,
    BalanceMulAdd,
    PvPlusCumulative,
}

#[derive(Clone, Copy, Debug)]
enum QuotientGraph {
    Divide,
    ReciprocalMultiply,
}

#[derive(Clone, Copy, Debug)]
struct ClosedFvCandidate {
    arith: Arith,
    factor: PowerGraph,
    delta: EmSource,
    quotient: QuotientGraph,
    fold: Fold,
}

#[derive(Clone, Copy, Debug)]
struct Candidate {
    arith: Arith,
    state: State,
    fold: Fold,
}

fn principal(arith: Arith, payment: f64, balance: f64, rate: f64) -> f64 {
    match arith {
        Arith::Strict => payment + balance * rate,
        // One legacy expression tree: balance*rate remains extended through
        // the addition, then the principal is stored to binary64.
        Arith::X87Period => X::f(payment).add(X::f(balance).mul(X::f(rate))).store(),
        // Assignment/store after the product, followed by an x87 add/store.
        Arith::X87StoredMul => {
            let interest_mag = X::f(balance).mul(X::f(rate)).store();
            X::f(payment).add(X::f(interest_mag)).store()
        }
    }
}

fn add(arith: Arith, lhs: f64, rhs: f64) -> f64 {
    match arith {
        Arith::Strict => lhs + rhs,
        Arith::X87Period | Arith::X87StoredMul => X::f(lhs).add(X::f(rhs)).store(),
    }
}

fn mul_add(arith: Arith, balance: f64, rate: f64, payment: f64) -> f64 {
    match arith {
        Arith::Strict => balance * (1.0 + rate) + payment,
        Arith::X87Period => X::f(balance)
            .mul(X::f(1.0).add(X::f(rate)))
            .add(X::f(payment))
            .store(),
        Arith::X87StoredMul => {
            let factor = X::f(1.0).add(X::f(rate)).store();
            let grown = X::f(balance).mul(X::f(factor)).store();
            X::f(grown).add(X::f(payment)).store()
        }
    }
}

fn op_mul(arith: Arith, lhs: f64, rhs: f64) -> f64 {
    match arith {
        Arith::Strict => lhs * rhs,
        Arith::X87Period | Arith::X87StoredMul => X::f(lhs).mul(X::f(rhs)).store(),
    }
}

fn op_div(arith: Arith, lhs: f64, rhs: f64) -> f64 {
    match arith {
        Arith::Strict => lhs / rhs,
        Arith::X87Period | Arith::X87StoredMul => X::f(lhs).div(X::f(rhs)).store(),
    }
}

fn discount_first(c: DiscountCandidate, row: &CumRow) -> f64 {
    let tau = -(row.nper as f64 * rx::excel_log1p(row.rate));
    let em = match c.em_source {
        EmSource::Internal => rx::excel_expm1_internal(tau),
        EmSource::Portable => rx::excel_expm1(tau),
        EmSource::ExpMinusOne => rx::excel_exp(tau) - 1.0,
    };
    let v = 1.0 + em;
    if matches!(c.arith, Arith::X87Period) {
        let (pv, r, em, v) = (X::f(row.pv), X::f(row.rate), X::f(em), X::f(v));
        return match c.first {
            FirstGraph::PvDivEmMulVMulR => pv.div(em).mul(v).mul(r),
            FirstGraph::PvMulVDivEmMulR => pv.mul(v).div(em).mul(r),
            FirstGraph::PvMulRDivEmMulV => pv.mul(r).div(em).mul(v),
            FirstGraph::PvMulRMulVDivEm => pv.mul(r).mul(v).div(em),
            FirstGraph::PvDivEmMulVR => pv.div(em).mul(v.mul(r)),
            FirstGraph::PvMulVMulRDivEm => pv.mul(v).mul(r).div(em),
        }
        .store();
    }
    match c.first {
        FirstGraph::PvDivEmMulVMulR => op_mul(
            c.arith,
            op_mul(c.arith, op_div(c.arith, row.pv, em), v),
            row.rate,
        ),
        FirstGraph::PvMulVDivEmMulR => op_mul(
            c.arith,
            op_div(c.arith, op_mul(c.arith, row.pv, v), em),
            row.rate,
        ),
        FirstGraph::PvMulRDivEmMulV => op_mul(
            c.arith,
            op_div(c.arith, op_mul(c.arith, row.pv, row.rate), em),
            v,
        ),
        FirstGraph::PvMulRMulVDivEm => op_div(
            c.arith,
            op_mul(c.arith, op_mul(c.arith, row.pv, row.rate), v),
            em,
        ),
        FirstGraph::PvDivEmMulVR => op_mul(
            c.arith,
            op_div(c.arith, row.pv, em),
            op_mul(c.arith, v, row.rate),
        ),
        FirstGraph::PvMulVMulRDivEm => op_div(
            c.arith,
            op_mul(c.arith, op_mul(c.arith, row.pv, v), row.rate),
            em,
        ),
    }
}

fn growth_factor(c: DiscountCandidate, row: &CumRow, exponent: i32) -> f64 {
    if exponent == 0 {
        return 1.0;
    }
    match c.growth {
        GrowthGraph::RepeatedMultiply => {
            let factor = add(c.arith, 1.0, row.rate);
            let mut out = 1.0;
            for _ in 0..exponent {
                out = op_mul(c.arith, out, factor);
            }
            out
        }
        GrowthGraph::PowChain => rx::excel_pow_chain(1.0 + row.rate, exponent as f64),
        GrowthGraph::PowPositive => rx::excel_pow_positive(1.0 + row.rate, exponent as f64),
        GrowthGraph::ExpLog1p => rx::excel_exp(exponent as f64 * rx::excel_log1p(row.rate)),
        GrowthGraph::Expm1Log1p => {
            1.0 + rx::excel_expm1_internal(exponent as f64 * rx::excel_log1p(row.rate))
        }
    }
}

fn discount_model(c: DiscountCandidate, row: &CumRow, published_payment: f64) -> f64 {
    let end_first = discount_first(c, row);
    let mut strict_total = 0.0;
    let mut x_total = X::f(0.0);
    for period in row.start..=row.end {
        let p = if row.timing == 0 {
            op_mul(c.arith, end_first, growth_factor(c, row, period - 1))
        } else if period == 1 {
            published_payment
        } else {
            op_mul(c.arith, end_first, growth_factor(c, row, period - 2))
        };
        match c.fold {
            Fold::Strict => strict_total += p,
            Fold::X87Continuous => x_total = x_total.add(X::f(p)),
        }
    }
    match c.fold {
        Fold::Strict => strict_total,
        Fold::X87Continuous => x_total.store(),
    }
}

fn powi_strict(base: f64, exponent: i32) -> f64 {
    let mut result = 1.0;
    let mut factor = base;
    let mut n = exponent;
    while n > 0 {
        if n & 1 != 0 {
            result *= factor;
        }
        n >>= 1;
        if n > 0 {
            factor *= factor;
        }
    }
    result
}

fn powi_x87_stored(base: f64, exponent: i32) -> f64 {
    let mut result = 1.0;
    let mut factor = base;
    let mut n = exponent;
    while n > 0 {
        if n & 1 != 0 {
            result = X::f(result).mul(X::f(factor)).store();
        }
        n >>= 1;
        if n > 0 {
            factor = X::f(factor).mul(X::f(factor)).store();
        }
    }
    result
}

fn direct_power(c: DirectCandidate, row: &CumRow, exponent: i32) -> f64 {
    if exponent == 0 {
        return 1.0;
    }
    let base = add(c.arith, 1.0, row.rate);
    match c.power {
        PowerGraph::StrictPowi => powi_strict(base, exponent),
        PowerGraph::X87StoredPowi => powi_x87_stored(base, exponent),
        PowerGraph::PowChain => rx::excel_pow_chain(base, exponent as f64),
        PowerGraph::PowPositive => rx::excel_pow_positive(base, exponent as f64),
        PowerGraph::ExpLog1p => rx::excel_exp(exponent as f64 * rx::excel_log1p(row.rate)),
        PowerGraph::Expm1Log1p => {
            1.0 + rx::excel_expm1_internal(exponent as f64 * rx::excel_log1p(row.rate))
        }
    }
}

fn direct_combine(c: DirectCandidate, pv: f64, numerator: f64, denominator: f64) -> f64 {
    if matches!(c.arith, Arith::X87Period) {
        return match c.combine {
            DirectCombine::RatioThenMul => X::f(0.0)
                .add(X::f(-pv))
                .mul(X::f(numerator).div(X::f(denominator))),
            DirectCombine::MulThenDiv => X::f(-pv).mul(X::f(numerator)).div(X::f(denominator)),
            DirectCombine::DivThenMul => X::f(-pv).div(X::f(denominator)).mul(X::f(numerator)),
        }
        .store();
    }
    match c.combine {
        DirectCombine::RatioThenMul => {
            op_mul(c.arith, -pv, op_div(c.arith, numerator, denominator))
        }
        DirectCombine::MulThenDiv => op_div(c.arith, op_mul(c.arith, -pv, numerator), denominator),
        DirectCombine::DivThenMul => op_mul(c.arith, op_div(c.arith, -pv, denominator), numerator),
    }
}

fn direct_model(c: DirectCandidate, row: &CumRow) -> f64 {
    let fnper = direct_power(c, row, row.nper);
    let denominator = add(c.arith, fnper, -1.0);
    if row.timing == 0 {
        let fend = direct_power(c, row, row.end);
        let fbefore = direct_power(c, row, row.start - 1);
        let numerator = add(c.arith, fend, -fbefore);
        return direct_combine(c, row.pv, numerator, denominator);
    }

    // Annuity-due principal schedule: period 1 is the beginning payment;
    // periods k>=2 are the ordinary first principal grown by F_(k-2).
    if row.start >= 2 {
        let fend = direct_power(c, row, row.end - 1);
        let fbefore = direct_power(c, row, row.start - 2);
        let numerator = add(c.arith, fend, -fbefore);
        return direct_combine(c, row.pv, numerator, denominator);
    }
    let fn_minus_one = direct_power(c, row, row.nper - 1);
    let payment_numerator = op_mul(c.arith, row.rate, fn_minus_one);
    let payment = direct_combine(c, row.pv, payment_numerator, denominator);
    if row.end == 1 {
        return payment;
    }
    let tail_numerator = add(c.arith, direct_power(c, row, row.end - 1), -1.0);
    add(
        c.arith,
        payment,
        direct_combine(c, row.pv, tail_numerator, denominator),
    )
}

fn stable_delta(c: StableBoundaryCandidate, row: &CumRow, exponent: i32) -> f64 {
    if exponent == 0 {
        return 0.0;
    }
    let t = op_mul(c.arith, exponent as f64, rx::excel_log1p(row.rate));
    match c.delta {
        EmSource::Internal => rx::excel_expm1_internal(t),
        EmSource::Portable => rx::excel_expm1(t),
        EmSource::ExpMinusOne => rx::excel_exp(t) - 1.0,
    }
}

fn stable_growth(c: StableBoundaryCandidate, row: &CumRow, exponent: i32) -> f64 {
    if exponent == 0 {
        return 1.0;
    }
    let base = add(c.arith, 1.0, row.rate);
    match c.growth {
        PowerGraph::StrictPowi => powi_strict(base, exponent),
        PowerGraph::X87StoredPowi => powi_x87_stored(base, exponent),
        PowerGraph::PowChain => rx::excel_pow_chain(base, exponent as f64),
        PowerGraph::PowPositive => rx::excel_pow_positive(base, exponent as f64),
        PowerGraph::ExpLog1p => {
            rx::excel_exp(op_mul(c.arith, exponent as f64, rx::excel_log1p(row.rate)))
        }
        PowerGraph::Expm1Log1p => {
            1.0 + rx::excel_expm1_internal(op_mul(
                c.arith,
                exponent as f64,
                rx::excel_log1p(row.rate),
            ))
        }
    }
}

fn stable_combine(c: StableBoundaryCandidate, pv: f64, numerator: f64, denominator: f64) -> f64 {
    direct_combine(
        DirectCandidate {
            arith: c.arith,
            power: c.growth,
            combine: c.combine,
        },
        pv,
        numerator,
        denominator,
    )
}

fn stable_boundary_model(c: StableBoundaryCandidate, row: &CumRow) -> f64 {
    let denominator = stable_delta(c, row, row.nper);
    let count = row.end - row.start + 1;
    let numerator = if row.timing == 0 {
        op_mul(
            c.arith,
            stable_growth(c, row, row.start - 1),
            stable_delta(c, row, count),
        )
    } else if row.start >= 2 {
        op_mul(
            c.arith,
            stable_growth(c, row, row.start - 2),
            stable_delta(c, row, count),
        )
    } else {
        let payment_term = op_mul(c.arith, row.rate, stable_growth(c, row, row.nper - 1));
        add(c.arith, payment_term, stable_delta(c, row, row.end - 1))
    };
    stable_combine(c, row.pv, numerator, denominator)
}

fn continuous_recurrence_model(state: ContinuousState, row: &CumRow, payment: f64) -> f64 {
    let pv = X::f(row.pv);
    let rate = X::f(row.rate);
    let payment = X::f(payment);
    let one_plus_rate = X::f(1.0).add(rate);
    let mut balance = pv;
    let mut cumulative = X::f(0.0);
    let mut total = X::f(0.0);
    for period in 1..=row.end {
        let p = if row.timing == 1 && period == 1 {
            payment
        } else {
            let entering = match state {
                ContinuousState::PvPlusCumulative => pv.add(cumulative),
                ContinuousState::BalanceAddPrincipal | ContinuousState::BalanceMulAdd => balance,
            };
            payment.add(entering.mul(rate))
        };
        cumulative = cumulative.add(p);
        match state {
            ContinuousState::BalanceAddPrincipal => balance = balance.add(p),
            ContinuousState::BalanceMulAdd => balance = balance.mul(one_plus_rate).add(payment),
            ContinuousState::PvPlusCumulative => {}
        }
        if period >= row.start {
            total = total.add(p);
        }
    }
    total.store()
}

fn closed_factor(c: ClosedFvCandidate, row: &CumRow, exponent: i32) -> f64 {
    stable_growth(
        StableBoundaryCandidate {
            arith: c.arith,
            delta: c.delta,
            growth: c.factor,
            combine: DirectCombine::RatioThenMul,
        },
        row,
        exponent,
    )
}

fn closed_delta(c: ClosedFvCandidate, row: &CumRow, exponent: i32) -> f64 {
    stable_delta(
        StableBoundaryCandidate {
            arith: c.arith,
            delta: c.delta,
            growth: c.factor,
            combine: DirectCombine::RatioThenMul,
        },
        row,
        exponent,
    )
}

fn closed_fv_principal(c: ClosedFvCandidate, row: &CumRow, payment: f64, period: i32) -> f64 {
    if row.timing == 1 && period == 1 {
        return payment;
    }
    let exponent = if row.timing == 1 {
        period - 2
    } else {
        period - 1
    };
    let factor = closed_factor(c, row, exponent);
    let delta = closed_delta(c, row, exponent);
    let quotient = match c.quotient {
        QuotientGraph::Divide => op_div(c.arith, delta, row.rate),
        QuotientGraph::ReciprocalMultiply => op_mul(c.arith, delta, op_div(c.arith, 1.0, row.rate)),
    };
    let pv_term = op_mul(c.arith, row.pv, factor);
    let timing_factor = if row.timing == 1 {
        add(c.arith, 1.0, row.rate)
    } else {
        1.0
    };
    let payment_term = op_mul(c.arith, op_mul(c.arith, payment, timing_factor), quotient);
    let rip = -(add(c.arith, pv_term, payment_term));
    let interest = op_mul(c.arith, rip, row.rate);
    let mut principal = add(c.arith, payment, -interest);
    if row.timing == 1 {
        // Excel IPMT's type-one path evaluates FV(m)-payment before applying
        // rate; retain this explicit boundary instead of algebraic collapse.
        let rip_due = add(c.arith, rip, -payment);
        let interest_due = op_mul(c.arith, rip_due, row.rate);
        principal = add(c.arith, payment, -interest_due);
    }
    principal
}

fn closed_fv_model(c: ClosedFvCandidate, row: &CumRow, payment: f64) -> f64 {
    let mut strict_total = 0.0;
    let mut x_total = X::f(0.0);
    for period in row.start..=row.end {
        let principal = closed_fv_principal(c, row, payment, period);
        match c.fold {
            Fold::Strict => strict_total += principal,
            Fold::X87Continuous => x_total = x_total.add(X::f(principal)),
        }
    }
    match c.fold {
        Fold::Strict => strict_total,
        Fold::X87Continuous => x_total.store(),
    }
}

fn model(c: Candidate, row: &CumRow, payment: f64) -> f64 {
    let mut balance = row.pv;
    let mut cumulative = 0.0;
    let mut total = 0.0;
    let mut total_x = X::f(0.0);
    let mut geometric = if row.timing == 1 {
        payment
    } else {
        principal(c.arith, payment, row.pv, row.rate)
    };

    for period in 1..=row.end {
        let p = if row.timing == 1 && period == 1 {
            payment
        } else {
            match c.state {
                State::GeometricPrincipal => {
                    if period == 1 {
                        geometric
                    } else {
                        geometric = match c.arith {
                            Arith::Strict => geometric * (1.0 + row.rate),
                            Arith::X87Period => {
                                X::f(geometric).mul(X::f(1.0).add(X::f(row.rate))).store()
                            }
                            Arith::X87StoredMul => {
                                let factor = X::f(1.0).add(X::f(row.rate)).store();
                                X::f(geometric).mul(X::f(factor)).store()
                            }
                        };
                        geometric
                    }
                }
                State::PvPlusCumulative => {
                    let entering = add(c.arith, row.pv, cumulative);
                    principal(c.arith, payment, entering, row.rate)
                }
                State::BalanceAddPrincipal | State::BalanceMulAdd => {
                    principal(c.arith, payment, balance, row.rate)
                }
            }
        };

        cumulative = add(c.arith, cumulative, p);
        match c.state {
            State::BalanceAddPrincipal => balance = add(c.arith, balance, p),
            State::BalanceMulAdd => balance = mul_add(c.arith, balance, row.rate, payment),
            State::PvPlusCumulative | State::GeometricPrincipal => {}
        }

        if period >= row.start {
            match c.fold {
                Fold::Strict => total += p,
                Fold::X87Continuous => total_x = total_x.add(X::f(p)),
            }
        }
    }
    match c.fold {
        Fold::Strict => total,
        Fold::X87Continuous => total_x.store(),
    }
}

#[derive(Default)]
struct Score {
    exact: usize,
    total: usize,
    max_ulp: u64,
    sum_ulp: u128,
    hist: BTreeMap<u64, usize>,
}

impl Score {
    fn add(&mut self, got: u64, want: u64) {
        let d = ulp(got, want);
        self.total += 1;
        self.exact += usize::from(d == 0);
        self.max_ulp = self.max_ulp.max(d);
        self.sum_ulp += u128::from(d);
        *self.hist.entry(d.min(16)).or_default() += 1;
    }
}

fn score_rows(c: Candidate, rows: &[CumRow], payments: &HashMap<PmtKey, f64>) -> Score {
    let mut score = Score::default();
    for row in rows {
        let payment = payments[&key(row.rate, row.nper as f64, row.pv, row.timing as f64)];
        score.add(model(c, row, payment).to_bits(), row.want);
    }
    score
}

fn score_amort(c: Candidate, rows: &[AmortRow]) -> Score {
    let mut score = Score::default();
    for a in rows {
        let row = CumRow {
            id: format!("amort-{}", a.per),
            rate: a.rate,
            nper: a.nper as i32,
            pv: a.pv,
            start: a.per,
            end: a.per,
            timing: 0,
            want: bits_hex(&a.cumprinc),
        };
        score.add(
            model(c, &row, f64::from_bits(bits_hex(&a.pmt))).to_bits(),
            row.want,
        );
    }
    score
}

fn score_key(
    c: Candidate,
    wanted_key: PmtKey,
    payment: f64,
    rows: &[CumRow],
    amort: &[AmortRow],
) -> Score {
    let mut score = Score::default();
    for row in rows {
        if key(row.rate, row.nper as f64, row.pv, row.timing as f64) == wanted_key {
            score.add(model(c, row, payment).to_bits(), row.want);
        }
    }
    for a in amort {
        if key(a.rate, a.nper, a.pv, 0.0) != wanted_key {
            continue;
        }
        let row = CumRow {
            id: format!("amort-{}", a.per),
            rate: a.rate,
            nper: a.nper as i32,
            pv: a.pv,
            start: a.per,
            end: a.per,
            timing: 0,
            want: bits_hex(&a.cumprinc),
        };
        score.add(model(c, &row, payment).to_bits(), row.want);
    }
    score
}

/// Recover the stored payment as a nuisance parameter, independently for each
/// loan.  Searching the adjacent 257 binary64 values is answer-blind with
/// respect to the graph: it only removes the unresolved upstream PMT low word.
fn optimized_payment_score(
    c: Candidate,
    rows: &[CumRow],
    payments: &HashMap<PmtKey, f64>,
    amort: &[AmortRow],
) -> (Score, BTreeMap<String, i64>) {
    let mut used = Vec::new();
    for row in rows {
        let k = key(row.rate, row.nper as f64, row.pv, row.timing as f64);
        if !used.contains(&k) {
            used.push(k);
        }
    }
    let mut aggregate = Score::default();
    let mut offsets = BTreeMap::new();
    for k in used {
        let published_bits = payments[&k].to_bits();
        let mut best: Option<(Score, i64)> = None;
        for offset in -128i64..=128 {
            let trial_bits = (i128::from(published_bits) + i128::from(offset)) as u64;
            let s = score_key(c, k, f64::from_bits(trial_bits), rows, amort);
            let rank = (
                std::cmp::Reverse(s.exact),
                s.max_ulp,
                s.sum_ulp,
                offset.abs(),
            );
            if best.as_ref().is_none_or(|(b, bo)| {
                rank < (std::cmp::Reverse(b.exact), b.max_ulp, b.sum_ulp, bo.abs())
            }) {
                best = Some((s, offset));
            }
        }
        let (s, offset) = best.expect("payment search");
        aggregate.exact += s.exact;
        aggregate.total += s.total;
        aggregate.max_ulp = aggregate.max_ulp.max(s.max_ulp);
        aggregate.sum_ulp += s.sum_ulp;
        for (d, n) in s.hist {
            *aggregate.hist.entry(d).or_default() += n;
        }
        offsets.insert(
            format!(
                "r={:016x}/n={}/pv={:016x}/type={}",
                k.0,
                f64::from_bits(k.1),
                k.2,
                f64::from_bits(k.3)
            ),
            offset,
        );
    }
    (aggregate, offsets)
}

fn score_discount(
    c: DiscountCandidate,
    rows: &[CumRow],
    payments: &HashMap<PmtKey, f64>,
    amort: &[AmortRow],
) -> Score {
    let mut score = Score::default();
    for row in rows {
        let payment = payments[&key(row.rate, row.nper as f64, row.pv, row.timing as f64)];
        score.add(discount_model(c, row, payment).to_bits(), row.want);
    }
    for a in amort {
        let row = CumRow {
            id: format!("amort-{}", a.per),
            rate: a.rate,
            nper: a.nper as i32,
            pv: a.pv,
            start: a.per,
            end: a.per,
            timing: 0,
            want: bits_hex(&a.cumprinc),
        };
        score.add(
            discount_model(c, &row, f64::from_bits(bits_hex(&a.pmt))).to_bits(),
            row.want,
        );
    }
    score
}

fn score_discount_timing(
    c: DiscountCandidate,
    rows: &[CumRow],
    payments: &HashMap<PmtKey, f64>,
    amort: &[AmortRow],
    timing: i32,
) -> Score {
    let mut score = Score::default();
    for row in rows.iter().filter(|row| row.timing == timing) {
        let payment = payments[&key(row.rate, row.nper as f64, row.pv, row.timing as f64)];
        score.add(discount_model(c, row, payment).to_bits(), row.want);
    }
    if timing == 0 {
        for a in amort {
            let row = CumRow {
                id: format!("amort-{}", a.per),
                rate: a.rate,
                nper: a.nper as i32,
                pv: a.pv,
                start: a.per,
                end: a.per,
                timing: 0,
                want: bits_hex(&a.cumprinc),
            };
            score.add(
                discount_model(c, &row, f64::from_bits(bits_hex(&a.pmt))).to_bits(),
                row.want,
            );
        }
    }
    score
}

fn score_direct(c: DirectCandidate, rows: &[CumRow], amort: &[AmortRow]) -> Score {
    let mut score = Score::default();
    for row in rows {
        score.add(direct_model(c, row).to_bits(), row.want);
    }
    for a in amort {
        let row = CumRow {
            id: format!("amort-{}", a.per),
            rate: a.rate,
            nper: a.nper as i32,
            pv: a.pv,
            start: a.per,
            end: a.per,
            timing: 0,
            want: bits_hex(&a.cumprinc),
        };
        score.add(direct_model(c, &row).to_bits(), row.want);
    }
    score
}

fn score_stable_boundary(c: StableBoundaryCandidate, rows: &[CumRow], amort: &[AmortRow]) -> Score {
    let mut score = Score::default();
    for row in rows {
        score.add(stable_boundary_model(c, row).to_bits(), row.want);
    }
    for a in amort {
        let row = CumRow {
            id: format!("amort-{}", a.per),
            rate: a.rate,
            nper: a.nper as i32,
            pv: a.pv,
            start: a.per,
            end: a.per,
            timing: 0,
            want: bits_hex(&a.cumprinc),
        };
        score.add(stable_boundary_model(c, &row).to_bits(), row.want);
    }
    score
}

fn score_continuous_rows(
    state: ContinuousState,
    rows: &[CumRow],
    payments: &HashMap<PmtKey, f64>,
) -> Score {
    let mut score = Score::default();
    for row in rows {
        let payment = payments[&key(row.rate, row.nper as f64, row.pv, row.timing as f64)];
        score.add(
            continuous_recurrence_model(state, row, payment).to_bits(),
            row.want,
        );
    }
    score
}

fn score_closed_rows(
    c: ClosedFvCandidate,
    rows: &[CumRow],
    payments: &HashMap<PmtKey, f64>,
) -> Score {
    let mut score = Score::default();
    for row in rows {
        let payment = payments[&key(row.rate, row.nper as f64, row.pv, row.timing as f64)];
        score.add(closed_fv_model(c, row, payment).to_bits(), row.want);
    }
    score
}

fn print_score(label: &str, score: &Score) {
    println!(
        "{label}: {}/{} exact max={} sum={} hist={:?}",
        score.exact, score.total, score.max_ulp, score.sum_ulp, score.hist
    );
}

fn discovery_invariants(rows: &[CumRow], payments: &HashMap<PmtKey, f64>) {
    let by_id = rows
        .iter()
        .map(|row| (row.id.as_str(), row))
        .collect::<HashMap<_, _>>();

    let mut full = Score::default();
    let mut full_partition = Score::default();
    let mut prefix_partition = Score::default();
    let mut scale_half = Score::default();
    let mut scale_double = Score::default();
    let mut due_first_payment = Score::default();

    for row in rows {
        if row.id.contains("-full-") {
            full.add((-row.pv).to_bits(), row.want);
            let prefix_id = row.id.replace("-full-", "-prefix_middle-");
            let suffix_id = row.id.replace("-full-", "-suffix_middle-");
            let got = f64::from_bits(by_id[prefix_id.as_str()].want)
                + f64::from_bits(by_id[suffix_id.as_str()].want);
            full_partition.add(got.to_bits(), row.want);
        }
        if row.id.contains("-prefix_middle-") {
            let early_id = row.id.replace("-prefix_middle-", "-prefix_early-");
            let interior_id = row.id.replace("-prefix_middle-", "-interior-");
            let got = f64::from_bits(by_id[early_id.as_str()].want)
                + f64::from_bits(by_id[interior_id.as_str()].want);
            prefix_partition.add(got.to_bits(), row.want);
        }
        if let Some(stem) = row.id.strip_suffix("v00") {
            let half = by_id[format!("{stem}v04").as_str()];
            let double = by_id[format!("{stem}v05").as_str()];
            scale_half.add((f64::from_bits(half.want) * 2.0).to_bits(), row.want);
            scale_double.add((f64::from_bits(row.want) * 2.0).to_bits(), double.want);
        }
        if row.timing == 1 && row.id.contains("-singleton_first-") {
            let payment = payments[&key(row.rate, row.nper as f64, row.pv, row.timing as f64)];
            due_first_payment.add(payment.to_bits(), row.want);
        }
    }

    println!("\ndiscovery metamorphic/decomposition invariants:");
    print_score("full schedule vs -PV", &full);
    print_score("RN(prefix_middle + suffix_middle) vs full", &full_partition);
    print_score(
        "RN(prefix_early + interior) vs prefix_middle",
        &prefix_partition,
    );
    print_score("2 * output(PV/2) vs output(PV)", &scale_half);
    print_score("2 * output(PV) vs output(2*PV)", &scale_double);
    print_score(
        "type1 singleton period1 vs published PMT",
        &due_first_payment,
    );
}

fn score_discovery(rows: &[CumRow], payments: &HashMap<PmtKey, f64>) {
    assert_eq!(rows.len(), 540);
    assert_eq!(payments.len(), 60);
    for row in rows {
        assert!(payments.contains_key(&key(row.rate, row.nper as f64, row.pv, row.timing as f64,)));
    }

    println!(
        "\n=== fresh answer-blind discovery batch: {} CUMPRINC / {} paired PMT ===",
        rows.len(),
        payments.len()
    );
    let mut shipping = Score::default();
    for row in rows {
        let got = cumprinc_kernel(
            row.rate,
            row.nper as f64,
            row.pv,
            row.start as f64,
            row.end as f64,
            row.timing as f64,
        )
        .expect("shipping discovery CUMPRINC")
        .to_bits();
        shipping.add(got, row.want);
    }
    print_score("shipping kernel", &shipping);
    discovery_invariants(rows, payments);

    let mut recurrence = Vec::new();
    for arith in [Arith::Strict, Arith::X87Period, Arith::X87StoredMul] {
        for state in [
            State::BalanceAddPrincipal,
            State::BalanceMulAdd,
            State::PvPlusCumulative,
            State::GeometricPrincipal,
        ] {
            for fold in [Fold::Strict, Fold::X87Continuous] {
                let c = Candidate { arith, state, fold };
                recurrence.push((score_rows(c, rows, payments), c));
            }
        }
    }
    recurrence.sort_by_key(|(s, _)| (std::cmp::Reverse(s.exact), s.max_ulp, s.sum_ulp));
    println!("\npublished-PMT recurrence top 12:");
    for (score, candidate) in recurrence.iter().take(12) {
        print_score(&format!("{candidate:?}"), score);
    }

    let mut discount = Vec::new();
    for arith in [Arith::Strict, Arith::X87Period, Arith::X87StoredMul] {
        for em_source in [
            EmSource::Internal,
            EmSource::Portable,
            EmSource::ExpMinusOne,
        ] {
            for first in [
                FirstGraph::PvDivEmMulVMulR,
                FirstGraph::PvMulVDivEmMulR,
                FirstGraph::PvMulRDivEmMulV,
                FirstGraph::PvMulRMulVDivEm,
                FirstGraph::PvDivEmMulVR,
                FirstGraph::PvMulVMulRDivEm,
            ] {
                for growth in [
                    GrowthGraph::RepeatedMultiply,
                    GrowthGraph::PowChain,
                    GrowthGraph::PowPositive,
                    GrowthGraph::ExpLog1p,
                    GrowthGraph::Expm1Log1p,
                ] {
                    for fold in [Fold::Strict, Fold::X87Continuous] {
                        let c = DiscountCandidate {
                            arith,
                            em_source,
                            first,
                            growth,
                            fold,
                        };
                        discount.push((score_discount(c, rows, payments, &[]), c));
                    }
                }
            }
        }
    }
    discount.sort_by_key(|(s, _)| (std::cmp::Reverse(s.exact), s.max_ulp, s.sum_ulp));
    println!("\npublished-PMT discount/geometric top 12:");
    for (score, candidate) in discount.iter().take(12) {
        print_score(&format!("{candidate:?}"), score);
    }

    let mut direct = Vec::new();
    for arith in [Arith::Strict, Arith::X87Period, Arith::X87StoredMul] {
        for power in [
            PowerGraph::StrictPowi,
            PowerGraph::X87StoredPowi,
            PowerGraph::PowChain,
            PowerGraph::PowPositive,
            PowerGraph::ExpLog1p,
            PowerGraph::Expm1Log1p,
        ] {
            for combine in [
                DirectCombine::RatioThenMul,
                DirectCombine::MulThenDiv,
                DirectCombine::DivThenMul,
            ] {
                let c = DirectCandidate {
                    arith,
                    power,
                    combine,
                };
                direct.push((score_direct(c, rows, &[]), c));
            }
        }
    }
    direct.sort_by_key(|(s, _)| (std::cmp::Reverse(s.exact), s.max_ulp, s.sum_ulp));
    println!("\nPMT-free direct-boundary top 12:");
    for (score, candidate) in direct.iter().take(12) {
        print_score(&format!("{candidate:?}"), score);
    }

    let mut stable = Vec::new();
    for arith in [Arith::Strict, Arith::X87Period, Arith::X87StoredMul] {
        for delta in [
            EmSource::Internal,
            EmSource::Portable,
            EmSource::ExpMinusOne,
        ] {
            for growth in [
                PowerGraph::StrictPowi,
                PowerGraph::X87StoredPowi,
                PowerGraph::PowChain,
                PowerGraph::PowPositive,
                PowerGraph::ExpLog1p,
                PowerGraph::Expm1Log1p,
            ] {
                for combine in [
                    DirectCombine::RatioThenMul,
                    DirectCombine::MulThenDiv,
                    DirectCombine::DivThenMul,
                ] {
                    let c = StableBoundaryCandidate {
                        arith,
                        delta,
                        growth,
                        combine,
                    };
                    stable.push((score_stable_boundary(c, rows, &[]), c));
                }
            }
        }
    }
    stable.sort_by_key(|(s, _)| (std::cmp::Reverse(s.exact), s.max_ulp, s.sum_ulp));
    println!("\nPMT-free stable-boundary top 12:");
    for (score, candidate) in stable.iter().take(12) {
        print_score(&format!("{candidate:?}"), score);
    }

    let mut continuous = [
        ContinuousState::BalanceAddPrincipal,
        ContinuousState::BalanceMulAdd,
        ContinuousState::PvPlusCumulative,
    ]
    .into_iter()
    .map(|state| (score_continuous_rows(state, rows, payments), state))
    .collect::<Vec<_>>();
    continuous.sort_by_key(|(s, _)| (std::cmp::Reverse(s.exact), s.max_ulp, s.sum_ulp));
    println!("\npublished-PMT continuous-x87 recurrence:");
    for (score, candidate) in &continuous {
        print_score(&format!("{candidate:?}"), score);
    }

    let mut closed = Vec::new();
    for arith in [Arith::Strict, Arith::X87Period, Arith::X87StoredMul] {
        for factor in [
            PowerGraph::StrictPowi,
            PowerGraph::X87StoredPowi,
            PowerGraph::PowChain,
            PowerGraph::PowPositive,
            PowerGraph::ExpLog1p,
            PowerGraph::Expm1Log1p,
        ] {
            for delta in [
                EmSource::Internal,
                EmSource::Portable,
                EmSource::ExpMinusOne,
            ] {
                for quotient in [QuotientGraph::Divide, QuotientGraph::ReciprocalMultiply] {
                    for fold in [Fold::Strict, Fold::X87Continuous] {
                        let c = ClosedFvCandidate {
                            arith,
                            factor,
                            delta,
                            quotient,
                            fold,
                        };
                        closed.push((score_closed_rows(c, rows, payments), c));
                    }
                }
            }
        }
    }
    closed.sort_by_key(|(s, _)| (std::cmp::Reverse(s.exact), s.max_ulp, s.sum_ulp));
    println!("\npublished-PMT closed-FV top 12:");
    for (score, candidate) in closed.iter().take(12) {
        print_score(&format!("{candidate:?}"), score);
    }
}

fn continuous_score_key(
    state: ContinuousState,
    wanted_key: PmtKey,
    payment: f64,
    rows: &[CumRow],
    amort: &[AmortRow],
) -> Score {
    let mut score = Score::default();
    for row in rows {
        if key(row.rate, row.nper as f64, row.pv, row.timing as f64) == wanted_key {
            score.add(
                continuous_recurrence_model(state, row, payment).to_bits(),
                row.want,
            );
        }
    }
    for a in amort {
        if key(a.rate, a.nper, a.pv, 0.0) != wanted_key {
            continue;
        }
        let row = CumRow {
            id: format!("amort-{}", a.per),
            rate: a.rate,
            nper: a.nper as i32,
            pv: a.pv,
            start: a.per,
            end: a.per,
            timing: 0,
            want: bits_hex(&a.cumprinc),
        };
        score.add(
            continuous_recurrence_model(state, &row, payment).to_bits(),
            row.want,
        );
    }
    score
}

fn optimized_continuous_score(
    state: ContinuousState,
    rows: &[CumRow],
    payments: &HashMap<PmtKey, f64>,
    amort: &[AmortRow],
) -> (Score, BTreeMap<String, i64>) {
    let mut used = Vec::new();
    for row in rows {
        let k = key(row.rate, row.nper as f64, row.pv, row.timing as f64);
        if !used.contains(&k) {
            used.push(k);
        }
    }
    let mut aggregate = Score::default();
    let mut offsets = BTreeMap::new();
    for k in used {
        let published_bits = payments[&k].to_bits();
        let mut best: Option<(Score, i64)> = None;
        for offset in -128i64..=128 {
            let trial_bits = (i128::from(published_bits) + i128::from(offset)) as u64;
            let s = continuous_score_key(state, k, f64::from_bits(trial_bits), rows, amort);
            let rank = (
                std::cmp::Reverse(s.exact),
                s.max_ulp,
                s.sum_ulp,
                offset.abs(),
            );
            if best.as_ref().is_none_or(|(b, bo)| {
                rank < (std::cmp::Reverse(b.exact), b.max_ulp, b.sum_ulp, bo.abs())
            }) {
                best = Some((s, offset));
            }
        }
        let (s, offset) = best.expect("continuous payment search");
        aggregate.exact += s.exact;
        aggregate.total += s.total;
        aggregate.max_ulp = aggregate.max_ulp.max(s.max_ulp);
        aggregate.sum_ulp += s.sum_ulp;
        offsets.insert(
            format!(
                "r={:016x}/n={}/pv={:016x}/type={}",
                k.0,
                f64::from_bits(k.1),
                k.2,
                f64::from_bits(k.3)
            ),
            offset,
        );
    }
    (aggregate, offsets)
}

fn closed_score_key(
    c: ClosedFvCandidate,
    wanted_key: PmtKey,
    payment: f64,
    rows: &[CumRow],
    amort: &[AmortRow],
) -> Score {
    let mut score = Score::default();
    for row in rows {
        if key(row.rate, row.nper as f64, row.pv, row.timing as f64) == wanted_key {
            score.add(closed_fv_model(c, row, payment).to_bits(), row.want);
        }
    }
    for a in amort {
        if key(a.rate, a.nper, a.pv, 0.0) != wanted_key {
            continue;
        }
        let row = CumRow {
            id: format!("amort-{}", a.per),
            rate: a.rate,
            nper: a.nper as i32,
            pv: a.pv,
            start: a.per,
            end: a.per,
            timing: 0,
            want: bits_hex(&a.cumprinc),
        };
        score.add(closed_fv_model(c, &row, payment).to_bits(), row.want);
    }
    score
}

fn optimized_closed_score(
    c: ClosedFvCandidate,
    rows: &[CumRow],
    payments: &HashMap<PmtKey, f64>,
    amort: &[AmortRow],
) -> (Score, BTreeMap<String, i64>) {
    let mut used = Vec::new();
    for row in rows {
        let k = key(row.rate, row.nper as f64, row.pv, row.timing as f64);
        if !used.contains(&k) {
            used.push(k);
        }
    }
    let mut aggregate = Score::default();
    let mut offsets = BTreeMap::new();
    for k in used {
        let published_bits = payments[&k].to_bits();
        let mut best: Option<(Score, i64)> = None;
        for offset in -128i64..=128 {
            let trial_bits = (i128::from(published_bits) + i128::from(offset)) as u64;
            let s = closed_score_key(c, k, f64::from_bits(trial_bits), rows, amort);
            let rank = (
                std::cmp::Reverse(s.exact),
                s.max_ulp,
                s.sum_ulp,
                offset.abs(),
            );
            if best.as_ref().is_none_or(|(b, bo)| {
                rank < (std::cmp::Reverse(b.exact), b.max_ulp, b.sum_ulp, bo.abs())
            }) {
                best = Some((s, offset));
            }
        }
        let (s, offset) = best.expect("closed payment search");
        aggregate.exact += s.exact;
        aggregate.total += s.total;
        aggregate.max_ulp = aggregate.max_ulp.max(s.max_ulp);
        aggregate.sum_ulp += s.sum_ulp;
        offsets.insert(
            format!(
                "r={:016x}/n={}/pv={:016x}/type={}",
                k.0,
                f64::from_bits(k.1),
                k.2,
                f64::from_bits(k.3)
            ),
            offset,
        );
    }
    (aggregate, offsets)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let base = PathBuf::from(
        args.get(1)
            .map(String::as_str)
            .unwrap_or("../../runs/w108-b2-financial"),
    );
    let (rows, payments) = load_w108(&base.join("excel_out.jsonl"));
    let amort: Vec<AmortRow> = std::fs::read_to_string(base.join("amort.jsonl"))
        .expect("read amort")
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str(line).expect("amort row"))
        .collect();
    let discovery = match (args.get(2), args.get(3)) {
        (Some(cum_path), Some(pmt_path)) => Some((
            load_cumprinc_answers(Path::new(cum_path)),
            load_pmt_answers(Path::new(pmt_path)),
        )),
        (None, None) => None,
        _ => panic!("pass both discovery files: <CUMPRINC answers> <paired PMT answers>"),
    };
    eprintln!(
        "loaded {} CUMPRINC ranges, {} captured PMTs, {} single-period decompositions",
        rows.len(),
        payments.len(),
        amort.len()
    );

    let mut ranked = Vec::new();
    for arith in [Arith::Strict, Arith::X87Period, Arith::X87StoredMul] {
        for state in [
            State::BalanceAddPrincipal,
            State::BalanceMulAdd,
            State::PvPlusCumulative,
            State::GeometricPrincipal,
        ] {
            for fold in [Fold::Strict, Fold::X87Continuous] {
                let c = Candidate { arith, state, fold };
                let frozen = score_rows(c, &rows, &payments);
                let singles = score_amort(c, &amort);
                let (optimized, offsets) = optimized_payment_score(c, &rows, &payments, &amort);
                ranked.push((frozen, singles, optimized, offsets, c));
            }
        }
    }
    ranked.sort_by_key(|(_, _, o, _, _)| (std::cmp::Reverse(o.exact), o.max_ulp, o.sum_ulp));
    println!(
        "candidate                                  published(ranges/singles) optimized  maxULP sumULP"
    );
    for (r, a, o, _, c) in &ranked {
        println!(
            "{:?}/{:?}/{:?}  {:2}/{:2} {:2}/{:2}           {:2}/{:2}     {:>6} {}",
            c.arith,
            c.state,
            c.fold,
            r.exact,
            r.total,
            a.exact,
            a.total,
            o.exact,
            o.total,
            o.max_ulp,
            o.sum_ulp
        );
    }

    let mut discount_ranked = Vec::new();
    for arith in [Arith::Strict, Arith::X87Period, Arith::X87StoredMul] {
        for em_source in [
            EmSource::Internal,
            EmSource::Portable,
            EmSource::ExpMinusOne,
        ] {
            for first in [
                FirstGraph::PvDivEmMulVMulR,
                FirstGraph::PvMulVDivEmMulR,
                FirstGraph::PvMulRDivEmMulV,
                FirstGraph::PvMulRMulVDivEm,
                FirstGraph::PvDivEmMulVR,
                FirstGraph::PvMulVMulRDivEm,
            ] {
                for growth in [
                    GrowthGraph::RepeatedMultiply,
                    GrowthGraph::PowChain,
                    GrowthGraph::PowPositive,
                    GrowthGraph::ExpLog1p,
                    GrowthGraph::Expm1Log1p,
                ] {
                    for fold in [Fold::Strict, Fold::X87Continuous] {
                        let c = DiscountCandidate {
                            arith,
                            em_source,
                            first,
                            growth,
                            fold,
                        };
                        discount_ranked.push((score_discount(c, &rows, &payments, &amort), c));
                    }
                }
            }
        }
    }
    discount_ranked.sort_by_key(|(s, _)| (std::cmp::Reverse(s.exact), s.max_ulp, s.sum_ulp));
    println!("\ndiscount/geometric top 30:");
    for (s, c) in discount_ranked.iter().take(30) {
        let t0 = score_discount_timing(*c, &rows, &payments, &amort, 0);
        let t1 = score_discount_timing(*c, &rows, &payments, &amort, 1);
        println!(
            "{:?}/{:?}/{:?}/{:?}/{:?}: {}/{} [t0 {}/{} t1 {}/{}] max={} sum={}",
            c.arith,
            c.em_source,
            c.first,
            c.growth,
            c.fold,
            s.exact,
            s.total,
            t0.exact,
            t0.total,
            t1.exact,
            t1.total,
            s.max_ulp,
            s.sum_ulp
        );
    }

    let mut direct_ranked = Vec::new();
    for arith in [Arith::Strict, Arith::X87Period, Arith::X87StoredMul] {
        for power in [
            PowerGraph::StrictPowi,
            PowerGraph::X87StoredPowi,
            PowerGraph::PowChain,
            PowerGraph::PowPositive,
            PowerGraph::ExpLog1p,
            PowerGraph::Expm1Log1p,
        ] {
            for combine in [
                DirectCombine::RatioThenMul,
                DirectCombine::MulThenDiv,
                DirectCombine::DivThenMul,
            ] {
                let c = DirectCandidate {
                    arith,
                    power,
                    combine,
                };
                direct_ranked.push((score_direct(c, &rows, &amort), c));
            }
        }
    }
    direct_ranked.sort_by_key(|(s, _)| (std::cmp::Reverse(s.exact), s.max_ulp, s.sum_ulp));
    println!("\ndirect-boundary top 30:");
    for (s, c) in direct_ranked.iter().take(30) {
        println!(
            "{:?}/{:?}/{:?}: {}/{} max={} sum={}",
            c.arith, c.power, c.combine, s.exact, s.total, s.max_ulp, s.sum_ulp
        );
    }

    let mut stable_ranked = Vec::new();
    for arith in [Arith::Strict, Arith::X87Period, Arith::X87StoredMul] {
        for delta in [
            EmSource::Internal,
            EmSource::Portable,
            EmSource::ExpMinusOne,
        ] {
            for growth in [
                PowerGraph::StrictPowi,
                PowerGraph::X87StoredPowi,
                PowerGraph::PowChain,
                PowerGraph::PowPositive,
                PowerGraph::ExpLog1p,
                PowerGraph::Expm1Log1p,
            ] {
                for combine in [
                    DirectCombine::RatioThenMul,
                    DirectCombine::MulThenDiv,
                    DirectCombine::DivThenMul,
                ] {
                    let c = StableBoundaryCandidate {
                        arith,
                        delta,
                        growth,
                        combine,
                    };
                    stable_ranked.push((score_stable_boundary(c, &rows, &amort), c));
                }
            }
        }
    }
    stable_ranked.sort_by_key(|(s, _)| (std::cmp::Reverse(s.exact), s.max_ulp, s.sum_ulp));
    println!("\nstable-boundary top 30:");
    for (s, c) in stable_ranked.iter().take(30) {
        println!(
            "{:?}/{:?}/{:?}/{:?}: {}/{} max={} sum={}",
            c.arith, c.delta, c.growth, c.combine, s.exact, s.total, s.max_ulp, s.sum_ulp
        );
    }

    println!("\ncontinuous-x87 recurrence with recovered stored PMT:");
    let mut continuous_ranked = Vec::new();
    for state in [
        ContinuousState::BalanceAddPrincipal,
        ContinuousState::BalanceMulAdd,
        ContinuousState::PvPlusCumulative,
    ] {
        let (score, offsets) = optimized_continuous_score(state, &rows, &payments, &amort);
        continuous_ranked.push((score, offsets, state));
    }
    continuous_ranked.sort_by_key(|(s, _, _)| (std::cmp::Reverse(s.exact), s.max_ulp, s.sum_ulp));
    for (s, offsets, state) in &continuous_ranked {
        println!(
            "{:?}: {}/{} max={} sum={} offsets={:?}",
            state, s.exact, s.total, s.max_ulp, s.sum_ulp, offsets
        );
    }

    let mut closed_ranked = Vec::new();
    for arith in [Arith::Strict, Arith::X87Period, Arith::X87StoredMul] {
        for factor in [
            PowerGraph::StrictPowi,
            PowerGraph::X87StoredPowi,
            PowerGraph::PowChain,
            PowerGraph::PowPositive,
            PowerGraph::ExpLog1p,
            PowerGraph::Expm1Log1p,
        ] {
            for delta in [
                EmSource::Internal,
                EmSource::Portable,
                EmSource::ExpMinusOne,
            ] {
                for quotient in [QuotientGraph::Divide, QuotientGraph::ReciprocalMultiply] {
                    for fold in [Fold::Strict, Fold::X87Continuous] {
                        let c = ClosedFvCandidate {
                            arith,
                            factor,
                            delta,
                            quotient,
                            fold,
                        };
                        let (score, offsets) = optimized_closed_score(c, &rows, &payments, &amort);
                        closed_ranked.push((score, offsets, c));
                    }
                }
            }
        }
    }
    closed_ranked.sort_by_key(|(s, _, _)| (std::cmp::Reverse(s.exact), s.max_ulp, s.sum_ulp));
    println!("\nclosed-FV top 30 with recovered stored PMT:");
    for (s, offsets, c) in closed_ranked.iter().take(30) {
        println!(
            "{:?}/{:?}/{:?}/{:?}/{:?}: {}/{} max={} sum={} offsets={:?}",
            c.arith,
            c.factor,
            c.delta,
            c.quotient,
            c.fold,
            s.exact,
            s.total,
            s.max_ulp,
            s.sum_ulp,
            offsets
        );
    }

    // Reproduce the two canonical catalog witnesses through the shipping
    // kernel before any experimental production edit.
    for (name, end, want) in [
        ("full-schedule", 12.0, 0xc08f_4000_0000_0001u64),
        ("half-schedule", 6.0, 0xc076_8ceb_86d1_d5a0u64),
    ] {
        let got = cumprinc_kernel(0.1, 12.0, 1000.0, 1.0, end, 0.0)
            .expect("shipping CUMPRINC")
            .to_bits();
        println!(
            "shipping {name}: got=0x{got:016x} want=0x{want:016x} ulp={}",
            ulp(got, want)
        );
    }

    let champion = ranked[0].4;
    println!("\nchampion recovered PMT raw-bit offsets:");
    for (k, offset) in &ranked[0].3 {
        println!("  {k}: {offset:+}");
    }
    println!("\nchampion residuals: {champion:?}");
    for row in &rows {
        let payment = payments[&key(row.rate, row.nper as f64, row.pv, row.timing as f64)];
        let got = model(champion, row, payment).to_bits();
        let d = ulp(got, row.want);
        if d != 0 {
            println!(
                "{} [{},{}] type{} got=0x{:016x} want=0x{:016x} ulp={}",
                row.id, row.start, row.end, row.timing, got, row.want, d
            );
        }
    }

    if let Some((discovery_rows, discovery_payments)) = discovery {
        score_discovery(&discovery_rows, &discovery_payments);
    }
}
