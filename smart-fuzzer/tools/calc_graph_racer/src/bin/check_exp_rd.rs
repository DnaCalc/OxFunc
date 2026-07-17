//! Validate oxfunc_core's exp_rd (round-toward-zero exp) by dumping bits for
//! args supplied as hex bit patterns on stdin, one per line. Compared against
//! mpmath's floor(true exp) on the Python side.

use oxfunc_core::functions::special_math_common::exp_rd;
use std::io::BufRead;

fn main() {
    let stdin = std::io::stdin();
    for line in stdin.lock().lines() {
        let line = line.unwrap();
        let s = line.trim().trim_start_matches("0x");
        if s.is_empty() {
            continue;
        }
        let bits = u64::from_str_radix(s, 16).unwrap();
        let x = f64::from_bits(bits);
        println!("{:016x} {:016x}", bits, exp_rd(x).to_bits());
    }
}
