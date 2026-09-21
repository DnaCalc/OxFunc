//! Score BINOM CDF vs production sum and BRATIO regularized_beta.

use oxfunc_core::functions::discrete_dist_family::binom_dist_kernel;
use oxfunc_core::functions::special_math_common::regularized_beta;
use serde::Deserialize;
use std::fs;

#[derive(Deserialize)]
struct Row {
    k: f64,
    n: f64,
    p: f64,
    cdf_bits: String,
}

fn bits(s: &str) -> u64 {
    u64::from_str_radix(s.trim_start_matches("0x"), 16).unwrap()
}

fn main() {
    let path = std::env::args().nth(1).unwrap_or_else(|| {
        "../../work/w109/lastbit-12h-20260912/binom-cdf/capture.jsonl".into()
    });
    let names = [
        "production sum",
        "I_{1-p}(n-k, k+1)",
        "1-I_p(k+1, n-k)",
        "I_p(n-k, k+1)",
        "I_{1-p}(k+1, n-k)",
    ];
    let mut exact = [0usize; 5];
    let mut tot = 0usize;
    let mut skip = 0usize;
    for line in fs::read_to_string(&path).unwrap().lines() {
        if line.is_empty() {
            continue;
        }
        let r: Row = serde_json::from_str(line).unwrap();
        let want = bits(&r.cdf_bits);
        let nk = r.n - r.k;
        let kp1 = r.k + 1.0;
        let q = 1.0 - r.p;
        let prod = binom_dist_kernel(r.k, r.n, r.p, true).unwrap();
        let mut cands = vec![prod];
        if nk > 0.0 {
            cands.push(regularized_beta(q, nk, kp1));
            cands.push(1.0 - regularized_beta(r.p, kp1, nk));
            cands.push(regularized_beta(r.p, nk, kp1));
            cands.push(regularized_beta(q, kp1, nk));
        } else {
            skip += 1;
            cands.extend_from_slice(&[f64::NAN, f64::NAN, f64::NAN, f64::NAN]);
        }
        tot += 1;
        for (i, v) in cands.iter().enumerate() {
            if v.is_finite() && v.to_bits() == want {
                exact[i] += 1;
            }
        }
    }
    println!("BINOM CDF vs Excel n={tot} (beta skipped k=n {skip})");
    for (i, name) in names.iter().enumerate() {
        let den = if i == 0 { tot } else { tot - skip };
        println!("  {:>5}/{:<5}  {name}", exact[i], den);
    }
}
