//! Score POISSON k=1 candidates vs live 20326 capture.

use oxfunc_core::excel_numeric::research as rx;
use oxfunc_core::functions::discrete_dist_family::poisson_dist_kernel;
use serde::Deserialize;
use std::fs;

#[derive(Deserialize)]
struct Row {
    lam: f64,
    pmf_bits: String,
}

fn bits(s: &str) -> u64 {
    u64::from_str_radix(s.trim_start_matches("0x"), 16).unwrap()
}

fn main() {
    let path = std::env::args().nth(1).unwrap_or_else(|| {
        "../../work/w109/lastbit-12h-20260912/poisson-k1/capture.jsonl".into()
    });
    let names = [
        "production",
        "exp(-l+ln l)",
        "exp(ln l - l)",
        "exp(-l+excel_ln l)",
        "exp(excel_ln l - l)",
        "l*exp(-l)",
        "exp(-l)*l",
        "x87_mul(l, exp(-l))",
        "x87_mul(exp(-l), l)",
        "exp(x87_mul(-1,l)+excel_ln l)",
    ];
    let mut exact = vec![0usize; names.len()];
    let mut tot = 0usize;
    for line in fs::read_to_string(&path).unwrap().lines() {
        if line.is_empty() {
            continue;
        }
        let r: Row = serde_json::from_str(line).unwrap();
        let want = bits(&r.pmf_bits);
        let l = r.lam;
        let e = rx::excel_exp(-l);
        let ln = rx::excel_ln(l);
        let real = [
            poisson_dist_kernel(1.0, l, false).unwrap(),
            rx::excel_exp(-l + l.ln()),
            rx::excel_exp(l.ln() - l),
            rx::excel_exp(-l + ln),
            rx::excel_exp(ln - l),
            l * e,
            e * l,
            rx::x87_mul(l, e),
            rx::x87_mul(e, l),
            rx::excel_exp(rx::x87_mul(-1.0, l) + ln),
        ];
        tot += 1;
        for (i, v) in real.iter().enumerate() {
            if v.to_bits() == want {
                exact[i] += 1;
            }
        }
    }
    println!("POISSON k=1 vs Excel n={tot}");
    for (i, name) in names.iter().enumerate() {
        println!("  {:>5}/{:<5}  {name}", exact[i], tot);
    }
}
