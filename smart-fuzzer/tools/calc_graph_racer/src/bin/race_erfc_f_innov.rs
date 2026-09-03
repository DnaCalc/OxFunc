//! Innovative F inverse probes (not more CF store-masks).
//! Implied-t Newton invert, AABB (c,k) pin-first, asymptotic series,
//! Cody x87 named-sites on direct. Heldouts unnamed. No landing.
//!
//!   cargo run --release --bin race_erfc_f_innov -- ../../work/w109/G3-01-dist [out-dir]

use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_sub, ext_to_f64, Ext80, CW_PC64_RN};
use std::fs;
use std::io::Write;
use std::path::PathBuf;

const P: [f64; 8] = [
    0.16506148041280876191828601e-03,
    0.15471455377139313353998665e-03,
    0.44852548090298868465196794e-04,
    -0.49177280017226285450486205e-05,
    -0.69353602078656412367801676e-05,
    -0.20508667787746282746857743e-05,
    -0.28982842617824971177267380e-06,
    -0.17272433544836633301127174e-07,
];
const Q: [f64; 8] = [
    1.0,
    0.16272656776533322859856317e+01,
    0.12040996037066026106794322e+01,
    0.52400246352158386907601472e+00,
    0.14497345252798672362384241e+00,
    0.25592517111042546492590736e-01,
    0.26869088293991371028123158e-02,
    0.13133767840925681614496481e-03,
];
const R: [f64; 9] = [
    0.145589721275038539045668824025,
    -0.273421931495426482902320421863,
    0.226008066916621506788789064272,
    -0.163571895523923805648814425592,
    0.102604312032193978662297299832,
    -0.548023266949835519254211506880e-01,
    0.241432239725390106956523668160e-01,
    -0.822062115403915116036874169600e-02,
    0.180296241564687154310619200000e-02,
];
const AA: [f64; 9] = [
    -0.45894433406309678202825375e-03,
    -0.12281298722544724287816236e-01,
    -0.91144359512342900801764781e-01,
    -0.28412489223839285652511367e-01,
    0.14083827189977123530129812e+01,
    0.11532175281537044570477189e+01,
    -0.72170903389442152112483632e+01,
    -0.19685597805218214001309225e+01,
    0.93846891504541841150916038e+01,
];
const BB: [f64; 12] = [
    1.0,
    0.25136329960926527692263725e+02,
    0.15349442087145759184067981e+03,
    -0.29971215958498680905476402e+03,
    -0.33876477506888115226730368e+04,
    0.28301829314924804988873701e+04,
    0.22979620942196507068034887e+05,
    -0.24280681522998071562462041e+05,
    -0.36680620673264731899504580e+05,
    0.42278731622295627627042436e+05,
    0.28834257644413614344549790e+03,
    0.70226293775648358646587341e+03,
];
const E0: f64 = 0.540464821348814822409610122136;
const E1: f64 = -0.261515522487415653487049835220e-01;
const E2: f64 = -0.288573438386338758794591212600e-02;

fn x87_horner(cs: &[f64], x: Ext80) -> Ext80 {
    let mut acc = ext_from_f64(0.0);
    for &c in cs.iter().rev() {
        acc = ext_add(
            &ext_mul(&acc, &x, CW_PC64_RN),
            &ext_from_f64(c),
            CW_PC64_RN,
        );
    }
    acc
}

fn pqr_t(x: f64, t: f64) -> f64 {
    let u = f::horner(&P, x);
    let v = f::horner(&Q, x);
    let mut acc = u / v;
    for &r in R.iter().rev() {
        acc = acc * t + r;
    }
    acc
}

fn pqr_x87_t(x: f64, t: f64) -> f64 {
    let xe = ext_from_f64(x);
    let te = ext_from_f64(t);
    let mut acc = ext_div(&x87_horner(&P, xe), &x87_horner(&Q, xe), CW_PC64_RN);
    for &r in R.iter().rev() {
        acc = ext_add(&ext_mul(&acc, &te, CW_PC64_RN), &ext_from_f64(r), CW_PC64_RN);
    }
    ext_to_f64(&acc, CW_PC64_RN)
}

