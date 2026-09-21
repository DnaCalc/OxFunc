//! Score NEGBINOM k=1 closed forms vs live modest capture.

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

fn main() {
    let path = std::env::args().nth(1).unwrap_or_else(|| {
        "../../work/w109/lastbit-12h-20260912/negbinom-modest/capture.jsonl".into()
    });
    let names = [
        "production",
        "s*pow_chain(p,s)*(1-p)",
        "pow_chain(p,s)*s*(1-p)",
        "x87_mul(s, pow_chain(p,s))*(1-p)",
        "x87_mul(x87_mul(s,pow_chain(p,s)), 1-p)",
        "NB0 * s * (1-p) via binom kn",
    ];
    let mut exact = [0usize; 6];
    let mut tot = 0usize;
    for line in fs::read_to_string(&path).unwrap().lines() {
        if line.is_empty() {
            continue;
        }
        let r: Row = serde_json::from_str(line).unwrap();
        if r.k != 1.0 {
            continue;
        }
        let want = bits(&r.nb_bits);
        let q = 1.0 - r.p;
        let ps = rx::excel_pow_chain(r.p, r.s);
        let nb0 = oxfunc_core::functions::discrete_dist_family::binom_dist_kernel(
            r.s, r.s, r.p, false,
        )
        .unwrap();
        let cands = [
            negbinom_dist_kernel(r.k, r.s, r.p, false).unwrap(),
            r.s * ps * q,
            ps * r.s * q,
            rx::x87_mul(r.s, ps) * q,
            rx::x87_mul(rx::x87_mul(r.s, ps), q),
            nb0 * r.s * q,
        ];
        tot += 1;
        for (i, v) in cands.iter().enumerate() {
            if v.to_bits() == want {
                exact[i] += 1;
            }
        }
    }
    println!("NEGBINOM k=1 vs Excel n={tot}");
    for (i, name) in names.iter().enumerate() {
        println!("  {:>5}/{:<5}  {name}", exact[i], tot);
    }
}
