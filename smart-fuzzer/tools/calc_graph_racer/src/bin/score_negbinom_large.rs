//! Score large-n NEGBINOM PMF vs live 20326 capture.

use oxfunc_core::excel_numeric::research as rx;
use oxfunc_core::functions::discrete_dist_family::negbinom_dist_kernel;
use serde::Deserialize;
use std::fs;

#[derive(Deserialize)]
struct Row {
    k: f64,
    s: f64,
    p: f64,
    nb_bits: String,
}

fn bits(s: &str) -> u64 {
    u64::from_str_radix(s.trim_start_matches("0x"), 16).unwrap()
}

fn ln_choose_native(n: u64, k: u64) -> f64 {
    let k = k.min(n - k);
    let mut acc = 0.0;
    for i in 1..=k {
        acc += ((n - k + i) as f64).ln() - (i as f64).ln();
    }
    acc
}

fn ln_choose_excel(n: u64, k: u64) -> f64 {
    let k = k.min(n - k);
    let mut acc = 0.0;
    for i in 1..=k {
        acc += rx::excel_ln((n - k + i) as f64) - rx::excel_ln(i as f64);
    }
    acc
}

fn main() {
    let path = std::env::args().nth(1).unwrap_or_else(|| {
        "../../work/w109/lastbit-12h-20260912/negbinom-large/capture.jsonl".into()
    });
    let names = [
        "production",
        "exp(native_lnc + s*ln p + k*ln q)",
        "exp(excel_lnc + s*eln p + k*eln q)",
        "excel_exp(native_lnc + s*eln p + k*eln q)",
        "excel_exp(excel_lnc + s*eln p + k*eln q)",
        "excel_pow(p,s)*excel_pow(q,k)*exp(native_lnc)",
    ];
    let mut exact = [0usize; 6];
    let mut tot = 0usize;
    for line in fs::read_to_string(&path).unwrap().lines() {
        if line.is_empty() {
            continue;
        }
        let r: Row = serde_json::from_str(line).unwrap();
        let want = bits(&r.nb_bits);
        let k = r.k as u64;
        let s = r.s as u64;
        let n = k + s - 1;
        let q = 1.0 - r.p;
        let lnc_n = ln_choose_native(n, k);
        let lnc_e = ln_choose_excel(n, k);
        let real = [
            negbinom_dist_kernel(r.k, r.s, r.p, false).unwrap(),
            (lnc_n + (s as f64) * r.p.ln() + (k as f64) * q.ln()).exp(),
            (lnc_e + (s as f64) * rx::excel_ln(r.p) + (k as f64) * rx::excel_ln(q)).exp(),
            rx::excel_exp(lnc_n + (s as f64) * rx::excel_ln(r.p) + (k as f64) * rx::excel_ln(q)),
            rx::excel_exp(lnc_e + (s as f64) * rx::excel_ln(r.p) + (k as f64) * rx::excel_ln(q)),
            rx::excel_pow_chain(r.p, s as f64)
                * rx::excel_pow_chain(q, k as f64)
                * lnc_n.exp(),
        ];
        tot += 1;
        for (i, v) in real.iter().enumerate() {
            if v.to_bits() == want {
                exact[i] += 1;
            }
        }
    }
    println!("NEGBINOM large vs Excel n={tot}");
    for (i, name) in names.iter().enumerate() {
        println!("  {:>5}/{:<5}  {name}", exact[i], tot);
    }
}