/// Undo F = Horner([R[0]..R[8], uv], t) via Newton from t0.
fn invert_t(f_or: f64, uv: f64, t0: f64) -> Option<f64> {
    let mut t = t0;
    for _ in 0..12 {
        let mut acc = uv;
        let mut dacc = 0.0;
        for &r in R.iter().rev() {
            dacc = dacc * t + acc;
            acc = acc * t + r;
        }
        let err = acc - f_or;
        if err.abs() == 0.0 {
            return Some(t);
        }
        if dacc == 0.0 || !dacc.is_finite() {
            return None;
        }
        let t1 = t - err / dacc;
        if !t1.is_finite() {
            return None;
        }
        if (t1 - t).abs() <= (t.abs() * 2.0f64.powi(-52)).max(2.0f64.powi(-60)) {
            return Some(t1);
        }
        t = t1;
    }
    Some(t)
}

fn aabb_ck(x: f64, c: f64, k: f64) -> f64 {
    let xe = ext_from_f64(x);
    let zz = ext_mul(&xe, &xe, CW_PC64_RN);
    let z = ext_div(
        &ext_from_f64(1.0),
        &ext_add(&ext_from_f64(c), &zz, CW_PC64_RN),
        CW_PC64_RN,
    );
    let t = ext_sub(
        &ext_mul(&ext_from_f64(k), &z, CW_PC64_RN),
        &ext_from_f64(1.0),
        CW_PC64_RN,
    );
    let r = ext_div(&x87_horner(&AA, z), &x87_horner(&BB, z), CW_PC64_RN);
    let mut acc = ext_add(&ext_mul(&r, &t, CW_PC64_RN), &ext_from_f64(E2), CW_PC64_RN);
    acc = ext_add(&ext_mul(&acc, &t, CW_PC64_RN), &ext_from_f64(E1), CW_PC64_RN);
    acc = ext_add(&ext_mul(&acc, &t, CW_PC64_RN), &ext_from_f64(E0), CW_PC64_RN);
    ext_to_f64(&ext_div(&acc, &xe, CW_PC64_RN), CW_PC64_RN)
}

/// Asymptotic erfcx series: F = RPINV/x * sum (-1)^m (2m-1)!! / (2x^2)^m
fn asymp_n(x: f64, nterms: u32) -> f64 {
    let u = 0.5 / (x * x);
    let mut term = 1.0;
    let mut s = 1.0;
    for m in 1..=nterms {
        term *= -(2.0 * m as f64 - 1.0) * u;
        s += term;
    }
    f::RPINV / x * s
}

fn cody_x87_site(y: f64, site: u8) -> f64 {
    const C: [f64; 9] = [
        0.564188496988670089,
        8.88314979438837594,
        66.1191906371416295,
        298.635138197400131,
        881.95222124176909,
        1712.04761263407058,
        2051.07837782607147,
        1230.33935479799725,
        2.15311535474403846e-8,
    ];
    const D: [f64; 8] = [
        15.7449261107098347,
        117.693950891312499,
        537.181101862009858,
        1621.38957456669019,
        3290.79923573345963,
        4362.61909014324716,
        3439.36767414372164,
        1230.33935480374942,
    ];
    fn spill(x: Ext80) -> Ext80 {
        ext_from_f64(ext_to_f64(&x, CW_PC64_RN))
    }
    let ye = ext_from_f64(y);
    if y > 4.0 {
        return f::cody_erfcx_f(y);
    }
    let mut xnum = ext_mul(&ext_from_f64(C[8]), &ye, CW_PC64_RN);
    let mut xden = ye;
    if site & 1 != 0 {
        xnum = spill(xnum);
    }
    if site & 2 != 0 {
        xden = spill(xden);
    }
    for i in 0..7 {
        xnum = ext_mul(&ext_add(&xnum, &ext_from_f64(C[i]), CW_PC64_RN), &ye, CW_PC64_RN);
        xden = ext_mul(&ext_add(&xden, &ext_from_f64(D[i]), CW_PC64_RN), &ye, CW_PC64_RN);
        if site & 4 != 0 {
            xnum = spill(xnum);
        }
        if site & 8 != 0 {
            xden = spill(xden);
        }
    }
    let n = ext_add(&xnum, &ext_from_f64(C[7]), CW_PC64_RN);
    let d = ext_add(&xden, &ext_from_f64(D[7]), CW_PC64_RN);
    let mut q = ext_div(&n, &d, CW_PC64_RN);
    if site & 16 != 0 {
        q = spill(q);
    }
    ext_to_f64(&q, CW_PC64_RN)
}

fn report(label: &str, rows: &[(f64, u64)], eval: impl Fn(f64) -> f64) {
    let (m, t) = f::score_f(rows, eval);
    println!("  {label:<36} mid {}  tail {}", f::fmt_acc(&m), f::fmt_acc(&t));
}

