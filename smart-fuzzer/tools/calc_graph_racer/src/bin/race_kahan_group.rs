//! W109 G6-01: Kahan expm1 GROUPING + denominator micro-search on the pure em
//! oracle (CSV double tau, r=2^-k, integer n; zero combine confound).
//! Production = ((u-1)*tau)/ln(u) mult-first. Kahan canonical = (u-1)*(tau/ln u).
use oxfunc_core::excel_numeric::research as rx;
use rx::{
    CW_PC53_RN, CW_PC64_RN, ext_div, ext_from_f64, ext_fyl2x, ext_ln2, ext_mul, ext_one, ext_sub,
    ext_to_f64,
};

fn lnu_ext_of_dbl(u: f64) -> f64 {
    // ln(u) via fyl2x on the double u, RN53 store (== excel_log basically)
    ext_to_f64(&ext_fyl2x(&ext_ln2(), &ext_from_f64(u), CW_PC64_RN), CW_PC53_RN)
}

fn main() {
    let csv = std::fs::read_to_string("../../work/w109/G6-solvers/expm1_intermediates.csv").unwrap();
    let cw = CW_PC64_RN;
    let labels = [
        "G1 ((u-1)*t)/lnu  [prod]",
        "G2 (u-1)*(t/lnu)  [Kahan]",
        "G3 (u-1)*t*(1/lnu)",
        "G4 (u-1)+(u-1)*(t-lnu)/lnu",
        "G5 (u-1)*(t/lnu) ext-div",
        "G6 ext num, ext (t/lnu), spill",
        "G7 (u-1)/lnu*t",
        "G8 y*(t/lnu) y=fl(u-1) lnu ext",
    ];
    let mut score = [0u32; 8];
    let mut misses = [0u32; 8];
    let mut tot = 0u32;
    for line in csv.lines().skip(1) {
        let f: Vec<&str> = line.split(',').collect();
        if f.len() < 6 { continue; }
        let tau = f64::from_bits(u64::from_str_radix(f[2], 16).unwrap());
        let pin = u64::from_str_radix(f[5], 16).unwrap();
        let u = rx::excel_exp(tau);
        let y = u - 1.0;
        let lnu = rx::excel_ln(u);
        let lnu2 = lnu_ext_of_dbl(u);

        let g1 = (y * tau / lnu).to_bits();
        let g2 = (y * (tau / lnu)).to_bits();
        let g3 = (y * tau * (1.0 / lnu)).to_bits();
        let g4 = (y + y * (tau - lnu) / lnu).to_bits();
        // ext divide t/lnu then double multiply
        let tql = ext_to_f64(&ext_div(&ext_from_f64(tau), &ext_from_f64(lnu), cw), CW_PC53_RN);
        let g5 = (y * tql).to_bits();
        // full ext: num=(u-1)*(tau/lnu) all ext off double u/tau/lnu, spill RN53
        let g6 = {
            let ye = ext_sub(&ext_from_f64(u), &ext_one(), cw);
            let q = ext_div(&ext_from_f64(tau), &ext_from_f64(lnu), cw);
            ext_to_f64(&ext_mul(&ye, &q, cw), CW_PC53_RN).to_bits()
        };
        let g7 = ((y / lnu) * tau).to_bits();
        let g8 = (y * (tau / lnu2)).to_bits();

        let gs = [g1, g2, g3, g4, g5, g6, g7, g8];
        for i in 0..8 { if gs[i] == pin { score[i] += 1; } else { misses[i] += 1; } }
        tot += 1;
    }
    println!("=== Kahan grouping race on pure em oracle, N={} ===", tot);
    for i in 0..8 {
        println!("  {:32} {:3}/{}  ({:.1}%)  misses={}", labels[i], score[i], tot,
                 100.0 * score[i] as f64 / tot as f64, misses[i]);
    }
}
