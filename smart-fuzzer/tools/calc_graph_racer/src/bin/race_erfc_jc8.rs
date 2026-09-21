//! C[8]/D[1]/D[7] pokes on j3432+0x20. Small search. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC64_RN};
use std::env;

const CW: u16 = CW_PC64_RN;
const ULP_CAP: u64 = 1 << 20;
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
fn poke(x: f64, k: i32) -> f64 {
    let mut v = x;
    if k > 0 {
        for _ in 0..k {
            v = v.next_up();
        }
    } else {
        for _ in 0..(-k) {
            v = v.next_down();
        }
    }
    v
}
fn maybe(x: Ext80, mask: u32, bit: u32) -> Ext80 {
    if bit < 32 && mask & (1u32 << bit) != 0 {
        ef(ext_to_f64(&x, CW))
    } else {
        x
    }
}
fn cody(y: f64, c: &[f64; 9], d: &[f64; 8], mask: u32) -> f64 {
    let ye = ef(y.abs());
    let mut xnum = ext_mul(&ef(c[8]), &ye, CW);
    let mut xden = ye;
    xnum = maybe(xnum, mask, 0);
    xden = maybe(xden, mask, 1);
    for i in 0..7 {
        xnum = ext_mul(&ext_add(&xnum, &ef(c[i]), CW), &ye, CW);
        xden = ext_mul(&ext_add(&xden, &ef(d[i]), CW), &ye, CW);
        xnum = maybe(xnum, mask, 2 + 2 * i as u32);
        xden = maybe(xden, mask, 3 + 2 * i as u32);
    }
    let mut q = ext_div(
        &ext_add(&xnum, &ef(c[7]), CW),
        &ext_add(&xden, &ef(d[7]), CW),
        CW,
    );
    q = maybe(q, mask, 16);
    ext_to_f64(&q, CW)
}
fn qw(z: f64, ff: f64) -> f64 {
    let v = f::w_rn53(z) * ff;
    if v.abs() < f64::MIN_POSITIVE {
        0.0
    } else {
        v
    }
}
fn score(rows: &[f::QRow], c: &[f64; 9], d: &[f64; 8], mask: u32) -> (usize, usize, u64, usize) {
    let mut qex = 0usize;
    let mut qd = 0usize;
    let mut qmx = 0u64;
    let mut fex = 0usize;
    for r in rows {
        if r.z < 0.5 || r.z >= 4.0 {
            continue;
        }
        let ff = cody(r.z, c, d, mask);
        let dq = ulp_distance(qw(r.z, ff), f64::from_bits(r.qbits)).unwrap_or(u64::MAX);
        if dq <= ULP_CAP {
            if dq == 0 {
                qex += 1;
                if r.direct {
                    qd += 1;
                }
            } else {
                qmx = qmx.max(dq);
            }
        }
        if let Some(fo) = f::f_or(r.z, r.qbits) {
            if ulp_distance(ff, fo).unwrap_or(99) == 0 {
                fex += 1;
            }
        }
    }
    (qex, qd, qmx, fex)
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    let mut jc = C0;
    jc[0] = poke(C0[0], 4);
    jc[1] = poke(C0[1], 1);
    jc[4] = poke(C0[4], -1);
    let mut jd = D0;
    jd[0] = poke(D0[0], -1);
    jd[4] = poke(D0[4], -1);
    let (q0, d0, m0, f0) = score(&rows, &jc, &jd, 0x20);
    println!("j3432+0x20 Q {q0} d={d0} max={m0} F_or {f0} (bars 3438/3021)");
    let deltas = [1i32, -1, 2, -2, 4, -4];
    println!("C[8] pokes:");
    for &k in &deltas {
        let mut c = jc;
        c[8] = poke(jc[8], k);
        let (q, d, m, fo) = score(&rows, &c, &jd, 0x20);
        if q >= q0 || fo >= f0 {
            println!("  C8 {k:+} Q {q} d={d} max={m} F_or {fo}");
        }
    }
    println!("D[1] pokes:");
    for &k in &deltas {
        let mut d = jd;
        d[1] = poke(jd[1], k);
        let (q, dd, m, fo) = score(&rows, &jc, &d, 0x20);
        if q >= q0 || fo >= f0 {
            println!("  D1 {k:+} Q {q} d={dd} max={m} F_or {fo}");
        }
    }
    println!("D[7] pokes:");
    for &k in &deltas {
        let mut d = jd;
        d[7] = poke(jd[7], k);
        let (q, dd, m, fo) = score(&rows, &jc, &d, 0x20);
        if q >= q0 || fo >= f0 {
            println!("  D7 {k:+} Q {q} d={dd} max={m} F_or {fo}");
        }
    }
    println!("C[2] pokes (bit6 store neighbor):");
    for &k in &deltas {
        let mut c = jc;
        c[2] = poke(jc[2], k);
        let (q, d, m, fo) = score(&rows, &c, &jd, 0x20);
        if q >= q0 || fo >= f0 {
            println!("  C2 {k:+} Q {q} d={d} max={m} F_or {fo}");
        }
    }
}
