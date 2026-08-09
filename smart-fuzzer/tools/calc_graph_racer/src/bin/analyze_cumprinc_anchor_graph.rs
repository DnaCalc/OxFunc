//! PMT-cancelled analysis of the W109 G6-07 CUMPRINC discovery battery.
//!
//! `CUMPRINC(...,1,1,0)` publishes the type-0 first principal.  Reusing that
//! observed value as an anchor cancels the unresolved PMT/expm1 helper and
//! lets this tool race only the principal-growth and range-fold graph.  The
//! paired PMT battery is used solely to test the first-principal boundary and
//! the exact type-1 period-1 identity.

use oxfunc_core::excel_numeric::research as rx;
use serde::Deserialize;
use std::collections::{BTreeMap, HashMap};
use std::path::Path;

const CW: u16 = rx::CW_PC64_RN;
const DEFAULT_CUM: &str =
    "../../work/w109/G6-cumprinc/answers-cumprinc-exact-discriminator-20260809.json";
const DEFAULT_PMT: &str =
    "../../work/w109/G6-cumprinc/answers-pmt-cumprinc-companion-20260809.json";

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

#[derive(Clone, Debug)]
struct Row {
    id: String,
    rate: f64,
    nper: i32,
    pv: f64,
    start: i32,
    end: i32,
    timing: i32,
    want: u64,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct LoanKey(u64, u64, u64, u64);

fn key(rate: f64, nper: f64, pv: f64, timing: f64) -> LoanKey {
    LoanKey(
        rate.to_bits(),
        nper.to_bits(),
        pv.to_bits(),
        timing.to_bits(),
    )
}

fn bits(text: &str) -> u64 {
    u64::from_str_radix(text.strip_prefix("0x").expect("0x bits"), 16).expect("hex bits")
}

fn ordered(value: u64) -> u64 {
    if value >> 63 != 0 {
        !value
    } else {
        value | (1_u64 << 63)
    }
}

fn ulp(got: u64, want: u64) -> u64 {
    ordered(got).abs_diff(ordered(want))
}

#[derive(Clone, Default)]
struct Score {
    exact: usize,
    total: usize,
    max_ulp: u64,
    sum_ulp: u128,
    hist: BTreeMap<u64, usize>,
}

impl Score {
    fn add(&mut self, got: u64, want: u64) {
        let distance = ulp(got, want);
        self.total += 1;
        self.exact += usize::from(distance == 0);
        self.max_ulp = self.max_ulp.max(distance);
        self.sum_ulp += u128::from(distance);
        *self.hist.entry(distance.min(32)).or_default() += 1;
    }

    fn rank(&self) -> (std::cmp::Reverse<usize>, u64, u128) {
        (std::cmp::Reverse(self.exact), self.max_ulp, self.sum_ulp)
    }

    fn merge(&mut self, other: Score) {
        self.exact += other.exact;
        self.total += other.total;
        self.max_ulp = self.max_ulp.max(other.max_ulp);
        self.sum_ulp += other.sum_ulp;
        for (distance, count) in other.hist {
            *self.hist.entry(distance).or_default() += count;
        }
    }
}

fn print_score(label: &str, score: &Score) {
    println!(
        "{label}: {}/{} max={} sum={} hist={:?}",
        score.exact, score.total, score.max_ulp, score.sum_ulp, score.hist
    );
}

fn load(path: &Path, function: &str) -> AnswerSet {
    let set: AnswerSet = serde_json::from_str(
        &std::fs::read_to_string(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display())),
    )
    .expect("parse answer set");
    assert_eq!(set.function, function);
    set
}

fn load_rows(path: &Path) -> Vec<Row> {
    load(path, "CUMPRINC")
        .witnesses
        .into_iter()
        .map(|witness| {
            assert_eq!(witness.args.len(), 6);
            let args = witness
                .args
                .iter()
                .map(|arg| f64::from_bits(bits(arg)))
                .collect::<Vec<_>>();
            Row {
                id: witness.id,
                rate: args[0],
                nper: args[1] as i32,
                pv: args[2],
                start: args[3] as i32,
                end: args[4] as i32,
                timing: args[5] as i32,
                want: bits(&witness.expected_bits),
            }
        })
        .collect()
}

fn load_payments(path: &Path) -> HashMap<LoanKey, f64> {
    load(path, "PMT")
        .witnesses
        .into_iter()
        .map(|witness| {
            assert_eq!(witness.args.len(), 5);
            let args = witness
                .args
                .iter()
                .map(|arg| f64::from_bits(bits(arg)))
                .collect::<Vec<_>>();
            (
                key(args[0], args[1], args[2], args[4]),
                f64::from_bits(bits(&witness.expected_bits)),
            )
        })
        .collect()
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
    fn mul(self, rhs: Self) -> Self {
        Self(rx::ext_mul(&self.0, &rhs.0, CW))
    }
    fn div(self, rhs: Self) -> Self {
        Self(rx::ext_div(&self.0, &rhs.0, CW))
    }
    fn sub(self, rhs: Self) -> Self {
        Self(rx::ext_sub(&self.0, &rhs.0, CW))
    }
    fn store(self) -> f64 {
        rx::ext_to_f64(&self.0, CW)
    }
}

