//! W109 G3-01: exact offline race for the clean `a = 2` GAMMA.DIST Taylor slice.
//!
//! The slice fixes `Gamma(2) = 1`, beta = 1, cumulative = TRUE, and 0 < x < 2,
//! so every remaining bit is attributable to the logarithm/EXP delivery,
//! Taylor recurrence, and publication graph.  No Excel process is used.
//!
//! Usage:
//!   race_g301_a2_taylor <G3-01-dist-work-dir>

use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use oxfunc_core::functions::special_math_common::regularized_gamma_p;
use rayon::prelude::*;
use rx::{
    CW_PC53_RN, CW_PC64_RN, Ext80, ext_abs, ext_add, ext_div, ext_f2xm1, ext_from_f64, ext_fyl2x,
    ext_l2e, ext_ln2, ext_mul, ext_one, ext_rndint, ext_scale, ext_sub, ext_to_f64,
};
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::fs;

const EPS: f64 = 1.110_223_024_625_156_5e-16;
const CW_RZ_BITS: u16 = 0x0c00;
const CW_RU_BITS: u16 = 0x0800;
const LEGACY: u8 = 1;
const B23: u8 = 2;
const B26: u8 = 4;

#[derive(Clone)]
struct Row {
    id: String,
    x: f64,
    expected: u64,
    cohorts: u8,
}

#[derive(Clone, Copy, Debug)]
enum ArgKind {
    StdStored,
    WorksheetStored,
    RawLnStored,
    RawT1Stored,
    StdLnRawT1Stored,
    RawT1Extended,
    StdLnRawT1Extended,
    FmaStored,
}

impl ArgKind {
    const ALL: [Self; 8] = [
        Self::StdStored,
        Self::WorksheetStored,
        Self::RawLnStored,
        Self::RawT1Stored,
        Self::StdLnRawT1Stored,
        Self::RawT1Extended,
        Self::StdLnRawT1Extended,
        Self::FmaStored,
    ];

