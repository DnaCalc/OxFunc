//! Test the Cephes/CRT rational expm1 (Pade) vs the exp-Kahan on the clean 46-oracle.
//! Cephes: for |x|<=0.5: r=x*P(x^2); r=r/(Q(x^2)-r); return 2r. Else exp(x)-1.
use oxfunc_core::excel_numeric::research as rx;
const CW: u16 = rx::CW_PC64_RN;
fn fb(h: &str) -> f64 {
    f64::from_bits(u64::from_str_radix(h.trim_start_matches("0x"), 16).unwrap())
}
fn e(x: f64) -> rx::Ext80 {
    rx::ext_from_f64(x)
}
fn tf(x: &rx::Ext80) -> f64 {
    rx::ext_to_f64(x, CW)
}
fn load(path: &str) -> Vec<(f64, i64, f64)> {
    let v: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let mut o = Vec::new();
    for (k, val) in v.as_object().unwrap() {
        let (rh, nh) = k.split_once('|').unwrap();
        o.push((fb(rh), nh.parse().unwrap(), fb(val.as_str().unwrap())));
    }
    o.sort_by(|a, b| (a.0, a.1).partial_cmp(&(b.0, b.1)).unwrap());
    o
}
const EP: [f64; 3] = [
    1.2617719307481059087798E-4,
    3.0299440770744196129956E-2,
    9.9999999999999999991025E-1,
];
const EQ: [f64; 4] = [
    3.0019850513866445504159E-6,
    2.5244834034968410419224E-3,
    2.2726554820815502876593E-1,
    2.0000000000000000000897E0,
];
fn polevl(x: f64, c: &[f64]) -> f64 {
    let mut a = c[0];
    for &ci in &c[1..] {
        a = a * x + ci;
    }
    a
}
// SSE (f64) cephes expm1
fn cephes_sse(x: f64, thr: f64) -> f64 {
    if x > thr || x < -thr {
        return rx::excel_exp(x) - 1.0;
    }
    let xx = x * x;
    let mut r = x * polevl(xx, &EP);
    r = r / (polevl(xx, &EQ) - r);
    r + r
}
// x87 (PC64 per-op double-rounded to f64) cephes
fn polevl_x87(x: f64, c: &[f64]) -> f64 {
    let mut a = c[0];
    for &ci in &c[1..] {
        a = tf(&rx::ext_add(&rx::ext_mul(&e(a), &e(x), CW), &e(ci), CW));
    }
    a
}
fn cephes_x87(x: f64, thr: f64) -> f64 {
    if x > thr || x < -thr {
        return rx::excel_exp(x) - 1.0;
    }
    let xx = tf(&rx::ext_mul(&e(x), &e(x), CW));
    let p = polevl_x87(xx, &EP);
    let mut r = tf(&rx::ext_mul(&e(x), &e(p), CW));
    let q = polevl_x87(xx, &EQ);
    r = tf(&rx::ext_div(
        &e(r),
        &e(tf(&rx::ext_sub(&e(q), &e(r), CW))),
        CW,
    ));
    tf(&rx::ext_add(&e(r), &e(r), CW))
}
fn main() {
    let rows = load("../../work/w109/G6-solvers/pmt_em_hdf_oracle.json");
    for thr in [0.5f64, 1.0] {
        for (name, x87) in [("cephes SSE", false), ("cephes x87", true)] {
            let mut ok = 0;
            let mut miss = Vec::new();
            for (r, n, em) in &rows {
                let t = -(*n as f64) * rx::excel_log1p(*r);
                let val = if x87 {
                    cephes_x87(t, thr)
                } else {
                    cephes_sse(t, thr)
                };
                if val.to_bits() == em.to_bits() {
                    ok += 1;
                } else {
                    miss.push((*r, *n, val.to_bits() as i64 - em.to_bits() as i64));
                }
            }
            print!("  thr={:.1} {:12} {:2}/46", thr, name, ok);
            if ok >= 38 {
                print!("  miss:");
                for (r, n, d) in &miss {
                    print!(" (2^{},{}):{:+}", (r.log2().round()) as i32, n, d);
                }
            }
            println!();
        }
    }
}
