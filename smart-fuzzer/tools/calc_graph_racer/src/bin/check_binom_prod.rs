//! Score the PRODUCTION BINOM.DIST kernel: stdin "k n p cum" hex bits per line
//! -> kernel value bits (or ERR). W109 lane-8 landing verification (agent-U).
use oxfunc_core::functions::discrete_dist_family::binom_dist_kernel;
use std::io::BufRead;

fn main() {
    let stdin = std::io::stdin();
    for line in stdin.lock().lines() {
        let line = line.unwrap();
        let p: Vec<&str> = line.split_whitespace().collect();
        if p.len() != 4 {
            continue;
        }
        let g =
            |s: &str| f64::from_bits(u64::from_str_radix(s.trim_start_matches("0x"), 16).unwrap());
        match binom_dist_kernel(g(p[0]), g(p[1]), g(p[2]), g(p[3]) != 0.0) {
            Ok(v) => println!("{:016x}", v.to_bits()),
            Err(_) => println!("ERR"),
        }
    }
}
