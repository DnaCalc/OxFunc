//! Discriminate w vs F vs product rounding on Cody C/D Q-miss-1. Not an ID.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC64_RN};
use std::env;

const CW: u16 = CW_PC64_RN;
const C: [f64; 9] = [
    0.564188496988670089,
    8.88314979438837594,
    66.1191906371416295,
    298.635138197400131,
    881.95222124176909,
    1712.04761263407058,
    2051.07837782607147,
    1230.33935479799725,
    2.15311535474403846e-8,
];
const D: [f64; 8] = [
    15.7449261107098347,
    117.693950891312499,
    537.181101862009858,
    1621.38957456669019,
    3290.79923573345963,
    4362.61909014324716,
    3439.36767414372164,
    1230.33935480374942,
];

fn ef(x: f64) -> Ext80 {
    ext_from_f64(x)
}
fn cody_cd(y: f64) -> f64 {
    let ye = ef(y.abs());
    let mut xnum = ext_mul(&ef(C[8]), &ye, CW);
    let mut xden = ye;
    for i in 0..7 {
        xnum = ext_mul(&ext_add(&xnum, &ef(C[i]), CW), &ye, CW);
        xden = ext_mul(&ext_add(&xden, &ef(D[i]), CW), &ye, CW);
    }
    ext_to_f64(
        &ext_div(&ext_add(&xnum, &ef(C[7]), CW), &ext_add(&xden, &ef(D[7]), CW), CW),
        CW,
    )
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    let mut n = 0usize;
    let mut qex = 0usize;
    let mut miss1 = 0usize;
    let mut miss1_f = 0usize; // |F - Qo/w|==1 and |w - Qo/F|>1
    let mut miss1_w = 0usize;
    let mut miss1_both = 0usize;
    let mut miss1_prod = 0usize; // F matches Qo/w and w matches Qo/F (product rounding)
    let mut miss1_x87_hits = 0usize;
    let mut miss1_next_hits = 0usize;
    let mut mid_tie = 0usize;
    for r in &rows {
        if r.z < 0.5 || r.z >= 4.0 {
            continue;
        }
        let ff = cody_cd(r.z);
        let w = f::w_rn53(r.z);
        if w == 0.0 || ff == 0.0 {
            continue;
        }
        let qg = w * ff;
        let qo = f64::from_bits(r.qbits);
        let dq = ulp_distance(qg, qo).unwrap_or(u64::MAX);
        n += 1;
        if dq == 0 {
            qex += 1;
            continue;
        }
        if dq != 1 {
            continue;
        }
        miss1 += 1;
        let f_or = qo / w;
        let w_or = qo / ff;
        let df = ulp_distance(ff, f_or).unwrap_or(u64::MAX);
        let dw = ulp_distance(w, w_or).unwrap_or(u64::MAX);
        if df <= 1 && dw <= 1 {
            miss1_prod += 1;
        } else if df <= 1 {
            miss1_f += 1;
        } else if dw <= 1 {
            miss1_w += 1;
        } else {
            miss1_both += 1;
        }
        let q80 = ext_to_f64(&ext_mul(&ef(w), &ef(ff), CW), CW);
        if q80 == qo {
            miss1_x87_hits += 1;
        }
        if qg.next_up() == qo || qg.next_down() == qo {
            miss1_next_hits += 1;
        }
        // midpoint: 80-bit product equally far
        let p80 = ext_mul(&ef(w), &ef(ff), CW);
        let lo = qg.min(qg.next_down());
        let hi = qg.max(qg.next_up());
        let _ = (p80, lo, hi);
        let _ = mid_tie;
    }
    println!("mid n={n} Qexact={qex} miss1={miss1}");
    println!("miss1 F-only={miss1_f} w-only={miss1_w} both-far={miss1_both} prod-round={miss1_prod}");
    println!("miss1 x87-mul hits Qo={miss1_x87_hits} nextafter(Qg) hits={miss1_next_hits}");
}
