//! C/D Horner x87 precision × rounding on 0x210 / mask0. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC24_RN, CW_PC53_RN, CW_PC64_RN};
use std::env;

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
fn maybe(x: Ext80, mask: u32, bit: u32, cw: u16) -> Ext80 {
    if bit < 32 && mask & (1u32 << bit) != 0 {
        ef(ext_to_f64(&x, cw))
    } else {
        x
    }
}
fn cody(y: f64, mask: u32, cw: u16) -> f64 {
    let ye = ef(y.abs());
    let mut xnum = ext_mul(&ef(C0[8]), &ye, cw);
    let mut xden = ye;
    xnum = maybe(xnum, mask, 0, cw);
    xden = maybe(xden, mask, 1, cw);
    for i in 0..7 {
        xnum = ext_mul(&ext_add(&xnum, &ef(C0[i]), cw), &ye, cw);
        xden = ext_mul(&ext_add(&xden, &ef(D0[i]), cw), &ye, cw);
        xnum = maybe(xnum, mask, 2 + 2 * i as u32, cw);
        xden = maybe(xden, mask, 3 + 2 * i as u32, cw);
    }
    let mut q = ext_div(
        &ext_add(&xnum, &ef(C0[7]), cw),
        &ext_add(&xden, &ef(D0[7]), cw),
        cw,
    );
    q = maybe(q, mask, 16, cw);
    ext_to_f64(&q, cw)
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
    let pcs = [("PC64", CW_PC64_RN), ("PC53", CW_PC53_RN), ("PC24", CW_PC24_RN)];
    let rcs = [("RN", 0u16), ("RD", 0x0400), ("RU", 0x0800), ("RZ", 0x0C00)];
    println!("C/D Horner PC×RC DIRECT dmid (bar PC64 RN 0x210 = 105):");
    let mut best = 0usize;
    let mut best_lab = String::new();
    for (pn, pc) in pcs {
        for (rn, rc) in rcs {
            let cw = pc | rc;
            for (mn, mask) in [("m0", 0u32), ("0x210", 0x210u32)] {
                let mut dmid = 0usize;
                let mut nd = 0usize;
                let mut mx = 0u64;
                let mut qe = 0usize;
                let mut nq = 0usize;
                for r in &rows {
                    if r.z < 0.5 || r.z >= 4.0 {
                        continue;
                    }
                    nq += 1;
                    let d = ulp_distance(qwf(r.z, cody(r.z, mask, cw)), f64::from_bits(r.qbits))
                        .unwrap_or(99);
                    if d == 0 {
                        qe += 1;
                    }
                    if r.direct {
                        nd += 1;
                        if d == 0 {
                            dmid += 1;
                        } else {
                            mx = mx.max(d);
                        }
                    }
                }
                let lab = format!("{pn} {rn} {mn}");
                if dmid >= 90 || (pn == "PC64" && rn == "RN") || dmid > best {
                    println!("  {lab:16} Q {qe}/{nq} dmid={dmid}/{nd} maxd={mx} cw={cw:#06x}");
                }
                if dmid > best {
                    best = dmid;
                    best_lab = lab;
                }
            }
        }
    }
    println!("BEST dmid {best} {best_lab}");
}
