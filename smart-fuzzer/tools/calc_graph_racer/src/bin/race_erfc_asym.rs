//! Large-z erfcx asymptotic / XBIG leftover. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_sqrt, ext_to_f64, Ext80, CW_PC64_RN};
use std::env;

const CW: u16 = CW_PC64_RN;
const RPINV: f64 = 0.56418958354775628695;
const CEPHES_P: [f64; 9] = [
    2.46196981473530512524e-10,
    5.64189564831068821977e-1,
    7.46321056442269912687e0,
    4.86371970985681366614e1,
    1.96520832956077098242e2,
    5.26445194995477358631e2,
    9.34528527171957607540e2,
    1.02755188689515710272e3,
    5.57535335369399327526e2,
];
const CEPHES_Q: [f64; 8] = [
    1.32281951154744992508e1,
    8.67072140885989742329e1,
    3.54937778887819891062e2,
    9.75708501743205489753e2,
    1.82390916687909736289e3,
    2.24633760818710981792e3,
    1.65666309194161350182e3,
    5.57535340817727675546e2,
];
const CEPHES_R: [f64; 6] = [
    5.64189583547755073984e-1,
    1.27536670759978104416e0,
    5.01905042251180477414e0,
    6.16021097993053585195e0,
    7.40974269950448939160e0,
    2.97886665372100240670e0,
];
const CEPHES_S: [f64; 6] = [
    2.26052863220117276590e0,
    9.39603524938001434673e0,
    1.20489539808096656605e1,
    1.70814450747565897222e1,
    9.60896809063285878198e0,
    3.36907645100081516050e0,
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
fn polevl_mask(x: Ext80, coef: &[f64], mask: u32, mut bit: u32) -> (Ext80, u32) {
    let mut ans = ef(coef[0]);
    for &c in &coef[1..] {
        ans = ext_add(&ext_mul(&ans, &x, CW), &ef(c), CW);
        ans = maybe(ans, mask, bit);
        bit += 1;
    }
    (ans, bit)
}
fn p1evl_mask(x: Ext80, coef: &[f64], mask: u32, mut bit: u32) -> (Ext80, u32) {
    let mut ans = ext_add(&x, &ef(coef[0]), CW);
    ans = maybe(ans, mask, bit);
    bit += 1;
    for &c in &coef[1..] {
        ans = ext_add(&ext_mul(&ans, &x, CW), &ef(c), CW);
        ans = maybe(ans, mask, bit);
        bit += 1;
    }
    (ans, bit)
}
fn cephes_5005(x: f64) -> f64 {
    let ax = x.abs();
    let xe = ef(ax);
    let mask = 0x5005u32;
    let (num, den, bit0) = if ax < 8.0 {
        let (p, b) = polevl_mask(xe, &CEPHES_P, mask, 0);
        let (q, b) = p1evl_mask(xe, &CEPHES_Q, mask, b);
        (p, q, b)
    } else {
        let (r, b) = polevl_mask(xe, &CEPHES_R, mask, 0);
        let (s, b) = p1evl_mask(xe, &CEPHES_S, mask, b);
        (r, s, b)
    };
    let mut q = ext_div(&num, &den, CW);
    q = maybe(q, mask, bit0);
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
fn asym_f64(z: f64, n: usize) -> f64 {
    let z2 = z * z;
    let inv = 1.0 / (2.0 * z2);
    let mut term = 1.0;
    let mut s = 1.0;
    for k in 1..n {
        term *= -(2.0 * k as f64 - 1.0) * inv;
        s += term;
    }
    RPINV / z * s
}
fn asym_80(z: f64, n: usize) -> f64 {
    let ze = ef(z);
    let z2 = ext_mul(&ze, &ze, CW);
    let inv = ext_div(&ef(1.0), &ext_mul(&ef(2.0), &z2, CW), CW);
    let mut term = ef(1.0);
    let mut s = ef(1.0);
    for k in 1..n {
        let odd = 2.0 * k as f64 - 1.0;
        term = ext_mul(&term, &ef(-odd), CW);
        term = ext_mul(&term, &inv, CW);
        s = ext_add(&s, &term, CW);
    }
    ext_to_f64(&ext_mul(&ext_div(&ef(RPINV), &ze, CW), &s, CW), CW)
}
fn asym_lead80(z: f64, n: usize) -> f64 {
    let ze = ef(z);
    let rp = ext_div(&ef(1.0), &ext_sqrt(&ef(std::f64::consts::PI), CW), CW);
    let z2 = ext_mul(&ze, &ze, CW);
    let inv = ext_div(&ef(1.0), &ext_mul(&ef(2.0), &z2, CW), CW);
    let mut term = ef(1.0);
    let mut s = ef(1.0);
    for k in 1..n {
        let odd = 2.0 * k as f64 - 1.0;
        term = ext_mul(&term, &ef(-odd), CW);
        term = ext_mul(&term, &inv, CW);
        s = ext_add(&s, &term, CW);
    }
    ext_to_f64(&ext_mul(&ext_div(&rp, &ze, CW), &s, CW), CW)
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    let xbig: Vec<&f::QRow> = rows
        .iter()
        .filter(|r| r.direct && r.z > 26.52 && r.z < 26.544)
        .collect();
    println!("XBIG leftover-ish n={}", xbig.len());
    for r in &xbig {
        let t = f64::from_bits(r.qbits);
        print!("  z={:.16} 5005={}", r.z, ulp_distance(qw(r.z, cephes_5005(r.z)), t).unwrap_or(99));
        for n in [1usize, 2, 3, 4, 6, 8] {
            print!(" a{n}={}", ulp_distance(qw(r.z, asym_f64(r.z, n)), t).unwrap_or(99));
        }
        println!();
    }
    println!("asymptotic as Q-tail (z>=4 / z>=8 / z>=20):");
    for (name, ev) in [
        ("a1 f64", Box::new(|z| asym_f64(z, 1)) as Box<dyn Fn(f64) -> f64>),
        ("a2 f64", Box::new(|z| asym_f64(z, 2))),
        ("a3 f64", Box::new(|z| asym_f64(z, 3))),
        ("a4 f64", Box::new(|z| asym_f64(z, 4))),
        ("a6 f64", Box::new(|z| asym_f64(z, 6))),
        ("a3 x87", Box::new(|z| asym_80(z, 3))),
        ("a6 x87", Box::new(|z| asym_80(z, 6))),
        ("a3 lead80", Box::new(|z| asym_lead80(z, 3))),
        ("RPINV/z", Box::new(|z| RPINV / z)),
        ("0x5005", Box::new(cephes_5005)),
    ] {
        let mut b4 = 0usize;
        let mut b8 = 0usize;
        let mut b20 = 0usize;
        let mut n4 = 0usize;
        let mut n8 = 0usize;
        let mut n20 = 0usize;
        let mut xhit = 0usize;
        for r in &rows {
            if r.z < 4.0 {
                continue;
            }
            let d = ulp_distance(qw(r.z, ev(r.z)), f64::from_bits(r.qbits)).unwrap_or(99);
            n4 += 1;
            if d == 0 {
                b4 += 1;
            }
            if r.z >= 8.0 {
                n8 += 1;
                if d == 0 {
                    b8 += 1;
                }
            }
            if r.z >= 20.0 {
                n20 += 1;
                if d == 0 {
                    b20 += 1;
                }
            }
            if r.direct && r.z > 26.52 && r.z < 26.544 && d == 0 {
                xhit += 1;
            }
        }
        println!("  {name:10} z>=4 {b4}/{n4}  z>=8 {b8}/{n8}  z>=20 {b20}/{n20}  xhit={xhit}");
    }
    println!("0x5005 then asymptotic cut (bar 1572):");
    for n in [1usize, 2, 3, 4, 6] {
        let mut best = 0usize;
        let mut best_c = 0.0;
        let mut best_x = 0usize;
        for k in 0..40 {
            let c = 8.0 + k as f64 * 0.5;
            let mut ex = 0usize;
            let mut xh = 0usize;
            for r in &rows {
                if r.z < 4.0 {
                    continue;
                }
                let ff = if r.z < c {
                    cephes_5005(r.z)
                } else {
                    asym_f64(r.z, n)
                };
                let d = ulp_distance(qw(r.z, ff), f64::from_bits(r.qbits)).unwrap_or(99);
                if d == 0 {
                    ex += 1;
                }
                if r.direct && r.z > 26.52 && r.z < 26.544 && d == 0 {
                    xh += 1;
                }
            }
            if ex > best || (ex == best && xh > best_x) {
                best = ex;
                best_c = c;
                best_x = xh;
            }
        }
        println!("  a{n} best Q {best} @ {best_c:.1} xhit={best_x}");
    }
}
