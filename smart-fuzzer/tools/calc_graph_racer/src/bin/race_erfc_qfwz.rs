//! Cluster F-only vs w-only Cody C/D Q-miss-1 by z. Not an identity.
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
fn band(z: f64) -> usize {
    ((z - 0.5) / 0.5).floor().clamp(0.0, 6.0) as usize
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    let mut f_only = Vec::new();
    let mut w_only = Vec::new();
    let mut both_far = Vec::new();
    let mut fb = [0usize; 7];
    let mut wb = [0usize; 7];
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
        if dq != 1 {
            continue;
        }
        let df = ulp_distance(ff, qo / w).unwrap_or(u64::MAX);
        let dw = ulp_distance(w, qo / ff).unwrap_or(u64::MAX);
        let b = band(r.z);
        if df <= 1 && dw > 1 {
            f_only.push(r.z);
            fb[b] += 1;
        } else if dw <= 1 && df > 1 {
            w_only.push(r.z);
            wb[b] += 1;
        } else if df > 1 && dw > 1 {
            both_far.push(r.z);
        }
    }
    println!("F-only n={} bands[0.5:0.5:4]={:?}", f_only.len(), fb);
    println!("w-only n={} bands={:?}", w_only.len(), wb);
    println!("both-far n={}", both_far.len());
    f_only.sort_by(|a, b| a.partial_cmp(b).unwrap());
    w_only.sort_by(|a, b| a.partial_cmp(b).unwrap());
    println!("F-only first10 {:?}", &f_only[..f_only.len().min(10)]);
    println!("F-only last10 {:?}", &f_only[f_only.len().saturating_sub(10)..]);
    println!("w-only first10 {:?}", &w_only[..w_only.len().min(10)]);
    println!("w-only last10 {:?}", &w_only[w_only.len().saturating_sub(10)..]);
}
