//! NSWC AABB HW=1,2 stores as Q=w*F on [2,4). Not a cube. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_sub, ext_to_f64, Ext80, CW_PC64_RN};
use std::env;

const CW: u16 = CW_PC64_RN;
const ULP_CAP: u64 = 1 << 20;
const NSITE: u32 = 28;
const AA: [f64; 9] = [
    -0.45894433406309678202825375e-03,
    -0.12281298722544724287816236e-01,
    -0.91144359512342900801764781e-01,
    -0.28412489223839285652511367e-01,
    0.14083827189977123530129812e+01,
    0.11532175281537044570477189e+01,
    -0.72170903389442152112483632e+01,
    -0.19685597805218214001309225e+01,
    0.93846891504541841150916038e+01,
];
const BB: [f64; 12] = [
    1.0,
    0.25136329960926527692263725e+02,
    0.15349442087145759184067981e+03,
    -0.29971215958498680905476402e+03,
    -0.33876477506888115226730368e+04,
    0.28301829314924804988873701e+04,
    0.22979620942196507068034887e+05,
    -0.24280681522998071562462041e+05,
    -0.36680620673264731899504580e+05,
    0.42278731622295627627042436e+05,
    0.28834257644413614344549790e+03,
    0.70226293775648358646587341e+03,
];
const E0: f64 = 0.540464821348814822409610122136;
const E1: f64 = -0.261515522487415653487049835220e-01;
const E2: f64 = -0.288573438386338758794591212600e-02;
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
fn horner(cs: &[f64], x: Ext80, mask: u32, mut bit: u32) -> (Ext80, u32) {
    let mut acc = ef(0.0);
    for &c in cs.iter().rev() {
        acc = ext_add(&ext_mul(&acc, &x, CW), &ef(c), CW);
        acc = maybe(acc, mask, bit);
        bit += 1;
    }
    (acc, bit)
}
fn aabb(x: f64, mask: u32) -> f64 {
    let xe = ef(x);
    let mut z = ext_div(&ef(1.0), &ext_add(&ef(2.5), &ext_mul(&xe, &xe, CW), CW), CW);
    z = maybe(z, mask, 0);
    let mut t = ext_sub(&ext_mul(&ef(13.0), &z, CW), &ef(1.0), CW);
    t = maybe(t, mask, 1);
    let (n, b) = horner(&AA, z, mask, 2);
    let (d, b) = horner(&BB, z, mask, b);
    let mut acc = ext_div(&n, &d, CW);
    acc = maybe(acc, mask, b);
    let mut bit = b + 1;
    acc = ext_add(&ext_mul(&acc, &t, CW), &ef(E2), CW);
    acc = maybe(acc, mask, bit);
    bit += 1;
    acc = ext_add(&ext_mul(&acc, &t, CW), &ef(E1), CW);
    acc = maybe(acc, mask, bit);
    bit += 1;
    acc = ext_add(&ext_mul(&acc, &t, CW), &ef(E0), CW);
    acc = maybe(acc, mask, bit);
    bit += 1;
    acc = ext_div(&acc, &xe, CW);
    acc = maybe(acc, mask, bit);
    ext_to_f64(&acc, CW)
}
fn cody74(y: f64) -> f64 {
    let ye = ef(y.abs());
    let mut xnum = ext_mul(&ef(C[8]), &ye, CW);
    let mut xden = ye;
    xnum = maybe(xnum, 0x74, 0);
    xden = maybe(xden, 0x74, 1);
    for i in 0..7 {
        xnum = ext_mul(&ext_add(&xnum, &ef(C[i]), CW), &ye, CW);
        xden = ext_mul(&ext_add(&xden, &ef(D[i]), CW), &ye, CW);
        xnum = maybe(xnum, 0x74, 2 + 2 * i as u32);
        xden = maybe(xden, 0x74, 3 + 2 * i as u32);
    }
    let mut q = ext_div(&ext_add(&xnum, &ef(C[7]), CW), &ext_add(&xden, &ef(D[7]), CW), CW);
    q = maybe(q, 0x74, 16);
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
    band24: usize,
    n24: usize,
}
fn score(rows: &[f::QRow], mask: u32) -> Mid {
    let mut exact = 0usize;
    let mut dmid = 0usize;
    let mut max_ulp = 0u64;
    let mut band24 = 0usize;
    let mut n24 = 0usize;
    for r in rows {
        if r.z < 0.5 || r.z >= 4.0 {
            continue;
        }
        let ff = if (2.0..4.0).contains(&r.z) {
            aabb(r.z, mask)
        } else {
            cody74(r.z)
        };
        let qg = qw(r.z, ff);
        if !qg.is_finite() {
            continue;
        }
        let d = ulp_distance(qg, f64::from_bits(r.qbits)).unwrap_or(u64::MAX);
        if d > ULP_CAP {
            continue;
        }
        if (2.0..4.0).contains(&r.z) {
            n24 += 1;
            if d == 0 {
                band24 += 1;
            }
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
        band24,
        n24,
    }
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    let base = score(&rows, 0);
    println!(
        "AABB mask0 {} dmid={} max={} [2,4) {}/{}",
        base.exact, base.dmid, base.max_ulp, base.band24, base.n24
    );
    let mut best1 = base;
    let mut lab1 = 0u32;
    println!("## HW=1");
    for b in 0..NSITE {
        let m = 1u32 << b;
        let sc = score(&rows, m);
        if sc.exact > best1.exact
            || (sc.exact == best1.exact && sc.dmid > best1.dmid)
            || (sc.exact == best1.exact && sc.dmid == best1.dmid && sc.band24 > best1.band24)
        {
            best1 = sc;
            lab1 = m;
            println!(
                "HIT bit={b} mask={m:#x} {} dmid={} max={} [2,4) {}/{}",
                sc.exact, sc.dmid, sc.max_ulp, sc.band24, sc.n24
            );
        }
    }
    println!(
        "best HW1 mask={lab1:#x} {} dmid={} [2,4) {}/{}",
        best1.exact, best1.dmid, best1.band24, best1.n24
    );

    println!("## HW=2");
    let mut best2 = best1;
    let mut lab2 = lab1;
    for i in 0..NSITE {
        for j in (i + 1)..NSITE {
            let m = (1u32 << i) | (1u32 << j);
            let sc = score(&rows, m);
            if sc.exact > best2.exact
                || (sc.exact == best2.exact && sc.dmid > best2.dmid)
                || (sc.exact == best2.exact && sc.dmid == best2.dmid && sc.band24 > best2.band24)
            {
                best2 = sc;
                lab2 = m;
                println!(
                    "HIT bits={i},{j} mask={m:#x} {} dmid={} max={} [2,4) {}/{}",
                    sc.exact, sc.dmid, sc.max_ulp, sc.band24, sc.n24
                );
            }
        }
    }
    println!(
        "best HW2 mask={lab2:#x} {} dmid={} max={} [2,4) {}/{}",
        best2.exact, best2.dmid, best2.max_ulp, best2.band24, best2.n24
    );
}
