//! W109 Phase-5: race the Cephes `lgam` algorithm (rational + Stirling)
//! against the cached published GAMMALN bits, in strict-double and
//! extended stagings with platform vs x87 ln.

use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{WitnessArg, WitnessSet, ulp_distance};
use oxfunc_core::excel_numeric::research as rx;

const A: [f64; 5] = [
    8.11614167470508450300e-4,
    -5.95061904284301438324e-4,
    7.93650340457716943945e-4,
    -2.77777777730099687205e-3,
    8.33333333333331927722e-2,
];
const B: [f64; 6] = [
    -1.37825152569120859100e3,
    -3.88016315134637840924e4,
    -3.31612992738871184744e5,
    -1.16237097492762307383e6,
    -1.72173700820839662146e6,
    -8.53555664245765465627e5,
];
const C: [f64; 6] = [
    -3.51815701436523470549e2,
    -1.70642106651881159223e4,
    -2.20528590553854454839e5,
    -1.13933444367982507207e6,
    -2.53252307177582951285e6,
    -2.01889141433532773231e6,
];
const LS2PI: f64 = 0.91893853320467274178;

fn polevl(x: f64, c: &[f64]) -> f64 {
    let mut r = c[0];
    for k in &c[1..] {
        r = r * x + k;
    }
    r
}
fn p1evl(x: f64, c: &[f64]) -> f64 {
    // leading coefficient 1.0
    let mut r = x + c[0];
    for k in &c[1..] {
        r = r * x + k;
    }
    r
}

/// Cephes lgam for x > 0, strict double, choice of ln.
fn cephes_double(mut x: f64, ln: fn(f64) -> f64) -> f64 {
    if x < 13.0 {
        let mut z = 1.0f64;
        let mut p = 0.0f64;
        let mut u = x;
        while u >= 3.0 {
            p -= 1.0;
            u = x + p;
            z *= u;
        }
        while u < 2.0 {
            z /= u;
            p += 1.0;
            u = x + p;
        }
        let z = z.abs();
        if u == 2.0 {
            return ln(z);
        }
        p -= 2.0;
        x += p;
        let q = x * polevl(x, &B) / p1evl(x, &C);
        return ln(z) + q;
    }
    let mut q = (x - 0.5) * ln(x) - x + LS2PI;
    if x > 1.0e8 {
        return q;
    }
    let p = 1.0 / (x * x);
    if x >= 1000.0 {
        q += ((7.9365079365079365079365e-4 * p - 2.7777777777777777777778e-3) * p
            + 0.0833333333333333333333)
            / x;
    } else {
        q += polevl(p, &A) / x;
    }
    q
}

/// Cephes lgam entirely in x87 extended (PC=64), one final store, fyl2x ln.
fn cephes_ext(x0: f64) -> f64 {
    let cw = rx::CW_PC64_RN;
    let e = rx::ext_from_f64;
    let ln = |v: &rx::Ext80| rx::ext_fyl2x(&rx::ext_ln2(), v, cw);
    let add = |a: &rx::Ext80, b: &rx::Ext80| rx::ext_add(a, b, cw);
    let sub = |a: &rx::Ext80, b: &rx::Ext80| rx::ext_sub(a, b, cw);
    let mul = |a: &rx::Ext80, b: &rx::Ext80| rx::ext_mul(a, b, cw);
    let div = |a: &rx::Ext80, b: &rx::Ext80| rx::ext_div(a, b, cw);
    let tof = |v: &rx::Ext80| rx::ext_to_f64(v, cw);

    let mut x = x0;
    if x0 < 13.0 {
        let mut z = rx::ext_one();
        let mut p = 0.0f64;
        let mut u = x0;
        while u >= 3.0 {
            p -= 1.0;
            u = x0 + p;
            z = mul(&z, &e(u));
        }
        while u < 2.0 {
            z = div(&z, &e(u));
            p += 1.0;
            u = x0 + p;
        }
        let z = rx::ext_abs(&z, cw);
        if u == 2.0 {
            return tof(&ln(&z));
        }
        p -= 2.0;
        x += p; // strict f64 shift of the small argument (exact here)
        let xe = e(x);
        let mut num = e(B[0]);
        for k in &B[1..] {
            num = add(&mul(&num, &xe), &e(*k));
        }
        let mut den = add(&xe, &e(C[0]));
        for k in &C[1..] {
            den = add(&mul(&den, &xe), &e(*k));
        }
        let q = div(&mul(&xe, &num), &den);
        return tof(&add(&ln(&z), &q));
    }
    let xe = e(x);
    let mut q = sub(&mul(&sub(&xe, &e(0.5)), &ln(&xe)), &xe);
    q = add(&q, &e(LS2PI));
    if x > 1.0e8 {
        return tof(&q);
    }
    let p = div(&rx::ext_one(), &mul(&xe, &xe));
    if x >= 1000.0 {
        let mut t = mul(&e(7.9365079365079365079365e-4), &p);
        t = sub(&t, &e(2.7777777777777777777778e-3));
        t = mul(&t, &p);
        t = add(&t, &e(0.0833333333333333333333));
        q = add(&q, &div(&t, &xe));
    } else {
        let mut t = e(A[0]);
        for k in &A[1..] {
            t = add(&mul(&t, &p), &e(*k));
        }
        q = add(&q, &div(&t, &xe));
    }
    tof(&q)
}

