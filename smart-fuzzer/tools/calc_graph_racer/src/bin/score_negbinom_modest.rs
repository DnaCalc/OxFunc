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

fn choose(n: u64, k: u64) -> f64 {
    let k = k.min(n - k);
    let mut acc = 1.0;
    for i in 1..=k {
        acc = acc * (n - k + i) as f64 / i as f64;
    }
    acc
}

fn main() {
    let path = std::env::args().nth(1).unwrap_or_else(|| {
        "../../work/w109/lastbit-12h-20260912/negbinom-modest/capture.jsonl".into()
    });
    let mut tot = 0usize;
    let mut e = [0usize; 4];
    let names = [
        "production",
        "C*pow_chain(p,s)*pow_chain(q,k)",
        "C*powi p * powi q",
        "pow_chain(p,s)*pow_chain(q,k)*C",
    ];
    for line in fs::read_to_string(&path).unwrap().lines() {
        if line.is_empty() {
            continue;
        }
        let r: Row = serde_json::from_str(line).unwrap();
        let want = bits(&r.nb_bits);
        let k = r.k as u64;
        let s = r.s as u64;
        let c = choose(k + s - 1, k);
        let q = 1.0 - r.p;
        let pc = rx::excel_pow_chain(r.p, s as f64);
        let qc = rx::excel_pow_chain(q, k as f64);
        let cands = [
            negbinom_dist_kernel(r.k, r.s, r.p, false).unwrap(),
            c * pc * qc,
            c * r.p.powi(s as i32) * q.powi(k as i32),
            pc * qc * c,
        ];
        tot += 1;
        for (i, v) in cands.iter().enumerate() {
            if v.to_bits() == want {
                e[i] += 1;
            }
        }
    }
    println!("NEGBINOM modest vs Excel n={tot}");
    for (i, name) in names.iter().enumerate() {
        println!("  {:>5}/{:<5}  {name}", e[i], tot);
    }
}
