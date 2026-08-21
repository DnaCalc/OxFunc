//! Score the PRODUCTION regularized_gamma_p path: stdin "x_bits a_bits" hex
//! per line -> prints P bits (GAMMA.DIST(x, a, 1, TRUE) kernel equivalent).
use oxfunc_core::functions::special_math_common::regularized_gamma_p;
use std::io::BufRead;

fn main() {
    let stdin = std::io::stdin();
    for line in stdin.lock().lines() {
        let line = line.unwrap();
        let p: Vec<&str> = line.split_whitespace().collect();
        if p.len() != 2 {
            continue;
        }
        let g =
            |s: &str| f64::from_bits(u64::from_str_radix(s.trim_start_matches("0x"), 16).unwrap());
        println!("{:016x}", regularized_gamma_p(g(p[1]), g(p[0])).to_bits());
    }
}
