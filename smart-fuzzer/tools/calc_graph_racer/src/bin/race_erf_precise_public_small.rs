//! W109 ERF.PRECISE clean-room race over public small-argument rational graphs.
//!
//! Families:
//! - NSWC/DCDFLIB `erfc1.f`, |x| <= 0.5 rational (used by the published
//!   TOMS-654 package's half-shape special-case substrate).
//! - Sun fdlibm `s_erf.c`, |x| < 0.84375 rational.
//!
//! The race changes arithmetic/storage graphs only.  Coefficients and algebra
//! remain those in the published sources.  It reads only the legacy discovery
//! banks named below and cannot open `answers-b9heldout.json`.
//!
//! Usage:
//!   race_erf_precise_public_small <G3-01-dist-directory>

use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use rx::{CW_PC53_RN, CW_PC64_RN, Ext80, ext_add, ext_div, ext_from_f64, ext_mul, ext_to_f64};
use std::collections::BTreeMap;

const DISCOVERY_BANKS: [&str; 7] = [
    "answers-b9train.json",
    "answers-erfp.json",
    "answers-erfm.json",
    "answers-b8erf.json",
    "answers-b7erf.json",
    "answers-b11.json",
    "answers-b10.json",
];

fn ef(x: f64) -> Ext80 {
    ext_from_f64(x)
}

fn dbl(x: &Ext80) -> f64 {
    ext_to_f64(x, CW_PC64_RN)
}

