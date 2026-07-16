//! W109 G3-01: identify Excel's incomplete-gamma kernel op-graph with TRUE x87
//! extended arithmetic (Ext80 @ CW_PC64_RN + the confirmed fFEXP/fFLN chains).
//!
//! Stage A: a in {1, 2} slices (Excel GAMMALN is exactly 0 there, so the
//! prefactor has NO lgamma unknown). Pins: series form (NR vs Cephes), CF form
//! (NR-Lentz vs Cephes rational), gser/gcf switch rule, prefactor staging
//! (ln/arg spills, op order), complement staging, convergence eps/test.
//!
//! Data: answered witness sets from smart-fuzzer/work/w109/G3-01-dist
//!   - answers-gammadist-modern.json  GAMMA.DIST(x, a, beta, TRUE) = P(a, x/beta)
//!   - answers-chidist.json           CHIDIST(x, df)             = Q(df/2, x/2)
//!
//! Usage: check_igamma <work-dir> [--amax N]

use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use rx::{
    CW_PC64_RN, Ext80, ext_abs, ext_add, ext_div, ext_f2xm1, ext_from_f64, ext_fyl2x, ext_l2e,
    ext_ln2, ext_mul, ext_one, ext_rndint, ext_scale, ext_sub, ext_to_f64,
};

const CW: u16 = CW_PC64_RN;

fn ef(x: f64) -> Ext80 {
    ext_from_f64(x)
}
fn dbl(x: &Ext80) -> f64 {
    ext_to_f64(x, CW)
}

/// fFEXP chain on an extended input (mirrors the proven exp_via_raw).
fn exp_ext(x: &Ext80) -> Ext80 {
    let t = ext_mul(x, &ext_l2e(), CW);
    let k = ext_rndint(&t, CW);
    let f = ext_sub(&t, &k, CW);
    let neg = dbl(&f) < 0.0;
    let w = ext_f2xm1(&ext_abs(&f, CW), CW);
    let mut m = ext_add(&w, &ext_one(), CW);
    if neg {
        m = ext_div(&ext_one(), &m, CW);
    }
    ext_scale(&m, &k, CW)
}

/// fFLN chain on an extended input.
fn ln_ext(x: &Ext80) -> Ext80 {
    ext_fyl2x(&ext_ln2(), x, CW)
}

