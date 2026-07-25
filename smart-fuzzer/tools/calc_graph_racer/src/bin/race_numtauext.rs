//! W109 G6-01: last operand-provenance isolation. u,lnu locked to captured doubles (ln is
//! catastrophically u-sensitive so u MUST spill). Only tau's extended bits survive without
//! amplification (exp well-conditioned → exp(tau_ext) spills to same u_dbl). Test numerator =
//! (u_dbl-1) * tau_EXT, where tau_ext = -n*log1p(r) kept 80-bit; denominator = captured lnu.
use oxfunc_core::excel_numeric::research as rx;
use rx::{CW_PC53_RN, CW_PC64_RN, ext_div, ext_from_f64, ext_fyl2xp1, ext_ln2, ext_mul, ext_to_f64};
const CW: u16 = CW_PC64_RN;
const RN53: u16 = CW_PC53_RN;
fn b(s: &str) -> f64 { f64::from_bits(u64::from_str_radix(s, 16).unwrap()) }

fn main() {
    let csv = std::fs::read_to_string("../../work/w109/G6-solvers/expm1_intermediates.csv").unwrap();
    let ln2 = ext_ln2();
    let names = [
        "0 baseline (u_dbl-1)*tau_dbl / lnu           ",
        "1 (u_dbl-1)*tau_EXT [dbl mul] / lnu          ",
        "2 (u_dbl-1)*tau_EXT [x87 num ext] / lnu RN53 ",
        "3 (u_dbl-1)*tau_EXT ext / lnu_ext x87 /PC64  ",
    ];
    let mut score = [0u32; 4]; let mut score_miss = [0u32; 4];
    let (mut tot, mut nmiss) = (0u32, 0u32);
    let mut tau_ext_ok = 0u32;
    let mut fix1: Vec<(i32,u32)> = Vec::new(); let mut break1 = 0u32;

    for line in csv.lines().skip(1) {
        let f: Vec<&str> = line.split(',').collect();
        if f.len() < 6 { continue; }
        let k: i32 = f[0].parse().unwrap(); let n: u32 = f[1].parse().unwrap();
        let tau = b(f[2]); let u = b(f[3]); let lnu = b(f[4]); let em = b(f[5]); let emb = em.to_bits();
        tot += 1;
        let r = 2f64.powi(k);
        let l1p = ext_fyl2xp1(&ln2, &ext_from_f64(r), CW);   // ln(1+r) 80-bit
        let tau_ext = ext_mul(&ext_from_f64(-(n as f64)), &l1p, CW);
        if ext_to_f64(&tau_ext, RN53).to_bits() == tau.to_bits() { tau_ext_ok += 1; }
        let a = u - 1.0;
        let a_ext = ext_from_f64(a);
        let lnu_ext = ext_from_f64(lnu);

        // c1: num = (u-1)*tau_ext but rounded to double before divide? do double mul of a with spilled tau_ext
        let tau_ext_as_dbl = ext_to_f64(&tau_ext, RN53); // == tau usually
        let c1 = (a * tau_ext_as_dbl / lnu).to_bits(); // (this == baseline where tau_ext spills to tau)
        // c2: numerator kept 80-bit ((u-1)*tau_ext), spill to double, then /lnu double
        let num_ext = ext_mul(&a_ext, &tau_ext, CW);
        let num_dbl = ext_to_f64(&num_ext, RN53);
        let c2 = (num_dbl / lnu).to_bits();
        // c3: numerator 80-bit, divide by captured lnu at x87 PC64, store RN53
        let c3 = ext_to_f64(&ext_div(&num_ext, &lnu_ext, CW), RN53).to_bits();
        let c0 = ((u - 1.0) * tau / lnu).to_bits();

        let cands = [c0, c1, c2, c3];
        let base_miss = c0 != emb;
        if base_miss { nmiss += 1; }
        for i in 0..4 { if cands[i]==emb { score[i]+=1; if base_miss { score_miss[i]+=1; } } }
        if base_miss { if c2==emb { fix1.push((k,n)); } } else if c2!=emb { break1+=1; }
    }
    println!("N={} misses={} | tau_ext spills==captured tau on {}/{}", tot, nmiss, tau_ext_ok, tot);
    println!("{:<46} {:>12} {:>10}", "candidate", "all", "on-miss");
    for i in 0..4 {
        println!("{:<46} {:>4}/{} ({:4.1}%) {:>4}/{}", names[i], score[i], tot,
                 100.0*score[i] as f64/tot as f64, score_miss[i], nmiss);
    }
    println!("\nnum-tau-ext(2) FIXES {} miss rows, BREAKS {} hit rows", fix1.len(), break1);
    print!("fixed: "); for x in fix1.iter().take(24) { print!("({},{}) ", x.0, x.1); } println!();
}
