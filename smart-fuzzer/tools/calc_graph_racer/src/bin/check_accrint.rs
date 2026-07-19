//! Score the PRODUCTION ACCRINT kernel: stdin "issue first settle rate par
//! freq basis calc" hex bits per line -> kernel value bits (or ERR). W109
//! G6-02 accumulation-staging lane.
use oxfunc_core::functions::bond_core_family::accrint_kernel;
use std::io::BufRead;

fn main() {
    let stdin = std::io::stdin();
    for line in stdin.lock().lines() {
        let line = line.unwrap();
        let p: Vec<&str> = line.split_whitespace().collect();
        if p.len() != 8 {
            continue;
        }
        let g =
            |s: &str| f64::from_bits(u64::from_str_radix(s.trim_start_matches("0x"), 16).unwrap());
        match accrint_kernel(
            g(p[0]),
            g(p[1]),
            g(p[2]),
            g(p[3]),
            Some(g(p[4])),
            g(p[5]),
            Some(g(p[6])),
            Some(g(p[7]) != 0.0),
        ) {
            Ok(v) => println!("{:016x}", v.to_bits()),
            Err(_) => println!("ERR"),
        }
    }
}
