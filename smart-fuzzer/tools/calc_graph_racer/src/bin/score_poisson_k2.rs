//! Score POISSON k=2,3 closed forms vs live capture.

use oxfunc_core::excel_numeric::research as rx;
use oxfunc_core::functions::discrete_dist_family::poisson_dist_kernel;
use serde::Deserialize;
use std::fs;

#[derive(Deserialize)]
struct Row {
    k: f64,
    lam: f64,
    pmf_bits: String,
}

fn bits(s: &str) -> u64 {
    u64::from_str_radix(s.trim_start_matches("0x"), 16).unwrap()
}

fn main() {
    let path = std::env::args().nth(1).unwrap_or_else(|| {
        "../../work/w109/lastbit-12h-20260912/poisson-kge2/capture.jsonl".into()
    });
    let mut tot2 = 0usize;
    let mut tot3 = 0usize;
    let mut e2 = [0usize; 5];
    let mut e3 = [0usize; 4];
    for line in fs::read_to_string(&path).unwrap().lines() {
        if line.is_empty() {
            continue;
        }
        let r: Row = serde_json::from_str(line).unwrap();
        let want = bits(&r.pmf_bits);
        let l = r.lam;
        let e = rx::excel_exp(-l);
        if r.k == 2.0 {
            let cands = [
                poisson_dist_kernel(2.0, l, false).unwrap(),
                l * l / 2.0 * e,
                (l * l) * e / 2.0,
                rx::x87_mul(l * l / 2.0, e),
                rx::excel_exp(-l + 2.0 * l.ln() - std::f64::consts::LN_2),
            ];
            tot2 += 1;
            for (i, v) in cands.iter().enumerate() {
                if v.to_bits() == want {
                    e2[i] += 1;
                }
            }
        } else if r.k == 3.0 {
            let cands = [
                poisson_dist_kernel(3.0, l, false).unwrap(),
                l * l * l / 6.0 * e,
                (l * l * l) * e / 6.0,
                rx::excel_exp(-l + 3.0 * l.ln() - (6.0_f64).ln()),
            ];
            tot3 += 1;
            for (i, v) in cands.iter().enumerate() {
                if v.to_bits() == want {
                    e3[i] += 1;
                }
            }
        }
    }
    println!("POISSON k=2 n={tot2}");
    println!("  prod {}/{}", e2[0], tot2);
    println!("  l*l/2*exp {}/{}", e2[1], tot2);
    println!("  (l*l)*exp/2 {}/{}", e2[2], tot2);
    println!("  x87 (l*l/2)*exp {}/{}", e2[3], tot2);
    println!("  exp(-l+2lnl-ln2) {}/{}", e2[4], tot2);
    println!("POISSON k=3 n={tot3}");
    println!("  prod {}/{}", e3[0], tot3);
    println!("  l^3/6*exp {}/{}", e3[1], tot3);
    println!("  (l^3)*exp/6 {}/{}", e3[2], tot3);
    println!("  exp log-compose {}/{}", e3[3], tot3);
}
