//! Score the PRODUCTION WEIBULL.DIST kernel: stdin "x a beta cum" hex bits
//! per line -> prints the kernel value's bits (or ERR on worksheet error).
use oxfunc_core::functions::special_dist_family::weibull_dist_kernel;
use std::io::BufRead;

fn main() {
    let stdin = std::io::stdin();
    for line in stdin.lock().lines() {
        let line = line.unwrap();
        let p: Vec<&str> = line.split_whitespace().collect();
        if p.len() != 4 {
            continue;
        }
        let g = |s: &str| f64::from_bits(u64::from_str_radix(s.trim_start_matches("0x"), 16).unwrap());
        match weibull_dist_kernel(g(p[0]), g(p[1]), g(p[2]), g(p[3]) != 0.0) {
            Ok(v) => println!("{:016x}", v.to_bits()),
            Err(_) => println!("ERR"),
        }
    }
}
