//! Test alternative (algebraically-equivalent) Kahan-correction ARRANGEMENTS on
//! the clean 46-row oracle. em = expm1(t), t=-n*log1p(r). Kahan branch |t|<1.
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
fn main() {
    let rows = load("../../work/w109/G6-solvers/pmt_em_hdf_oracle.json");
    type V = Box<dyn Fn(f64, f64, f64) -> f64>; // (u,t,lnu)->em  (Kahan branch only)
    let variants: Vec<(&str, V)> = vec![
        ("A0 (u-1)*t/lnu", Box::new(|u, t, lnu| (u - 1.0) * t / lnu)),
        (
            "A1 (u-1)+(u-1)*(t-lnu)/lnu",
            Box::new(|u, t, lnu| {
                let a = u - 1.0;
                a + a * (t - lnu) / lnu
            }),
        ),
        (
            "A2 (u-1)*(1+(t-lnu)/lnu)",
            Box::new(|u, t, lnu| {
                let a = u - 1.0;
                a * (1.0 + (t - lnu) / lnu)
            }),
        ),
        (
            "A3 (u-1)+(u-1)/lnu*(t-lnu)",
            Box::new(|u, t, lnu| {
                let a = u - 1.0;
                a + a / lnu * (t - lnu)
            }),
        ),
        (
            "A4 (u-1)+ (t-lnu)*((u-1)/lnu)",
            Box::new(|u, t, lnu| {
                let a = u - 1.0;
                a + (t - lnu) * (a / lnu)
            }),
        ),
        // extended: correction term extended, added f64
        (
            "A5 (u-1)+ext[(u-1)*(t-lnu)/lnu]",
            Box::new(|u, t, lnu| {
                let a = u - 1.0;
                let corr = tf(&rx::ext_div(
                    &rx::ext_mul(&e(a), &e(t - lnu), CW),
                    &e(lnu),
                    CW,
                ));
                a + corr
            }),
        ),
        // t-lnu computed extended (t,lnu both f64 here)
        (
            "A6 (u-1)+(u-1)*ext(t-lnu)/lnu",
            Box::new(|u, t, lnu| {
                let a = u - 1.0;
                let d = tf(&rx::ext_sub(&e(t), &e(lnu), CW));
                a + a * d / lnu
            }),
        ),
        // whole additive form in x87 PC64, single store
        (
            "A7 ext-all additive",
            Box::new(|u, t, lnu| {
                let a = e(u - 1.0);
                let d = rx::ext_sub(&e(t), &e(lnu), CW);
                let corr = rx::ext_div(&rx::ext_mul(&a, &d, CW), &e(lnu), CW);
                tf(&rx::ext_add(&a, &corr, CW))
            }),
        ),
    ];
    for (name, f) in &variants {
        let mut ok = 0;
        let mut miss = Vec::new();
        for (r, n, em) in &rows {
            let t = -(*n as f64) * rx::excel_log1p(*r);
            let u = rx::excel_exp(t);
            let val = if u == 1.0 {
                t
            } else if t.abs() < 1.0 {
                f(u, t, rx::excel_ln(u))
            } else {
                u - 1.0
            };
            if val.to_bits() == em.to_bits() {
                ok += 1;
            } else {
                miss.push((*r, *n, val.to_bits() as i64 - em.to_bits() as i64));
            }
        }
        print!("  {:34} {:2}/46", name, ok);
        if ok >= 39 {
            print!("  miss:");
            for (r, n, d) in &miss {
                print!(" (2^{},{}):{:+}", (r.log2().round()) as i32, n, d);
            }
        }
        println!();
    }
}