#[derive(Clone, Copy, Debug)]
enum Arith {
    Strict,
    X87Store,
}

fn add(arith: Arith, left: f64, right: f64) -> f64 {
    match arith {
        Arith::Strict => left + right,
        Arith::X87Store => X::f(left).add(X::f(right)).store(),
    }
}

fn mul(arith: Arith, left: f64, right: f64) -> f64 {
    match arith {
        Arith::Strict => left * right,
        Arith::X87Store => X::f(left).mul(X::f(right)).store(),
    }
}

fn div(arith: Arith, left: f64, right: f64) -> f64 {
    match arith {
        Arith::Strict => left / right,
        Arith::X87Store => X::f(left).div(X::f(right)).store(),
    }
}

#[derive(Clone, Copy, Debug)]
enum LogSource {
    Portable,
    LnOnePlus,
    Fyl2xp1,
}

fn log1p(source: LogSource, rate: f64) -> f64 {
    match source {
        LogSource::Portable => rx::excel_log1p(rate),
        LogSource::LnOnePlus => rx::excel_ln(1.0 + rate),
        LogSource::Fyl2xp1 => rx::ext_to_f64(
            &rx::ext_fyl2xp1(&rx::ext_ln2(), &rx::ext_from_f64(rate), CW),
            CW,
        ),
    }
}

#[derive(Clone, Copy, Debug)]
enum Growth {
    RepeatedFactor,
    PowiStrict,
    PowiX87Stored,
    PowChain,
    PowPositive,
    ExpSse(LogSource),
    ExpX87Product(LogSource),
    Expm1Internal(LogSource),
}

fn powi(base: f64, exponent: i32, arith: Arith) -> f64 {
    let mut result = 1.0;
    let mut factor = base;
    let mut n = exponent;
    while n > 0 {
        if n & 1 != 0 {
            result = mul(arith, result, factor);
        }
        n >>= 1;
        if n > 0 {
            factor = mul(arith, factor, factor);
        }
    }
    result
}

fn repeated_factor(rate: f64, exponent: i32, arith: Arith) -> f64 {
    let factor = add(arith, 1.0, rate);
    let mut value = 1.0;
    for _ in 0..exponent {
        value = mul(arith, value, factor);
    }
    value
}

