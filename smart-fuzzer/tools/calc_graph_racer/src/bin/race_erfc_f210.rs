//! 0x210 [2,4) Q-hard vs F_or: last-mul vs F-body. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC64_RN};
use std::env;

const CW: u16 = CW_PC64_RN;
const C0: [f64; 9] = [
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
const D0: [f64; 8] = [
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
fn maybe(x: Ext80, mask: u32, bit: u32) -> Ext80 {
    if bit < 32 && mask & (1u32 << bit) != 0 {
        ef(ext_to_f64(&x, CW))
    } else {
        x
    }
}
fn cody(y: f64, mask: u32) -> f64 {
    let ye = ef(y.abs());
    let mut xnum = ext_mul(&ef(C0[8]), &ye, CW);
    let mut xden = ye;
    xnum = maybe(xnum, mask, 0);
    xden = maybe(xden, mask, 1);
    for i in 0..7 {
        xnum = ext_mul(&ext_add(&xnum, &ef(C0[i]), CW), &ye, CW);
        xden = ext_mul(&ext_add(&xden, &ef(D0[i]), CW), &ye, CW);
        xnum = maybe(xnum, mask, 2 + 2 * i as u32);
        xden = maybe(xden, mask, 3 + 2 * i as u32);
    }
    let mut q = ext_div(
        &ext_add(&xnum, &ef(C0[7]), CW),
        &ext_add(&xden, &ef(D0[7]), CW),
        CW,
    );
    q = maybe(q, mask, 16);
    ext_to_f64(&q, CW)
}
fn qwf(z: f64, ff: f64) -> f64 {
    let v = f::w_rn53(z) * ff;
    if v.abs() < f64::MIN_POSITIVE {
        0.0
    } else {
        v
    }
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    let mut qe_fe = 0usize;
    let mut qe_fm = 0usize;
    let mut qm_fe = 0usize;
    let mut qm_fm = 0usize;
    let mut n = 0usize;
    let mut fex = 0usize;
    let mut qex = 0usize;
    println!("[2,4) DIRECT 0x210 Q vs F_or (last-mul leftover = Q miss F exact):");
    for r in &rows {
        if !r.direct || r.z < 2.0 || r.z >= 4.0 {
            continue;
        }
        n += 1;
        let ff = cody(r.z, 0x210);
        let dq = ulp_distance(qwf(r.z, ff), f64::from_bits(r.qbits)).unwrap_or(99);
        let df = match f::f_or(r.z, r.qbits) {
            Some(fo) => ulp_distance(ff, fo).unwrap_or(99),
            None => 99,
        };
        let qe = dq == 0;
        let fe = df == 0;
        if qe {
            qex += 1;
        }
        if fe {
            fex += 1;
        }
        match (qe, fe) {
            (true, true) => qe_fe += 1,
            (true, false) => {
                qe_fm += 1;
                if qe_fm <= 6 {
                    println!("  Qexact Fmiss z={:.16} dF={df}", r.z);
                }
            }
            (false, true) => {
                qm_fe += 1;
                if qm_fe <= 8 {
                    println!(
                        "  Qmiss Fexact z={:.16} dQ={dq} {}",
                        r.z,
                        if qwf(r.z, ff) < f64::from_bits(r.qbits) {
                            "Q low"
                        } else {
                            "Q high"
                        }
                    );
                }
            }
            (false, false) => {
                qm_fm += 1;
                if dq >= 4 {
                    println!("  both-miss ulp4 z={:.16} dQ={dq} dF={df}", r.z);
                }
            }
        }
    }
    println!(
        "n={n} Qexact={qex} Fexact={fex} both_ex={qe_fe} Qex_Fmiss={qe_fm} Qmiss_Fex={qm_fe} both_miss={qm_fm}"
    );

    println!("full mid DIRECT 0x210 Q vs F_or:");
    let mut qe = 0usize;
    let mut fe = 0usize;
    let mut nmid = 0usize;
    let mut qmfe = 0usize;
    let mut bothm = 0usize;
    for r in &rows {
        if !r.direct || r.z < 0.5 || r.z >= 4.0 {
            continue;
        }
        nmid += 1;
        let ff = cody(r.z, 0x210);
        let dq = ulp_distance(qwf(r.z, ff), f64::from_bits(r.qbits)).unwrap_or(99);
        let df = match f::f_or(r.z, r.qbits) {
            Some(fo) => ulp_distance(ff, fo).unwrap_or(99),
            None => 99,
        };
        if dq == 0 {
            qe += 1;
        }
        if df == 0 {
            fe += 1;
        }
        if dq != 0 && df == 0 {
            qmfe += 1;
        }
        if dq != 0 && df != 0 && dq >= 2 && df >= 2 {
            bothm += 1;
        }
    }
    println!("  n={nmid} Q dmid={qe} F_or dmid={fe} Qmiss_Fex={qmfe} both_hard={bothm}");
}
