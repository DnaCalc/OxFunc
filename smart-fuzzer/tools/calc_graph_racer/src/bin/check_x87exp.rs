//! W109 hardware-chain exp verification: for each stdin arg (hex bits), print
//! RN53 and RZ53 publications of the REAL x87 fFEXP chain (hardware F2XM1 via
//! the Ext80 ops), plus the -expm1(-x) staged wrapper value.
//!
//! Output per line: "x_bits rn53_bits rz53_bits expm1neg_bits"

use oxfunc_core::excel_numeric::research as rx;
use rx::{
    CW_PC64_RN, ext_abs, ext_add, ext_div, ext_f2xm1, ext_from_f64, ext_l2e, ext_mul, ext_one,
    ext_rndint, ext_scale, ext_sub, ext_to_f64,
};
use std::io::BufRead;

const CW_RN: u16 = CW_PC64_RN; // 0x133F: PC=64, RC=nearest
const CW_RZ: u16 = CW_PC64_RN | 0x0C00; // RC=11: toward zero at the final store

fn exp_chain_ext(x: f64) -> rx::Ext80 {
    let cw = CW_RN;
    let t = ext_mul(&ext_from_f64(x), &ext_l2e(), cw);
    let k = ext_rndint(&t, cw);
    let f = ext_sub(&t, &k, cw);
    let neg = ext_to_f64(&f, cw) < 0.0;
    let w = ext_f2xm1(&ext_abs(&f, cw), cw);
    let mut m = ext_add(&w, &ext_one(), cw);
    if neg {
        m = ext_div(&ext_one(), &m, cw);
    }
    ext_scale(&m, &k, cw)
}

fn main() {
    let stdin = std::io::stdin();
    for line in stdin.lock().lines() {
        let line = line.unwrap();
        let s = line.trim().trim_start_matches("0x");
        if s.is_empty() {
            continue;
        }
        let x = f64::from_bits(u64::from_str_radix(s, 16).unwrap());
        let v = exp_chain_ext(x);
        let rn = ext_to_f64(&v, CW_RN);
        let rz = ext_to_f64(&v, CW_RZ);
        let em1 = -rx::excel_expm1_internal(-x);
        println!(
            "{:016x} {:016x} {:016x} {:016x}",
            x.to_bits(),
            rn.to_bits(),
            rz.to_bits(),
            em1.to_bits()
        );
    }
}

// (variant harness appended for the series-site test: input y bits, compute
// t1 = 2*ln(y) - y fully EXTENDED via fyl2x, feed extended into the chain.)
