//! NSWC AABB x87 t-map as Q=w*F on [2,4). Documented DERFC0 piece. Not an ID.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_sub, ext_to_f64, Ext80, CW_PC64_RN};
use std::env;

const CW: u16 = CW_PC64_RN;
const ULP_CAP: u64 = 1 << 20;
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
fn horner_hi(cs: &[f64], x: Ext80) -> Ext80 {
    let mut acc = ef(0.0);
    for &c in cs.iter().rev() {
        acc = ext_add(&ext_mul(&acc, &x, CW), &ef(c), CW);
    }
    acc
}
fn aabb(x: f64, tmode: u8) -> f64 {
    let xe = ef(x);
    let xx = ext_mul(&xe, &xe, CW);
    let z = match tmode {
        0 => ext_div(&ef(1.0), &ext_add(&ef(2.5), &xx, CW), CW),
        1 => ext_div(&ef(1.0), &ext_add(&ext_div(&ef(5.0), &ef(2.0), CW), &xx, CW), CW),
        _ => ext_div(&ef(1.0), &ext_add(&ext_add(&ef(2.0), &ef(0.5), CW), &xx, CW), CW),
    };
    let t = match tmode {
        0 => ext_sub(&ext_mul(&ef(13.0), &z, CW), &ef(1.0), CW),
        _ => ext_sub(&ext_mul(&ef(13.0), &z, CW), &ef(1.0), CW),
    };
    let ratio = ext_div(&horner_hi(&AA, z), &horner_hi(&BB, z), CW);
    let mut acc = ext_add(&ext_mul(&ratio, &t, CW), &ef(E2), CW);
    acc = ext_add(&ext_mul(&acc, &t, CW), &ef(E1), CW);
    acc = ext_add(&ext_mul(&acc, &t, CW), &ef(E0), CW);
    ext_to_f64(&ext_div(&acc, &xe, CW), CW)
}
fn maybe(x: Ext80, mask: u32, bit: u32) -> Ext80 {
    if bit < 32 && mask & (1u32 << bit) != 0 {
        ef(ext_to_f64(&x, CW))
    } else {
        x
    }
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

#[derive(Default)]
struct Acc {
    exact: usize,
    n: usize,
    max_ulp: u64,
    dmid: usize,
    dn: usize,
}
impl Acc {
    fn add(&mut self, d: u64, direct: bool) {
        self.n += 1;
        if direct {
            self.dn += 1;
        }
        if d == 0 {
            self.exact += 1;
            if direct {
                self.dmid += 1;
            }
        } else {
            self.max_ulp = self.max_ulp.max(d);
        }
    }
}
fn fmt(a: &Acc) -> String {
    format!("{}/{} max={} d={}/{}", a.exact, a.n, a.max_ulp, a.dmid, a.dn)
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    let graphs: [(&str, fn(f64) -> f64); 7] = [
        ("cody74 global", |z| qw(z, cody74(z))),
        ("AABB x87 2.5/13 z in[2,4) else cody74", |z| {
            if (2.0..4.0).contains(&z) {
                qw(z, aabb(z, 0))
            } else if z < 4.0 {
                qw(z, cody74(z))
            } else {
                qw(z, f::nswc_ccdd_f(z))
            }
        }),
        ("AABB 5/2 z in[2,4) else cody74", |z| {
            if (2.0..4.0).contains(&z) {
                qw(z, aabb(z, 1))
            } else if z < 4.0 {
                qw(z, cody74(z))
            } else {
                qw(z, f::nswc_ccdd_f(z))
            }
        }),
        ("AABB 2+0.5 z in[2,4) else cody74", |z| {
            if (2.0..4.0).contains(&z) {
                qw(z, aabb(z, 2))
            } else if z < 4.0 {
                qw(z, cody74(z))
            } else {
                qw(z, f::nswc_ccdd_f(z))
            }
        }),
        ("AABB x87 global z>=2", |z| {
            if z >= 2.0 && z <= 4.0 {
                qw(z, aabb(z, 0))
            } else if z < 2.0 {
                qw(z, cody74(z))
            } else {
                qw(z, f::nswc_derfc0(z))
            }
        }),
        ("nswc_derfc0 * w", |z| qw(z, f::nswc_derfc0(z))),
        ("native AABB [2,4) via derfc0", |z| {
            if (2.0..4.0).contains(&z) {
                qw(z, f::nswc_derfc0(z))
            } else if z < 4.0 {
                qw(z, cody74(z))
            } else {
                qw(z, f::nswc_ccdd_f(z))
            }
        }),
    ];
    for (name, ev) in graphs {
        let mut mid = Acc::default();
        let mut tail = Acc::default();
        for r in &rows {
            if r.z < 0.5 {
                continue;
            }
            let qg = ev(r.z);
            if !qg.is_finite() {
                continue;
            }
            let d = ulp_distance(qg, f64::from_bits(r.qbits)).unwrap_or(u64::MAX);
            if d > ULP_CAP {
                continue;
            }
            if r.z < 4.0 {
                mid.add(d, r.direct);
            } else {
                tail.add(d, r.direct);
            }
        }
        println!("{name:48} mid {} tail {}", fmt(&mid), fmt(&tail));
    }

    println!("\n## Cody 0x74 direct-mid misses by band");
    let mut nmiss = [0usize; 3];
    let mut n = [0usize; 3];
    for r in &rows {
        if !r.direct || r.z < 0.5 || r.z >= 4.0 {
            continue;
        }
        let b = if r.z < 1.0 {
            0
        } else if r.z < 2.0 {
            1
        } else {
            2
        };
        n[b] += 1;
        let qg = qw(r.z, cody74(r.z));
        let d = ulp_distance(qg, f64::from_bits(r.qbits)).unwrap_or(u64::MAX);
        if d != 0 {
            nmiss[b] += 1;
        }
    }
    println!(
        "[0.5,1) miss {}/{}  [1,2) {}/{}  [2,4) {}/{}",
        nmiss[0], n[0], nmiss[1], n[1], nmiss[2], n[2]
    );
}
