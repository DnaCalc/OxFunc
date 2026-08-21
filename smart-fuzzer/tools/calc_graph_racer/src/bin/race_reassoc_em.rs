//! W109 G6-01: the Goldberg/Kahan expm1 correction form. tau/lnu ≈ 1 (lnu=log(exp(tau))≈tau),
//! so the natural human code is em = (u-1) * (tau/log(u))  [correction factor ~1], which rounds
//! DIFFERENTLY from the left-associated ((u-1)*tau)/log(u). Test all associations of the
//! two multiplies and one divide over the three exact operands a=(u-1), tau, lnu. Confound-free
//! oracle (u,tau,lnu are Excel's exact captures).
use oxfunc_core::excel_numeric::research as rx;
use rx::{CW_PC53_RN, CW_PC64_RN, ext_div, ext_from_f64, ext_mul, ext_to_f64};

const RN64: u16 = CW_PC64_RN;
const RN53: u16 = CW_PC53_RN;

fn b(s: &str) -> f64 {
    f64::from_bits(u64::from_str_radix(s, 16).unwrap())
}

fn main() {
    let csv =
        std::fs::read_to_string("../../work/w109/G6-solvers/expm1_intermediates.csv").unwrap();
    let names = [
        "0 ((a*tau)/lnu)         baseline dbl",
        "1 (a*(tau/lnu))         corr-form dbl",
        "2 ((a/lnu)*tau)         dbl",
        "3 (tau*(a/lnu))         dbl",
        "4 a*(tau/lnu) tau/lnu EXT then *a dbl",
        "5 (a*tau)/lnu all-x87 PC64->RN53",
        "6 a*(tau/lnu) all-x87 PC64->RN53",
        "7 (a/lnu)*tau all-x87 PC64->RN53",
    ];
    let mut score = [0u32; 8];
    let mut score_miss = [0u32; 8];
    let (mut tot, mut nmiss) = (0u32, 0u32);
    let mut fix1: Vec<(i32, u32)> = Vec::new();
    let mut break1 = 0u32;

    for line in csv.lines().skip(1) {
        let f: Vec<&str> = line.split(',').collect();
        if f.len() < 6 {
            continue;
        }
        let k: i32 = f[0].parse().unwrap();
        let n: u32 = f[1].parse().unwrap();
        let tau = b(f[2]);
        let u = b(f[3]);
        let lnu = b(f[4]);
        let em = b(f[5]);
        let emb = em.to_bits();
        tot += 1;
        let a = u - 1.0;

        // double associations
        let c0 = ((a * tau) / lnu).to_bits();
        let c1 = (a * (tau / lnu)).to_bits();
        let c2 = ((a / lnu) * tau).to_bits();
        let c3 = (tau * (a / lnu)).to_bits();
        // tau/lnu extended, then * a, store double
        let ae = ext_from_f64(a);
        let te = ext_from_f64(tau);
        let le = ext_from_f64(lnu);
        let q_ext = ext_div(&te, &le, RN64);
        let c4 = ext_to_f64(&ext_mul(&ae, &q_ext, RN64), RN53).to_bits();
        // all-x87 PC64 spill-loop variants
        let c5 = ext_to_f64(&ext_div(&ext_mul(&ae, &te, RN64), &le, RN64), RN53).to_bits();
        let c6 = ext_to_f64(&ext_mul(&ae, &ext_div(&te, &le, RN64), RN64), RN53).to_bits();
        let c7 = ext_to_f64(&ext_mul(&ext_div(&ae, &le, RN64), &te, RN64), RN53).to_bits();

        let cands = [c0, c1, c2, c3, c4, c5, c6, c7];
        let base_miss = c0 != emb;
        if base_miss {
            nmiss += 1;
        }
        for i in 0..8 {
            if cands[i] == emb {
                score[i] += 1;
                if base_miss {
                    score_miss[i] += 1;
                }
            }
        }
        if base_miss {
            if c1 == emb {
                fix1.push((k, n));
            }
        } else if c1 != emb {
            break1 += 1;
        }
    }

    println!("N={} misses={}", tot, nmiss);
    println!("{:<40} {:>12} {:>10}", "candidate", "all", "on-miss");
    for i in 0..8 {
        println!(
            "{:<40} {:>4}/{} ({:4.1}%) {:>4}/{}",
            names[i],
            score[i],
            tot,
            100.0 * score[i] as f64 / tot as f64,
            score_miss[i],
            nmiss
        );
    }
    println!(
        "\ncorr-form(1) FIXES {} miss rows, BREAKS {} hit rows",
        fix1.len(),
        break1
    );
    print!("fixed: ");
    for x in fix1.iter().take(20) {
        print!("({},{}) ", x.0, x.1);
    }
    println!();
}
