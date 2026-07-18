//! Score the PRODUCTION EXPON.DIST kernel: stdin "x lambda cum" hex bits per
//! line -> kernel value bits (or ERR).
use oxfunc_core::functions::discrete_dist_family::expon_dist_kernel;
use std::io::BufRead;

fn main() {
    let stdin = std::io::stdin();
    for line in stdin.lock().lines() {
        let line = line.unwrap();
        let p: Vec<&str> = line.split_whitespace().collect();
        if p.len() != 3 {
            continue;
        }
        let g = |s: &str| f64::from_bits(u64::from_str_radix(s.trim_start_matches("0x"), 16).unwrap());
        match expon_dist_kernel(g(p[0]), g(p[1]), g(p[2]) != 0.0) {
            Ok(v) => println!("{:016x}", v.to_bits()),
            Err(_) => println!("ERR"),
        }
    }
}
