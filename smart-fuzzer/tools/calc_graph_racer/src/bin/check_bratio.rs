//! Validate oxfunc_core's bratio (incomplete-beta-ratio kernel) by dumping
//! result bits for (a, b, x, y) tuples supplied as hex bit patterns on stdin,
//! one tuple per line ("a b x y", each a bare 16-hex-digit u64, optionally
//! 0x-prefixed). Prints "w_bits w1_bits" per line as 016x hex. Compared against
//! the Python reference agentA_bratio.bratio on the Python side.

use oxfunc_core::functions::special_math_common::bratio;
use std::io::BufRead;

fn parse_bits(tok: &str) -> u64 {
    let s = tok.trim().trim_start_matches("0x");
    u64::from_str_radix(s, 16).unwrap()
}

fn main() {
    let stdin = std::io::stdin();
    for line in stdin.lock().lines() {
        let line = line.unwrap();
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let toks: Vec<&str> = line.split_whitespace().collect();
        if toks.len() != 4 {
            panic!("expected 4 hex tokens per line, got {}", toks.len());
        }
        let a = f64::from_bits(parse_bits(toks[0]));
        let b = f64::from_bits(parse_bits(toks[1]));
        let x = f64::from_bits(parse_bits(toks[2]));
        let y = f64::from_bits(parse_bits(toks[3]));
        let (w, w1) = bratio(a, b, x, y);
        println!("{:016x} {:016x}", w.to_bits(), w1.to_bits());
    }
}
