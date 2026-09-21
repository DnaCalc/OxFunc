//! Score BINOM k=0 candidates vs live 20326 capture.

use oxfunc_core::excel_numeric::research as rx;
use oxfunc_core::functions::discrete_dist_family::binom_dist_kernel;
use serde::Deserialize;
use std::fs;

#[derive(Deserialize)]
struct Row {
    n: f64,
    p: f64,
    binom_bits: String,
}

fn bits(s: &str) -> u64 {
    u64::from_str_radix(s.trim_start_matches("0x"), 16).unwrap()
}

fn main() {
    let path = std::env::args().nth(1).unwrap_or_else(|| {
        "../../work/w109/lastbit-12h-20260912/binom-k0/capture.jsonl".into()
    });
    let names = [
        "production",
        "pow_chain(q,n)",
        "exp(n*ln q)",
        "exp(lnq*n)",
        "exp(x87 n*lnq)",
        "exp(x87 lnq*n)",
        "pow_positive(q,n)",
        "exp(-n*p)*?skip",
    ];
    let mut exact = [0usize; 7];
    let mut tot = 0usize;
    let mut p_lt = (0usize, 0usize);
    let mut p_ge = (0usize, 0usize);
    for line in fs::read_to_string(&path).unwrap().lines() {
        if line.is_empty() {
            continue;
        }
        let r: Row = serde_json::from_str(line).unwrap();
        let want = bits(&r.binom_bits);
        let q = 1.0 - r.p;
        let lnq = rx::excel_ln(q);
        let real = [
            binom_dist_kernel(0.0, r.n, r.p, false).unwrap(),
            rx::excel_pow_chain(q, r.n),
            rx::excel_exp(r.n * lnq),
            rx::excel_exp(lnq * r.n),
            rx::excel_exp(rx::x87_mul(r.n, lnq)),
            rx::excel_exp(rx::x87_mul(lnq, r.n)),
            rx::excel_pow_positive(q, r.n),
        ];
        tot += 1;
        for (i, v) in real.iter().enumerate() {
            if v.to_bits() == want {
                exact[i] += 1;
            }
        }
        let slot = if r.p < 0.1 { &mut p_lt } else { &mut p_ge };
        slot.1 += 1;
        if real[0].to_bits() == want {
            slot.0 += 1;
        }
    }
    println!("BINOM k=0 vs Excel n={tot}");
    for (i, name) in names.iter().take(7).enumerate() {
        println!("  {:>5}/{:<5}  {name}", exact[i], tot);
    }
    println!("production p<0.1 {}/{}  p>=0.1 {}/{}", p_lt.0, p_lt.1, p_ge.0, p_ge.1);
}