    fn label(self) -> &'static str {
        match self {
            Self::StdStored => "arg:std-ln/f64-t1",
            Self::WorksheetStored => "arg:worksheet-ln/f64-t1",
            Self::RawLnStored => "arg:raw-ln-store/f64-t1",
            Self::RawT1Stored => "arg:raw-ln/raw-t1/store",
            Self::StdLnRawT1Stored => "arg:std-ln/raw-t1/store",
            Self::RawT1Extended => "arg:raw-ln/raw-t1/extended",
            Self::StdLnRawT1Extended => "arg:std-ln/raw-t1/extended",
            Self::FmaStored => "arg:std-ln/fma-t1",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ExpKind {
    StdNearest,
    Raw64Nearest,
    Raw64TowardZero,
    Raw64Up,
    Raw64Extended,
    Raw53Nearest,
    Raw53TowardZero,
}

impl ExpKind {
    const ALL: [Self; 7] = [
        Self::StdNearest,
        Self::Raw64Nearest,
        Self::Raw64TowardZero,
        Self::Raw64Up,
        Self::Raw64Extended,
        Self::Raw53Nearest,
        Self::Raw53TowardZero,
    ];

    fn label(self) -> &'static str {
        match self {
            Self::StdNearest => "exp:std-rn",
            Self::Raw64Nearest => "exp:x87pc64-rn53",
            Self::Raw64TowardZero => "exp:x87pc64-rz53",
            Self::Raw64Up => "exp:x87pc64-ru53",
            Self::Raw64Extended => "exp:x87pc64-extended",
            Self::Raw53Nearest => "exp:x87pc53-rn53",
            Self::Raw53TowardZero => "exp:x87pc53-rz53",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SeriesKind {
    ForwardDivFirst,
    ForwardMulFirst,
    ForwardTailThenOne,
    ForwardReverse,
    ForwardPairwise,
    ForwardKahan,
    DistributedDivFirst,
    DistributedMulFirst,
    WkBackward,
    X87DrDivFirst,
    X87DrMulFirst,
    X87ContinuousDivFirst,
    X87ContinuousMulFirst,
}

impl SeriesKind {
    const ALL: [Self; 13] = [
        Self::ForwardDivFirst,
        Self::ForwardMulFirst,
        Self::ForwardTailThenOne,
        Self::ForwardReverse,
        Self::ForwardPairwise,
        Self::ForwardKahan,
        Self::DistributedDivFirst,
        Self::DistributedMulFirst,
        Self::WkBackward,
        Self::X87DrDivFirst,
        Self::X87DrMulFirst,
        Self::X87ContinuousDivFirst,
        Self::X87ContinuousMulFirst,
    ];

    fn label(self) -> &'static str {
        match self {
            Self::ForwardDivFirst => "series:fwd-div-first",
            Self::ForwardMulFirst => "series:fwd-mul-first",
            Self::ForwardTailThenOne => "series:fwd-tail-then-one",
            Self::ForwardReverse => "series:fwd-reverse",
            Self::ForwardPairwise => "series:fwd-pairwise",
            Self::ForwardKahan => "series:fwd-kahan",
            Self::DistributedDivFirst => "series:distributed-div-first",
            Self::DistributedMulFirst => "series:distributed-mul-first",
            Self::WkBackward => "series:wk-backward",
            Self::X87DrDivFirst => "series:x87dr-div-first",
            Self::X87DrMulFirst => "series:x87dr-mul-first",
            Self::X87ContinuousDivFirst => "series:x87-cont-div-first",
            Self::X87ContinuousMulFirst => "series:x87-cont-mul-first",
        }
    }

    fn is_distributed(self) -> bool {
        matches!(self, Self::DistributedDivFirst | Self::DistributedMulFirst)
    }
}

#[derive(Clone, Copy, Debug)]
enum PubKind {
    F64HalfFirst,
    F64MulFirst,
    F64FactorFirst,
    F64Additive,
    X87ContinuousHalfFirstRn,
    X87ContinuousMulFirstRn,
    X87ContinuousFactorFirstRn,
    X87ContinuousAdditiveRn,
    X87ContinuousHalfFirstRz,
    X87ContinuousMulFirstRz,
    X87DrHalfFirst,
    X87DrMulFirst,
    X87RzHalfFirst,
    X87RzMulFirst,
    FactorF64,
    FactorX87Rn,
    FactorX87Rz,
}

impl PubKind {
    const ANS: [Self; 14] = [
        Self::F64HalfFirst,
        Self::F64MulFirst,
        Self::F64FactorFirst,
        Self::F64Additive,
        Self::X87ContinuousHalfFirstRn,
        Self::X87ContinuousMulFirstRn,
        Self::X87ContinuousFactorFirstRn,
        Self::X87ContinuousAdditiveRn,
        Self::X87ContinuousHalfFirstRz,
        Self::X87ContinuousMulFirstRz,
        Self::X87DrHalfFirst,
        Self::X87DrMulFirst,
        Self::X87RzHalfFirst,
        Self::X87RzMulFirst,
    ];
    const FACTOR: [Self; 3] = [Self::FactorF64, Self::FactorX87Rn, Self::FactorX87Rz];

    fn label(self) -> &'static str {
        match self {
            Self::F64HalfFirst => "pub:f64-(e/2)*ans",
            Self::F64MulFirst => "pub:f64-(e*ans)/2",
            Self::F64FactorFirst => "pub:f64-e*(ans/2)",
            Self::F64Additive => "pub:f64-e/2+(e/2)*tail",
            Self::X87ContinuousHalfFirstRn => "pub:x87-cont-(e/2)*ans-rn",
            Self::X87ContinuousMulFirstRn => "pub:x87-cont-(e*ans)/2-rn",
            Self::X87ContinuousFactorFirstRn => "pub:x87-cont-e*(ans/2)-rn",
            Self::X87ContinuousAdditiveRn => "pub:x87-cont-additive-rn",
            Self::X87ContinuousHalfFirstRz => "pub:x87-cont-(e/2)*ans-rz",
            Self::X87ContinuousMulFirstRz => "pub:x87-cont-(e*ans)/2-rz",
            Self::X87DrHalfFirst => "pub:x87dr-(e/2)*ans",
            Self::X87DrMulFirst => "pub:x87dr-(e*ans)/2",
            Self::X87RzHalfFirst => "pub:x87rz-(e/2)*ans",
            Self::X87RzMulFirst => "pub:x87rz-(e*ans)/2",
            Self::FactorF64 => "pub:f64-e*factor",
            Self::FactorX87Rn => "pub:x87-cont-e*factor-rn",
            Self::FactorX87Rz => "pub:x87-cont-e*factor-rz",
        }
    }
}

#[derive(Clone)]
struct ArgValue {
    arg_f64: f64,
    exp64: Ext80,
    exp53: Ext80,
}

#[derive(Clone)]
struct SeriesValue {
    ans_f64: f64,
    ans_ext: Ext80,
    factor_f64: f64,
    factor_ext: Ext80,
}

#[derive(Clone)]
struct Prepared {
    row: Row,
    args: Vec<ArgValue>,
    series: Vec<SeriesValue>,
}

#[derive(Clone, Copy)]
struct Candidate {
    arg: usize,
    exp: ExpKind,
    series: usize,
    publication: PubKind,
}

#[derive(Clone)]
struct ResultRow {
    candidate: Candidate,
    exact: usize,
    max_ulp: u64,
    sum_ulp: u128,
    residual_hist: BTreeMap<i64, usize>,
}

fn scalar(arg: &WitnessArg) -> Option<f64> {
    match arg {
        WitnessArg::Scalar(s) => parse_bits_hex(s),
        WitnessArg::Array(_) => None,
    }
}

fn load_rows(dir: &str) -> Vec<Row> {
    let sources = [
        ("answers-b5.json", LEGACY),
        ("answers-gammadist-modern.json", LEGACY),
        ("answers-b23-gd.json", B23),
        ("answers-b26-gd.json", B26),
    ];
    let mut rows: BTreeMap<u64, Row> = BTreeMap::new();
    for (name, cohort) in sources {
        let text = fs::read_to_string(format!("{dir}/{name}"))
            .unwrap_or_else(|e| panic!("cannot read {name}: {e}"));
        let set: WitnessSet =
            serde_json::from_str(&text).unwrap_or_else(|e| panic!("cannot parse {name}: {e}"));
        assert_eq!(set.function, "GAMMA.DIST");
        for witness in set.witnesses {
            if witness.args.len() != 4 {
                continue;
            }
            let Some(x) = scalar(&witness.args[0]) else {
                continue;
            };
            let Some(a) = scalar(&witness.args[1]) else {
                continue;
            };
            let Some(beta) = scalar(&witness.args[2]) else {
                continue;
            };
            let Some(cumulative) = scalar(&witness.args[3]) else {
                continue;
            };
            if a != 2.0 || beta != 1.0 || cumulative != 1.0 || !(x > 0.0 && x < 2.0) {
                continue;
            }
            let id = witness.id.unwrap_or_default();
            if cohort == B23 && !(id.starts_with("b23A-") || id.starts_with("b23B")) {
                continue;
            }
            if cohort == B26 && !(id.starts_with("b26A-") || id.starts_with("b26X-")) {
                continue;
            }
            let expected = parse_bits_hex(&witness.expected_bits)
                .unwrap_or_else(|| panic!("non-numeric expected in {name} row {id}"))
                .to_bits();
            rows.entry(x.to_bits())
                .and_modify(|old| {
                    assert_eq!(
                        old.expected, expected,
                        "conflicting Excel answers for x={x:?}: {} vs {id}",
                        old.id
                    );
                    old.cohorts |= cohort;
                })
                .or_insert(Row {
                    id: format!("{name}:{id}"),
                    x,
                    expected,
                    cohorts: cohort,
                });
        }
    }
    rows.into_values().collect()
}

fn raw_ln_ext(x: f64) -> Ext80 {
    ext_fyl2x(&ext_ln2(), &ext_from_f64(x), CW_PC64_RN)
}

fn exp_chain(arg: &Ext80, cw: u16) -> Ext80 {
    let t = ext_mul(arg, &ext_l2e(), cw);
    let k = ext_rndint(&t, cw);
    let f = ext_sub(&t, &k, cw);
    let negative = ext_to_f64(&f, cw) < 0.0;
    let w = ext_f2xm1(&ext_abs(&f, cw), cw);
    let mut m = ext_add(&w, &ext_one(), cw);
    if negative {
        m = ext_div(&ext_one(), &m, cw);
    }
    ext_scale(&m, &k, cw)
}

fn prepare_arg(x: f64, kind: ArgKind) -> ArgValue {
    let two = ext_from_f64(2.0);
    let xe = ext_from_f64(x);
    let std_ln = x.ln();
    let worksheet_ln = rx::excel_ln(x);
    let raw_ln = raw_ln_ext(x);
    let arg_ext = match kind {
        ArgKind::StdStored => ext_from_f64(2.0 * std_ln - x),
        ArgKind::WorksheetStored => ext_from_f64(2.0 * worksheet_ln - x),
        ArgKind::RawLnStored => {
            let ln = ext_to_f64(&raw_ln, CW_PC64_RN);
            ext_from_f64(2.0 * ln - x)
        }
        ArgKind::RawT1Stored => {
            let product = ext_mul(&two, &raw_ln, CW_PC64_RN);
            ext_from_f64(ext_to_f64(&ext_sub(&product, &xe, CW_PC64_RN), CW_PC64_RN))
        }
        ArgKind::StdLnRawT1Stored => {
            let product = ext_mul(&two, &ext_from_f64(std_ln), CW_PC64_RN);
            ext_from_f64(ext_to_f64(&ext_sub(&product, &xe, CW_PC64_RN), CW_PC64_RN))
        }
        ArgKind::RawT1Extended => {
            let product = ext_mul(&two, &raw_ln, CW_PC64_RN);
            ext_sub(&product, &xe, CW_PC64_RN)
        }
        ArgKind::StdLnRawT1Extended => {
            let product = ext_mul(&two, &ext_from_f64(std_ln), CW_PC64_RN);
            ext_sub(&product, &xe, CW_PC64_RN)
        }
        ArgKind::FmaStored => ext_from_f64(2.0f64.mul_add(std_ln, -x)),
    };
    let arg_f64 = ext_to_f64(&arg_ext, CW_PC64_RN);
    ArgValue {
        arg_f64,
        exp64: exp_chain(&arg_ext, CW_PC64_RN),
        exp53: exp_chain(&arg_ext, CW_PC53_RN),
    }
}

fn dr_add(a: f64, b: f64) -> f64 {
    ext_to_f64(
        &ext_add(&ext_from_f64(a), &ext_from_f64(b), CW_PC64_RN),
        CW_PC64_RN,
    )
}

fn dr_mul(a: f64, b: f64) -> f64 {
    ext_to_f64(
        &ext_mul(&ext_from_f64(a), &ext_from_f64(b), CW_PC64_RN),
        CW_PC64_RN,
    )
}

fn dr_div(a: f64, b: f64) -> f64 {
    ext_to_f64(
        &ext_div(&ext_from_f64(a), &ext_from_f64(b), CW_PC64_RN),
        CW_PC64_RN,
    )
}

fn terms(x: f64, mul_first: bool) -> Vec<f64> {
    let mut rr = 2.0;
    let mut c = 1.0;
    let mut out = Vec::new();
    let mut ans_for_stop = 1.0;
    loop {
        rr += 1.0;
        c = if mul_first {
            (c * x) / rr
        } else {
            c * (x / rr)
        };
        out.push(c);
        ans_for_stop += c;
        if c / ans_for_stop <= EPS {
            break;
        }
        assert!(out.len() < 256);
    }
    out
}

fn pairwise_sum(mut values: Vec<f64>) -> f64 {
    while values.len() > 1 {
        let mut next = Vec::with_capacity(values.len().div_ceil(2));
        let mut it = values.chunks_exact(2);
        for pair in &mut it {
            next.push(pair[0] + pair[1]);
        }
        if let Some(&last) = it.remainder().first() {
            next.push(last);
        }
        values = next;
    }
    values.first().copied().unwrap_or(0.0)
}

fn series_f64(x: f64, kind: SeriesKind) -> (f64, f64) {
    match kind {
        SeriesKind::ForwardDivFirst | SeriesKind::ForwardMulFirst => {
            let mut ans = 1.0;
            for c in terms(x, matches!(kind, SeriesKind::ForwardMulFirst)) {
                ans += c;
            }
            (ans, ans / 2.0)
        }
        SeriesKind::ForwardTailThenOne => {
            let mut tail = 0.0;
            for c in terms(x, false) {
                tail += c;
            }
            let ans = 1.0 + tail;
            (ans, ans / 2.0)
        }
        SeriesKind::ForwardReverse => {
            let mut ts = terms(x, false);
            ts.reverse();
            let mut ans = 1.0;
            for c in ts {
                ans += c;
            }
            (ans, ans / 2.0)
        }
        SeriesKind::ForwardPairwise => {
            let mut ts = terms(x, false);
            ts.push(1.0);
            let ans = pairwise_sum(ts);
            (ans, ans / 2.0)
        }
        SeriesKind::ForwardKahan => {
            let mut ans = 1.0;
            let mut correction = 0.0;
            for c in terms(x, false) {
                let y = c - correction;
                let t = ans + y;
                correction = (t - ans) - y;
                ans = t;
            }
            (ans, ans / 2.0)
        }
        SeriesKind::DistributedDivFirst | SeriesKind::DistributedMulFirst => {
            let mut rr = 2.0;
            let mut d = 0.5;
            let mut sum = 0.5;
            loop {
                rr += 1.0;
                d = if matches!(kind, SeriesKind::DistributedMulFirst) {
                    (d * x) / rr
                } else {
                    d * (x / rr)
                };
                sum += d;
                if d.abs() < sum.abs() * EPS {
                    break;
                }
            }
            (sum * 2.0, sum)
        }
        SeriesKind::WkBackward => {
            let mut wk = [0.0; 21];
            let mut apn = 3.0;
            let mut t = x / apn;
            wk[1] = t;
            let mut n = 20usize;
            let mut broke = false;
            for n_ in 2..=20 {
                apn += 1.0;
                t *= x / apn;
                if t <= 1e-3 {
                    n = n_;
                    broke = true;
                    break;
                }
                wk[n_] = t;
            }
            if !broke {
                n = 20;
            }
            let mut sum = t;
            loop {
                apn += 1.0;
                t *= x / apn;
                sum += t;
                if t <= 2.5e-15 {
                    break;
                }
            }
            for _ in 0..n - 1 {
                n -= 1;
                sum += wk[n];
            }
            let ans = 1.0 + sum;
            (ans, ans / 2.0)
        }
        SeriesKind::X87DrDivFirst | SeriesKind::X87DrMulFirst => {
            let mut rr = 2.0;
            let mut c = 1.0;
            let mut ans = 1.0;
            loop {
                rr += 1.0;
                c = if matches!(kind, SeriesKind::X87DrMulFirst) {
                    dr_div(dr_mul(c, x), rr)
                } else {
                    dr_mul(c, dr_div(x, rr))
                };
                ans = dr_add(ans, c);
                if dr_div(c, ans) <= EPS {
                    break;
                }
            }
            (ans, ans / 2.0)
        }
        SeriesKind::X87ContinuousDivFirst | SeriesKind::X87ContinuousMulFirst => {
            unreachable!("continuous series handled separately")
        }
    }
}

fn series_ext(x: f64, mul_first: bool) -> (Ext80, Ext80) {
    let xe = ext_from_f64(x);
    let mut rr = 2.0;
    let mut c = ext_one();
    let mut ans = ext_one();
    loop {
        rr += 1.0;
        let re = ext_from_f64(rr);
        c = if mul_first {
            ext_div(&ext_mul(&c, &xe, CW_PC64_RN), &re, CW_PC64_RN)
        } else {
            ext_mul(&c, &ext_div(&xe, &re, CW_PC64_RN), CW_PC64_RN)
        };
        ans = ext_add(&ans, &c, CW_PC64_RN);
        let ratio = ext_div(&c, &ans, CW_PC64_RN);
        if ext_to_f64(&ratio, CW_PC64_RN) <= EPS {
            break;
        }
    }
    let factor = ext_div(&ans, &ext_from_f64(2.0), CW_PC64_RN);
    (ans, factor)
}

fn prepare_series(x: f64, kind: SeriesKind) -> SeriesValue {
    if matches!(
        kind,
        SeriesKind::X87ContinuousDivFirst | SeriesKind::X87ContinuousMulFirst
    ) {
        let (ans_ext, factor_ext) =
            series_ext(x, matches!(kind, SeriesKind::X87ContinuousMulFirst));
        return SeriesValue {
            ans_f64: ext_to_f64(&ans_ext, CW_PC64_RN),
            ans_ext,
            factor_f64: ext_to_f64(&factor_ext, CW_PC64_RN),
            factor_ext,
        };
    }
    let (ans_f64, factor_f64) = series_f64(x, kind);
    SeriesValue {
        ans_f64,
        ans_ext: ext_from_f64(ans_f64),
        factor_f64,
        factor_ext: ext_from_f64(factor_f64),
    }
}

fn prepare(row: Row) -> Prepared {
    let args = ArgKind::ALL
        .iter()
        .copied()
        .map(|kind| prepare_arg(row.x, kind))
        .collect();
    let series = SeriesKind::ALL
        .iter()
        .copied()
        .map(|kind| prepare_series(row.x, kind))
        .collect();
    Prepared { row, args, series }
}

fn exp_source(arg: &ArgValue, kind: ExpKind) -> (f64, Ext80) {
    match kind {
        ExpKind::StdNearest => {
            let value = arg.arg_f64.exp();
            (value, ext_from_f64(value))
        }
        ExpKind::Raw64Nearest => {
            let value = ext_to_f64(&arg.exp64, CW_PC64_RN);
            (value, ext_from_f64(value))
        }
        ExpKind::Raw64TowardZero => {
            let value = ext_to_f64(&arg.exp64, CW_PC64_RN | CW_RZ_BITS);
            (value, ext_from_f64(value))
        }
        ExpKind::Raw64Up => {
            let value = ext_to_f64(&arg.exp64, CW_PC64_RN | CW_RU_BITS);
            (value, ext_from_f64(value))
        }
        ExpKind::Raw64Extended => (ext_to_f64(&arg.exp64, CW_PC64_RN), arg.exp64),
        ExpKind::Raw53Nearest => {
            let value = ext_to_f64(&arg.exp53, CW_PC53_RN);
            (value, ext_from_f64(value))
        }
        ExpKind::Raw53TowardZero => {
            let value = ext_to_f64(&arg.exp53, CW_PC53_RN | CW_RZ_BITS);
            (value, ext_from_f64(value))
        }
    }
}

fn eval(candidate: Candidate, prepared: &Prepared) -> f64 {
    let arg = &prepared.args[candidate.arg];
    let series = &prepared.series[candidate.series];
    let (exp_f64, exp_ext) = exp_source(arg, candidate.exp);
    let two = ext_from_f64(2.0);
    let final_rn = |value: &Ext80| ext_to_f64(value, CW_PC64_RN);
    let final_rz = |value: &Ext80| ext_to_f64(value, CW_PC64_RN | CW_RZ_BITS);
    match candidate.publication {
        PubKind::F64HalfFirst => (exp_f64 / 2.0) * series.ans_f64,
        PubKind::F64MulFirst => (exp_f64 * series.ans_f64) / 2.0,
        PubKind::F64FactorFirst => exp_f64 * (series.ans_f64 / 2.0),
        PubKind::F64Additive => {
            let half = exp_f64 / 2.0;
            half + half * (series.ans_f64 - 1.0)
        }
        PubKind::X87ContinuousHalfFirstRn => final_rn(&ext_mul(
            &ext_div(&exp_ext, &two, CW_PC64_RN),
            &series.ans_ext,
            CW_PC64_RN,
        )),
        PubKind::X87ContinuousMulFirstRn => final_rn(&ext_div(
            &ext_mul(&exp_ext, &series.ans_ext, CW_PC64_RN),
            &two,
            CW_PC64_RN,
        )),
        PubKind::X87ContinuousFactorFirstRn => {
            final_rn(&ext_mul(&exp_ext, &series.factor_ext, CW_PC64_RN))
        }
        PubKind::X87ContinuousAdditiveRn => {
            let half = ext_div(&exp_ext, &two, CW_PC64_RN);
            let tail = ext_sub(&series.ans_ext, &ext_one(), CW_PC64_RN);
            final_rn(&ext_add(
                &half,
                &ext_mul(&half, &tail, CW_PC64_RN),
                CW_PC64_RN,
            ))
        }
        PubKind::X87ContinuousHalfFirstRz => final_rz(&ext_mul(
            &ext_div(&exp_ext, &two, CW_PC64_RN),
            &series.ans_ext,
            CW_PC64_RN,
        )),
        PubKind::X87ContinuousMulFirstRz => final_rz(&ext_div(
            &ext_mul(&exp_ext, &series.ans_ext, CW_PC64_RN),
            &two,
            CW_PC64_RN,
        )),
        PubKind::X87DrHalfFirst => {
            let half = ext_to_f64(&ext_div(&exp_ext, &two, CW_PC64_RN), CW_PC64_RN);
            ext_to_f64(
                &ext_mul(&ext_from_f64(half), &series.ans_ext, CW_PC64_RN),
                CW_PC64_RN,
            )
        }
        PubKind::X87DrMulFirst => {
            let product = ext_to_f64(&ext_mul(&exp_ext, &series.ans_ext, CW_PC64_RN), CW_PC64_RN);
            product / 2.0
        }
        PubKind::X87RzHalfFirst => {
            let half = ext_to_f64(
                &ext_div(&exp_ext, &two, CW_PC64_RN),
                CW_PC64_RN | CW_RZ_BITS,
            );
            ext_to_f64(
                &ext_mul(&ext_from_f64(half), &series.ans_ext, CW_PC64_RN),
                CW_PC64_RN,
            )
        }
        PubKind::X87RzMulFirst => {
            let product = ext_to_f64(
                &ext_mul(&exp_ext, &series.ans_ext, CW_PC64_RN),
                CW_PC64_RN | CW_RZ_BITS,
            );
            product / 2.0
        }
        PubKind::FactorF64 => exp_f64 * series.factor_f64,
        PubKind::FactorX87Rn => final_rn(&ext_mul(&exp_ext, &series.factor_ext, CW_PC64_RN)),
        PubKind::FactorX87Rz => final_rz(&ext_mul(&exp_ext, &series.factor_ext, CW_PC64_RN)),
    }
}

fn signed_ulp(expected: u64, got: u64) -> i64 {
    if expected >= got {
        expected.saturating_sub(got).min(i64::MAX as u64) as i64
    } else {
        -(got.saturating_sub(expected).min(i64::MAX as u64) as i64)
    }
}

fn candidates() -> Vec<Candidate> {
    let mut out = Vec::new();
    for arg in 0..ArgKind::ALL.len() {
        for exp in ExpKind::ALL {
            for series in 0..SeriesKind::ALL.len() {
                let kind = SeriesKind::ALL[series];
                if !kind.is_distributed() {
                    for publication in PubKind::ANS {
                        out.push(Candidate {
                            arg,
                            exp,
                            series,
                            publication,
                        });
                    }
                }
                for publication in PubKind::FACTOR {
                    out.push(Candidate {
                        arg,
                        exp,
                        series,
                        publication,
                    });
                }
            }
        }
    }
    out
}

fn discovery_row(row: &Prepared) -> bool {
    row.row.cohorts & (LEGACY | B26) != 0 || row.row.x.to_bits() % 29 == 0
}

fn score(
    candidate: Candidate,
    rows: &[Prepared],
    discovery_only: bool,
    collect_histogram: bool,
) -> ResultRow {
    let mut exact = 0usize;
    let mut max_ulp = 0u64;
    let mut sum_ulp = 0u128;
    let mut residual_hist = BTreeMap::new();
    for row in rows {
        if discovery_only && !discovery_row(row) {
            continue;
        }
        let got = eval(candidate, row).to_bits();
        let delta = signed_ulp(row.row.expected, got);
        if delta == 0 {
            exact += 1;
        } else if collect_histogram {
            *residual_hist.entry(delta).or_insert(0) += 1;
        }
        let distance = delta.unsigned_abs();
        max_ulp = max_ulp.max(distance);
        sum_ulp += distance as u128;
    }
    ResultRow {
        candidate,
        exact,
        max_ulp,
        sum_ulp,
        residual_hist,
    }
}

fn result_order(a: &ResultRow, b: &ResultRow) -> Ordering {
    b.exact
        .cmp(&a.exact)
        .then_with(|| a.max_ulp.cmp(&b.max_ulp))
        .then_with(|| a.sum_ulp.cmp(&b.sum_ulp))
}

fn candidate_name(candidate: Candidate) -> String {
    format!(
        "{} | {} | {} | {}",
        ArgKind::ALL[candidate.arg].label(),
        candidate.exp.label(),
        SeriesKind::ALL[candidate.series].label(),
        candidate.publication.label(),
    )
}

fn cohort_score(candidate: Candidate, rows: &[Prepared], cohort: u8) -> (usize, usize, u64) {
    let mut exact = 0usize;
    let mut count = 0usize;
    let mut max_ulp = 0u64;
    for row in rows {
        if row.row.cohorts & cohort == 0 {
            continue;
        }
        count += 1;
        let got = eval(candidate, row).to_bits();
        let delta = signed_ulp(row.row.expected, got);
        if delta == 0 {
            exact += 1;
        }
        max_ulp = max_ulp.max(delta.unsigned_abs());
    }
    (exact, count, max_ulp)
}

fn predicate_score(
    candidate: Candidate,
    rows: &[Prepared],
    predicate: impl Fn(&Row) -> bool,
) -> (usize, usize, u64) {
    let mut exact = 0usize;
    let mut count = 0usize;
    let mut max_ulp = 0u64;
    for row in rows {
        if !predicate(&row.row) {
            continue;
        }
        count += 1;
        let got = eval(candidate, row).to_bits();
        let delta = signed_ulp(row.row.expected, got);
        exact += usize::from(delta == 0);
        max_ulp = max_ulp.max(delta.unsigned_abs());
    }
    (exact, count, max_ulp)
}

fn current_production(rows: &[Prepared], cohort: Option<u8>) -> (usize, usize, u64) {
    let mut exact = 0usize;
    let mut count = 0usize;
    let mut max_ulp = 0u64;
    for row in rows {
        if cohort.is_some_and(|mask| row.row.cohorts & mask == 0) {
            continue;
        }
        count += 1;
        let got = regularized_gamma_p(2.0, row.row.x).to_bits();
        let delta = signed_ulp(row.row.expected, got);
        exact += usize::from(delta == 0);
        max_ulp = max_ulp.max(delta.unsigned_abs());
    }
    (exact, count, max_ulp)
}

fn main() {
    let dir = std::env::args().nth(1).expect("G3-01-dist work directory");
    let rows: Vec<Prepared> = load_rows(&dir).into_iter().map(prepare).collect();
    let discovery_count = rows.iter().filter(|row| discovery_row(row)).count();
    let legacy_count = rows
        .iter()
        .filter(|row| row.row.cohorts & LEGACY != 0)
        .count();
    let b23_count = rows.iter().filter(|row| row.row.cohorts & B23 != 0).count();
    let b26_count = rows.iter().filter(|row| row.row.cohorts & B26 != 0).count();
    println!(
        "a=2 exact-normalizer rows: unique={} legacy={} b23={} b26={} deterministic-discovery={}",
        rows.len(),
        legacy_count,
        b23_count,
        b26_count,
        discovery_count
    );

    for (label, cohort) in [
        ("all", None),
        ("legacy", Some(LEGACY)),
        ("b23", Some(B23)),
        ("b26", Some(B26)),
    ] {
        let (hit, count, max_ulp) = current_production(&rows, cohort);
        println!("production {label:7}: {hit}/{count} max_ulp={max_ulp}");
    }

    let production_graph = Candidate {
        arg: 0,
        exp: ExpKind::Raw64TowardZero,
        series: 0,
        publication: PubKind::F64HalfFirst,
    };
    let production_graph_score = score(production_graph, &rows, false, false);
    let production_graph_disagreements = rows
        .iter()
        .filter(|row| {
            eval(production_graph, row).to_bits() != regularized_gamma_p(2.0, row.row.x).to_bits()
        })
        .count();
    println!(
        "production graph replay: {}/{} max_ulp={} disagreements_with_kernel={} | {}",
        production_graph_score.exact,
        rows.len(),
        production_graph_score.max_ulp,
        production_graph_disagreements,
        candidate_name(production_graph)
    );

    let candidates = candidates();
    println!(
        "racing {} legal candidates on discovery rows",
        candidates.len()
    );
    let mut discovery: Vec<ResultRow> = candidates
        .par_iter()
        .copied()
        .map(|candidate| score(candidate, &rows, true, false))
        .collect();
    discovery.sort_by(result_order);
    let survivors = discovery
        .iter()
        .filter(|result| result.exact == discovery_count)
        .count();
    println!("discovery exact survivors: {survivors}");
    println!("top discovery candidates:");
    for (rank, result) in discovery.iter().take(20).enumerate() {
        println!(
            "  {:02} {}/{} max={} sum={} | {}",
            rank + 1,
            result.exact,
            discovery_count,
            result.max_ulp,
            result.sum_ulp,
            candidate_name(result.candidate)
        );
    }
    println!("best discovery score by argument delivery:");
    for (arg, kind) in ArgKind::ALL.iter().enumerate() {
        let best = discovery
            .iter()
            .find(|result| result.candidate.arg == arg)
            .unwrap();
        println!(
            "  {:31} {}/{} max={} sum={}",
            kind.label(),
            best.exact,
            discovery_count,
            best.max_ulp,
            best.sum_ulp
        );
    }
    println!("best discovery score by EXP delivery/publication:");
    for kind in ExpKind::ALL {
        let best = discovery
            .iter()
            .find(|result| result.candidate.exp == kind)
            .unwrap();
        println!(
            "  {:24} {}/{} max={} sum={}",
            kind.label(),
            best.exact,
            discovery_count,
            best.max_ulp,
            best.sum_ulp
        );
    }
    println!("best discovery score by series schedule:");
    for (series, kind) in SeriesKind::ALL.iter().enumerate() {
        let best = discovery
            .iter()
            .find(|result| result.candidate.series == series)
            .unwrap();
        println!(
            "  {:32} {}/{} max={} sum={}",
            kind.label(),
            best.exact,
            discovery_count,
            best.max_ulp,
            best.sum_ulp
        );
    }

    let mut b26_rank: Vec<(usize, u64, Candidate)> = candidates
        .par_iter()
        .copied()
        .map(|candidate| {
            let (exact, _, max_ulp) = cohort_score(candidate, &rows, B26);
            (exact, max_ulp, candidate)
        })
        .collect();
    b26_rank.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));
    println!("top clean-b26 candidates:");
    for (rank, (exact, max_ulp, candidate)) in b26_rank.iter().take(10).enumerate() {
        println!(
            "  {:02} {exact}/{b26_count} max={max_ulp} | {}",
            rank + 1,
            candidate_name(*candidate)
        );
    }

