//! Score CHIDIST df=1 vs live 20326 after ERFC complement.

use oxfunc_core::functions::chi_f_t_family::chisq_dist_rt_kernel;
use oxfunc_core::functions::special_dist_family::erfc_of_sqrt_half_x;
use serde::Deserialize;
use std::fs;

#[derive(Deserialize)]
struct Row {
    x: f64,
    rt_bits: String,
}

fn bits(s: &str) -> u64 {
    u64::from_str_radix(s.trim_start_matches("0x"), 16).unwrap()
}

fn main() {
    let path = std::env::args().nth(1).unwrap_or_else(|| {
        "../../work/w109/lastbit-12h-20260912/chidist-df1-small/capture.jsonl".into()
    });
    let mut n = 0usize;
    let mut chi = 0usize;
    let mut er = 0usize;
    for line in fs::read_to_string(&path).unwrap().lines() {
        if line.is_empty() {
            continue;
        }
        let r: Row = serde_json::from_str(line).unwrap();
        let want = bits(&r.rt_bits);
        n += 1;
        if chisq_dist_rt_kernel(r.x, 1.0).unwrap().to_bits() == want {
            chi += 1;
        }
        if erfc_of_sqrt_half_x(r.x).unwrap().to_bits() == want {
            er += 1;
        }
    }
    println!("CHIDIST df=1 vs Excel {chi}/{n}  ERFC(sqrt(x/2)) {er}/{n}");
}
