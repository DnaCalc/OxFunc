//! Direct-only named F and the ulp>=2 direct misses of CR / 0x210.
//! Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC64_RN};
use std::env;

const CW: u16 = CW_PC64_RN;
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
const P7: [f64; 8] = [
    3.004592610201616005e2,
    4.519189537118729422e2,
    3.393208167343436870e2,
    1.529892850469404039e2,
    4.316222722205673530e1,
    7.211758250883093659,
    5.641955174789739711e-1,
    -1.368648573827167067e-7,
];
const Q7: [f64; 8] = [
    3.004592609569832933e2,
    7.909509253278980272e2,
    9.313540948506096211e2,
    6.389802644656311665e2,
    2.775854447439876434e2,
    7.700015293522947295e1,
    1.278272731962942351e1,
    1.0,
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
fn horner(cs: &[f64], x: Ext80) -> Ext80 {
    let mut acc = ef(0.0);
    for &c in cs.iter().rev() {
        acc = ext_add(&ext_mul(&acc, &x, CW), &ef(c), CW);
    }
    acc
}
fn cody(y: f64, mask: u32) -> f64 {
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
fn aabb(x: f64) -> f64 {
    let xe = ef(x);
    let z = ext_div(&ef(1.0), &ext_add(&ef(2.5), &ext_mul(&xe, &xe, CW), CW), CW);
    let t = ext_sub_13(z);
    let ratio = ext_div(&horner(&AA, z), &horner(&BB, z), CW);
    let mut acc = ext_add(&ext_mul(&ratio, &t, CW), &ef(-0.288573438386338758794591212600e-02), CW);
    acc = ext_add(&ext_mul(&acc, &t, CW), &ef(-0.261515522487415653487049835220e-01), CW);
    acc = ext_add(&ext_mul(&acc, &t, CW), &ef(0.540464821348814822409610122136), CW);
    ext_to_f64(&ext_div(&acc, &xe, CW), CW)
}
fn ext_sub_13(z: Ext80) -> Ext80 {
    rx::ext_sub(&ext_mul(&ef(13.0), &z, CW), &ef(1.0), CW)
}
fn n7(y: f64) -> f64 {
    let ye = ef(y.abs());
    ext_to_f64(&ext_div(&horner(&P7, ye), &horner(&Q7, ye), CW), CW)
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
    dmid: usize,
    dn: usize,
    m1d: usize,
    max_d: u64,
    exact: usize,
}
fn add_row(a: &mut Acc, d: u64, direct: bool, is_exact: bool) {
    if is_exact {
        a.exact += 1;
    }
    if direct {
        a.dn += 1;
        a.max_d = a.max_d.max(d);
        if d == 0 {
            a.dmid += 1;
        } else if d == 1 {
            a.m1d += 1;
        }
    }
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    type Ev = fn(f64) -> f64;
    let graphs: [(&str, Ev); 8] = [
        ("CR mask0", |z| qw(z, cody(z, 0))),
        ("CR 0x210", |z| qw(z, cody(z, 0x210))),
        ("CR 0x74", |z| qw(z, cody(z, 0x74))),
        ("AABB[2,4)+cody0", |z| {
            if (2.0..4.0).contains(&z) {
                qw(z, aabb(z))
            } else {
                qw(z, cody(z, 0))
            }
        }),
        ("Cody1969 n=7", |z| qw(z, n7(z))),
        ("nswc_derfc0", |z| qw(z, f::nswc_derfc0(z))),
        ("cephes_f", |z| qw(z, f::cephes_f(z))),
        ("libm::erfc", libm::erfc),
    ];
    println!("## named direct-mid");
    for (name, ev) in graphs {
        let mut a = Acc::default();
        for r in &rows {
            if r.z < 0.5 || r.z >= 4.0 {
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
            add_row(&mut a, d, r.direct, d == 0);
        }
        println!(
            "{name:22} exact={} dmid={}/{} m1d={} maxd={} w1={}",
            a.exact,
            a.dmid,
            a.dn,
            a.m1d,
            a.max_d,
            a.dmid + a.m1d
        );
    }

    println!("## CR mask0 direct ulp>=2");
    for r in &rows {
        if !r.direct || r.z < 0.5 || r.z >= 4.0 {
            continue;
        }
        let qg = qw(r.z, cody(r.z, 0));
        let d = ulp_distance(qg, f64::from_bits(r.qbits)).unwrap_or(99);
        if d >= 2 {
            let sign = if qg > f64::from_bits(r.qbits) { '+' } else { '-' };
            println!("  z={:.17} ulp={d} {sign} bits={:016x}", r.z, r.qbits);
        }
    }
    println!("## CR 0x210 direct ulp>=2");
    for r in &rows {
        if !r.direct || r.z < 0.5 || r.z >= 4.0 {
            continue;
        }
        let qg = qw(r.z, cody(r.z, 0x210));
        let d = ulp_distance(qg, f64::from_bits(r.qbits)).unwrap_or(99);
        if d >= 2 {
            let sign = if qg > f64::from_bits(r.qbits) { '+' } else { '-' };
            println!("  z={:.17} ulp={d} {sign} bits={:016x}", r.z, r.qbits);
        }
    }
}
