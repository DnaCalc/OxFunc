//! W109 G3-01 x87-exp last mile: race the CORRECTED gratio staging (forward-sum
//! Taylor, CR-quality normalizer) with exp/log models {std, RN53(x87 chain)}
//! against the full gamma-side corpus.
//!
//! Usage: check_gratio_x87 <work-dir>

use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use rx::{
    CW_PC64_RN, Ext80, ext_abs, ext_add, ext_div, ext_f2xm1, ext_from_f64, ext_fyl2x, ext_l2e,
    ext_ln2, ext_mul, ext_one, ext_rndint, ext_scale, ext_sub, ext_to_f64,
};
use std::collections::BTreeMap;

const CW: u16 = CW_PC64_RN;

fn x87_exp(x: f64) -> f64 {
    let xe = ext_from_f64(x);
    let t = ext_mul(&xe, &ext_l2e(), CW);
    let k = ext_rndint(&t, CW);
    let f = ext_sub(&t, &k, CW);
    let neg = ext_to_f64(&f, CW) < 0.0;
    let w = ext_f2xm1(&ext_abs(&f, CW), CW);
    let mut m = ext_add(&w, &ext_one(), CW);
    if neg {
        m = ext_div(&ext_one(), &m, CW);
    }
    ext_to_f64(&ext_scale(&m, &k, CW), CW)
}

fn x87_ln(x: f64) -> f64 {
    ext_to_f64(&ext_fyl2x(&ext_ln2(), &ext_from_f64(x), CW), CW)
}

fn norm_gamma(a: f64) -> f64 {
    if a >= 1.0 && a <= 22.0 && a == a.floor() {
        let mut f = 1.0f64;
        let mut k = 2.0f64;
        while k < a {
            f *= k;
            k += 1.0;
        }
        return f;
    }
    let nh = a - 0.5;
    if a >= 0.5 && a <= 22.0 && nh == nh.floor() {
        let n = nh as i32;
        let mut df = 1.0f64;
        let mut m = 1.0f64;
        for _ in 0..n {
            df *= m;
            m += 2.0;
        }
        return df * std::f64::consts::PI.sqrt() / f64::powi(2.0, n);
    }
    // corpus only uses integer/half-integer + a few fractional; CR via libm proxy
    statrs_lgamma_exp(a)
}

fn statrs_lgamma_exp(a: f64) -> f64 {
    // exp(ln_gamma) via a Lanczos identical to production ln_gamma
    const C: [f64; 9] = [
        0.999_999_999_999_809_9,
        676.520_368_121_885_1,
        -1_259.139_216_722_402_8,
        771.323_428_777_653_1,
        -176.615_029_162_140_6,
        12.507_343_278_686_905,
        -0.138_571_095_265_720_12,
        9.984_369_578_019_572e-6,
        1.505_632_735_149_311_6e-7,
    ];
    let z = a;
    let x = C
        .iter()
        .enumerate()
        .skip(1)
        .fold(C[0], |acc, (i, c)| acc + c / (z - 1.0 + i as f64));
    let t = z - 1.0 + 7.5;
    ((2.0 * std::f64::consts::PI).ln() * 0.5 + (z - 0.5) * t.ln() - t + x.ln()).exp()
}

#[derive(Clone, Copy)]
struct Cfg {
    x87e: bool,
    x87l: bool,
}

fn expf(c: &Cfg, x: f64) -> f64 {
    if c.x87e { x87_exp(x) } else { x.exp() }
}
fn lnf(c: &Cfg, x: f64) -> f64 {
    if c.x87l { x87_ln(x) } else { x.ln() }
}

