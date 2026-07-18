//! W109 lane-1: WEIBULL pdf staging primitives (b27 pdf corpus).
//!
//! Stdin: "x_bits alpha_bits beta_bits" per row.  Emits the x87-dependent
//! primitives; the Python scorer composes candidate op-graphs (plain RN53
//! products/divisions) from them.
//!
//! Columns (all hex bits):
//!   ratio  = RN53(x/beta)
//!   am1    = RN53(alpha-1)
//!   pd     = chain_pow(ratio, am1)          direct, signed exponent
//!   pr     = am1<0 ? x87_recip(chain_pow(ratio,-am1)) : pd   POWER-style
//!   tp     = chain_pow(ratio, alpha)        the cdf-site pow
//!   e1     = excel_exp(-tp)
//!   e2     = excel_exp(-RN53(pd*ratio))     t derived as p*ratio
//!   e3     = excel_exp(-RN53(pr*ratio))
//!
//! chain_pow = excel_exp(x87_mul(y, excel_ln(base))) — the b27-pinned
//! double-rounded product staging, no shortcuts.  chain_pow(base, 0) = 1.

use oxfunc_core::excel_numeric::research as rx;
use std::io::BufRead;

fn chain_pow(base: f64, y: f64) -> f64 {
    rx::excel_exp(rx::x87_mul(y, rx::excel_ln(base)))
}

fn main() {
    let stdin = std::io::stdin();
    for line in stdin.lock().lines() {
        let line = line.unwrap();
        let mut it = line.split_whitespace();
        let (Some(xs), Some(as_), Some(bs)) = (it.next(), it.next(), it.next()) else {
            continue;
        };
        let g = |s: &str| f64::from_bits(u64::from_str_radix(s.trim_start_matches("0x"), 16).unwrap());
        let (x, alpha, beta) = (g(xs), g(as_), g(bs));

        let ratio = x / beta;
        let am1 = alpha - 1.0;
        let pd = chain_pow(ratio, am1);
        let pr = if am1 < 0.0 {
            rx::x87_recip(chain_pow(ratio, -am1))
        } else {
            pd
        };
        let tp = chain_pow(ratio, alpha);
        let e1 = rx::excel_exp(-tp);
        let e2 = rx::excel_exp(-(pd * ratio));
        let e3 = rx::excel_exp(-(pr * ratio));

        println!(
            "{:016x} {:016x} {:016x} {:016x} {:016x} {:016x} {:016x} {:016x}",
            ratio.to_bits(),
            am1.to_bits(),
            pd.to_bits(),
            pr.to_bits(),
            tp.to_bits(),
            e1.to_bits(),
            e2.to_bits(),
            e3.to_bits()
        );
    }
}