fn main() {
    let dir = std::env::args()
        .nth(1)
        .expect("G3-01-dist (heldout files are not named)");
    assert!(!dir.contains("heldout"));
    let out = PathBuf::from(
        std::env::args()
            .nth(2)
            .unwrap_or_else(|| "../../work/w109/erfc-f-innov".into()),
    );
    fs::create_dir_all(&out).unwrap();
    let tagged = f::load_q_rows_tagged(&dir);
    let rows: Vec<(f64, u64)> = tagged.iter().map(|r| (r.z, r.qbits)).collect();
    let direct: Vec<(f64, u64)> = tagged
        .iter()
        .filter(|r| r.direct)
        .map(|r| (r.z, r.qbits))
        .collect();

    println!("## implied-t Newton invert on leftover PQR [0.5,1.5)");
    let mut tf = fs::File::create(out.join("implied-t.tsv")).unwrap();
    writeln!(tf, "z\tdirect\tulp_pub\tt_pub\tt_imp\tulp_t\tmatch").unwrap();
    let mut hist = [0usize; 8];
    let mut n_exact_t = 0usize;
    let mut n_fail = 0usize;
    let mut n_row = 0usize;
    for r in &tagged {
        if r.z < 0.5 || r.z >= 1.5 {
            continue;
        }
        let Some(fo) = f::f_or(r.z, r.qbits) else {
            continue;
        };
        let t_pub = (r.z - 3.75) / (r.z + 3.75);
        let fg = pqr_t(r.z, t_pub);
        let d0 = ulp_distance(fg, fo).unwrap_or(u64::MAX);
        if d0 == 0 {
            continue;
        }
        n_row += 1;
        let uv = f::horner(&P, r.z) / f::horner(&Q, r.z);
        let Some(t_imp) = invert_t(fo, uv, t_pub) else {
            n_fail += 1;
            continue;
        };
        let dt = ulp_distance(t_imp, t_pub).unwrap_or(u64::MAX);
        let bucket = match dt {
            0 => 0,
            1 => 1,
            2 => 2,
            3..=7 => 3,
            8..=63 => 4,
            64..=1023 => 5,
            1024..=1048576 => 6,
            _ => 7,
        };
        hist[bucket] += 1;
        if dt == 0 {
            n_exact_t += 1;
        }
        writeln!(
            tf,
            "{:.16e}\t{}\t{d0}\t{}\t{}\t{dt}\t{}",
            r.z,
            r.direct as u8,
            t_pub.to_bits(),
            t_imp.to_bits(),
            if dt == 0 { "t_pub" } else { "other" }
        )
        .unwrap();
    }
    println!(
        "  leftover PQR n={n_row} invert_fail={n_fail} implied-t exact vs published={n_exact_t}"
    );
    println!("  ulp(t_imp, t_pub) hist 0,1,2,3-7,8-63,64-1023,1k-2^20,big: {hist:?}");

    println!("\n## t-map candidates scored as x87 PQR on leftover-aware full mid");
    let maps: [(&str, fn(f64) -> f64); 6] = [
        ("(x-3.75)/(x+3.75)", |x| (x - 3.75) / (x + 3.75)),
        ("1-7.5/(x+3.75)", |x| 1.0 - 7.5 / (x + 3.75)),
        ("(1-3.75/x)/(1+3.75/x)", |x| {
            let r = 3.75 / x;
            (1.0 - r) / (1.0 + r)
        }),
        ("(x-4)/(x+4)", |x| (x - 4.0) / (x + 4.0)),
        ("(x-2.5)/(x+2.5)", |x| (x - 2.5) / (x + 2.5)),
        ("(x-13/4)/(x+13/4)", |x| {
            let a = 3.25;
            (x - a) / (x + a)
        }),
    ];
    for (name, tm) in maps {
        report(name, &rows, |z| pqr_x87_t(z, tm(z)));
    }
    report("x87 PQR t_divfirst cut1.5+AABB", &rows, |z| {
        if z < 1.5 {
            let r = 3.75 / z;
            pqr_x87_t(z, (1.0 - r) / (1.0 + r))
        } else if z < 4.0 {
            aabb_ck(z, 2.5, 13.0)
        } else {
            f::nswc_ccdd_f(z)
        }
    });
    report("x87 PQR t_pub next_up cut1.5+AABB", &rows, |z| {
        if z < 1.5 {
            pqr_x87_t(z, ((z - 3.75) / (z + 3.75)).next_up())
        } else if z < 4.0 {
            aabb_ck(z, 2.5, 13.0)
        } else {
            f::nswc_ccdd_f(z)
        }
    });
    report("x87 PQR t_pub next_down cut1.5+AABB", &rows, |z| {
        if z < 1.5 {
            pqr_x87_t(z, ((z - 3.75) / (z + 3.75)).next_down())
        } else if z < 4.0 {
            aabb_ck(z, 2.5, 13.0)
        } else {
            f::nswc_ccdd_f(z)
        }
    });

    println!("\n## AABB (c,k) pin-first then AABB-band [1.5,4)");
    let pin = 2.125;
    let fo_pin = tagged
        .iter()
        .find(|r| r.z == pin)
        .and_then(|r| f::f_or(pin, r.qbits))
        .expect("pin 2.125");
    let aabb_band: Vec<(f64, u64)> = rows
        .iter()
        .copied()
        .filter(|(z, _)| *z >= 1.5 && *z < 4.0)
        .collect();
    let cs = [2.0, 2.25, 2.5, 2.75, 3.0, 3.75, 4.0, 5.0];
    let ks = [8.0, 10.0, 12.0, 13.0, 14.0, 16.0, 20.0];
    let mut best_pin = (u64::MAX, 0.0, 0.0, 0usize);
    for &c in &cs {
        for &k in &ks {
            let dp = ulp_distance(aabb_ck(pin, c, k), fo_pin).unwrap_or(u64::MAX);
            let (m, _) = f::score_f(&aabb_band, |z| aabb_ck(z, c, k));
            if dp < best_pin.0 || (dp == best_pin.0 && m.exact > best_pin.3) {
                best_pin = (dp, c, k, m.exact);
                println!(
                    "  c={c} k={k} pin_ulp={dp} AABB-band {}",
                    f::fmt_acc(&m)
                );
            }
        }
    }
    println!(
        "  best pin c={} k={} pin_ulp={} AABB exact={}",
        best_pin.1, best_pin.2, best_pin.0, best_pin.3
    );

    println!("\n## truncated asymptotic series (tail F)");
    println!("  {:>4} {:>22} {:>22}", "n", "native tail", "direct tail");
    for n in [1u32, 2, 4, 8, 12, 16, 21, 24, 32, 40] {
        let (_, t) = f::score_f(&rows, |z| asymp_n(z, n));
        let (_, td) = f::score_f(&direct, |z| asymp_n(z, n));
        println!("  {n:>4} {:>22} {:>22}", f::fmt_acc(&t), f::fmt_acc(&td));
    }

    println!("\n## Cody x87 named-site 5-bit on direct mid");
    let dmid: Vec<(f64, u64)> = direct
        .iter()
        .copied()
        .filter(|(z, _)| *z >= 0.5 && *z < 4.0)
        .collect();
    let mut best_c = (0u8, 0usize);
    for site in 0u8..=31 {
        let (m, _) = f::score_f(&dmid, |z| cody_x87_site(z, site));
        if m.exact > best_c.1 {
            best_c = (site, m.exact);
            println!("  site=0x{site:02x} mid {}", f::fmt_acc(&m));
        }
    }
    println!("  best Cody site=0x{:02x} exact={}", best_c.0, best_c.1);

    println!("\n## close-z pairs among direct leftover (ulp(z)<8)");
    let mut leftover_d: Vec<(f64, u64, u64)> = Vec::new();
    for r in tagged.iter().filter(|r| r.direct && r.z >= 0.5 && r.z < 4.0) {
        let Some(fo) = f::f_or(r.z, r.qbits) else {
            continue;
        };
        let tpub = (r.z - 3.75) / (r.z + 3.75);
        let fg = if r.z < 1.5 {
            pqr_x87_t(r.z, tpub)
        } else {
            aabb_ck(r.z, 2.5, 13.0)
        };
        let d = ulp_distance(fg, fo).unwrap_or(u64::MAX);
        if d != 0 {
            leftover_d.push((r.z, fo.to_bits(), d));
        }
    }
    leftover_d.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    let mut npair = 0usize;
    let mut same_res = 0usize;
    for w in leftover_d.windows(2) {
        let dz = ulp_distance(w[0].0, w[1].0).unwrap_or(u64::MAX);
        if dz == 0 || dz > 8 {
            continue;
        }
        npair += 1;
        if w[0].2 == w[1].2 {
            same_res += 1;
        }
        if npair <= 12 {
            println!(
                "  z={:.16e} vs {:.16e} dz_ulp={dz} res=({},{})",
                w[0].0, w[1].0, w[0].2, w[1].2
            );
        }
    }
    println!("  close pairs n={npair} same-residual-ulp={same_res}");
    println!("artifacts in {}", out.display());
}
