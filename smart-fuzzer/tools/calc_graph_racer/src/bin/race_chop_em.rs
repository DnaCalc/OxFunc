//! W109 G6-01 (Fable inverse-lane #3 + diagnostic): the D-histogram says Excel's em is the
//! TOWARD-ZERO NEIGHBOR of the correctly-rounded Kahan quotient. Mechanistic test: is the FINAL
//! DIVIDE chopped toward zero (RZ / truncate) rather than round-nearest? On rows where the true
//! quotient q sits just above the RN midpoint (RN rounds UP to baseline), truncation toward zero
//! lands on em_excel exactly. Also dumps the required per-row direction/magnitude (inverse solve).
use oxfunc_core::excel_numeric::research as rx;
use rx::{CW_PC53_RN, CW_PC64_RN, Ext80, ext_div, ext_from_f64, ext_mul, ext_to_f64};

const RN53: u16 = CW_PC53_RN;
const RN64: u16 = CW_PC64_RN;
const RZ53: u16 = CW_PC53_RN | 0x0C00; // PC53, round-toward-zero
const RZ64: u16 = CW_PC64_RN | 0x0C00; // PC64, round-toward-zero

fn b(s: &str) -> f64 { f64::from_bits(u64::from_str_radix(s, 16).unwrap()) }
fn ulp_dist(x: f64, y: f64) -> i64 { x.to_bits() as i64 - y.to_bits() as i64 } // same-sign magnitudes

fn main() {
    let csv = std::fs::read_to_string("../../work/w109/G6-solvers/expm1_intermediates.csv").unwrap();
    let names = [
        "0 baseline RN53(num_dbl/lnu)",
        "1 RZ53 final divide (chop, PC53)         ",
        "2 RZ53-store of RN64 divide (PC64->chop) ",
        "3 RZ53 divide, numerator=RN64 product    ",
        "4 num chopped RZ53, divide RN53          ",
        "5 num chopped RZ64->RN53, divide RN53    ",
    ];
    let mut score = [0u32; 6];
    let mut score_miss = [0u32; 6];
    let (mut tot, mut nmiss) = (0u32, 0u32);
    let (mut toward, mut away, mut faroff) = (0u32, 0u32, 0u32);
    let mut c1_fix = 0u32; let mut c1_break = 0u32;
    let mut dump: Vec<String> = Vec::new();

    for line in csv.lines().skip(1) {
        let f: Vec<&str> = line.split(',').collect();
        if f.len() < 6 { continue; }
        let k: i32 = f[0].parse().unwrap();
        let n: u32 = f[1].parse().unwrap();
        let tau = b(f[2]);
        let u = b(f[3]);
        let lnu = b(f[4]);
        let em = b(f[5]);
        let emb = em.to_bits();
        tot += 1;

        let a = u - 1.0;
        let num_dbl = a * tau;
        let a_ext = ext_from_f64(a);
        let tau_ext = ext_from_f64(tau);
        let num_ext = ext_mul(&a_ext, &tau_ext, RN64); // 80-bit product
        let num_rn64 = ext_to_f64(&num_ext, RN53);     // product as double (RN53) == num_dbl usually
        let num_dbl_e = ext_from_f64(num_dbl);
        let lnu_e = ext_from_f64(lnu);

        let base = num_dbl / lnu;
        // candidate roundings of the FINAL divide (denominator = captured double lnu):
        let c1 = ext_to_f64(&ext_div(&num_dbl_e, &lnu_e, RZ53), RN53);           // chop divide PC53
        let c2 = ext_to_f64(&ext_div(&num_dbl_e, &lnu_e, RN64), RZ53);           // RN64 divide, chop store
        let c3 = ext_to_f64(&ext_div(&ext_from_f64(num_rn64), &lnu_e, RZ53), RN53);
        // numerator chopped toward zero, divide round-nearest:
        let num_rz = ext_to_f64(&num_ext, RZ53);
        let c4 = num_rz / lnu;
        let num_rz64 = ext_to_f64(&ext_from_f64(ext_to_f64(&num_ext, RZ64)), RN53);
        let c5 = num_rz64 / lnu;

        let cands = [base, c1, c2, c3, c4, c5];
        let base_miss = base.to_bits() != emb;
        if base_miss { nmiss += 1; }
        for i in 0..6 { if cands[i].to_bits() == emb { score[i] += 1; if base_miss { score_miss[i] += 1; } } }
        if base_miss { if c1.to_bits() == emb { c1_fix += 1; } }
        else if c1.to_bits() != emb { c1_break += 1; }

        if base_miss {
            // direction: is em toward zero (|em|<|base|) or away?
            let d = ulp_dist(em, base); // em bits - base bits; for negatives, toward zero = bits DECREASE in magnitude
            // For em<0, base<0: toward zero means em > base (less negative) => em.to_bits() < base.to_bits() (sign bit set, smaller magnitude has smaller bits)
            let toward_zero = em.abs() < base.abs();
            if toward_zero { toward += 1; } else { away += 1; }
            if d.abs() > 1 { faroff += 1; }
            if dump.len() < 24 {
                // required denominator D* = num_dbl/em, offset from lnu in double-ulp(lnu)
                let dstar = num_dbl / em;
                let doff = ulp_dist(dstar, lnu);
                // where does true quotient q sit vs base and em (in ulp of the quotient)
                dump.push(format!("k={:3} n={:3} em-base={:+} toward0={} reqD_off_ulp(lnu)={:+} c1hit={}",
                    k, n, d, toward_zero, doff, c1.to_bits()==emb));
            }
        }
    }

    println!("N={} baseline misses={} (toward-zero={}, away={}, |off|>1ulp={})", tot, nmiss, toward, away, faroff);
    println!("{:<44} {:>12}  {:>12}", "candidate", "all", "on-miss");
    for i in 0..6 {
        println!("{:<44} {:>4}/{} ({:4.1}%) {:>4}/{}", names[i], score[i], tot,
                 100.0*score[i] as f64/tot as f64, score_miss[i], nmiss);
    }
    println!("\nchop-final-divide(1) FIXES {} miss rows, BREAKS {} hit rows", c1_fix, c1_break);
    println!("\n-- sample miss rows (inverse solve) --");
    for d in &dump { println!("  {}", d); }
}
