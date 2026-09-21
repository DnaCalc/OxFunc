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
        "../../work/w109/lastbit-12h-20260912/binom-kn/capture.jsonl".into()
    });
    let mut tot = 0usize;
    let mut e = [0usize; 3];
    for line in fs::read_to_string(&path).unwrap().lines() {
        if line.is_empty() {
            continue;
        }
        let r: Row = serde_json::from_str(line).unwrap();
        let want = bits(&r.pmf_bits);
        let cands = [
            binom_dist_kernel(r.n, r.n, r.p, false).unwrap(),
            rx::excel_pow_chain(r.p, r.n),
            rx::excel_pow_positive(r.p, r.n),
        ];
        tot += 1;
        for (i, v) in cands.iter().enumerate() {
            if v.to_bits() == want {
                e[i] += 1;
            }
        }
    }
    println!("BINOM k=n vs Excel n={tot}");
    println!("  prod {}/{}", e[0], tot);
    println!("  pow_chain {}/{}", e[1], tot);
    println!("  pow_positive {}/{}", e[2], tot);
}
