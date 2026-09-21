//! 0x210 then NSWC AABB [2,4); extra store bits vs only-210 pair. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_sub, ext_to_f64, Ext80, CW_PC64_RN};
use std::env;

const CW: u16 = CW_PC64_RN;
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
const ONLY: [f64; 2] = [3.2708333333333335, 3.46875];

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
fn horner_hi(cs: &[f64], x: Ext80) -> Ext80 {
    let mut acc = ef(0.0);
    for &c in cs.iter().rev() {
        acc = ext_add(&ext_mul(&acc, &x, CW), &ef(c), CW);
    }
    acc
}
fn aabb(x: f64) -> f64 {
    let xe = ef(x);
    let xx = ext_mul(&xe, &xe, CW);
    let z = ext_div(&ef(1.0), &ext_add(&ef(2.5), &xx, CW), CW);
    let t = ext_sub(&ext_mul(&ef(13.0), &z, CW), &ef(1.0), CW);
    let ratio = ext_div(&horner_hi(&AA, z), &horner_hi(&BB, z), CW);
    let mut acc = ext_add(&ext_mul(&ratio, &t, CW), &ef(E2), CW);
    acc = ext_add(&ext_mul(&acc, &t, CW), &ef(E1), CW);
    acc = ext_add(&ext_mul(&acc, &t, CW), &ef(E0), CW);
    ext_to_f64(&ext_div(&acc, &xe, CW), CW)
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
    let score = |ff: &dyn Fn(f64) -> f64| {
        let mut dmid = 0usize;
        let mut n = 0usize;
        let mut mx = 0u64;
        let mut only = 0usize;
        let mut b24 = 0usize;
        let mut n24 = 0usize;
        for r in &rows {
            if !r.direct || r.z < 0.5 || r.z >= 4.0 {
                continue;
            }
            n += 1;
            let d = ulp_distance(qwf(r.z, ff(r.z)), f64::from_bits(r.qbits)).unwrap_or(99);
            if d == 0 {
                dmid += 1;
            } else {
                mx = mx.max(d);
            }
            if r.z >= 2.0 {
                n24 += 1;
                if d == 0 {
                    b24 += 1;
                }
            }
            if ONLY.iter().any(|&z| (r.z - z).abs() < 1e-12) && d == 0 {
                only += 1;
            }
        }
        (dmid, n, mx, b24, n24, only)
    };

    println!("bars 0x210 dmid 105/226 [2,4) hard 37/128");
    for (name, ff) in [
        ("0x210", Box::new(|z| cody(z, 0x210)) as Box<dyn Fn(f64) -> f64>),
        ("AABB", Box::new(aabb)),
        ("derfc0", Box::new(|z| f::nswc_derfc0(z))),
        ("0x210 z<2 else AABB", Box::new(|z| {
            if z < 2.0 {
                cody(z, 0x210)
            } else {
                aabb(z)
            }
        })),
        ("0x210 z<2 else derfc0", Box::new(|z| {
            if z < 2.0 {
                cody(z, 0x210)
            } else {
                f::nswc_derfc0(z)
            }
        })),
        ("0x210 z<2.5 else AABB", Box::new(|z| {
            if z < 2.5 {
                cody(z, 0x210)
            } else {
                aabb(z)
            }
        })),
    ] {
        let (dmid, n, mx, b24, n24, only) = score(&*ff);
        println!("  {name:24} dmid={dmid}/{n} maxd={mx} [2,4) {b24}/{n24} only210={only}/2");
    }

    println!("0x210-then-AABB cut scan (print if dmid>=105 or only>0):");
    let mut best = 0usize;
    let mut best_c = 0.0;
    for k in 0..=40 {
        let c = 1.5 + k as f64 * 0.05;
        let (dmid, n, mx, b24, n24, only) = score(&|z| {
            if z < c {
                cody(z, 0x210)
            } else {
                aabb(z)
            }
        });
        if dmid >= 105 || only > 0 {
            println!(
                "  cut={c:.2} dmid={dmid}/{n} maxd={mx} [2,4) {b24}/{n24} only={only}"
            );
        }
        if dmid > best {
            best = dmid;
            best_c = c;
        }
    }
    println!("  BEST dmid {best} @ {best_c:.2} (bar 105)");

    println!("0x210 | extra bit (print dmid>=105 or only>0):");
    for b in 0..17u32 {
        if b == 4 || b == 9 {
            continue;
        }
        let m = 0x210 | (1u32 << b);
        let (dmid, n, mx, _, _, only) = score(&|z| cody(z, m));
        if dmid >= 105 || only > 0 {
            println!("  bit={b} mask={m:#x} dmid={dmid}/{n} maxd={mx} only={only}");
        }
    }
}
