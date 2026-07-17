//! W109 G3-02 landing verification: dump the ported Excel GAMMALN kernel's bits
//! for positive arguments supplied as hex bit patterns on stdin (one per line).
//! Exercises the WIRED production surface path
//! (`special_dist_family::gammaln_kernel`, positive x -> `excel_numeric::
//! gammaln_excel`). Compared on the Python side (`agentP_score.py`) against the
//! same answer files the reference scorer (`agentL_composite3.py`) uses.
//!
//! Output per line: `<argbits> <resultbits>` (both 16 hex digits). A `#NUM!`
//! domain result (x <= 0 / non-finite) prints `ffffffffffffffff`.

use oxfunc_core::functions::special_dist_family::gammaln_kernel;
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
        let out = match gammaln_kernel(x) {
            Ok(v) => v.to_bits(),
            Err(_) => u64::MAX,
        };
        println!("{bits:016x} {out:016x}");
    }
}
