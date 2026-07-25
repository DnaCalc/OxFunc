//! agent-P: per-case perturbation decode of the n=1 Kahan em mismatches.
//! em = fl( fl((u-1)*t) / lnu ). For each PINNED case where the f64 Kahan
//! disagrees with Excel, find which single perturbation reproduces em_excel:
//!  - product rounded up/down (extended product kept)
//!  - lnu off by +-1 ulp (i.e. Excel's ln(u) differs)
//!  - u off by +-1 ulp (Excel's exp differs)
//!  - t off by +-1 ulp (Excel's log1p differs)
//! Reports the class distribution.
use oxfunc_core::excel_numeric::research as rx;
use std::collections::BTreeMap;

const CW: u16 = rx::CW_PC64_RN;

fn fb(h: &str) -> f64 {
    f64::from_bits(u64::from_str_radix(h.trim_start_matches("0x"), 16).unwrap())
}
fn nextafter(x: f64, dir: i64) -> f64 {
    f64::from_bits((x.to_bits() as i64 + dir) as u64)
}
fn e(x: f64) -> rx::Ext80 {
    rx::ext_from_f64(x)
}
fn tf(x: &rx::Ext80) -> f64 {
    rx::ext_to_f64(x, CW)
}

fn load_pinned(path: &str) -> Vec<(f64, i64, f64)> {
    let v: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(path).expect("read")).expect("json");
    let mut out = Vec::new();
    for (k, val) in v.as_object().unwrap() {
        if val.is_null() {
            continue;
        }
        let (rh, nh) = k.split_once('|').unwrap();
        out.push((fb(rh), nh.parse().unwrap(), fb(val.as_str().unwrap())));
    }
    out
}

fn main() {
    let rows = load_pinned("../../work/w109/G6-solvers/pmt_em_pinned.json");
    let n1: Vec<_> = rows.iter().filter(|(_, n, _)| *n == 1).cloned().collect();

    let mut classes: BTreeMap<&str, u32> = BTreeMap::new();
    let mut multi = 0u32;
    let mut none = 0u32;
    let mut mism = 0u32;
    for (r, _, em_x) in &n1 {
        let t = -rx::excel_log1p(*r);
        let u = rx::excel_exp(t);
        if u == 1.0 || t.abs() >= 1.0 {
            continue;
        }
        let um1 = u - 1.0;
        let lnu = rx::excel_ln(u);
        let p = um1 * t; // f64 product (RN)
        let em = p / lnu; // f64 divide (RN)
        if em.to_bits() == em_x.to_bits() {
            continue;
        }
        mism += 1;

        // candidate explanations (each a single-op deviation) -> em must equal em_x
        let mut hits: Vec<&str> = Vec::new();
        // (a) extended product kept for the divide (product not stored)
        let em_extprod = tf(&rx::ext_div(&rx::ext_mul(&e(um1), &e(t), CW), &e(lnu), CW));
        if em_extprod.to_bits() == em_x.to_bits() {
            hits.push("extProd");
        }
        // (b) product rounded the other way (+-1 ulp on p)
        for d in [-1i64, 1] {
            if (nextafter(p, d) / lnu).to_bits() == em_x.to_bits() {
                hits.push(if d < 0 { "p-1" } else { "p+1" });
            }
        }
        // (c) lnu off by +-1 ulp
        for d in [-1i64, 1] {
            if (p / nextafter(lnu, d)).to_bits() == em_x.to_bits() {
                hits.push(if d < 0 { "lnu-1" } else { "lnu+1" });
            }
        }
        // (d) u off by +-1 ulp (recompute um1, lnu, p, em)
        for d in [-1i64, 1] {
            let u2 = nextafter(u, d);
            let um1b = u2 - 1.0;
            let lnu2 = rx::excel_ln(u2);
            let em2 = (um1b * t) / lnu2;
            if em2.to_bits() == em_x.to_bits() {
                hits.push(if d < 0 { "u-1" } else { "u+1" });
            }
        }
        // (e) t off by +-1 ulp
        for d in [-1i64, 1] {
            let t2 = nextafter(t, d);
            if ((um1 * t2) / lnu).to_bits() == em_x.to_bits() {
                hits.push(if d < 0 { "t-1" } else { "t+1" });
            }
        }
        // (f) final divide rounded other way (em +-1) -- trivially always exists as a class marker
        // record
        if hits.is_empty() {
            none += 1;
            classes.entry("UNEXPLAINED").and_modify(|c| *c += 1).or_insert(1);
        } else if hits.len() == 1 {
            classes.entry(hits[0]).and_modify(|c| *c += 1).or_insert(1);
        } else {
            multi += 1;
            // record the multi combo
            let key: &'static str = Box::leak(hits.join("+").into_boxed_str());
            classes.entry(key).and_modify(|c| *c += 1).or_insert(1);
        }
    }
    println!("n=1 mismatches: {}", mism);
    println!("  single-explanation classes + multi combos:");
    for (k, c) in &classes {
        println!("    {:24} {}", k, c);
    }
    println!("  (multi-hit cases: {}, unexplained: {})", multi, none);
}