/// Corrected-staging gratio for the corpus paths (a <= 20; no Temme needed
/// except a=15 stays under big=20).
fn gratio(c: &Cfg, a: f64, x: f64) -> (f64, f64) {
    let acc = 5e-15f64;
    if x == 0.0 {
        return (0.0, 1.0);
    }
    if a < 1.0 {
        if x < 1.1 {
            // small-a series (160/190/200)
            let mut an = 3.0;
            let mut cc = x;
            let mut summ = x / (a + 3.0);
            let tol = 3.0 * acc / (a + 1.0);
            loop {
                an += 1.0;
                cc = -cc * (x / an);
                let t = cc / (a + an);
                summ += t;
                if t.abs() <= tol {
                    break;
                }
            }
            let j = a * x * ((summ / 6.0 - 0.5 / (a + 2.0)) * x + 1.0 / (a + 1.0));
            let zl = a * lnf(c, x);
            let h = gam1(a);
            let g = 1.0 + h;
            let go200 = if x < 0.25 { zl > -0.13394 } else { a < x / 2.59 };
            if go200 {
                let l = rexp(c, zl);
                let w = 0.5 + (0.5 + l);
                let q = (w * j - l) * g - h;
                if q < 0.0 {
                    return (1.0, 0.0);
                }
                return (0.5 + (0.5 - q), q);
            }
            let w = expf(c, zl);
            let p = w * g * (0.5 + (0.5 - j));
            return (p, 0.5 + (0.5 - p));
        }
        let t1 = a * lnf(c, x) - x;
        let u = a * expf(c, t1);
        if u == 0.0 {
            return (1.0, 0.0);
        }
        let r = u * (1.0 + gam1(a));
        return cf(c, a, x, r, acc);
    }
    // a >= 1 (corpus a <= 15 < big=20)
    if !(a > x || x >= 31.0) {
        let twoa = a + a;
        let m = twoa as i64;
        if twoa == m as f64 {
            let i = m / 2;
            let (mut summ, mut t, mut n, mut cc);
            if a == i as f64 {
                summ = expf(c, -x);
                t = summ;
                n = 1i64;
                cc = 0.0f64;
            } else {
                let rtx = x.sqrt();
                summ = erfc_via(c, x);
                t = expf(c, -x) / (1.77245385090552 * rtx);
                n = 0;
                cc = -0.5;
            }
            while n != i {
                n += 1;
                cc += 1.0;
                t = (x * t) / cc;
                summ += t;
            }
            let q = summ;
            return (0.5 + (0.5 - q), q);
        }
    }
    let t1 = a * lnf(c, x) - x;
    let r = expf(c, t1) / norm_gamma(a);
    if r == 0.0 {
        return if x <= a { (0.0, 1.0) } else { (1.0, 0.0) };
    }
    if x <= a.max(2.30258509299405) {
        // forward-summed Taylor, 1/a outer
        let mut apn = a + 1.0;
        let mut cc = x / apn;
        let mut summ = cc;
        let tol = 0.5 * acc;
        loop {
            apn += 1.0;
            cc *= x / apn;
            summ += cc;
            if cc <= tol {
                break;
            }
        }
        let p = (r / a) * (1.0 + summ);
        return (p, 0.5 + (0.5 - p));
    }
    if x < 31.0 {
        return cf(c, a, x, r, acc);
    }
    // asymptotic (x >= 31): forward + backward per NSWC (few corpus rows)
    let mut amn = a - 1.0;
    let mut t = amn / x;
    let mut wk = [0.0f64; 21];
    wk[1] = t;
    let mut n = 20usize;
    for n_ in 2..=20 {
        amn -= 1.0;
        t *= amn / x;
        if t.abs() <= 1e-3 {
            n = n_;
            break;
        }
        wk[n_] = t;
    }
    let mut summ = t;
    while t.abs() > acc {
        amn -= 1.0;
        t *= amn / x;
        summ += t;
    }
    let mx = n - 1;
    for _ in 0..mx {
        n -= 1;
        summ += wk[n];
    }
    let q = (r / x) * (1.0 + summ);
    (0.5 + (0.5 - q), q)
}

fn erfc_via(c: &Cfg, x: f64) -> f64 {
    gratio(c, 0.5, x).1
}

fn gam1(a: f64) -> f64 {
    const P: [f64; 7] = [
        0.577215664901533e+00,
        -0.409078193005776e+00,
        -0.230975380857675e+00,
        0.597275330452234e-01,
        0.766968181649490e-02,
        -0.514889771323592e-02,
        0.589597428611429e-03,
    ];
    const Q: [f64; 5] = [
        1.0,
        0.427569613095214e+00,
        0.158451672430138e+00,
        0.261132021441447e-01,
        0.423244297896961e-02,
    ];
    let mut t = a;
    let d = a - 0.5;
    if d > 0.0 {
        t = d - 0.5;
    }
    if t == 0.0 {
        return 0.0;
    }
    if t > 0.0 {
        let top = (((((P[6] * t + P[5]) * t + P[4]) * t + P[3]) * t + P[2]) * t + P[1]) * t + P[0];
        let bot = (((Q[4] * t + Q[3]) * t + Q[2]) * t + Q[1]) * t + 1.0;
        let w = top / bot;
        if d > 0.0 {
            return (t / a) * ((w - 0.5) - 0.5);
        }
        return a * w;
    }
    0.0 // corpus never hits t<0 here
}

