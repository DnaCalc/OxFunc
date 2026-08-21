//! W109 G6-01 (Fable inverse-lane #1): EXTENDED-DENOMINATOR LINKAGE.
//! Hypothesis: Excel's PMT em = (1+r)^-n - 1 uses the Kahan form (u-1)*t/ln(u), but
//! ln(u) is delivered EXTENDED (80-bit FYL2X result, kept in ST(0)) and consumed by an
//! x87 FDIV, while u is the spilled double (matches the worksheet EXP capture). The
//! worksheet LN oracle only ever sees RN53(lnu_ext), so the ~11 sub-double bits of the
//! denominator are INVISIBLE to it — precisely the freedom the double-only spill search
//! (<=165) could not reach. u,tau,lnu here are Excel's EXACT captures (expm1_intermediates.csv).
use oxfunc_core::excel_numeric::research as rx;
use rx::{
    CW_PC53_RN, CW_PC64_RN, Ext80, ext_div, ext_from_f64, ext_fyl2x, ext_fyl2xp1, ext_mul,
    ext_to_f64,
};

/// FLDLN2 — the 80-bit ln2 constant the x87 ln routine loads (not the double-rounded f64).
fn ext_ln2() -> Ext80 {
    let mut out = Ext80([0u8; 10]);
    // SAFETY: one push (fldln2), one pop (fstp tbyte); pointer store only.
    unsafe {
        core::arch::asm!("fldln2", "fstp tbyte ptr [{o}]", o = in(reg) out.0.as_mut_ptr());
    }
    out
}

fn b(s: &str) -> f64 {
    f64::from_bits(u64::from_str_radix(s, 16).unwrap())
}