/// Cephes lgam with per-STATEMENT stores (32-bit MSVC x87 codegen semantics):
/// each C assignment's RHS evaluates in extended and stores once to a double;
/// `log` is the x87 CRT ln returning a stored double.
fn cephes_stmt(x0: f64) -> f64 {
    let cw = rx::CW_PC64_RN;
    let e = rx::ext_from_f64;
    let tof = |v: &rx::Ext80| rx::ext_to_f64(v, cw);
    // one-statement helpers: extended expression, single store
    let horner_step = |r: f64, x: f64, c: f64| -> f64 {
        tof(&rx::ext_add(&rx::ext_mul(&e(r), &e(x), cw), &e(c), cw))
    };
    let polevl_s = |x: f64, c: &[f64]| -> f64 {
        let mut r = c[0];
        for k in &c[1..] {
            r = horner_step(r, x, *k);
        }
        r
    };
    let p1evl_s = |x: f64, c: &[f64]| -> f64 {
        let mut r = tof(&rx::ext_add(&e(x), &e(c[0]), cw));
        for k in &c[1..] {
            r = horner_step(r, x, *k);
        }
        r
    };

    let mut x = x0;
    if x0 < 13.0 {
        let mut z = 1.0f64;
        let mut p = 0.0f64;
        let mut u = x0;
        while u >= 3.0 {
            p -= 1.0;
            u = x0 + p; // exact
            z = tof(&rx::ext_mul(&e(z), &e(u), cw));
        }
        while u < 2.0 {
            z = tof(&rx::ext_div(&e(z), &e(u), cw));
            p += 1.0;
            u = x0 + p;
        }
        let z = z.abs();
        if u == 2.0 {
            return rx::excel_ln(z);
        }
        p -= 2.0;
        x += p;
        // p = x * polevl(x,B,5) / p1evl(x,C,6);  (one store)
        let num = polevl_s(x, &B);
        let den = p1evl_s(x, &C);
        let q = tof(&rx::ext_div(&rx::ext_mul(&e(x), &e(num), cw), &e(den), cw));
        // return log(z) + q;  (ln stored by the CRT, sum stored on return)
        let lnz = rx::excel_ln(z);
        return tof(&rx::ext_add(&e(lnz), &e(q), cw));
    }
    // q = (x - 0.5)*log(x) - x + LS2PI;  (log stored, rest extended, one store)
    let lnx = rx::excel_ln(x);
    let q0 = tof(&rx::ext_add(
        &rx::ext_sub(&rx::ext_mul(&rx::ext_sub(&e(x), &e(0.5), cw), &e(lnx), cw), &e(x), cw),
        &e(LS2PI),
        cw,
    ));
    if x > 1.0e8 {
        return q0;
    }
    // p = 1.0/(x*x);
    let p = tof(&rx::ext_div(&rx::ext_one(), &rx::ext_mul(&e(x), &e(x), cw), cw));
    if x >= 1000.0 {
        let mut t = tof(&rx::ext_sub(
            &rx::ext_mul(&e(7.9365079365079365079365e-4), &e(p), cw),
            &e(2.7777777777777777777778e-3),
            cw,
        ));
        t = tof(&rx::ext_add(
            &rx::ext_mul(&e(t), &e(p), cw),
            &e(0.0833333333333333333333),
            cw,
        ));
        tof(&rx::ext_add(&e(q0), &rx::ext_div(&e(t), &e(x), cw), cw))
    } else {
        let t = polevl_s(p, &A);
        tof(&rx::ext_add(&e(q0), &rx::ext_div(&e(t), &e(x), cw), cw))
    }
}

fn main() {
    let files: Vec<String> = std::env::args().skip(1).collect();
    let mut rows: Vec<(f64, f64)> = Vec::new();
    for file in &files {
        let ws: WitnessSet =
            serde_json::from_str(&std::fs::read_to_string(file).expect("read")).expect("parse");
        for w in &ws.witnesses {
            let x = match &w.args[0] {
                WitnessArg::Scalar(s) => parse_bits_hex(s).unwrap(),
                _ => continue,
            };
            if let Some(v) = parse_bits_hex(&w.expected_bits) {
                rows.push((x, v));
            }
        }
    }
    println!("{} GAMMALN witnesses", rows.len());
    let cands: Vec<(&str, Box<dyn Fn(f64) -> f64>)> = vec![
        ("cephes-double-platform", Box::new(|x| cephes_double(x, f64::ln))),
        ("cephes-double-x87ln", Box::new(|x| cephes_double(x, rx::excel_ln))),
        ("cephes-ext", Box::new(cephes_ext)),
        ("cephes-stmt-spill", Box::new(cephes_stmt)),
    ];
    for (name, f) in &cands {
        let mut small = (0u32, 0u32, 0u64); // exact, total, max_ulp
        let mut large = (0u32, 0u32, 0u64);
        for &(x, want) in &rows {
            let v = f(x);
            let s = if x < 13.0 { &mut small } else { &mut large };
            s.1 += 1;
            if v.to_bits() == want.to_bits() {
                s.0 += 1;
            } else {
                s.2 = s.2.max(ulp_distance(v, want).unwrap_or(u64::MAX));
            }
        }
        println!(
            "{name:24} small(x<13): {}/{} max {}   stirling: {}/{} max {}",
            small.0, small.1, small.2, large.0, large.1, large.2
        );
    }
}
