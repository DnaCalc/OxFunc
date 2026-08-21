//! W109 G6-01 (Fable inverse-lane → identification candidate): the inverse solve shows 57/71
//! misses are toward-zero with the denominator wanting EXACTLY +1 double-ULP (more negative).
//! Natural human-coded cause: the careful expm1 uses log1p(u-1) via FYL2XP1 for the denominator
//! (accurate when u≈1), NOT log(u) via FYL2X. That gives a DIFFERENT last-bit double than the
//! worksheet LN (=FYL2X, proven CR). Test: denominator = RN53(FYL2XP1(ln2, u-1)) [spilled to
//! double], with the natural branch to FYL2X when |u-1| >= 1 - sqrt(2)/2 (FYL2XP1 domain limit).
use oxfunc_core::excel_numeric::research as rx;
use rx::{
    CW_PC53_RN, CW_PC64_RN, ext_from_f64, ext_fyl2x, ext_fyl2xp1, ext_ln2, ext_mul, ext_to_f64,
};

const RN53: u16 = CW_PC53_RN;
const RN64: u16 = CW_PC64_RN;
const P1_DOMAIN: f64 = 0.2928932188134524; // 1 - sqrt(2)/2

fn b(s: &str) -> f64 {
    f64::from_bits(u64::from_str_radix(s, 16).unwrap())
}

fn main() {
    let csv =
        std::fs::read_to_string("../../work/w109/G6-solvers/expm1_intermediates.csv").unwrap();
    let ln2 = ext_ln2();

    // candidates for em, denominator via log1p(u-1):
    let names = [
        "0 baseline num_dbl/lnu(FYL2X)              ",
        "1 den=RN53(FYL2XP1(u-1)) dbl, num_dbl, /RN53",
        "2 den=RN53(FYL2XP1(u-1)) dbl, num=RN64prod  ",
        "3 den=FYL2XP1 EXT, num_dbl, x87 /PC64->RN53 ",
        "4 den=RN53(FYL2XP1) , divide x87 PC64->RN53 ",
    ];
    let mut score = [0u32; 5];
    let mut score_miss = [0u32; 5];
    let (mut tot, mut nmiss) = (0u32, 0u32);
    let mut den_differs = 0u32; // rows where FYL2XP1-double != FYL2X-double
    let mut used_p1 = 0u32;
    let mut c1_fix = 0u32;
    let mut c1_break = 0u32;
    let mut still_miss: Vec<(i32, u32, i64)> = Vec::new();

    for line in csv.lines().skip(1) {
        let f: Vec<&str> = line.split(',').collect();
        if f.len() < 6 {
            continue;
        }
        let k: i32 = f[0].parse().unwrap();
        let n: u32 = f[1].parse().unwrap();
        let tau = b(f[2]);
        let u = b(f[3]);
        let lnu = b(f[4]); // = worksheet LN = FYL2X-double
        let em = b(f[5]);
        let emb = em.to_bits();
        tot += 1;

        let a = u - 1.0; // exact
        let num_dbl = a * tau; // double product
        let a_ext = ext_from_f64(a);
        let tau_ext = ext_from_f64(tau);
        let num_ext = ext_mul(&a_ext, &tau_ext, RN64);
        let num_rn64_dbl = ext_to_f64(&num_ext, RN53);
        let u_ext = ext_from_f64(u);

        let use_p1 = a.abs() < P1_DOMAIN;
        if use_p1 {
            used_p1 += 1;
        }
        // denominator: log1p(u-1) via FYL2XP1 when in domain, else log(u) via FYL2X
        let den_ext = if use_p1 {
            ext_fyl2xp1(&ln2, &a_ext, RN64)
        } else {
            ext_fyl2x(&ln2, &u_ext, RN64)
        };
        let den_dbl = ext_to_f64(&den_ext, RN53);
        if den_dbl.to_bits() != lnu.to_bits() {
            den_differs += 1;
        }

        let base = num_dbl / lnu;
        let c1 = num_dbl / den_dbl;
        let c2 = num_rn64_dbl / den_dbl;
        // x87 divide of num_dbl by EXTENDED den, PC64 then RN53 store
        let num_dbl_e = ext_from_f64(num_dbl);
        let c3 = ext_to_f64(&rx::ext_div(&num_dbl_e, &den_ext, RN64), RN53);
        // x87 divide of num_dbl by den_dbl (double), PC64 then RN53 store (double-round)
        let c4 = ext_to_f64(&rx::ext_div(&num_dbl_e, &ext_from_f64(den_dbl), RN64), RN53);

        let cands = [base, c1, c2, c3, c4];
        let base_miss = base.to_bits() != emb;
        if base_miss {
            nmiss += 1;
        }
        for i in 0..5 {
            if cands[i].to_bits() == emb {
                score[i] += 1;
                if base_miss {
                    score_miss[i] += 1;
                }
            }
        }
        if base_miss {
            if c1.to_bits() == emb {
                c1_fix += 1;
            } else {
                still_miss.push((k, n, c1.to_bits() as i64 - emb as i64));
            }
        } else if c1.to_bits() != emb {
            c1_break += 1;
        }
    }

    println!(
        "N={} misses={} | used FYL2XP1 on {} rows | FYL2XP1-dbl != FYL2X-dbl on {} rows",
        tot, nmiss, used_p1, den_differs
    );
    println!("{:<46} {:>12}  {:>10}", "candidate", "all", "on-miss");
    for i in 0..5 {
        println!(
            "{:<46} {:>4}/{} ({:4.1}%) {:>4}/{}",
            names[i],
            score[i],
            tot,
            100.0 * score[i] as f64 / tot as f64,
            score_miss[i],
            nmiss
        );
    }
    println!(
        "\nlog1p-denominator(1) FIXES {} miss rows, BREAKS {} hit rows",
        c1_fix, c1_break
    );
    println!("still-miss after c1 ({}): first 20:", still_miss.len());
    for x in still_miss.iter().take(20) {
        println!("  k={:3} n={:3} c1-em={:+}", x.0, x.1, x.2);
    }
}
