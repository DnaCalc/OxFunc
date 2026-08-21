//! W109 G6-01 (Fable H1): the x87 SPILL-LOOP Kahan. Same algebra as the 163
//! model, but each op = PC=64 in-register THEN FSTP to double = fl53(fl64(.)),
//! a genuine double-rounding per op. Differs from SSE2 RN53 exactly in the
//! 2^-12-ulp midpoint windows this adversarial oracle lives in. Excel-exact
//! u, tau, lnu from expm1_intermediates.csv.
use oxfunc_core::excel_numeric::research as rx;
use rx::{
    CW_PC53_RN, CW_PC64_RN, ext_div, ext_from_f64, ext_fyl2x, ext_ln2, ext_mul, ext_one, ext_sub,
    ext_to_f64,
};

const RN53: u16 = CW_PC53_RN;

// fl53(fl64(a*b)) : x87 multiply at PC64, spill to double
fn mul_spill(a: f64, b: f64) -> f64 {
    ext_to_f64(
        &ext_mul(&ext_from_f64(a), &ext_from_f64(b), CW_PC64_RN),
        RN53,
    )
}
// fl53(fl64(a/b)) : x87 divide at PC64, spill to double
fn div_spill(a: f64, b: f64) -> f64 {
    ext_to_f64(
        &ext_div(&ext_from_f64(a), &ext_from_f64(b), CW_PC64_RN),
        RN53,
    )
}
// ln(u) via FYL2X kept in register (extended), returned as Ext80
fn ln_ext(u: f64) -> oxfunc_core::excel_numeric::research::Ext80 {
    ext_fyl2x(&ext_ln2(), &ext_from_f64(u), CW_PC64_RN)
}

fn main() {
    let csv =
        std::fs::read_to_string("../../work/w109/G6-solvers/expm1_intermediates.csv").unwrap();
    let labels = [
        "S0 SSE2 (y*x)/lnu [163]",
        "S1 num spill, div SSE2",
        "S2 num spill, div spill",
        "S3 num SSE2, div spill",
        "S4 num spill / lnu_ext(reg), store",
        "S5 num_ext(reg) / lnu_ext(reg), store",
        "S6 num spill, div spill, lnu_ext den",
        "S7 y*x reg / lnu double reg, store", // fl64(y*x)/lnu one store (my old 145?)
    ];
    let mut score = [0u32; 8];
    let mut tot = 0u32;
    let mut miss7: Vec<(i32, u32, i64)> = Vec::new();
    let best_idx_track = 2usize;
    for line in csv.lines().skip(1) {
        let f: Vec<&str> = line.split(',').collect();
        if f.len() < 6 {
            continue;
        }
        let k: i32 = f[0].parse().unwrap();
        let n: u32 = f[1].parse().unwrap();
        let tau = f64::from_bits(u64::from_str_radix(f[2], 16).unwrap());
        let u = f64::from_bits(u64::from_str_radix(f[3], 16).unwrap());
        let lnu = f64::from_bits(u64::from_str_radix(f[4], 16).unwrap());
        let pin = u64::from_str_radix(f[5], 16).unwrap();
        let y = u - 1.0;

        // S0 pure SSE2
        let s0 = ((y * tau) / lnu).to_bits();
        // S1 num spilled (double-rounded), divide SSE2
        let num_sp = mul_spill(y, tau);
        let s1 = (num_sp / lnu).to_bits();
        // S2 num spill + div spill
        let s2 = div_spill(num_sp, lnu).to_bits();
        // S3 num SSE2, div spill
        let num_sse = y * tau;
        let s3 = div_spill(num_sse, lnu).to_bits();
        // S4 num spill divided by extended ln (register), final store RN53
        let s4 = ext_to_f64(
            &ext_div(&ext_from_f64(num_sp), &ln_ext(u), CW_PC64_RN),
            RN53,
        )
        .to_bits();
        // S5 num extended (register) / ln extended (register), one final store
        let num_ext = ext_mul(&ext_from_f64(y), &ext_from_f64(tau), CW_PC64_RN);
        let s5 = ext_to_f64(&ext_div(&num_ext, &ln_ext(u), CW_PC64_RN), RN53).to_bits();
        // S6 num spill, then divide by lnu(double) spill, but ln kept extended for the divide operand? = S4 basically; make S6 = num spill, div spill, but den = fl53(lnu) (same)
        let s6 = div_spill(num_sp, lnu).to_bits(); // identical to S2 (kept as sanity)
        // S7 fl64(y*x) register / lnu(double) register, single store (register-resident, my old test)
        let s7 = ext_to_f64(&ext_div(&num_ext, &ext_from_f64(lnu), CW_PC64_RN), RN53).to_bits();

        let cands = [s0, s1, s2, s3, s4, s5, s6, s7];
        for i in 0..8 {
            if cands[i] == pin {
                score[i] += 1;
            }
        }
        if cands[best_idx_track] != pin {
            miss7.push((k, n, (cands[best_idx_track] as i64) - (pin as i64)));
        }
        tot += 1;
    }
    println!("=== SPILL-LOOP Kahan lattice (Fable H1), N={} ===", tot);
    for i in 0..8 {
        println!(
            "  {:34} {:3}/{}  ({:.1}%)",
            labels[i],
            score[i],
            tot,
            100.0 * score[i] as f64 / tot as f64
        );
    }
    println!(
        "\nS2 (num spill+div spill) misses: {} ; dirs +{}/-{}",
        miss7.len(),
        miss7.iter().filter(|m| m.2 > 0).count(),
        miss7.iter().filter(|m| m.2 < 0).count()
    );
    for m in miss7.iter().take(15) {
        println!("    k={:3} n={:3} d={:+}", m.0, m.1, m.2);
    }
}
