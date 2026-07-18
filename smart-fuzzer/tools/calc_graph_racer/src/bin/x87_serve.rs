//! W109 excel-primitive op server: line-oriented request/response so Python
//! candidate-graph explorers can call the REAL hardware chains per op.
//!
//! Request per stdin line:  "<op> <hex> [<hex>]"
//!   exp x      — excel_exp (fFEXP chain, RN53 publish)
//!   expz x     — excel_exp_rz (chop publish, series r-site)
//!   ln x       — excel_ln (fyl2x chain)
//!   expm1 x    — excel_expm1_internal (Kahan)
//!   mul a b    — x87 double-rounded product RN53(RN64(a*b))
//!   div a b    — x87 double-rounded quotient RN53(RN64(a/b))
//!   recip x    — x87 double-rounded reciprocal
//!   cexpext a b — chain-exp of the EXTENDED product a*b (PC=64, no spill
//!                 between the multiply and the fFEXP chain entry)
//! Response: one hex64 per line, same order.  Unknown op -> "ERR".

use oxfunc_core::excel_numeric::research as rx;
use std::io::{BufRead, Write};

fn main() {
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let mut out = std::io::BufWriter::new(stdout.lock());
    let g = |s: &str| f64::from_bits(u64::from_str_radix(s.trim_start_matches("0x"), 16).unwrap());
    for line in stdin.lock().lines() {
        let line = line.unwrap();
        let mut it = line.split_whitespace();
        let Some(op) = it.next() else { continue };
        let a = it.next().map(g);
        let b = it.next().map(g);
        let r = match (op, a, b) {
            ("exp", Some(x), _) => rx::excel_exp(x),
            ("expz", Some(x), _) => rx::excel_exp_rz(x),
            ("ln", Some(x), _) => rx::excel_ln(x),
            ("expm1", Some(x), _) => rx::excel_expm1_internal(x),
            ("mul", Some(x), Some(y)) => rx::x87_mul(x, y),
            ("div", Some(x), Some(y)) => {
                use rx::{CW_PC64_RN, ext_div, ext_from_f64, ext_to_f64};
                ext_to_f64(
                    &ext_div(&ext_from_f64(x), &ext_from_f64(y), CW_PC64_RN),
                    CW_PC64_RN,
                )
            }
            ("recip", Some(x), _) => rx::x87_recip(x),
            ("cexpext", Some(x), Some(y)) => {
                use rx::{
                    CW_PC64_RN, ext_abs, ext_add, ext_div, ext_f2xm1, ext_from_f64, ext_l2e,
                    ext_mul, ext_one, ext_rndint, ext_scale, ext_sub, ext_to_f64,
                };
                let cw = CW_PC64_RN;
                let xe = ext_mul(&ext_from_f64(x), &ext_from_f64(y), cw);
                let t = ext_mul(&xe, &ext_l2e(), cw);
                let k = ext_rndint(&t, cw);
                let f = ext_sub(&t, &k, cw);
                let neg = ext_to_f64(&f, cw) < 0.0;
                let w = ext_f2xm1(&ext_abs(&f, cw), cw);
                let mut m = ext_add(&w, &ext_one(), cw);
                if neg {
                    m = ext_div(&ext_one(), &m, cw);
                }
                let r = ext_scale(&m, &k, cw);
                ext_to_f64(&r, cw)
            }
            _ => {
                writeln!(out, "ERR").unwrap();
                continue;
            }
        };
        writeln!(out, "{:016x}", r.to_bits()).unwrap();
    }
}
