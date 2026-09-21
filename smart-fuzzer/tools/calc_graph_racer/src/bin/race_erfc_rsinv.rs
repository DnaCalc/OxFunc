//! Cephes R/S in 1/z, z², 1/z² on z≥8. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC64_RN};
use std::env;

const CW: u16 = CW_PC64_RN;
const R: [f64; 6] = [
    5.64189583547755073984e-1,
    1.27536670759978104416e0,
    5.01905042251180477414e0,
    6.16021097993053585195e0,
    7.40974269950448939160e0,
    2.97886665372100240670e0,
];
const S: [f64; 6] = [
    2.26052863220117276590e0,
    9.39603524938001434673e0,
    1.20489539808096656605e1,
    1.70814450747565897222e1,
    9.60896809063285878198e0,
    3.36907645100081516050e0,
];
const RPINV: f64 = 0.56418958354775628695;

fn ef(x: f64) -> Ext80 {
    ext_from_f64(x)
}
fn polevl(x: Ext80, coef: &[f64]) -> Ext80 {
    let mut ans = ef(coef[0]);
    for &c in &coef[1..] {
        ans = ext_add(&ext_mul(&ans, &x, CW), &ef(c), CW);
    }
    ans
}
fn p1evl(x: Ext80, coef: &[f64]) -> Ext80 {
    let mut ans = ext_add(&x, &ef(coef[0]), CW);
    for &c in &coef[1..] {
        ans = ext_add(&ext_mul(&ans, &x, CW), &ef(c), CW);
    }
    ans
}
fn qw(z: f64, ff: f64) -> f64 {
    let v = f::w_rn53(z) * ff;
    if v.abs() < f64::MIN_POSITIVE {
        0.0
    } else {
        v
    }
}
fn rs_of(arg: Ext80) -> f64 {
    ext_to_f64(&ext_div(&polevl(arg, &R), &p1evl(arg, &S), CW), CW)
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    let graphs: [(&str, Box<dyn Fn(f64) -> f64>); 8] = [
        ("R/S(z)", Box::new(|z| rs_of(ef(z)))),
        ("R/S(1/z)", Box::new(|z| rs_of(ef(1.0 / z)))),
        ("R/S(1/z)/z", Box::new(|z| rs_of(ef(1.0 / z)) / z)),
        ("R/S(z²)", Box::new(|z| rs_of(ef(z * z)))),
        ("R/S(1/z²)", Box::new(|z| rs_of(ef(1.0 / (z * z))))),
        ("(rp-R/S(1/z²))/z", Box::new(|z| (RPINV - rs_of(ef(1.0 / (z * z)))) / z)),
        ("R/S(z)/z", Box::new(|z| rs_of(ef(z)) / z)),
        ("cephes_f", Box::new(|z| f::cephes_f(z))),
    ];
    println!("R/S argument map z>=8 (bar cephes 0x5005 z>=8 = 991):");
    for (name, ev) in &graphs {
        let mut ex8 = 0usize;
        let mut n8 = 0usize;
        let mut ex20 = 0usize;
        let mut n20 = 0usize;
        let mut xhit = 0usize;
        let mut mx = 0u64;
        for r in &rows {
            if r.z < 8.0 {
                continue;
            }
            let d = ulp_distance(qw(r.z, ev(r.z)), f64::from_bits(r.qbits)).unwrap_or(u64::MAX);
            n8 += 1;
            if d == 0 {
                ex8 += 1;
            } else if d < (1 << 20) {
                mx = mx.max(d);
            }
            if r.z >= 20.0 {
                n20 += 1;
                if d == 0 {
                    ex20 += 1;
                }
            }
            if r.direct && r.z > 26.52 && r.z < 26.544 && d == 0 {
                xhit += 1;
            }
        }
        println!("  {name:18} z>=8 {ex8}/{n8} z>=20 {ex20}/{n20} max={mx} xhit={xhit}");
    }
}
