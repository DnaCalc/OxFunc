//! C/D Horner store-after-add vs store-after-mul (HW=1/2). Not an identity.
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
fn split(y: f64, mask: u32) -> f64 {
    let ye = ef(y.abs());
    let mut xnum = ext_mul(&ef(C0[8]), &ye, CW);
    let mut xden = ye;
    xnum = maybe(xnum, mask, 0);
    xden = maybe(xden, mask, 1);
    for i in 0..7 {
        let b = 2 + 4 * i as u32;
        xnum = ext_add(&xnum, &ef(C0[i]), CW);
        xnum = maybe(xnum, mask, b);
        xnum = ext_mul(&xnum, &ye, CW);
        xnum = maybe(xnum, mask, b + 1);
        xden = ext_add(&xden, &ef(D0[i]), CW);
        xden = maybe(xden, mask, b + 2);
        xden = ext_mul(&xden, &ye, CW);
        xden = maybe(xden, mask, b + 3);
    }
    let mut q = ext_div(
        &ext_add(&xnum, &ef(C0[7]), CW),
        &ext_add(&xden, &ef(D0[7]), CW),
        CW,
    );
    q = maybe(q, mask, 30);
    ext_to_f64(&q, CW)
}
fn fused(y: f64, mask: u32) -> f64 {
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
fn dmid_of(rows: &[f::QRow], ev: impl Fn(f64) -> f64) -> (usize, usize, u64) {
    let mut dmid = 0usize;
    let mut n = 0usize;
    let mut mx = 0u64;
    for r in rows {
        if !r.direct || r.z < 0.5 || r.z >= 4.0 {
            continue;
        }
        n += 1;
        let d = ulp_distance(qwf(r.z, ev(r.z)), f64::from_bits(r.qbits)).unwrap_or(99);
        if d == 0 {
            dmid += 1;
        } else {
            mx = mx.max(d);
        }
    }
    (dmid, n, mx)
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    let (b, n, mx) = dmid_of(&rows, |z| fused(z, 0x210));
    println!("fused 0x210 dmid={b}/{n} maxd={mx}");
    let (b0, _, mx0) = dmid_of(&rows, |z| split(z, 0));
    println!("split mask0 dmid={b0}/{n} maxd={mx0}");
    println!("split HW=1 (print dmid>=96):");
    let mut best = b0;
    let mut best_m = 0u32;
    for i in 0..31u32 {
        let m = 1u32 << i;
        let (dmid, _, mx) = dmid_of(&rows, |z| split(z, m));
        if dmid >= 96 {
            println!("  bit={i} mask={m:#x} dmid={dmid} maxd={mx}");
        }
        if dmid > best {
            best = dmid;
            best_m = m;
        }
    }
    println!("HW=1 best dmid {best} mask={best_m:#x}");
    println!("split HW=2 (print dmid>=105):");
    let mut best2 = best;
    let mut best2m = best_m;
    for i in 0..31u32 {
        for j in (i + 1)..31u32 {
            let m = (1u32 << i) | (1u32 << j);
            let (dmid, _, mx) = dmid_of(&rows, |z| split(z, m));
            if dmid >= 105 {
                println!("  bits={i},{j} mask={m:#x} dmid={dmid} maxd={mx}");
            }
            if dmid > best2 {
                best2 = dmid;
                best2m = m;
            }
        }
    }
    println!("HW=2 best dmid {best2} mask={best2m:#x} (bar fused 105)");
}
