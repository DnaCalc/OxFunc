//! W109 G6-01 DIAGNOSTIC: pin down what em_pinned IS, op by op.
//! Uses the CSV's exact double tau (tau_bits) so we match the pinning's argument
//! and isolate the Kahan internal roundings. em is negative; "toward zero" = up.
use oxfunc_core::excel_numeric::research as rx;
use rx::{
    CW_PC53_RN, CW_PC64_RN, Ext80, ext_div, ext_from_f64, ext_fyl2x, ext_ln2, ext_mul, ext_one,
    ext_sub, ext_to_f64,
};

const RZ53: u16 = CW_PC53_RN | 0x0C00; // toward zero, double precision store
const RN53: u16 = CW_PC53_RN;

// near-true em = (1+r)^-n - 1 in ext80 (PC64 throughout), returned as Ext80
fn em_true_ext(r: f64, n: u32) -> Ext80 {
    let cw = CW_PC64_RN;
    let opr = ext_from_f64(1.0 + r);
    // (1+r)^n by ext binexp
    let mut result = ext_one();
    let mut b = opr;
    let mut e = n;
    while e > 0 {
        if e & 1 == 1 { result = ext_mul(&result, &b, cw); }
        e >>= 1;
        if e > 0 { b = ext_mul(&b, &b, cw); }
    }
    let v = ext_div(&ext_one(), &result, cw); // (1+r)^-n
    ext_sub(&v, &ext_one(), cw)
}

fn main() {
    let csv = std::fs::read_to_string("../../work/w109/G6-solvers/expm1_intermediates.csv").unwrap();
    let cw = CW_PC64_RN;
    // counters
    let (mut n_true_rn, mut n_true_rz, mut n_kahan, mut n_kahan_qrz, mut n_lnu_rz,
         mut n_num_rz, mut n_kahan_ext_rz) = (0u32, 0u32, 0u32, 0u32, 0u32, 0u32, 0u32);
    let mut tot = 0u32;
    // co-occurrence: among production-kahan MISSES, how many each alt fixes
    let (mut miss, mut fix_true_rz, mut fix_qrz, mut fix_lnu_rz, mut fix_num_rz) = (0u32,0u32,0u32,0u32,0u32);
    for line in csv.lines().skip(1) {
        let f: Vec<&str> = line.split(',').collect();
        if f.len() < 6 { continue; }
        let n: u32 = f[1].parse().unwrap();
        let tau_bits = u64::from_str_radix(f[2], 16).unwrap();
        let pin = u64::from_str_radix(f[5], 16).unwrap();
        let k: i32 = f[0].parse().unwrap();
        let r = 2f64.powi(k);
        let tau = f64::from_bits(tau_bits);
        tot += 1;

        // true em roundings
        let te = em_true_ext(r, n);
        let true_rn = ext_to_f64(&te, RN53).to_bits();
        let true_rz = ext_to_f64(&te, RZ53).to_bits();

        // production double Kahan on the CSV tau
        let u = rx::excel_exp(tau);
        let kahan = ((u - 1.0) * tau / rx::excel_ln(u)).to_bits();
        // Kahan with quotient stored RZ (num,den in double, divide toward zero)
        let num_d = (u - 1.0) * tau;
        let q_rz = ext_to_f64(&ext_div(&ext_from_f64(num_d), &ext_from_f64(rx::excel_ln(u)), cw), RZ53).to_bits();
        // Kahan with lnu computed toward zero (ext ln of double u, RZ store), rest double
        let lnu_rz = ext_to_f64(&ext_fyl2x(&ext_ln2(), &ext_from_f64(u), cw), RZ53);
        let kahan_lnu_rz = ((u - 1.0) * tau / lnu_rz).to_bits();
        // Kahan with numerator product toward zero
        let num_rz = ext_to_f64(&ext_mul(&ext_sub(&ext_from_f64(u), &ext_one(), cw), &ext_from_f64(tau), cw), RZ53);
        let kahan_num_rz = (num_rz / rx::excel_ln(u)).to_bits();
        // Kahan fully ext off double u/tau, quotient RZ
        let num_e = ext_mul(&ext_sub(&ext_from_f64(u), &ext_one(), cw), &ext_from_f64(tau), cw);
        let den_e = ext_fyl2x(&ext_ln2(), &ext_from_f64(u), cw);
        let kahan_ext_rz = ext_to_f64(&ext_div(&num_e, &den_e, cw), RZ53).to_bits();

        if true_rn == pin { n_true_rn += 1; }
        if true_rz == pin { n_true_rz += 1; }
        if kahan == pin { n_kahan += 1; }
        if q_rz == pin { n_kahan_qrz += 1; }
        if kahan_lnu_rz == pin { n_lnu_rz += 1; }
        if kahan_num_rz == pin { n_num_rz += 1; }
        if kahan_ext_rz == pin { n_kahan_ext_rz += 1; }

        if kahan != pin {
            miss += 1;
            if true_rz == pin { fix_true_rz += 1; }
            if q_rz == pin { fix_qrz += 1; }
            if kahan_lnu_rz == pin { fix_lnu_rz += 1; }
            if kahan_num_rz == pin { fix_num_rz += 1; }
        }
        let _ = k;
    }
    println!("N = {}", tot);
    println!("  true (1+r)^-n-1  RN53 == pin : {}/{}", n_true_rn, tot);
    println!("  true (1+r)^-n-1  RZ53 == pin : {}/{}", n_true_rz, tot);
    println!("  double-Kahan (RN)      == pin : {}/{}  <- production baseline", n_kahan, tot);
    println!("  Kahan quotient RZ      == pin : {}/{}", n_kahan_qrz, tot);
    println!("  Kahan lnu RZ           == pin : {}/{}", n_lnu_rz, tot);
    println!("  Kahan numerator RZ     == pin : {}/{}", n_num_rz, tot);
    println!("  Kahan ext-off-dbl qRZ  == pin : {}/{}", n_kahan_ext_rz, tot);
    println!("\nAmong {} production-Kahan MISSES, fixed by:", miss);
    println!("  true RZ      : {}", fix_true_rz);
    println!("  quotient RZ  : {}", fix_qrz);
    println!("  lnu RZ       : {}", fix_lnu_rz);
    println!("  numerator RZ : {}", fix_num_rz);
}
