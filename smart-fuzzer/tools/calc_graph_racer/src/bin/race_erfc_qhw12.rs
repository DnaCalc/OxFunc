//! Hamming-weight 1 and 2 store masks on SPECFUN C/D, Q=w*F. Human-era temps.
//! Not a 24-bit cube. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC64_RN};
use std::env;

const CW: u16 = CW_PC64_RN;
const ULP_CAP: u64 = 1 << 20;
const NSITE: u32 = 17;
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
fn maybe(x: Ext80, mask: u32, bit: u32) -> Ext80 {
    if bit < 32 && mask & (1u32 << bit) != 0 {
        ef(ext_to_f64(&x, CW))
    } else {
        x
    }
}
fn cody_cd(y: f64, mask: u32) -> f64 {
    let ye = ef(y.abs());
    let mut xnum = ext_mul(&ef(C[8]), &ye, CW);
    let mut xden = ye;
    xnum = maybe(xnum, mask, 0);
    xden = maybe(xden, mask, 1);
    for i in 0..7 {
        xnum = ext_mul(&ext_add(&xnum, &ef(C[i]), CW), &ye, CW);
        xden = ext_mul(&ext_add(&xden, &ef(D[i]), CW), &ye, CW);
        xnum = maybe(xnum, mask, 2 + 2 * i as u32);
        xden = maybe(xden, mask, 3 + 2 * i as u32);
    }
    let mut q = ext_div(&ext_add(&xnum, &ef(C[7]), CW), &ext_add(&xden, &ef(D[7]), CW), CW);
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

#[derive(Clone, Copy)]
struct Mid {
    exact: usize,
    dmid: usize,
    max_ulp: u64,
}
fn score(rows: &[f::QRow], mask: u32) -> Mid {
    let mut exact = 0usize;
    let mut dmid = 0usize;
    let mut max_ulp = 0u64;
    for r in rows {
        if r.z < 0.5 || r.z >= 4.0 {
            continue;
        }
        let qg = qw(r.z, cody_cd(r.z, mask));
        if !qg.is_finite() {
            continue;
        }
        let d = ulp_distance(qg, f64::from_bits(r.qbits)).unwrap_or(u64::MAX);
        if d > ULP_CAP {
            continue;
        }
        if d == 0 {
            exact += 1;
            if r.direct {
                dmid += 1;
            }
        } else {
            max_ulp = max_ulp.max(d);
        }
    }
    Mid {
        exact,
        dmid,
        max_ulp,
    }
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    let base = score(&rows, 0);
    println!(
        "mask0 {} dmid={} max={}",
        base.exact, base.dmid, base.max_ulp
    );
    let mut best1 = base;
    let mut lab1 = 0u32;
    println!("## HW=1");
    for b in 0..NSITE {
        let m = 1u32 << b;
        let sc = score(&rows, m);
        if sc.exact > best1.exact || (sc.exact == best1.exact && sc.dmid > best1.dmid) {
            best1 = sc;
            lab1 = m;
            println!(
                "HIT bit={b} mask={m:#x} {} dmid={} max={}",
                sc.exact, sc.dmid, sc.max_ulp
            );
        }
    }
    println!(
        "best HW1 mask={lab1:#x} {} dmid={}",
        best1.exact, best1.dmid
    );

    println!("## HW=2");
    let mut best2 = best1;
    let mut lab2 = lab1;
    for i in 0..NSITE {
        for j in (i + 1)..NSITE {
            let m = (1u32 << i) | (1u32 << j);
            let sc = score(&rows, m);
            if sc.exact > best2.exact || (sc.exact == best2.exact && sc.dmid > best2.dmid) {
                best2 = sc;
                lab2 = m;
                println!(
                    "HIT bits={i},{j} mask={m:#x} {} dmid={} max={}",
                    sc.exact, sc.dmid, sc.max_ulp
                );
            }
        }
    }
    println!(
        "best HW2 mask={lab2:#x} {} dmid={} max={}",
        best2.exact, best2.dmid, best2.max_ulp
    );
}
