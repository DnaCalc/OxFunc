//! Score GAMMALN x<0.7 compose: production vs excel_log reduction.

use oxfunc_core::excel_numeric::research as rx;
use oxfunc_core::functions::special_dist_family::gammaln_kernel;
use serde::Deserialize;
use std::fs;

#[derive(Deserialize)]
struct Row {
    x: f64,
    gammaln_bits: String,
}

fn bits(s: &str) -> u64 {
    u64::from_str_radix(s.trim_start_matches("0x"), 16).unwrap()
}

fn main() {
    let path = std::env::args().nth(1).unwrap_or_else(|| {
        "../../work/w109/lastbit-12h-20260912/gammaln-composed/capture.jsonl".into()
    });
    let mut tot = 0usize;
    let mut prod = 0usize;
    let mut eln = 0usize;
    let mut both_same = 0usize;
    for line in fs::read_to_string(&path).unwrap().lines() {
        if line.is_empty() {
            continue;
        }
        let r: Row = serde_json::from_str(line).unwrap();
        if r.x <= 0.0 || r.x >= 0.7 {
            continue;
        }
        let want = bits(&r.gammaln_bits);
        tot += 1;
        let g = gammaln_kernel(r.x).unwrap();
        let alt = gammaln_kernel(r.x + 1.0).unwrap() - rx::excel_ln(r.x);
        if g.to_bits() == want {
            prod += 1;
        }
        if alt.to_bits() == want {
            eln += 1;
        }
        if g.to_bits() == alt.to_bits() {
            both_same += 1;
        }
    }
    println!("composed x<0.7 n={tot}");
    println!("  production {prod}/{tot}");
    println!("  GAMMALN(x+1)-excel_ln(x) {eln}/{tot}");
    println!("  prod==excel_ln compose {both_same}/{tot}");
}
