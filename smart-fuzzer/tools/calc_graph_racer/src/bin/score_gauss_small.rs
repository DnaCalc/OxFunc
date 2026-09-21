//! Score GAUSS/NORMSDIST vs live 20326 after ERFC (0,0.5] complement.

use oxfunc_core::functions::gauss_fn::gauss_kernel;
use oxfunc_core::functions::normal_log_family::identified_std_normal_cdf;
use serde::Deserialize;
use std::fs;

#[derive(Deserialize)]
struct Row {
    x: f64,
    gauss_bits: String,
    ns_bits: String,
}

fn bits(s: &str) -> u64 {
    u64::from_str_radix(s.trim_start_matches("0x"), 16).unwrap()
}

fn main() {
    let path = std::env::args().nth(1).unwrap_or_else(|| {
        "../../work/w109/lastbit-12h-20260912/gauss-small/capture.jsonl".into()
    });
    let mut n = 0usize;
    let mut g = 0usize;
    let mut ns = 0usize;
    let mut z_le = (0usize, 0usize);
    for line in fs::read_to_string(&path).unwrap().lines() {
        if line.is_empty() {
            continue;
        }
        let r: Row = serde_json::from_str(line).unwrap();
        n += 1;
        if gauss_kernel(r.x).unwrap().to_bits() == bits(&r.gauss_bits) {
            g += 1;
        }
        if identified_std_normal_cdf(r.x).to_bits() == bits(&r.ns_bits) {
            ns += 1;
        }
        let z = r.x.abs() * f64::from_bits(0x3fe6a09e667f3bcd);
        if z > 0.0 && z <= 0.5 {
            z_le.1 += 1;
            if identified_std_normal_cdf(r.x).to_bits() == bits(&r.ns_bits) {
                z_le.0 += 1;
            }
        }
    }
    println!("GAUSS {g}/{n}  NORMSDIST {ns}/{n}  stored-z in (0,0.5] {0}/{1}", z_le.0, z_le.1);
}
