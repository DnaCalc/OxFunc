//! W109 G6-01 (Fable follow-up A1): a DIFFERENT ln IMPLEMENTATION as the Goldberg denominator.
//! Hypothesis: Excel's financial routine forms ln(u) = log10(u)·LN10 (x87 FLDLG2/FYL2X for
//! log10, then a double-multiply by the C literal LN10=2.302585092994046, which is +0.4..0.9
//! ULP53 HIGH). log10(u)<0 × slightly-too-large positive const → lnu more negative → |denom|
//! larger → em TOWARD ZERO — matching the 57-row toward-zero majority. Also log2(u)·LN2 (LN2 is
//! LOW → away bias) and log2(u)/LOG2E. All vs the confound-free captured operands.
use oxfunc_core::excel_numeric::research as rx;
use rx::{CW_PC53_RN, CW_PC64_RN, Ext80, ext_fyl2x, ext_from_f64, ext_mul, ext_to_f64};
const CW: u16 = CW_PC64_RN;
const RN53: u16 = CW_PC53_RN;

fn ext_const(inst: &str) -> Ext80 {
    let mut out = Ext80([0u8; 10]);
    unsafe {
        match inst {
            "lg2" => core::arch::asm!("fldlg2", "fstp tbyte ptr [{o}]", o = in(reg) out.0.as_mut_ptr()),
            "ln2" => core::arch::asm!("fldln2", "fstp tbyte ptr [{o}]", o = in(reg) out.0.as_mut_ptr()),
            "l2e" => core::arch::asm!("fldl2e", "fstp tbyte ptr [{o}]", o = in(reg) out.0.as_mut_ptr()),
            _ => unreachable!(),
        }
    }
    out
}
fn b(s: &str) -> f64 { f64::from_bits(u64::from_str_radix(s, 16).unwrap()) }

fn main() {
    let csv = std::fs::read_to_string("../../work/w109/G6-solvers/expm1_intermediates.csv").unwrap();
    let lg2 = ext_const("lg2"); // log10(2), 80-bit
    let ln2c = ext_const("ln2");
    let l2e = ext_const("l2e");
    let ln10_dbl = std::f64::consts::LN_10;   // 2.302585092994046 (HIGH ~+0.9 ULP53)
    let ln2_dbl = std::f64::consts::LN_2;      // 0.6931471805599453 (LOW)
    let log2e_dbl = std::f64::consts::LOG2_E;  // 1.4426950408889634 (LOW, weak)

    let names = [
        "0 baseline num/lnu(FYL2X direct)          ",
        "1 A1 log10(u)_dbl * LN10_dbl              ",
        "2 A1 log10(u)_ext * LN10_dbl, ext, spill  ",
        "3 A1 log10(u)_dbl * LN10_ext(dbl-widened) ",
        "4 B  log2(u)_dbl * LN2_dbl                ",
        "5 B  log2(u)_dbl / LOG2E_dbl              ",
        "6 A1 log10(u)_ext * LN10, den EXT, x87 div",
    ];
    let mut score = [0u32; 7];
    let mut score_miss = [0u32; 7];
    let (mut tot, mut nmiss) = (0u32, 0u32);
    let mut a1_denom_dir: std::collections::HashMap<i64, u32> = std::collections::HashMap::new();
    let mut fix1: Vec<(i32,u32)> = Vec::new(); let mut break1 = 0u32;

    for line in csv.lines().skip(1) {
        let f: Vec<&str> = line.split(',').collect();
        if f.len() < 6 { continue; }
        let k: i32 = f[0].parse().unwrap(); let n: u32 = f[1].parse().unwrap();
        let tau = b(f[2]); let u = b(f[3]); let lnu = b(f[4]); let em = b(f[5]); let emb = em.to_bits();
        tot += 1;
        let a = u - 1.0;
        let num_dbl = a * tau;
        let num_e = ext_from_f64(num_dbl);
        let u_e = ext_from_f64(u);

        // log10(u) via FLDLG2/FYL2X
        let log10_ext = ext_fyl2x(&lg2, &u_e, CW);
        let log10_dbl = ext_to_f64(&log10_ext, RN53);
        // log2(u) via FYL2X with y=1
        let log2_ext = ext_fyl2x(&ext_from_f64(1.0), &u_e, CW);
        let log2_dbl = ext_to_f64(&log2_ext, RN53);

        // A1 candidates (natural log = log10 * ln10)
        let ln10e = ext_from_f64(ln10_dbl);
        let lnu_1 = log10_dbl * ln10_dbl;                                    // both double
        let lnu_2 = ext_to_f64(&ext_mul(&log10_ext, &ln10e, CW), RN53);      // log10 ext * ln10, spill
        let lnu_3 = log10_dbl * ln10_dbl;                                    // (same as 1; dup slot)
        let lnu_4 = log2_dbl * ln2_dbl;
        let lnu_5 = log2_dbl / log2e_dbl;
        let den6_ext = ext_mul(&log10_ext, &ln10e, CW);                      // ext denom for x87 divide

        let c0 = (num_dbl / lnu).to_bits();
        let c1 = (num_dbl / lnu_1).to_bits();
        let c2 = (num_dbl / lnu_2).to_bits();
        let c3 = (num_dbl / lnu_3).to_bits();
        let c4 = (num_dbl / lnu_4).to_bits();
        let c5 = (num_dbl / lnu_5).to_bits();
        let c6 = ext_to_f64(&rx::ext_div(&num_e, &den6_ext, CW), RN53).to_bits();

        // fingerprint: A1(1) denominator vs captured lnu
        let dd = lnu_1.to_bits() as i64 - lnu.to_bits() as i64;
        *a1_denom_dir.entry(dd).or_insert(0) += 1;

        let cands = [c0, c1, c2, c3, c4, c5, c6];
        let base_miss = c0 != emb;
        if base_miss { nmiss += 1; }
        for i in 0..7 { if cands[i]==emb { score[i]+=1; if base_miss { score_miss[i]+=1; } } }
        if base_miss { if c1==emb { fix1.push((k,n)); } } else if c1!=emb { break1+=1; }
    }

    println!("N={} misses={}", tot, nmiss);
    let mut dirs: Vec<_> = a1_denom_dir.iter().collect(); dirs.sort();
    print!("A1 denom(log10*ln10) - captured lnu, ulp-diff dist: ");
    for (d,c) in dirs { print!("{}:{} ", d, c); } println!();
    println!("{:<44} {:>12} {:>10}", "candidate", "all", "on-miss");
    for i in 0..7 {
        println!("{:<44} {:>4}/{} ({:4.1}%) {:>4}/{}", names[i], score[i], tot,
                 100.0*score[i] as f64/tot as f64, score_miss[i], nmiss);
    }
    println!("\nA1(1) FIXES {} miss rows, BREAKS {} hit rows", fix1.len(), break1);
    print!("fixed: "); for x in fix1.iter().take(30) { print!("({},{}) ", x.0, x.1); } println!();
}