fn rexp(c: &Cfg, x: f64) -> f64 {
    if x.abs() <= 0.15 {
        const P1: f64 = 0.914041914819518e-09;
        const P2: f64 = 0.238082361044469e-01;
        const Q1: f64 = -0.499999999085958e+00;
        const Q2: f64 = 0.107141568980644e+00;
        const Q3: f64 = -0.119041179760821e-01;
        const Q4: f64 = 0.595130811860248e-03;
        return x * (((P2 * x + P1) * x + 1.0) / ((((Q4 * x + Q3) * x + Q2) * x + Q1) * x + 1.0));
    }
    let w = expf(c, x);
    if x <= 0.0 {
        return (w - 0.5) - 0.5;
    }
    w * (0.5 + (0.5 - 1.0 / w))
}

fn cf(c: &Cfg, a: f64, x: f64, r: f64, acc: f64) -> (f64, f64) {
    let tol = (5.0f64 * 2.220446049250313e-16).max(acc);
    let mut a2nm1 = 1.0;
    let mut a2n = 1.0;
    let mut b2nm1 = x;
    let mut b2n = x + (1.0 - a);
    let mut cc = 1.0;
    loop {
        a2nm1 = x * a2n + cc * a2nm1;
        b2nm1 = x * b2n + cc * b2nm1;
        let am0 = a2nm1 / b2nm1;
        cc += 1.0;
        let cma = cc - a;
        a2n = a2nm1 + cma * a2n;
        b2n = b2nm1 + cma * b2n;
        let an0 = a2n / b2n;
        if !((an0 - am0).abs() >= tol * an0) {
            let q = r * an0;
            return (0.5 + (0.5 - q), q);
        }
    }
}

fn main() {
    let dir = std::env::args().nth(1).expect("work dir");
    let mut rows: Vec<(String, f64, f64, bool, u64)> = Vec::new(); // id, a, x, is_p, bits
    for (name, is_p) in [
        ("answers-gammadist-modern.json", true),
        ("answers-b5.json", true),
        ("answers-chidist.json", false),
    ] {
        let Ok(txt) = std::fs::read_to_string(format!("{dir}/{name}")) else {
            continue;
        };
        let ws: WitnessSet = serde_json::from_str(&txt).unwrap();
        for w in &ws.witnesses {
            let s = |i: usize| -> f64 {
                match &w.args[i] {
                    WitnessArg::Scalar(s) => parse_bits_hex(s).unwrap(),
                    _ => f64::NAN,
                }
            };
            let Some(e) = parse_bits_hex(&w.expected_bits) else {
                continue;
            };
            let (a, x) = if is_p {
                (s(1), s(0) / s(2))
            } else {
                (s(1) / 2.0, s(0) / 2.0)
            };
            if a == 1.0 && is_p {
                continue; // wrapper-dispatched (expm1) — out of kernel scope
            }
            rows.push((w.id.clone().unwrap_or_default(), a, x, is_p, e.to_bits()));
        }
    }
    // dedupe on (a,x,view)
    let mut ded: BTreeMap<(u64, u64, bool), (String, f64, f64, bool, u64)> = BTreeMap::new();
    for r in rows {
        ded.insert((r.1.to_bits(), r.2.to_bits(), r.3), r);
    }
    let rows: Vec<_> = ded.into_values().collect();
    println!("{} rows", rows.len());

    for (tag, cfg) in [
        ("std-exp/std-ln", Cfg { x87e: false, x87l: false }),
        ("x87-exp/std-ln", Cfg { x87e: true, x87l: false }),
        ("std-exp/x87-ln", Cfg { x87e: false, x87l: true }),
        ("x87-exp/x87-ln", Cfg { x87e: true, x87l: true }),
    ] {
        let mut exact = 0usize;
        let mut per_a: BTreeMap<u64, (usize, usize)> = BTreeMap::new();
        let mut maxu: i64 = 0;
        for (_, a, x, is_p, eb) in &rows {
            let (p, q) = gratio(&cfg, *a, *x);
            let v = if *is_p { p } else { q };
            let e = per_a.entry(a.to_bits()).or_insert((0, 0));
            e.1 += 1;
            fn key(i: u64) -> i64 {
                let i = i as i64;
                if i < 0 { !i } else { i }
            }
            let d = (key(*eb) - key(v.to_bits())).abs();
            if d == 0 {
                exact += 1;
                e.0 += 1;
            } else {
                maxu = maxu.max(d);
            }
        }
        print!("{tag}: {exact}/{} max {maxu} | ", rows.len());
        for (ab, (hit, n)) in &per_a {
            let a = f64::from_bits(*ab);
            if *n >= 10 {
                print!("a{a}:{hit}/{n} ");
            }
        }
        println!();
    }
}
