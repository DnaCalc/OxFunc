//! W109 G6-01: Kahan arithmetic precision-schedule matrix, using Excel's
//! GROUND-TRUTH u, tau, lnu (from expm1_intermediates.csv, verified == live
//! Excel EXP/LN captures). Only the combine arithmetic varies: numerator product
//! PC53/PC64, divide PC53/PC64, final store RN53/RZ53. em is negative.
use oxfunc_core::excel_numeric::research as rx;
use rx::{CW_PC53_RN, CW_PC64_RN, ext_div, ext_from_f64, ext_mul, ext_one, ext_sub, ext_to_f64};

const RN53: u16 = CW_PC53_RN;
const RZ53: u16 = CW_PC53_RN | 0x0C00;

fn main() {
    let csv =
        std::fs::read_to_string("../../work/w109/G6-solvers/expm1_intermediates.csv").unwrap();
    // schedules: (label, num_pc, div_pc, store)
    let scheds: [(&str, u16, u16, u16); 10] = [
        ("num53 div53 RN [prod163]", CW_PC53_RN, CW_PC53_RN, RN53),
        ("num64 div53 RN [165?]", CW_PC64_RN, CW_PC53_RN, RN53),
        ("num53 div64 RN", CW_PC53_RN, CW_PC64_RN, RN53),
        ("num64 div64 RN", CW_PC64_RN, CW_PC64_RN, RN53),
        ("num64 div64 RZ", CW_PC64_RN, CW_PC64_RN, RZ53),
        ("num64 div53 RZ", CW_PC64_RN, CW_PC53_RN, RZ53),
        ("num53 div53 RZ", CW_PC53_RN, CW_PC53_RN, RZ53),
        (
            "num64 div64 store64->RN53",
            CW_PC64_RN,
            CW_PC64_RN,
            CW_PC64_RN,
        ),
        ("yexact num64 div64 RN", CW_PC64_RN, CW_PC64_RN, RN53), // y kept exact ext
        ("yexact num64 div53 RN", CW_PC64_RN, CW_PC53_RN, RN53),
    ];
    let mut score = [0u32; 10];
    let mut tot = 0u32;
    let mut misses: Vec<[i64; 10]> = Vec::new();
    let mut meta: Vec<(i32, u32)> = Vec::new();
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
        let ue = ext_from_f64(u);
        let taue = ext_from_f64(tau);
        let lnue = ext_from_f64(lnu);
        let mut row = [0i64; 10];
        for (i, (lbl, npc, dpc, st)) in scheds.iter().enumerate() {
            let y = if lbl.starts_with("yexact") {
                ext_sub(&ue, &ext_one(), CW_PC64_RN) // exact (fits) y=u-1
            } else {
                // y = fl(u-1.0) as a double, re-widened
                ext_from_f64(u - 1.0)
            };
            let num = ext_mul(&y, &taue, *npc);
            let q = ext_div(&num, &lnue, *dpc);
            let em = ext_to_f64(&q, *st).to_bits();
            if em == pin {
                score[i] += 1;
            }
            row[i] = (em as i64) - (pin as i64);
        }
        misses.push(row);
        meta.push((k, n));
        tot += 1;
    }
    println!(
        "=== Kahan precision-schedule matrix (Excel ground-truth u,tau,lnu) N={} ===",
        tot
    );
    for (i, (lbl, _, _, _)) in scheds.iter().enumerate() {
        println!(
            "  {:30} {:3}/{}  ({:.1}%)",
            lbl,
            score[i],
            tot,
            100.0 * score[i] as f64 / tot as f64
        );
    }
    // best schedule miss structure
    let best = (0..10).max_by_key(|&i| score[i]).unwrap();
    let plus = misses.iter().filter(|r| r[best] > 0).count();
    let minus = misses.iter().filter(|r| r[best] < 0).count();
    println!(
        "\nbest = [{}]  miss dirs +{} / -{}",
        scheds[best].0, plus, minus
    );
    print!("  sample misses (k,n,d): ");
    let mut c = 0;
    for (j, r) in misses.iter().enumerate() {
        if r[best] != 0 && c < 15 {
            print!("({},{},{:+}) ", meta[j].0, meta[j].1, r[best]);
            c += 1;
        }
    }
    println!();
}
