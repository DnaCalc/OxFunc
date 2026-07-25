//! W109 G6-01: Kahan numerator using the ACCURATE u-1 from the exp reduction
//! (u-1 = -w*m, w=F2XM1(|f|), m=1/(1+w)) instead of RN(u)-1 — exposes F2XM1's
//! tail in the numerator only, keeping the Kahan /ln(u) denominator. Matches the
//! D-histogram "upstream numerator ~0.5 ULP" signature. Excel-exact tau,u,lnu.
use oxfunc_core::excel_numeric::research as rx;
use rx::{
    CW_PC53_RN, CW_PC64_RN, Ext80, ext_abs, ext_add, ext_chs, ext_div, ext_f2xm1, ext_from_f64,
    ext_l2e, ext_mul, ext_one, ext_rndint, ext_sub, ext_to_f64,
};
const RN53: u16 = CW_PC53_RN;
const CW: u16 = CW_PC64_RN;
fn sp(v: &Ext80) -> f64 { ext_to_f64(v, RN53) }

// accurate (u-1) for K=0 neg branch: -w*m, m=1/(1+w). Returns Ext80.
fn accurate_um1(tau: f64) -> (Ext80, f64 /*K*/, bool) {
    let t = ext_mul(&ext_from_f64(tau), &ext_l2e(), CW);
    let k = ext_rndint(&t, CW);
    let kv = ext_to_f64(&k, CW);
    let f = ext_sub(&t, &k, CW);
    let neg = ext_to_f64(&f, CW) < 0.0;
    let w = ext_f2xm1(&ext_abs(&f, CW), CW);
    let onepw = ext_add(&w, &ext_one(), CW);
    let m = ext_div(&ext_one(), &onepw, CW);
    let um1 = ext_chs(&ext_mul(&w, &m, CW), CW); // -w*m
    (um1, kv, neg)
}

fn main() {
    let csv = std::fs::read_to_string("../../work/w109/G6-solvers/expm1_intermediates.csv").unwrap();
    let labels = [
        "W0 y=RN(u)-1 spill-Kahan [165]",
        "W1 y=fl53(-w*m) spill-Kahan",
        "W2 y=-w*m ext, num ext, /lnu spill",
        "W3 y=-w*m ext, num ext, /lnu ext",
    ];
    let mut score = [0u32; 4];
    let mut score_k0 = [0u32; 4];
    let (mut tot, mut tot_k0) = (0u32, 0u32);
    for line in csv.lines().skip(1) {
        let ff: Vec<&str> = line.split(',').collect();
        if ff.len() < 6 { continue; }
        let tau = f64::from_bits(u64::from_str_radix(ff[2], 16).unwrap());
        let u = f64::from_bits(u64::from_str_radix(ff[3], 16).unwrap());
        let lnu = f64::from_bits(u64::from_str_radix(ff[4], 16).unwrap());
        let pin = u64::from_str_radix(ff[5], 16).unwrap();
        let (um1_ext, kv, neg) = accurate_um1(tau);
        let is_k0 = kv == 0.0 && neg;

        // W0 standard spill-Kahan
        let y0 = u - 1.0;
        let num0 = sp(&ext_mul(&ext_from_f64(y0), &ext_from_f64(tau), CW));
        let w0 = sp(&ext_div(&ext_from_f64(num0), &ext_from_f64(lnu), CW)).to_bits();
        // W1 accurate y spilled
        let y1 = sp(&um1_ext);
        let num1 = sp(&ext_mul(&ext_from_f64(y1), &ext_from_f64(tau), CW));
        let w1 = sp(&ext_div(&ext_from_f64(num1), &ext_from_f64(lnu), CW)).to_bits();
        // W2 accurate y extended, numerator extended, spill divide by double lnu
        let num2 = ext_mul(&um1_ext, &ext_from_f64(tau), CW);
        let w2 = sp(&ext_div(&ext_from_f64(sp(&num2)), &ext_from_f64(lnu), CW)).to_bits();
        // W3 full extended (y, num, and lnu widened), one final store
        let w3 = ext_to_f64(&ext_div(&num2, &ext_from_f64(lnu), CW), RN53).to_bits();

        let cands = [w0, w1, w2, w3];
        for i in 0..4 { if cands[i] == pin { score[i] += 1; if is_k0 { score_k0[i] += 1; } } }
        if is_k0 { tot_k0 += 1; }
        tot += 1;
    }
    println!("=== Kahan with w-based accurate numerator, N={} (K0={}) ===", tot, tot_k0);
    for i in 0..4 {
        println!("  {:34} all {:3}/{} ({:4.1}%)  K0 {:3}/{} ({:4.1}%)",
                 labels[i], score[i], tot, 100.0*score[i] as f64/tot as f64,
                 score_k0[i], tot_k0, 100.0*score_k0[i] as f64/tot_k0 as f64);
    }
}
