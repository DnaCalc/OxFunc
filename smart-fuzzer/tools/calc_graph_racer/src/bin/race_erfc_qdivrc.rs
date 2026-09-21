//! Last-div rounding of SPECFUN C/D, then Q=w*F64. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC64_RN};
use std::env;

const CW: u16 = CW_PC64_RN;
const CW_RD: u16 = 0x173F;
const CW_RU: u16 = 0x1B3F;
const CW_RZ: u16 = 0x1F3F;
const ULP_CAP: u64 = 1 << 20;
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
fn cody_cd_div(y: f64, div_cw: u16) -> f64 {
    let ye = ef(y.abs());
    let mut xnum = ext_mul(&ef(C[8]), &ye, CW);
    let mut xden = ye;
    for i in 0..7 {
        xnum = ext_mul(&ext_add(&xnum, &ef(C[i]), CW), &ye, CW);
        xden = ext_mul(&ext_add(&xden, &ef(D[i]), CW), &ye, CW);
    }
    let n = ext_add(&xnum, &ef(C[7]), CW);
    let d = ext_add(&xden, &ef(D[7]), CW);
    ext_to_f64(&ext_div(&n, &d, div_cw), div_cw)
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    for (name, cw) in [("RN", CW), ("RD", CW_RD), ("RU", CW_RU), ("RZ", CW_RZ)] {
        let mut ex = 0usize;
        let mut n = 0usize;
        let mut max = 0u64;
        let mut dmid = 0usize;
        for r in &rows {
            if r.z < 0.5 || r.z >= 4.0 {
                continue;
            }
            let qg = f::w_rn53(r.z) * cody_cd_div(r.z, cw);
            if !qg.is_finite() {
                continue;
            }
            let d = ulp_distance(qg, f64::from_bits(r.qbits)).unwrap_or(u64::MAX);
            if d > ULP_CAP {
                continue;
            }
            n += 1;
            if d == 0 {
                ex += 1;
                if r.direct {
                    dmid += 1;
                }
            } else {
                max = max.max(d);
            }
        }
        println!("{name} {ex}/{n} max={max} dmid={dmid}");
    }
}
