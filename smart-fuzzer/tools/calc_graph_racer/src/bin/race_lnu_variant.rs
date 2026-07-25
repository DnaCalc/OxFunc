//! W109 G6-01: the expm1 denominator ln(u). Inversion showed Excel's internal
//! ln(u) is ~1 ULP off my worksheet fyl2x(ln2,u), biased toward -inf. Hypothesis:
//! the code holds y=u-1 (small) and computes ln(u)=log1p(y) via FYL2XP1 (accurate
//! for u~1), not FYL2X(u). Race denominator variants on the pure em oracle.
use oxfunc_core::excel_numeric::research as rx;
use rx::{CW_PC53_RN, CW_PC64_RN, ext_from_f64, ext_fyl2x, ext_fyl2xp1, ext_ln2, ext_to_f64};

const RZ64: u16 = CW_PC64_RN | 0x0C00;
const RN64: u16 = CW_PC64_RN;
const RN53: u16 = CW_PC53_RN;
const RZ53: u16 = CW_PC53_RN | 0x0C00;

// ln(u) via fyl2x(ln2, u) at a given control word (store to f64)
fn lnu_fyl2x(u: f64, cw_compute: u16, cw_store: u16) -> f64 {
    ext_to_f64(&ext_fyl2x(&ext_ln2(), &ext_from_f64(u), cw_compute), cw_store)
}
// ln(u) = log1p(y) via fyl2xp1(ln2, y), y=u-1 as double
fn lnu_fyl2xp1(y: f64, cw_compute: u16, cw_store: u16) -> f64 {
    ext_to_f64(&ext_fyl2xp1(&ext_ln2(), &ext_from_f64(y), cw_compute), cw_store)
}

fn main() {
    let csv = std::fs::read_to_string("../../work/w109/G6-solvers/expm1_intermediates.csv").unwrap();
    let labels = [
        "L0 fyl2x PC64 RN [prod]",
        "L1 fyl2x PC53 RN",
        "L2 fyl2xp1(y) PC64 RN",
        "L3 fyl2xp1(y) PC53 RN",
        "L4 fyl2x PC64 RZ",
        "L5 fyl2xp1(y) PC64 RZ",
        "L6 fyl2xp1(y) PC53 RZ",
        "L7 fyl2x PC53 RZ",
    ];
    let mut score = [0u32; 8];
    let mut dirbias = [[0i32; 3]; 8]; // [toward0, exact, towardinf] rough
    let mut tot = 0u32;
    for line in csv.lines().skip(1) {
        let f: Vec<&str> = line.split(',').collect();
        if f.len() < 6 { continue; }
        let tau = f64::from_bits(u64::from_str_radix(f[2], 16).unwrap());
        let pin = u64::from_str_radix(f[5], 16).unwrap();
        let u = rx::excel_exp(tau);
        let y = u - 1.0;
        let num = y * tau; // one double product
        let dens = [
            lnu_fyl2x(u, RN64, RN53),
            lnu_fyl2x(u, RN53, RN53),
            lnu_fyl2xp1(y, RN64, RN53),
            lnu_fyl2xp1(y, RN53, RN53),
            lnu_fyl2x(u, RN64, RZ53),
            lnu_fyl2xp1(y, RN64, RZ53),
            lnu_fyl2xp1(y, RN53, RZ53),
            lnu_fyl2x(u, RN53, RZ53),
        ];
        for i in 0..8 {
            let em = (num / dens[i]).to_bits();
            if em == pin { score[i] += 1; } else {
                let d = (em as i64) - (pin as i64);
                if d > 0 { dirbias[i][0] += 1; } else { dirbias[i][2] += 1; }
            }
        }
        tot += 1;
    }
    println!("=== ln(u) denominator variants, pure em oracle N={} ===", tot);
    for i in 0..8 {
        println!("  {:26} {:3}/{}  ({:.1}%)  miss_dir[+/-]={:?}", labels[i], score[i], tot,
                 100.0 * score[i] as f64 / tot as f64, (dirbias[i][0], dirbias[i][2]));
    }
}