fn spill(x: Ext80) -> Ext80 {
    ef(dbl(&x))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Family {
    NswcErfc1,
    Fdlibm,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Arithmetic {
    /// One register-continuous PC=64 expression.
    X87Continuous,
    /// PC=64 operation, then a binary64 store after every arithmetic node.
    X87EveryOp53,
    /// PC=64 multiply+add, then a binary64 store after each Horner stage.
    X87HornerStage53,
    /// PC=53 x87 operations: a single binary53 rounding per arithmetic node.
    X87Pc53,
    /// Native Rust f64 arithmetic (SSE2 control).
    Native53,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TMode {
    X87Continuous,
    X87DoubleRounded,
    X87Pc53,
    Native53,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FinalGraph {
    XTimesRatioContinuous,
    XTimesRatioStored,
    XTimesTopThenDivideContinuous,
    XTimesTopThenDivideStored,
    XDivideBotThenTopContinuous,
    XDivideBotThenTopStored,
    FdlibmXPlusXYContinuous,
    FdlibmXPlusXYStored,
    FdlibmXTimesOnePlusYContinuous,
    FdlibmXTimesOnePlusYStored,
    FdlibmXPlusXrOverSContinuous,
    FdlibmXPlusXrOverSStored,
}

#[derive(Clone, Copy, Debug)]
struct Cfg {
    family: Family,
    t: TMode,
    numerator: Arithmetic,
    denominator: Arithmetic,
    final_graph: FinalGraph,
    nswc_fold_final_one: bool,
}

fn add(a: &Ext80, b: &Ext80, mode: Arithmetic) -> Ext80 {
    match mode {
        Arithmetic::X87Continuous | Arithmetic::X87HornerStage53 => ext_add(a, b, CW_PC64_RN),
        Arithmetic::X87EveryOp53 => spill(ext_add(a, b, CW_PC64_RN)),
        Arithmetic::X87Pc53 => ext_add(a, b, CW_PC53_RN),
        Arithmetic::Native53 => ef(dbl(a) + dbl(b)),
    }
}

fn mul(a: &Ext80, b: &Ext80, mode: Arithmetic) -> Ext80 {
    match mode {
        Arithmetic::X87Continuous | Arithmetic::X87HornerStage53 => ext_mul(a, b, CW_PC64_RN),
        Arithmetic::X87EveryOp53 => spill(ext_mul(a, b, CW_PC64_RN)),
        Arithmetic::X87Pc53 => ext_mul(a, b, CW_PC53_RN),
        Arithmetic::Native53 => ef(dbl(a) * dbl(b)),
    }
}

fn madd(acc: &Ext80, x: &Ext80, c: f64, mode: Arithmetic) -> Ext80 {
    let value = add(&mul(acc, x, mode), &ef(c), mode);
    if matches!(mode, Arithmetic::X87HornerStage53) {
        spill(value)
    } else {
        value
    }
}

fn t_value(x: f64, mode: TMode) -> Ext80 {
    match mode {
        TMode::X87Continuous => ext_mul(&ef(x), &ef(x), CW_PC64_RN),
        TMode::X87DoubleRounded => spill(ext_mul(&ef(x), &ef(x), CW_PC64_RN)),
        TMode::X87Pc53 => ext_mul(&ef(x), &ef(x), CW_PC53_RN),
        TMode::Native53 => ef(x * x),
    }
}

const NSWC_A: [f64; 5] = [
    0.771058495001320e-04,
    -0.133733772997339e-02,
    0.323076579225834e-01,
    0.479137145607681e-01,
    0.128379167095513e+00,
];
const NSWC_B: [f64; 3] = [
    0.301048631703895e-02,
    0.538971687740286e-01,
    0.375795757275549e+00,
];

const FDLIBM_P: [f64; 5] = [
    1.28379167095512558561e-01,
    -3.25042107247001499370e-01,
    -2.84817495755985104766e-02,
    -5.77027029648944159157e-03,
    -2.37630166566501626084e-05,
];
const FDLIBM_Q: [f64; 5] = [
    3.97917223959155352819e-01,
    6.50222499887672944485e-02,
    5.08130628187576562776e-03,
    1.32494738004321644526e-04,
    -3.96022827877536812320e-06,
];

fn horner(coefficients: &[f64], x: &Ext80, mode: Arithmetic) -> Ext80 {
    let mut value = ef(coefficients[0]);
    for &coefficient in &coefficients[1..] {
        value = madd(&value, x, coefficient, mode);
    }
    value
}

fn nswc_parts(t: &Ext80, cfg: &Cfg) -> (Ext80, Ext80) {
    let mut top = if cfg.nswc_fold_final_one {
        let mut coefficients = NSWC_A;
        coefficients[4] += 1.0;
        horner(&coefficients, t, cfg.numerator)
    } else {
        let value = horner(&NSWC_A, t, cfg.numerator);
        add(&value, &ef(1.0), cfg.numerator)
    };
    if matches!(cfg.numerator, Arithmetic::X87HornerStage53) {
        top = spill(top);
    }
    let mut bot = horner(&NSWC_B, t, cfg.denominator);
    bot = mul(&bot, t, cfg.denominator);
    bot = add(&bot, &ef(1.0), cfg.denominator);
    if matches!(cfg.denominator, Arithmetic::X87HornerStage53) {
        bot = spill(bot);
    }
    (top, bot)
}

fn fdlibm_parts(t: &Ext80, cfg: &Cfg) -> (Ext80, Ext80) {
    // The C source is right-nested: pp0 + z*(pp1 + z*(... + z*pp4)).
    let mut p = FDLIBM_P;
    p.reverse();
    let r = horner(&p, t, cfg.numerator);
    let mut q = FDLIBM_Q;
    q.reverse();
    let q_inner = horner(&q, t, cfg.denominator);
    let mut s = add(
        &mul(t, &q_inner, cfg.denominator),
        &ef(1.0),
        cfg.denominator,
    );
    if matches!(cfg.denominator, Arithmetic::X87HornerStage53) {
        s = spill(s);
    }
    (r, s)
}

fn maybe_store(value: Ext80, stored: bool) -> Ext80 {
    if stored { spill(value) } else { value }
}

fn eval(x: f64, cfg: &Cfg) -> f64 {
    let xe = ef(x);
    let t = t_value(x, cfg.t);
    let (p, q) = match cfg.family {
        Family::NswcErfc1 => nswc_parts(&t, cfg),
        Family::Fdlibm => fdlibm_parts(&t, cfg),
    };
    let v = match cfg.final_graph {
        FinalGraph::XTimesRatioContinuous | FinalGraph::XTimesRatioStored => {
            let y = ext_div(&p, &q, CW_PC64_RN);
            let y = maybe_store(y, matches!(cfg.final_graph, FinalGraph::XTimesRatioStored));
            ext_mul(&xe, &y, CW_PC64_RN)
        }
        FinalGraph::XTimesTopThenDivideContinuous | FinalGraph::XTimesTopThenDivideStored => {
            let numerator = ext_mul(&xe, &p, CW_PC64_RN);
            let numerator = maybe_store(
                numerator,
                matches!(cfg.final_graph, FinalGraph::XTimesTopThenDivideStored),
            );
            ext_div(&numerator, &q, CW_PC64_RN)
        }
        FinalGraph::XDivideBotThenTopContinuous | FinalGraph::XDivideBotThenTopStored => {
            let ratio = ext_div(&xe, &q, CW_PC64_RN);
            let ratio = maybe_store(
                ratio,
                matches!(cfg.final_graph, FinalGraph::XDivideBotThenTopStored),
            );
            ext_mul(&ratio, &p, CW_PC64_RN)
        }
        FinalGraph::FdlibmXPlusXYContinuous | FinalGraph::FdlibmXPlusXYStored => {
            let y = ext_div(&p, &q, CW_PC64_RN);
            let y = maybe_store(
                y,
                matches!(cfg.final_graph, FinalGraph::FdlibmXPlusXYStored),
            );
            ext_add(&xe, &ext_mul(&xe, &y, CW_PC64_RN), CW_PC64_RN)
        }
        FinalGraph::FdlibmXTimesOnePlusYContinuous | FinalGraph::FdlibmXTimesOnePlusYStored => {
            let y = ext_div(&p, &q, CW_PC64_RN);
            let one_plus_y = ext_add(&ef(1.0), &y, CW_PC64_RN);
            let one_plus_y = maybe_store(
                one_plus_y,
                matches!(cfg.final_graph, FinalGraph::FdlibmXTimesOnePlusYStored),
            );
            ext_mul(&xe, &one_plus_y, CW_PC64_RN)
        }
        FinalGraph::FdlibmXPlusXrOverSContinuous | FinalGraph::FdlibmXPlusXrOverSStored => {
            let xr = ext_mul(&xe, &p, CW_PC64_RN);
            let xr = maybe_store(
                xr,
                matches!(cfg.final_graph, FinalGraph::FdlibmXPlusXrOverSStored),
            );
            ext_add(&xe, &ext_div(&xr, &q, CW_PC64_RN), CW_PC64_RN)
        }
    };
    dbl(&v)
}

fn ordered(bits: u64) -> i64 {
    let signed = bits as i64;
    if signed < 0 { !signed } else { signed }
}

fn load_rows(dir: &str) -> Vec<(f64, u64)> {
    let mut rows = BTreeMap::new();
    for name in DISCOVERY_BANKS {
        let path = format!("{dir}/{name}");
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        let bank: WitnessSet =
            serde_json::from_str(&text).unwrap_or_else(|e| panic!("failed to parse {path}: {e}"));
        for witness in &bank.witnesses {
            let x = match &witness.args[0] {
                WitnessArg::Scalar(s) => parse_bits_hex(s).expect("scalar bits"),
                _ => continue,
            };
            let Some(expected) = parse_bits_hex(&witness.expected_bits) else {
                continue;
            };
            if x > 0.0 && x < 0.5 {
                if let Some(old) = rows.insert(x.to_bits(), expected.to_bits()) {
                    assert_eq!(old, expected.to_bits(), "conflicting oracle bits");
                }
            }
        }
    }
    rows.into_iter()
        .map(|(x, expected)| (f64::from_bits(x), expected))
        .collect()
}

#[derive(Debug)]
struct Score {
    exact: usize,
    max_ulp: i64,
    sum_abs_ulp: u64,
    cfg: Cfg,
}

fn main() {
    let dir = std::env::args().nth(1).expect("G3-01-dist directory");
    let rows = load_rows(&dir);
    println!(
        "{} distinct z<0.5 discovery rows; heldout path is absent from the reader",
        rows.len()
    );
    let arithmetic = [
        Arithmetic::X87Continuous,
        Arithmetic::X87EveryOp53,
        Arithmetic::X87HornerStage53,
        Arithmetic::X87Pc53,
        Arithmetic::Native53,
    ];
    let t_modes = [
        TMode::X87Continuous,
        TMode::X87DoubleRounded,
        TMode::X87Pc53,
        TMode::Native53,
    ];
    let families = [Family::NswcErfc1, Family::Fdlibm];
    let nswc_finals = [
        FinalGraph::XTimesRatioContinuous,
        FinalGraph::XTimesRatioStored,
        FinalGraph::XTimesTopThenDivideContinuous,
        FinalGraph::XTimesTopThenDivideStored,
        FinalGraph::XDivideBotThenTopContinuous,
        FinalGraph::XDivideBotThenTopStored,
    ];
    let fdlibm_finals = [
        FinalGraph::FdlibmXPlusXYContinuous,
        FinalGraph::FdlibmXPlusXYStored,
        FinalGraph::FdlibmXTimesOnePlusYContinuous,
        FinalGraph::FdlibmXTimesOnePlusYStored,
        FinalGraph::FdlibmXPlusXrOverSContinuous,
        FinalGraph::FdlibmXPlusXrOverSStored,
    ];
    let mut scores = Vec::new();
    for family in families {
        let finals = match family {
            Family::NswcErfc1 => &nswc_finals,
            Family::Fdlibm => &fdlibm_finals,
        };
        for t in t_modes {
            for numerator in arithmetic {
                for denominator in arithmetic {
                    for &final_graph in finals {
                        for nswc_fold_final_one in [false, true] {
                            if family == Family::Fdlibm && nswc_fold_final_one {
                                continue;
                            }
                            let cfg = Cfg {
                                family,
                                t,
                                numerator,
                                denominator,
                                final_graph,
                                nswc_fold_final_one,
                            };
                            let (mut exact, mut max_ulp, mut sum_abs_ulp) = (0usize, 0i64, 0u64);
                            for &(x, expected) in &rows {
                                let got = eval(x, &cfg).to_bits();
                                let delta = ordered(expected) - ordered(got);
                                exact += usize::from(delta == 0);
                                let abs = delta.unsigned_abs();
                                max_ulp = max_ulp.max(abs as i64);
                                sum_abs_ulp = sum_abs_ulp.saturating_add(abs);
                            }
                            scores.push(Score {
                                exact,
                                max_ulp,
                                sum_abs_ulp,
                                cfg,
                            });
                        }
                    }
                }
            }
        }
    }
    scores.sort_by_key(|s| (usize::MAX - s.exact, s.max_ulp, s.sum_abs_ulp));
    println!("raced {} source-preserving graphs", scores.len());
    for (rank, score) in scores.iter().take(24).enumerate() {
        println!(
            "#{:02} exact={:4}/{} max_ulp={} sum_abs_ulp={} {:?}",
            rank + 1,
            score.exact,
            rows.len(),
            score.max_ulp,
            score.sum_abs_ulp,
            score.cfg
        );
    }
    println!("best score by public family:");
    for family in families {
        let score = scores
            .iter()
            .find(|score| score.cfg.family == family)
            .unwrap();
        println!(
            "  {:?}: exact={}/{} max_ulp={} sum_abs_ulp={} {:?}",
            family,
            score.exact,
            rows.len(),
            score.max_ulp,
            score.sum_abs_ulp,
            score.cfg
        );
    }
}