fn growth(growth: Growth, rate: f64, exponent: i32, arith: Arith) -> f64 {
    if exponent == 0 {
        return 1.0;
    }
    match growth {
        Growth::RepeatedFactor => repeated_factor(rate, exponent, arith),
        Growth::PowiStrict => powi(1.0 + rate, exponent, Arith::Strict),
        Growth::PowiX87Stored => powi(1.0 + rate, exponent, Arith::X87Store),
        Growth::PowChain => rx::excel_pow_chain(1.0 + rate, exponent as f64),
        Growth::PowPositive => rx::excel_pow_positive(1.0 + rate, exponent as f64),
        Growth::ExpSse(source) => rx::excel_exp(exponent as f64 * log1p(source, rate)),
        Growth::ExpX87Product(source) => {
            rx::excel_exp(rx::x87_mul(exponent as f64, log1p(source, rate)))
        }
        Growth::Expm1Internal(source) => {
            1.0 + rx::excel_expm1_internal(exponent as f64 * log1p(source, rate))
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum Fold {
    Strict,
    X87Continuous,
}

#[derive(Clone, Copy, Debug)]
struct ClosedCandidate {
    growth: Growth,
    arith: Arith,
    fold: Fold,
}

fn anchor_id(row: &Row) -> String {
    let mut id = row.id.replace("-t1-", "-t0-");
    for shape in [
        "singleton_early",
        "singleton_middle",
        "singleton_last",
        "prefix_early",
        "prefix_middle",
        "suffix_middle",
        "interior",
        "full",
    ] {
        id = id.replace(&format!("-{shape}-"), "-singleton_first-");
    }
    id
}

fn closed_model(
    candidate: ClosedCandidate,
    row: &Row,
    anchors: &HashMap<&str, &Row>,
    payments: &HashMap<LoanKey, f64>,
) -> f64 {
    let anchor = f64::from_bits(anchors[anchor_id(row).as_str()].want);
    let payment = payments[&key(row.rate, row.nper as f64, row.pv, row.timing as f64)];
    let mut strict_total = 0.0;
    let mut x_total = X::f(0.0);
    for period in row.start..=row.end {
        let principal = if row.timing == 1 && period == 1 {
            payment
        } else {
            let exponent = if row.timing == 1 {
                period - 2
            } else {
                period - 1
            };
            mul(
                candidate.arith,
                anchor,
                growth(candidate.growth, row.rate, exponent, candidate.arith),
            )
        };
        match candidate.fold {
            Fold::Strict => strict_total += principal,
            Fold::X87Continuous => x_total = x_total.add(X::f(principal)),
        }
    }
    match candidate.fold {
        Fold::Strict => strict_total,
        Fold::X87Continuous => x_total.store(),
    }
}

#[derive(Clone, Copy, Debug)]
enum StepState {
    StrictStored,
    X87Stored,
    X87ContinuousStoredAddend,
    X87ContinuousExtendedAddend,
}

#[derive(Clone, Copy, Debug)]
enum FactorState {
    StrictStored,
    X87Stored,
    X87Extended,
}

#[derive(Clone, Copy, Debug)]
struct IterCandidate {
    step: StepState,
    factor: FactorState,
    fold: Fold,
}

fn iterative_model(
    candidate: IterCandidate,
    row: &Row,
    anchors: &HashMap<&str, &Row>,
    payments: &HashMap<LoanKey, f64>,
) -> f64 {
    let anchor = f64::from_bits(anchors[anchor_id(row).as_str()].want);
    let payment = payments[&key(row.rate, row.nper as f64, row.pv, row.timing as f64)];
    let stored_factor = match candidate.factor {
        FactorState::StrictStored => 1.0 + row.rate,
        FactorState::X87Stored | FactorState::X87Extended => X::one().add(X::f(row.rate)).store(),
    };
    let extended_factor = X::one().add(X::f(row.rate));
    let mut stored_principal = anchor;
    let mut extended_principal = X::f(anchor);
    let mut strict_total = 0.0;
    let mut extended_total = X::f(0.0);

    for period in 1..=row.end {
        let (stored_addend, extended_addend) = if row.timing == 1 && period == 1 {
            (payment, X::f(payment))
        } else {
            match candidate.step {
                StepState::X87ContinuousExtendedAddend => {
                    (extended_principal.store(), extended_principal)
                }
                StepState::X87ContinuousStoredAddend => {
                    let stored = extended_principal.store();
                    (stored, X::f(stored))
                }
                StepState::StrictStored | StepState::X87Stored => {
                    (stored_principal, X::f(stored_principal))
                }
            }
        };
        if period >= row.start {
            match candidate.fold {
                Fold::Strict => strict_total += stored_addend,
                Fold::X87Continuous => extended_total = extended_total.add(extended_addend),
            }
        }

        let advance = row.timing == 0 || period >= 2;
        if advance {
            match candidate.step {
                StepState::StrictStored => stored_principal *= stored_factor,
                StepState::X87Stored => {
                    stored_principal = X::f(stored_principal).mul(X::f(stored_factor)).store();
                }
                StepState::X87ContinuousStoredAddend | StepState::X87ContinuousExtendedAddend => {
                    extended_principal = extended_principal.mul(match candidate.factor {
                        FactorState::X87Extended => extended_factor,
                        FactorState::StrictStored | FactorState::X87Stored => X::f(stored_factor),
                    });
                }
            }
        }
    }
    match candidate.fold {
        Fold::Strict => strict_total,
        Fold::X87Continuous => extended_total.store(),
    }
}

fn score_closed(
    candidate: ClosedCandidate,
    rows: &[Row],
    anchors: &HashMap<&str, &Row>,
    payments: &HashMap<LoanKey, f64>,
) -> Score {
    let mut score = Score::default();
    for row in rows {
        score.add(
            closed_model(candidate, row, anchors, payments).to_bits(),
            row.want,
        );
    }
    score
}

fn score_iterative(
    candidate: IterCandidate,
    rows: &[Row],
    anchors: &HashMap<&str, &Row>,
    payments: &HashMap<LoanKey, f64>,
) -> Score {
    let mut score = Score::default();
    for row in rows {
        score.add(
            iterative_model(candidate, row, anchors, payments).to_bits(),
            row.want,
        );
    }
    score
}

#[derive(Clone, Copy, Debug)]
enum Delta {
    FactorMinusOne,
    Portable,
    Internal,
    ExpMinusOne,
    InternalX87Product,
}

#[derive(Clone, Copy, Debug)]
enum PaymentSource {
    End,
    Timing,
}

#[derive(Clone, Copy, Debug)]
enum Association {
    PvFactorThenRate,
    PvThenFactorRate,
    PvRateThenFactor,
}

#[derive(Clone, Copy, Debug)]
enum BodyArith {
    Strict,
    X87Stored,
    X87Continuous,
}

#[derive(Clone, Copy, Debug)]
enum DueDivide {
    Divide,
    ReciprocalMultiply,
}

#[derive(Clone, Copy, Debug)]
struct LoanCandidate {
    factor: Growth,
    delta: Delta,
    payment_source: PaymentSource,
    association: Association,
    body: BodyArith,
    due_divide: DueDivide,
    fold: Fold,
}

fn delta(candidate: LoanCandidate, row: &Row, exponent: i32, factor: f64) -> f64 {
    if exponent == 0 {
        return 0.0;
    }
    let t = exponent as f64 * log1p(LogSource::Portable, row.rate);
    match candidate.delta {
        Delta::FactorMinusOne => match candidate.body {
            BodyArith::Strict => factor - 1.0,
            BodyArith::X87Stored | BodyArith::X87Continuous => X::f(factor).sub(X::one()).store(),
        },
        Delta::Portable => rx::excel_expm1(t),
        Delta::Internal => rx::excel_expm1_internal(t),
        Delta::ExpMinusOne => rx::excel_exp(t) - 1.0,
        Delta::InternalX87Product => rx::excel_expm1_internal(rx::x87_mul(
            exponent as f64,
            log1p(LogSource::Portable, row.rate),
        )),
    }
}

fn loan_principal(
    candidate: LoanCandidate,
    row: &Row,
    period: i32,
    pmt_end: f64,
    pmt_timing: f64,
) -> f64 {
    if row.timing == 1 && period == 1 {
        return pmt_timing;
    }
    let exponent = period - 1;
    let factor_arith = match candidate.body {
        BodyArith::Strict => Arith::Strict,
        BodyArith::X87Stored | BodyArith::X87Continuous => Arith::X87Store,
    };
    let factor = growth(candidate.factor, row.rate, exponent, factor_arith);
    let delta = delta(candidate, row, exponent, factor);
    let interest_payment = match candidate.payment_source {
        PaymentSource::End => pmt_end,
        PaymentSource::Timing => pmt_timing,
    };

    match candidate.body {
        BodyArith::Strict | BodyArith::X87Stored => {
            let arith = match candidate.body {
                BodyArith::Strict => Arith::Strict,
                BodyArith::X87Stored => Arith::X87Store,
                BodyArith::X87Continuous => unreachable!(),
            };
            let pv_term = match candidate.association {
                Association::PvFactorThenRate => mul(arith, mul(arith, row.pv, factor), row.rate),
                Association::PvThenFactorRate => mul(arith, row.pv, mul(arith, factor, row.rate)),
                Association::PvRateThenFactor => mul(arith, mul(arith, row.pv, row.rate), factor),
            };
            let pmt_term = mul(arith, interest_payment, delta);
            let mut interest = -add(arith, pv_term, pmt_term);
            if row.timing == 1 {
                let timing_factor = add(arith, 1.0, row.rate);
                interest = match candidate.due_divide {
                    DueDivide::Divide => div(arith, interest, timing_factor),
                    DueDivide::ReciprocalMultiply => {
                        mul(arith, interest, div(arith, 1.0, timing_factor))
                    }
                };
            }
            add(arith, pmt_timing, -interest)
        }
        BodyArith::X87Continuous => {
            let (pv, factor, rate) = (X::f(row.pv), X::f(factor), X::f(row.rate));
            let pv_term = match candidate.association {
                Association::PvFactorThenRate => pv.mul(factor).mul(rate),
                Association::PvThenFactorRate => pv.mul(factor.mul(rate)),
                Association::PvRateThenFactor => pv.mul(rate).mul(factor),
            };
            let pmt_term = X::f(interest_payment).mul(X::f(delta));
            let mut interest = X::f(0.0).sub(pv_term.add(pmt_term));
            if row.timing == 1 {
                let timing_factor = X::one().add(rate);
                interest = match candidate.due_divide {
                    DueDivide::Divide => interest.div(timing_factor),
                    DueDivide::ReciprocalMultiply => interest.mul(X::one().div(timing_factor)),
                };
            }
            X::f(pmt_timing).sub(interest).store()
        }
    }
}

fn score_loan(candidate: LoanCandidate, rows: &[Row], payments: &HashMap<LoanKey, f64>) -> Score {
    let mut score = Score::default();
    for row in rows {
        let pmt_end = payments[&key(row.rate, row.nper as f64, row.pv, 0.0)];
        let pmt_timing = payments[&key(row.rate, row.nper as f64, row.pv, row.timing as f64)];
        let mut strict_total = 0.0;
        let mut x_total = X::f(0.0);
        for period in row.start..=row.end {
            let principal = loan_principal(candidate, row, period, pmt_end, pmt_timing);
            match candidate.fold {
                Fold::Strict => strict_total += principal,
                Fold::X87Continuous => x_total = x_total.add(X::f(principal)),
            }
        }
        let got = match candidate.fold {
            Fold::Strict => strict_total,
            Fold::X87Continuous => x_total.store(),
        };
        score.add(got.to_bits(), row.want);
    }
    score
}

fn race_public_loan_formula(rows: &[Row], payments: &HashMap<LoanKey, f64>) {
    let factors = [
        Growth::PowiStrict,
        Growth::PowiX87Stored,
        Growth::PowChain,
        Growth::PowPositive,
        Growth::ExpSse(LogSource::Portable),
        Growth::ExpX87Product(LogSource::Portable),
        Growth::Expm1Internal(LogSource::Portable),
        Growth::ExpSse(LogSource::LnOnePlus),
        Growth::ExpX87Product(LogSource::LnOnePlus),
        Growth::Expm1Internal(LogSource::LnOnePlus),
        Growth::ExpSse(LogSource::Fyl2xp1),
        Growth::ExpX87Product(LogSource::Fyl2xp1),
        Growth::Expm1Internal(LogSource::Fyl2xp1),
    ];
    let deltas = [
        Delta::FactorMinusOne,
        Delta::Portable,
        Delta::Internal,
        Delta::ExpMinusOne,
        Delta::InternalX87Product,
    ];

    // Stage 1 selects only the factor/delta provider pair.  The broader body
    // race below uses the leading pairs; this is discovery ranking, never a
    // held-out validation claim.
    let mut provider_rank = Vec::new();
    for factor in factors {
        for delta in deltas {
            let candidate = LoanCandidate {
                factor,
                delta,
                payment_source: PaymentSource::End,
                association: Association::PvFactorThenRate,
                body: BodyArith::Strict,
                due_divide: DueDivide::Divide,
                fold: Fold::Strict,
            };
            provider_rank.push((score_loan(candidate, rows, payments), factor, delta));
        }
    }
    provider_rank.sort_by_key(|(score, _, _)| score.rank());
    println!("\npublic loan.fs provider stage top 15:");
    for (score, factor, delta) in provider_rank.iter().take(15) {
        print_score(&format!("factor={factor:?}/delta={delta:?}"), score);
    }

    let mut finalists = Vec::new();
    for (_, factor, delta) in provider_rank.iter().take(12) {
        for payment_source in [PaymentSource::End, PaymentSource::Timing] {
            for association in [
                Association::PvFactorThenRate,
                Association::PvThenFactorRate,
                Association::PvRateThenFactor,
            ] {
                for body in [
                    BodyArith::Strict,
                    BodyArith::X87Stored,
                    BodyArith::X87Continuous,
                ] {
                    for due_divide in [DueDivide::Divide, DueDivide::ReciprocalMultiply] {
                        for fold in [Fold::Strict, Fold::X87Continuous] {
                            let candidate = LoanCandidate {
                                factor: *factor,
                                delta: *delta,
                                payment_source,
                                association,
                                body,
                                due_divide,
                                fold,
                            };
                            finalists.push((score_loan(candidate, rows, payments), candidate));
                        }
                    }
                }
            }
        }
    }
    finalists.sort_by_key(|(score, _)| score.rank());
    println!("\npublic loan.fs full-axis top 30:");
    for (score, candidate) in finalists.iter().take(30) {
        print_score(&format!("{candidate:?}"), score);
    }
    let best = finalists[0].1;
    for timing in [0, 1] {
        let subset = rows
            .iter()
            .filter(|row| row.timing == timing)
            .cloned()
            .collect::<Vec<_>>();
        print_score(
            &format!("best loan.fs timing={timing}"),
            &score_loan(best, &subset, payments),
        );
    }
}

fn first_principal_scores(rows: &[Row], payments: &HashMap<LoanKey, f64>) {
    let first = rows
        .iter()
        .filter(|row| row.timing == 0 && row.id.contains("-singleton_first-"))
        .collect::<Vec<_>>();
    let mut scores = (0..7).map(|_| Score::default()).collect::<Vec<_>>();
    for row in first {
        let pmt0 = payments[&key(row.rate, row.nper as f64, row.pv, 0.0)];
        let pmt1 = payments[&key(row.rate, row.nper as f64, row.pv, 1.0)];
        let strict_product = row.pv * row.rate;
        let x_product = X::f(row.pv).mul(X::f(row.rate));
        let candidates = [
            pmt0 + strict_product,
            X::f(pmt0).add(x_product).store(),
            X::f(pmt0).add(X::f(strict_product)).store(),
            (pmt1 * (1.0 + row.rate)) + strict_product,
            X::f(pmt1)
                .mul(X::one().add(X::f(row.rate)))
                .add(x_product)
                .store(),
            pmt1 + (row.pv + pmt1) * row.rate,
            X::f(pmt1)
                .add(X::f(row.pv).add(X::f(pmt1)).mul(X::f(row.rate)))
                .store(),
        ];
        for (score, got) in scores.iter_mut().zip(candidates) {
            score.add(got.to_bits(), row.want);
        }
    }
    println!("first-principal boundary (30 type-0 rows):");
    for (label, score) in [
        ("strict pmt0 + pv*r", &scores[0]),
        ("x87 continuous pmt0 + pv*r", &scores[1]),
        ("x87 stored-product pmt0 + pv*r", &scores[2]),
        ("strict pmt1*(1+r) + pv*r", &scores[3]),
        ("x87 pmt1*(1+r) + pv*r", &scores[4]),
        ("strict pmt1 + (pv+pmt1)*r", &scores[5]),
        ("x87 pmt1 + (pv+pmt1)*r", &scores[6]),
    ] {
        print_score(label, score);
    }
}

#[derive(Clone, Copy, Debug)]
enum HiddenAddend {
    Extended,
    Stored,
}

#[derive(Clone, Copy, Debug)]
struct HiddenCandidate {
    addend: HiddenAddend,
    fold: Fold,
}

fn ext_offset(anchor: f64, offset: i32) -> Option<X> {
    let mut value = rx::ext_from_f64(anchor);
    let mut bytes = [0_u8; 8];
    bytes.copy_from_slice(&value.0[..8]);
    let significand = u64::from_le_bytes(bytes);
    let adjusted = if offset >= 0 {
        significand.checked_add(offset as u64)?
    } else {
        significand.checked_sub(offset.unsigned_abs() as u64)?
    };
    value.0[..8].copy_from_slice(&adjusted.to_le_bytes());
    (rx::ext_to_f64(&value, CW).to_bits() == anchor.to_bits()).then_some(X(value))
}

fn hidden_model(candidate: HiddenCandidate, row: &Row, hidden_anchor: X) -> f64 {
    assert_eq!(row.timing, 0);
    let factor = X::one().add(X::f(row.rate));
    let mut principal = hidden_anchor;
    let mut strict_total = 0.0;
    let mut extended_total = X::f(0.0);
    for period in 1..=row.end {
        if period >= row.start {
            match (candidate.fold, candidate.addend) {
                (Fold::Strict, _) => strict_total += principal.store(),
                (Fold::X87Continuous, HiddenAddend::Extended) => {
                    extended_total = extended_total.add(principal)
                }
                (Fold::X87Continuous, HiddenAddend::Stored) => {
                    extended_total = extended_total.add(X::f(principal.store()))
                }
            }
        }
        principal = principal.mul(factor);
    }
    match candidate.fold {
        Fold::Strict => strict_total,
        Fold::X87Continuous => extended_total.store(),
    }
}

fn search_hidden_anchors(rows: &[Row]) {
    let mut groups = HashMap::<LoanKey, Vec<&Row>>::new();
    for row in rows.iter().filter(|row| row.timing == 0) {
        groups
            .entry(key(row.rate, row.nper as f64, row.pv, 0.0))
            .or_default()
            .push(row);
    }
    assert_eq!(groups.len(), 30);

    println!("\nhidden-x87 first-principal search (type0, nine rows per loan):");
    for candidate in [
        HiddenCandidate {
            addend: HiddenAddend::Extended,
            fold: Fold::X87Continuous,
        },
        HiddenCandidate {
            addend: HiddenAddend::Stored,
            fold: Fold::X87Continuous,
        },
        HiddenCandidate {
            addend: HiddenAddend::Stored,
            fold: Fold::Strict,
        },
    ] {
        let mut aggregate = Score::default();
        let mut exact_groups = 0usize;
        let mut offsets = BTreeMap::new();
        for (loan, group) in &groups {
            let anchor_row = group
                .iter()
                .find(|row| row.id.contains("-singleton_first-"))
                .expect("type0 first-principal row");
            let published_anchor = f64::from_bits(anchor_row.want);
            let mut best: Option<(Score, i32)> = None;
            for offset in -2048..=2048 {
                let Some(hidden_anchor) = ext_offset(published_anchor, offset) else {
                    continue;
                };
                let mut score = Score::default();
                for row in group {
                    score.add(
                        hidden_model(candidate, row, hidden_anchor).to_bits(),
                        row.want,
                    );
                }
                let rank = (score.rank(), offset.unsigned_abs());
                if best
                    .as_ref()
                    .is_none_or(|(old, old_offset)| rank < (old.rank(), old_offset.unsigned_abs()))
                {
                    best = Some((score, offset));
                }
            }
            let (score, offset) = best.expect("at least exact widened anchor");
            exact_groups += usize::from(score.exact == score.total);
            aggregate.merge(score);
            offsets.insert(
                format!(
                    "r={:016x}/n={}/pv={:016x}",
                    loan.0,
                    f64::from_bits(loan.1),
                    loan.2
                ),
                offset,
            );
        }
        print_score(
            &format!("{candidate:?} exact_groups={exact_groups}/30"),
            &aggregate,
        );
        println!("  offsets={offsets:?}");
    }
}

fn coefficient_publication_search(rows: &[Row]) {
    let mut groups = HashMap::<String, Vec<&Row>>::new();
    for row in rows {
        let stem = row
            .id
            .strip_suffix(&row.id[row.id.len() - 3..])
            .expect("PV variant suffix")
            .to_owned();
        groups.entry(stem).or_default().push(row);
    }
    assert_eq!(groups.len(), 90);
    for group in groups.values_mut() {
        group.sort_by_key(|row| row.id[row.id.len() - 2..].parse::<usize>().unwrap());
        assert_eq!(group.len(), 6);
    }

    let mut f64_aggregate = Score::default();
    let mut f64_exact_groups = 0usize;
    let mut ext_aggregate = Score::default();
    let mut ext_exact_groups = 0usize;
    let splits: [(&[usize], &[usize]); 3] = [
        (&[0], &[1, 2, 3, 4, 5]),
        (&[0, 1], &[2, 3, 4, 5]),
        (&[0, 1, 2, 3], &[4, 5]),
    ];
    let split_labels = [
        "train v00; validate v01-v05",
        "train v00-v01; validate v02-v05",
        "train v00-v03; validate half/double",
    ];
    let mut split_train_aggregates = [Score::default(), Score::default(), Score::default()];
    let mut split_validation_aggregates = [Score::default(), Score::default(), Score::default()];
    let mut split_validation_exact_groups = [0usize; 3];

    for group in groups.values() {
        let estimate = f64::from_bits(group[0].want) / group[0].pv;
        let estimate_bits = estimate.to_bits();
        let mut best_f64: Option<Score> = None;
        for delta in -4096_i64..=4096 {
            let candidate_bits = (i128::from(estimate_bits) + i128::from(delta)) as u64;
            let coefficient = f64::from_bits(candidate_bits);
            if !coefficient.is_finite() {
                continue;
            }
            let mut score = Score::default();
            for row in group {
                score.add((row.pv * coefficient).to_bits(), row.want);
            }
            if best_f64
                .as_ref()
                .is_none_or(|old| score.rank() < old.rank())
            {
                best_f64 = Some(score);
            }
        }
        let best_f64 = best_f64.expect("finite coefficient neighborhood");
        f64_exact_groups += usize::from(best_f64.exact == best_f64.total);
        f64_aggregate.merge(best_f64);

        let mut seen = std::collections::BTreeSet::new();
        let mut best_ext: Option<Score> = None;
        let mut split_best = vec![None::<(Score, Score, i64)>; splits.len()];
        for f64_delta in -16_i64..=16 {
            let rounded =
                f64::from_bits((i128::from(estimate_bits) + i128::from(f64_delta)) as u64);
            if !rounded.is_finite() {
                continue;
            }
            for ext_delta in -2048..=2048 {
                let Some(coefficient) = ext_offset(rounded, ext_delta) else {
                    continue;
                };
                if !seen.insert(coefficient.0.0) {
                    continue;
                }
                let predictions = group
                    .iter()
                    .map(|row| X::f(row.pv).mul(coefficient).store().to_bits())
                    .collect::<Vec<_>>();
                let mut score = Score::default();
                for (row, got) in group.iter().zip(&predictions) {
                    score.add(*got, row.want);
                }
                if best_ext
                    .as_ref()
                    .is_none_or(|old| score.rank() < old.rank())
                {
                    best_ext = Some(score);
                }

                // These are explicit answer-fitting audits, not candidate models.  Each split
                // still has one independently fitted coefficient per query context; validation
                // only holds aside PV metamers within that same context.  Rank and tie-break use
                // training answers alone, never validation answers.
                let proximity = (f64_delta * 2048 + i64::from(ext_delta)).abs();
                for (split_index, (training, validation)) in splits.iter().enumerate() {
                    let mut train_score = Score::default();
                    for &index in *training {
                        train_score.add(predictions[index], group[index].want);
                    }
                    let mut validation_score = Score::default();
                    for &index in *validation {
                        validation_score.add(predictions[index], group[index].want);
                    }
                    let rank = (train_score.rank(), proximity);
                    if split_best[split_index].as_ref().is_none_or(
                        |(old_train, _, old_proximity)| rank < (old_train.rank(), *old_proximity),
                    ) {
                        split_best[split_index] = Some((train_score, validation_score, proximity));
                    }
                }
            }
        }
        let best_ext = best_ext.expect("finite extended coefficient neighborhood");
        ext_exact_groups += usize::from(best_ext.exact == best_ext.total);
        ext_aggregate.merge(best_ext);
        for (split_index, best) in split_best.into_iter().enumerate() {
            let (training, validation, _) = best.expect("finite split coefficient neighborhood");
            split_validation_exact_groups[split_index] +=
                usize::from(validation.exact == validation.total);
            split_train_aggregates[split_index].merge(training);
            split_validation_aggregates[split_index].merge(validation);
        }
    }

    println!("\neffective-coefficient publication search (six PV metamers/query):");
    print_score(
        &format!("stored-f64 coefficient exact_groups={f64_exact_groups}/90"),
        &f64_aggregate,
    );
    print_score(
        &format!("hidden-Ext80 coefficient exact_groups={ext_exact_groups}/90"),
        &ext_aggregate,
    );
    println!("per-query Ext80 PV-metamer splits (90 fitted parameters; no context holdout):");
    for split_index in 0..splits.len() {
        print_score(
            &format!("  {} training", split_labels[split_index]),
            &split_train_aggregates[split_index],
        );
        print_score(
            &format!(
                "  {} validation exact_groups={}/90",
                split_labels[split_index], split_validation_exact_groups[split_index]
            ),
            &split_validation_aggregates[split_index],
        );
    }
}

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    let rows = load_rows(Path::new(
        args.get(1).map(String::as_str).unwrap_or(DEFAULT_CUM),
    ));
    let payments = load_payments(Path::new(
        args.get(2).map(String::as_str).unwrap_or(DEFAULT_PMT),
    ));
    assert_eq!(rows.len(), 540);
    assert_eq!(payments.len(), 60);
    let anchors = rows
        .iter()
        .map(|row| (row.id.as_str(), row))
        .collect::<HashMap<_, _>>();

    first_principal_scores(&rows, &payments);
    search_hidden_anchors(&rows);
    coefficient_publication_search(&rows);
    race_public_loan_formula(&rows, &payments);

    let log_sources = [
        LogSource::Portable,
        LogSource::LnOnePlus,
        LogSource::Fyl2xp1,
    ];
    let mut growths = vec![
        Growth::RepeatedFactor,
        Growth::PowiStrict,
        Growth::PowiX87Stored,
        Growth::PowChain,
        Growth::PowPositive,
    ];
    for source in log_sources {
        growths.extend([
            Growth::ExpSse(source),
            Growth::ExpX87Product(source),
            Growth::Expm1Internal(source),
        ]);
    }
    let mut closed = Vec::new();
    for growth in growths {
        for arith in [Arith::Strict, Arith::X87Store] {
            for fold in [Fold::Strict, Fold::X87Continuous] {
                let candidate = ClosedCandidate {
                    growth,
                    arith,
                    fold,
                };
                closed.push((
                    score_closed(candidate, &rows, &anchors, &payments),
                    candidate,
                ));
            }
        }
    }
    closed.sort_by_key(|(score, _)| score.rank());
    println!("\nobserved-anchor closed-growth top 20:");
    for (score, candidate) in closed.iter().take(20) {
        print_score(&format!("{candidate:?}"), score);
    }

    let mut iterative = Vec::new();
    for step in [
        StepState::StrictStored,
        StepState::X87Stored,
        StepState::X87ContinuousStoredAddend,
        StepState::X87ContinuousExtendedAddend,
    ] {
        for factor in [
            FactorState::StrictStored,
            FactorState::X87Stored,
            FactorState::X87Extended,
        ] {
            if matches!(step, StepState::StrictStored | StepState::X87Stored)
                && matches!(factor, FactorState::X87Extended)
            {
                continue;
            }
            for fold in [Fold::Strict, Fold::X87Continuous] {
                let candidate = IterCandidate { step, factor, fold };
                iterative.push((
                    score_iterative(candidate, &rows, &anchors, &payments),
                    candidate,
                ));
            }
        }
    }
    iterative.sort_by_key(|(score, _)| score.rank());
    println!("\nobserved-anchor iterative-growth candidates:");
    for (score, candidate) in &iterative {
        print_score(&format!("{candidate:?}"), score);
    }

    let singleton_rows = rows
        .iter()
        .filter(|row| row.id.contains("-singleton_"))
        .cloned()
        .collect::<Vec<_>>();
    let range_rows = rows
        .iter()
        .filter(|row| !row.id.contains("-singleton_"))
        .cloned()
        .collect::<Vec<_>>();
    println!("\nbest candidate localization:");
    let best_closed = closed[0].1;
    print_score(
        "best closed / singletons",
        &score_closed(best_closed, &singleton_rows, &anchors, &payments),
    );
    print_score(
        "best closed / ranges",
        &score_closed(best_closed, &range_rows, &anchors, &payments),
    );
    let best_iterative = iterative[0].1;
    print_score(
        "best iterative / singletons",
        &score_iterative(best_iterative, &singleton_rows, &anchors, &payments),
    );
    print_score(
        "best iterative / ranges",
        &score_iterative(best_iterative, &range_rows, &anchors, &payments),
    );
}
