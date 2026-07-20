//! Score the PRODUCTION DURATION kernel: stdin "settle mat coupon yld freq
//! basis" hex bits per line -> kernel value bits (or ERR). W109 G6-03c lane.
use oxfunc_core::functions::bond_core_family::duration_kernel;
use std::io::BufRead;

fn main() {
    let stdin = std::io::stdin();
    for line in stdin.lock().lines() {
        let line = line.unwrap();
        let p: Vec<&str> = line.split_whitespace().collect();
        if p.len() != 6 {
            continue;
        }
        let g =
            |s: &str| f64::from_bits(u64::from_str_radix(s.trim_start_matches("0x"), 16).unwrap());
        match duration_kernel(g(p[0]), g(p[1]), g(p[2]), g(p[3]), g(p[4]), Some(g(p[5]))) {
            Ok(v) => println!("{:016x}", v.to_bits()),
            Err(_) => println!("ERR"),
        }
    }
}