fn main() {
    let csv =
        std::fs::read_to_string("../../work/w109/G6-solvers/expm1_intermediates.csv").unwrap();
    let ln2 = ext_ln2();

    let names = [
        "1 base RN53(num_dbl/lnu_captured)",
        "2 sanity RN53(num_dbl/myFYL2X_dbl)",
        "3*EXTDENOM fyl2x  RN53(num_dbl / lnu_ext)   [PC64 div]",
        "4*EXTDENOM fyl2xp1 RN53(num_dbl / lnu_ext)  [PC64 div]",
        "5 EXTBOTH  fyl2x  RN53(num_ext / lnu_ext)   [PC64 div]",
        "6 EXTNUM   RN53(num_ext / lnu_captured_ext) [PC64 div]",
        "7 EXTDENOM fyl2x  PC53-div  RN53(num_dbl/lnu_ext)",
        "8 EXTDENOM fyl2x  num_ext   RN53(num_ext/lnu_ext_2x) dup guard",
    ];
    let mut score = [0u32; 8];
    let mut score_miss = [0u32; 8]; // score restricted to baseline-miss rows
    let mut myln_matches_capture = 0u32;
    let mut p1_valid = 0u32;
    let (mut tot, mut nmiss) = (0u32, 0u32);
    // per-candidate: on baseline-miss rows, how it moves (hit / still-miss)
    let mut ex3_fix: Vec<(i32, u32)> = Vec::new();
    let mut ex3_break: Vec<(i32, u32)> = Vec::new();

    for line in csv.lines().skip(1) {
        let f: Vec<&str> = line.split(',').collect();
        if f.len() < 6 {
            continue;
        }
        let k: i32 = f[0].parse().unwrap();
        let n: u32 = f[1].parse().unwrap();
        let tau = b(f[2]);
        let u = b(f[3]);
        let lnu_cap = b(f[4]);
        let em = b(f[5]).to_bits();
        tot += 1;

        let a = u - 1.0; // exact (Sterbenz, u in [0.5,2))
        let num_dbl = a * tau; // double product
        let a_ext = ext_from_f64(a);
        let tau_ext = ext_from_f64(tau);
        let u_ext = ext_from_f64(u);
        let num_ext = ext_mul(&a_ext, &tau_ext, CW_PC64_RN);
        let num_dbl_ext = ext_from_f64(num_dbl);
        let lnu_cap_ext = ext_from_f64(lnu_cap);

        // denominators
        let lnu_ext_2x = ext_fyl2x(&ln2, &u_ext, CW_PC64_RN); // ln(u) 80-bit
        let myln_dbl = ext_to_f64(&lnu_ext_2x, CW_PC53_RN);
        if myln_dbl.to_bits() == lnu_cap.to_bits() {
            myln_matches_capture += 1;
        }
        let a_abs = a.abs();
        let use_p1 = a_abs < 0.2928; // FYL2XP1 domain |x| < 1 - sqrt(2)/2
        if use_p1 {
            p1_valid += 1;
        }
        let lnu_ext_p1 = if use_p1 {
            ext_fyl2xp1(&ln2, &a_ext, CW_PC64_RN)
        } else {
            lnu_ext_2x
        };

        // candidates
        let c1 = (num_dbl / lnu_cap).to_bits();
        let c2 = (num_dbl / myln_dbl).to_bits();
        let c3 = ext_to_f64(&ext_div(&num_dbl_ext, &lnu_ext_2x, CW_PC64_RN), CW_PC53_RN).to_bits();
        let c4 = if use_p1 {
            ext_to_f64(&ext_div(&num_dbl_ext, &lnu_ext_p1, CW_PC64_RN), CW_PC53_RN).to_bits()
        } else {
            u64::MAX
        };
        let c5 = ext_to_f64(&ext_div(&num_ext, &lnu_ext_2x, CW_PC64_RN), CW_PC53_RN).to_bits();
        let c6 = ext_to_f64(&ext_div(&num_ext, &lnu_cap_ext, CW_PC64_RN), CW_PC53_RN).to_bits();
        // c7: FDIV at PC53 (single-round the true quotient num_dbl/lnu_ext, no double round)
        let c7 = ext_to_f64(&ext_div(&num_dbl_ext, &lnu_ext_2x, CW_PC53_RN), CW_PC53_RN).to_bits();
        let c8 = c5; // placeholder dup

        let cands = [c1, c2, c3, c4, c5, c6, c7, c8];
        let base_miss = c1 != em;
        if base_miss {
            nmiss += 1;
        }
        for i in 0..8 {
            if cands[i] == em {
                score[i] += 1;
                if base_miss {
                    score_miss[i] += 1;
                }
            }
        }
        if base_miss {
            if c3 == em {
                ex3_fix.push((k, n));
            }
        } else if c3 != em {
            ex3_break.push((k, n));
        }
    }

    println!(
        "N={} rows, baseline misses={}, FYL2XP1-valid rows={}",
        tot, nmiss, p1_valid
    );
    println!(
        "my-FYL2X(u)->dbl == captured lnu on {}/{} rows (ln2/fyl2x sanity)\n",
        myln_matches_capture, tot
    );
    println!(
        "{:<52} {:>10}  {:>14}",
        "candidate", "all", "on-miss-subset"
    );
    for i in 0..8 {
        println!(
            "{:<52} {:>4}/{} ({:4.1}%) {:>6}/{}",
            names[i],
            score[i],
            tot,
            100.0 * score[i] as f64 / tot as f64,
            score_miss[i],
            nmiss
        );
    }
    println!(
        "\nEXTDENOM(3) FIXES {} baseline-miss rows; BREAKS {} baseline-hit rows",
        ex3_fix.len(),
        ex3_break.len()
    );
    print!("  fixed(k,n): ");
    for x in ex3_fix.iter().take(16) {
        print!("({},{}) ", x.0, x.1);
    }
    println!();
    print!("  broke(k,n): ");
    for x in ex3_break.iter().take(16) {
        print!("({},{}) ", x.0, x.1);
    }
    println!();
}
