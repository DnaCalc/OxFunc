//! Series-site variant: input y bits; t1 = 2*fyl2x-ln(y) - y all EXTENDED;
//! exp chain on the extended argument; print rn53 and rz53.
use oxfunc_core::excel_numeric::research as rx;
use rx::{
    CW_PC64_RN, ext_abs, ext_add, ext_div, ext_f2xm1, ext_from_f64, ext_fyl2x, ext_l2e, ext_ln2,
    ext_mul, ext_one, ext_rndint, ext_scale, ext_sub, ext_to_f64,
};
use std::io::BufRead;

const CW: u16 = CW_PC64_RN;
const CW_RZ: u16 = CW_PC64_RN | 0x0C00;

fn main() {
    let stdin = std::io::stdin();
    for line in stdin.lock().lines() {
        let line = line.unwrap();
        let s = line.trim().trim_start_matches("0x");
        if s.is_empty() {
            continue;
        }
        let y = f64::from_bits(u64::from_str_radix(s, 16).unwrap());
        let ye = ext_from_f64(y);
        let l = ext_fyl2x(&ext_ln2(), &ye, CW); // ln(y) extended
        let t1 = ext_sub(&ext_mul(&ext_from_f64(2.0), &l, CW), &ye, CW);
        // exp chain on extended argument
        let t = ext_mul(&t1, &ext_l2e(), CW);
        let k = ext_rndint(&t, CW);
        let f = ext_sub(&t, &k, CW);
        let neg = ext_to_f64(&f, CW) < 0.0;
        let w = ext_f2xm1(&ext_abs(&f, CW), CW);
        let mut m = ext_add(&w, &ext_one(), CW);
        if neg {
            m = ext_div(&ext_one(), &m, CW);
        }
        let v = ext_scale(&m, &k, CW);
        println!(
            "{:016x} {:016x} {:016x}",
            y.to_bits(),
            ext_to_f64(&v, CW).to_bits(),
            ext_to_f64(&v, CW_RZ).to_bits()
        );
    }
}