fn ext_lt(a: &Ext80, b: &Ext80) -> bool {
    dbl(&ext_sub(a, b, CW)) < 0.0
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Series {
    Nr,     // sum = 1/a + terms                  (NR gser)
    Cephes, // ans = 1 + terms; return ans*ax/a   (Cephes igam)
}
#[derive(Clone, Copy, Debug, PartialEq)]
enum Cf {
    NrLentz, // modified Lentz on b = x+1-a       (NR gcf)
    Cephes,  // rational recurrence + big renorm  (Cephes igamc)
}
#[derive(Clone, Copy, Debug, PartialEq)]
enum Switch {
    CephesEntry, // CF when x > 1 && x > a
    NrA1,        // CF when x >= a + 1
    XgtA,        // CF when x > a
}
#[derive(Clone, Copy, Debug, PartialEq)]
enum PrefOrder {
    Nr,     // -x + a*ln(x) - gln
    Cephes, // a*ln(x) - x - gln
}
#[derive(Clone, Copy, Debug, PartialEq)]
enum Comp {
    Dbl, // complement on the double-rounded value: 1 - RN53(v)
    Ext, // complement in extended: RN53(1 - v_ext)
}

#[derive(Clone, Copy, Debug)]
struct Cfg {
    series: Series,
    cf: Cf,
    switch_: Switch,
    pref: PrefOrder,
    ln_spill: bool,
    arg_spill: bool,
    term_div_first: bool, // term *= x/r (true) vs term = term*x/r (false)
    comp: Comp,
    eps: f64,
}

fn prefactor(cfg: &Cfg, a: f64, x: f64, gln: &Ext80) -> Ext80 {
    let mut lnx = ln_ext(&ef(x));
    if cfg.ln_spill {
        lnx = ef(dbl(&lnx));
    }
    let alnx = ext_mul(&ef(a), &lnx, CW);
    let mut arg = match cfg.pref {
        PrefOrder::Nr => ext_sub(&ext_add(&ef(-x), &alnx, CW), gln, CW),
        PrefOrder::Cephes => ext_sub(&ext_sub(&alnx, &ef(x), CW), gln, CW),
    };
    if cfg.arg_spill {
        arg = ef(dbl(&arg));
    }
    exp_ext(&arg)
}

/// Lower series; returns P(a,x) as Ext80 (before the publish rounding).
fn gser(cfg: &Cfg, a: f64, x: f64, gln: &Ext80) -> Ext80 {
    let eps = ef(cfg.eps);
    let ax = prefactor(cfg, a, x, gln);
    match cfg.series {
        Series::Nr => {
            let mut ap = ef(a);
            let mut sum = ext_div(&ext_one(), &ef(a), CW);
            let mut del = sum;
            for _ in 0..700 {
                ap = ext_add(&ap, &ext_one(), CW);
                del = if cfg.term_div_first {
                    ext_mul(&del, &ext_div(&ef(x), &ap, CW), CW)
                } else {
                    ext_div(&ext_mul(&del, &ef(x), CW), &ap, CW)
                };
                sum = ext_add(&sum, &del, CW);
                if ext_lt(&ext_abs(&del, CW), &ext_mul(&ext_abs(&sum, CW), &eps, CW)) {
                    break;
                }
            }
            ext_mul(&sum, &ax, CW)
        }
        Series::Cephes => {
            let mut r = ef(a);
            let mut c = ext_one();
            let mut ans = ext_one();
            for _ in 0..700 {
                r = ext_add(&r, &ext_one(), CW);
                c = if cfg.term_div_first {
                    ext_mul(&c, &ext_div(&ef(x), &r, CW), CW)
                } else {
                    ext_div(&ext_mul(&c, &ef(x), CW), &r, CW)
                };
                ans = ext_add(&ans, &c, CW);
                if !ext_lt(&eps, &ext_div(&c, &ans, CW)) {
                    break; // while (c/ans > MACHEP)
                }
            }
            ext_div(&ext_mul(&ans, &ax, CW), &ef(a), CW)
        }
    }
}

/// Upper CF; returns Q(a,x) as Ext80.
fn gcf(cfg: &Cfg, a: f64, x: f64, gln: &Ext80) -> Ext80 {
    let eps = ef(cfg.eps);
    let ax = prefactor(cfg, a, x, gln);
    match cfg.cf {
        Cf::NrLentz => {
            let fpmin = ef(1e-300);
            let mut b = ext_sub(&ext_add(&ef(x), &ext_one(), CW), &ef(a), CW);
            let mut c = ext_div(&ext_one(), &fpmin, CW);
            let mut d = ext_div(&ext_one(), &b, CW);
            let mut h = d;
            for i in 1..=700 {
                let i_e = ef(i as f64);
                let an = ext_mul(&ef(-(i as f64)), &ext_sub(&i_e, &ef(a), CW), CW);
                b = ext_add(&b, &ef(2.0), CW);
                d = ext_add(&ext_mul(&an, &d, CW), &b, CW);
                if ext_lt(&ext_abs(&d, CW), &fpmin) {
                    d = fpmin;
                }
                c = ext_add(&b, &ext_div(&an, &c, CW), CW);
                if ext_lt(&ext_abs(&c, CW), &fpmin) {
                    c = fpmin;
                }
                d = ext_div(&ext_one(), &d, CW);
                let del = ext_mul(&d, &c, CW);
                h = ext_mul(&h, &del, CW);
                if ext_lt(&ext_abs(&ext_sub(&del, &ext_one(), CW), CW), &eps) {
                    break;
                }
            }
            ext_mul(&ax, &h, CW)
        }
        Cf::Cephes => {
            let big = ef(4.503599627370496e15);
            let biginv = ef(2.220_446_049_250_313e-16);
            let mut y = ext_sub(&ext_one(), &ef(a), CW);
            let mut z = ext_add(&ext_add(&ef(x), &y, CW), &ext_one(), CW);
            let mut c = ef(0.0);
            let mut pkm2 = ext_one();
            let mut qkm2 = ef(x);
            let mut pkm1 = ext_add(&ef(x), &ext_one(), CW);
            let mut qkm1 = ext_mul(&z, &ef(x), CW);
            let mut ans = ext_div(&pkm1, &qkm1, CW);
            for _ in 0..700 {
                c = ext_add(&c, &ext_one(), CW);
                y = ext_add(&y, &ext_one(), CW);
                z = ext_add(&z, &ef(2.0), CW);
                let yc = ext_mul(&y, &c, CW);
                let pk = ext_sub(&ext_mul(&pkm1, &z, CW), &ext_mul(&pkm2, &yc, CW), CW);
                let qk = ext_sub(&ext_mul(&qkm1, &z, CW), &ext_mul(&qkm2, &yc, CW), CW);
                let t;
                if dbl(&qk) != 0.0 {
                    let r = ext_div(&pk, &qk, CW);
                    t = ext_abs(&ext_div(&ext_sub(&ans, &r, CW), &r, CW), CW);
                    ans = r;
                } else {
                    t = ext_one();
                }
                pkm2 = pkm1;
                pkm1 = pk;
                qkm2 = qkm1;
                qkm1 = qk;
                if ext_lt(&big, &ext_abs(&pk, CW)) {
                    pkm2 = ext_mul(&pkm2, &biginv, CW);
                    pkm1 = ext_mul(&pkm1, &biginv, CW);
                    qkm2 = ext_mul(&qkm2, &biginv, CW);
                    qkm1 = ext_mul(&qkm1, &biginv, CW);
                }
                if !ext_lt(&eps, &t) {
                    break; // while t > MACHEP
                }
            }
            ext_mul(&ans, &ax, CW)
        }
    }
}

fn use_cf(cfg: &Cfg, a: f64, x: f64) -> bool {
    match cfg.switch_ {
        Switch::CephesEntry => x > 1.0 && x > a,
        Switch::NrA1 => x >= a + 1.0,
        Switch::XgtA => x > a,
    }
}

/// Published P(a,x).
fn p_publish(cfg: &Cfg, a: f64, x: f64, gln: &Ext80) -> f64 {
    if x <= 0.0 {
        return 0.0;
    }
    if use_cf(cfg, a, x) {
        let q = gcf(cfg, a, x, gln);
        match cfg.comp {
            Comp::Dbl => 1.0 - dbl(&q),
            Comp::Ext => dbl(&ext_sub(&ext_one(), &q, CW)),
        }
    } else {
        dbl(&gser(cfg, a, x, gln))
    }
}

/// Published Q(a,x).
fn q_publish(cfg: &Cfg, a: f64, x: f64, gln: &Ext80) -> f64 {
    if x <= 0.0 {
        return 1.0;
    }
    if use_cf(cfg, a, x) {
        dbl(&gcf(cfg, a, x, gln))
    } else {
        let p = gser(cfg, a, x, gln);
        match cfg.comp {
            Comp::Dbl => 1.0 - dbl(&p),
            Comp::Ext => dbl(&ext_sub(&ext_one(), &p, CW)),
        }
    }
}

fn ulp_signed(a: f64, b: f64) -> i64 {
    fn toi(x: f64) -> i64 {
        let i = x.to_bits() as i64;
        if i < 0 { i64::MIN.wrapping_sub(i).wrapping_neg() ^ i64::MIN } else { i }
    }
    // total-order mapping: for negatives flip
    fn key(x: f64) -> i64 {
        let i = x.to_bits() as i64;
        if i < 0 { !i } else { i }
    }
    let _ = toi;
    key(a) - key(b)
}

struct Row {
    id: String,
    a: f64,
    x: f64,
    is_p: bool, // P view (GAMMA.DIST) or Q view (CHIDIST)
    excel: f64,
}

fn load_rows(dir: &str, amax: f64) -> Vec<Row> {
    let mut rows = Vec::new();
    let gd: WitnessSet = serde_json::from_str(
        &std::fs::read_to_string(format!("{dir}/answers-gammadist-modern.json")).unwrap(),
    )
    .unwrap();
    for w in &gd.witnesses {
        let (Some(id), Some(excel)) = (&w.id, parse_bits_hex(&w.expected_bits)) else {
            continue;
        };
        let s = |i: usize| -> f64 {
            match &w.args[i] {
                WitnessArg::Scalar(s) => parse_bits_hex(s).unwrap(),
                _ => f64::NAN,
            }
        };
        let (x, a, beta) = (s(0), s(1), s(2));
        if a <= amax {
            rows.push(Row { id: id.clone(), a, x: x / beta, is_p: true, excel });
        }
    }
    let chi: WitnessSet = serde_json::from_str(
        &std::fs::read_to_string(format!("{dir}/answers-chidist.json")).unwrap(),
    )
    .unwrap();
    for w in &chi.witnesses {
        let (Some(id), Some(excel)) = (&w.id, parse_bits_hex(&w.expected_bits)) else {
            continue;
        };
        let s = |i: usize| -> f64 {
            match &w.args[i] {
                WitnessArg::Scalar(s) => parse_bits_hex(s).unwrap(),
                _ => f64::NAN,
            }
        };
        let (x, df) = (s(0), s(1));
        let a = df / 2.0;
        if a <= amax && (a == 1.0 || a == 2.0 || a == 3.0 || a == 5.0) {
            rows.push(Row { id: id.clone(), a, x: x / 2.0, is_p: false, excel });
        }
    }
    rows
}

fn main() {
    let dir = std::env::args().nth(1).expect("work dir");
    let amax: f64 = std::env::args()
        .nth(2)
        .map(|s| s.parse().unwrap())
        .unwrap_or(2.0);
    let rows = load_rows(&dir, amax);
    println!("loaded {} rows with a <= {amax} (P view + Q view)", rows.len());

    let gln0 = ef(0.0);
    // Stage A only supports gln = 0 slices (a = 1, 2); ln 2 handled in stage B.
    let usable: Vec<&Row> = rows.iter().filter(|r| r.a == 1.0 || r.a == 2.0).collect();
    println!("stage-A usable (gln = 0): {}", usable.len());

    let mut results: Vec<(usize, i64, Cfg, Vec<(String, f64, f64, i64)>)> = Vec::new();
    for series in [Series::Nr, Series::Cephes] {
        for cf in [Cf::NrLentz, Cf::Cephes] {
            for switch_ in [Switch::CephesEntry, Switch::NrA1, Switch::XgtA] {
                for pref in [PrefOrder::Nr, PrefOrder::Cephes] {
                    for ln_spill in [false, true] {
                        for arg_spill in [false, true] {
                            for term_div_first in [true, false] {
                                for comp in [Comp::Dbl, Comp::Ext] {
                                    for eps in
                                        [1.110_223_024_625_156_5e-16, 2.220_446_049_250_313e-16, 1e-15]
                                    {
                                        let cfg = Cfg {
                                            series,
                                            cf,
                                            switch_,
                                            pref,
                                            ln_spill,
                                            arg_spill,
                                            term_div_first,
                                            comp,
                                            eps,
                                        };
                                        let mut exact = 0usize;
                                        let mut maxd = 0i64;
                                        let mut misses = Vec::new();
                                        for r in &usable {
                                            let v = if r.is_p {
                                                p_publish(&cfg, r.a, r.x, &gln0)
                                            } else {
                                                q_publish(&cfg, r.a, r.x, &gln0)
                                            };
                                            let d = ulp_signed(r.excel, v);
                                            if d == 0 {
                                                exact += 1;
                                            } else {
                                                maxd = maxd.max(d.abs());
                                                if misses.len() < 40 {
                                                    misses.push((r.id.clone(), r.a, r.x, d));
                                                }
                                            }
                                        }
                                        results.push((exact, maxd, cfg, misses));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    results.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
    println!("\ntop configs (exact/{}):", usable.len());
    for (exact, maxd, cfg, _) in results.iter().take(14) {
        println!(
            "  {exact:4}/{} max|d|={maxd:3}  {:?} {:?} {:?} {:?} lnS={} argS={} divF={} {:?} eps={:.3e}",
            usable.len(),
            cfg.series,
            cfg.cf,
            cfg.switch_,
            cfg.pref,
            cfg.ln_spill as u8,
            cfg.arg_spill as u8,
            cfg.term_div_first as u8,
            cfg.comp,
            cfg.eps
        );
    }
    let (exact, maxd, cfg, misses) = &results[0];
    println!("\nbest cfg misses ({} exact, max {maxd}): {:?}", exact, cfg);
    for (id, a, x, d) in misses {
        println!("  d={d:+3}  a={a} x={x:<12} {id}");
    }
}
