use oxfunc_core::excel_numeric::research as rx;
use oxfunc_core::functions::discrete_dist_family::binom_dist_kernel;
use serde::Deserialize;
use std::fs;

#[derive(Deserialize)]
struct Row {
    n: f64,
    p: f64,
    pmf_bits: String,
}

fn bits(s: &str) -> u64 {
    u64::from_str_radix(s.trim_start_matches("0x"), 16).unwrap()
}

fn main() {
    let path = std::env::args().nth(1).unwrap_or_else(|| {
        "../../work/w109/lastbit-12h-20260912/binom-k1/capture.jsonl".into()
    });
    let mut tot = 0usize;
    let mut e = [0usize; 5];
    let names = [
        "production",
        "n*p*pow_chain(q,n-1)",
        "p*n*pow_chain(q,n-1)",
        "x87_mul(n, x87_mul(p, pow_chain q))",
        "pow_chain(q,n-1)*n*p",
    ];
    for line in fs::read_to_string(&path).unwrap().lines() {
        if line.is_empty() {
            continue;
        }
        let r: Row = serde_json::from_str(line).unwrap();
        let want = bits(&r.pmf_bits);
        let q = 1.0 - r.p;
        let pw = rx::excel_pow_chain(q, r.n - 1.0);
        let cands = [
            binom_dist_kernel(1.0, r.n, r.p, false).unwrap(),
            r.n * r.p * pw,
            r.p * r.n * pw,
            rx::x87_mul(r.n, rx::x87_mul(r.p, pw)),
            pw * r.n * r.p,
        ];
        tot += 1;
        for (i, v) in cands.iter().enumerate() {
            if v.to_bits() == want {
                e[i] += 1;
            }
        }
    }
    println!("BINOM k=1 vs Excel n={tot}");
    for (i, name) in names.iter().enumerate() {
        println!("  {:>5}/{:<5}  {name}", e[i], tot);
    }
}