    println!("re-scoring all candidates on the full bank");
    let mut full: Vec<ResultRow> = candidates
        .par_iter()
        .copied()
        .map(|candidate| score(candidate, &rows, false, false))
        .collect();
    full.sort_by(result_order);
    println!("top full-bank candidates (all legal graphs):");
    for (rank, result) in full.iter().take(20).enumerate() {
        let legacy = cohort_score(result.candidate, &rows, LEGACY);
        let b23 = cohort_score(result.candidate, &rows, B23);
        let b26 = cohort_score(result.candidate, &rows, B26);
        println!(
            "  {:02} {}/{} max={} sum={} legacy={}/{}({}) b23={}/{}({}) b26={}/{}({}) | {}",
            rank + 1,
            result.exact,
            rows.len(),
            result.max_ulp,
            result.sum_ulp,
            legacy.0,
            legacy.1,
            legacy.2,
            b23.0,
            b23.1,
            b23.2,
            b26.0,
            b26.1,
            b26.2,
            candidate_name(result.candidate)
        );
    }

    let best = &full[0];
    let best_with_histogram = score(best.candidate, &rows, false, true);
    println!(
        "best residual histogram: {:?}",
        best_with_histogram.residual_hist
    );
    for (label, needle) in [
        ("b23A", "b23A-"),
        ("b23B", "b23B"),
        ("b26A", "b26A-"),
        ("b26X", "b26X-"),
    ] {
        let subgroup = predicate_score(best.candidate, &rows, |row| row.id.contains(needle));
        println!(
            "best subgroup {label:5}: {}/{} max_ulp={}",
            subgroup.0, subgroup.1, subgroup.2
        );
    }
    for (label, lo, hi) in [
        ("tiny", 0.0, 0.01),
        ("small", 0.01, 0.1),
        ("mid", 0.1, 0.3),
        ("clean", 0.3, 1.6),
        ("upper", 1.6, 2.0),
    ] {
        let band = predicate_score(best.candidate, &rows, |row| row.x >= lo && row.x < hi);
        println!(
            "best x-band {label:5} [{lo},{hi}): {}/{} max_ulp={}",
            band.0, band.1, band.2
        );
    }
    let mut reachable = 0usize;
    let mut unreachable_examples = Vec::new();
    let finalists: Vec<Candidate> = full
        .iter()
        .take(96)
        .map(|result| result.candidate)
        .collect();
    for row in &rows {
        let hit = finalists
            .iter()
            .any(|&candidate| eval(candidate, row).to_bits() == row.row.expected);
        if hit {
            reachable += 1;
        } else if unreachable_examples.len() < 20 {
            unreachable_examples.push(format!("{} x=0x{:016x}", row.row.id, row.row.x.to_bits()));
        }
    }
    println!(
        "per-row reachability by the 96 finalist graphs: {reachable}/{}; first unreachable: {:?}",
        rows.len(),
        unreachable_examples
    );
}
