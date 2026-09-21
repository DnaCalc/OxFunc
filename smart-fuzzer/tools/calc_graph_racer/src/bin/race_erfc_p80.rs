//! SPECFUN/Cody P/Q (y≥4) 80-bit printed decimals as Q=w*F tail. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_sqrt, ext_sub, ext_to_f64, Ext80, CW_PC64_RN};
use std::env;

const CW: u16 = CW_PC64_RN;
const ULP_CAP: u64 = 1 << 20;
const P: [f64; 6] = [
    0.305326634961232344,
    0.360344899949804439,
    0.125781726111229246,
    0.0160837851487422766,
    6.58749161529837803e-4,
    0.0163153871373020978,
];
const Q: [f64; 5] = [
    2.56852019228982242,
    1.87295284992346047,
    0.527905102951428412,
    0.0605183413124413191,
    0.00233520497626869185,
];
const SQRPI: f64 = 0.56418958354775628695;
const P80: [Ext80; 6] = [
    Ext80([0x1b, 0xae, 0x2f, 0x83, 0xcf, 0xc5, 0x53, 0x9c, 0xfd, 0x3f]),
    Ext80([0xe5, 0xbd, 0x2d, 0x21, 0x71, 0x20, 0x7f, 0xb8, 0xfd, 0x3f]),
    Ext80([0x6e, 0x74, 0x21, 0x54, 0xc0, 0xec, 0xcc, 0x80, 0xfc, 0x3f]),
    Ext80([0x41, 0xab, 0x2d, 0xb6, 0x66, 0x24, 0xc2, 0x83, 0xf9, 0x3f]),
    Ext80([0x8d, 0xde, 0x19, 0x8f, 0x6b, 0xe8, 0xaf, 0xac, 0xf4, 0x3f]),
    Ext80([0xe4, 0x38, 0x2e, 0xa4, 0xc5, 0xd8, 0xa7, 0x85, 0xf9, 0x3f]),
];
const Q80: [Ext80; 5] = [
    Ext80([0xef, 0xdb, 0x06, 0x40, 0x84, 0xa2, 0x62, 0xa4, 0x00, 0x40]),
    Ext80([0xb9, 0x68, 0x85, 0xaf, 0x42, 0xeb, 0xbc, 0xef, 0xff, 0x3f]),
    Ext80([0x31, 0xc5, 0x61, 0x91, 0xf0, 0xc9, 0x24, 0x87, 0xfe, 0x3f]),
    Ext80([0xf2, 0x37, 0xef, 0xeb, 0x8b, 0x14, 0xe2, 0xf7, 0xfa, 0x3f]),
    Ext80([0x81, 0x89, 0xd6, 0xa5, 0x00, 0x3d, 0x0a, 0x99, 0xf6, 0x3f]),
];
const SQRPI80: Ext80 = Ext80([0x8d, 0x68, 0xdb, 0x14, 0x82, 0xba, 0x6e, 0x90, 0xfe, 0x3f]);

fn ef(x: f64) -> Ext80 {
    ext_from_f64(x)
}
fn pq_f64(y: f64) -> f64 {
    let ye = ef(y.abs());
    let ysq = ext_div(&ef(1.0), &ext_mul(&ye, &ye, CW), CW);
    let mut xnum = ext_mul(&ef(P[5]), &ysq, CW);
    let mut xden = ysq;
    for i in 0..4 {
        xnum = ext_mul(&ext_add(&xnum, &ef(P[i]), CW), &ysq, CW);
        xden = ext_mul(&ext_add(&xden, &ef(Q[i]), CW), &ysq, CW);
    }
    let r = ext_div(
        &ext_mul(&ysq, &ext_add(&xnum, &ef(P[4]), CW), CW),
        &ext_add(&xden, &ef(Q[4]), CW),
        CW,
    );
    ext_to_f64(&ext_div(&ext_sub(&ef(SQRPI), &r, CW), &ye, CW), CW)
}
fn pq_x87_sqrpi(y: f64) -> f64 {
    let ye = ef(y.abs());
    let ysq = ext_div(&ef(1.0), &ext_mul(&ye, &ye, CW), CW);
    let mut xnum = ext_mul(&ef(P[5]), &ysq, CW);
    let mut xden = ysq;
    for i in 0..4 {
        xnum = ext_mul(&ext_add(&xnum, &ef(P[i]), CW), &ysq, CW);
        xden = ext_mul(&ext_add(&xden, &ef(Q[i]), CW), &ysq, CW);
    }
    let r = ext_div(
        &ext_mul(&ysq, &ext_add(&xnum, &ef(P[4]), CW), CW),
        &ext_add(&xden, &ef(Q[4]), CW),
        CW,
    );
    let s = ext_div(&ef(1.0), &ext_sqrt(&ef(std::f64::consts::PI), CW), CW);
    ext_to_f64(&ext_div(&ext_sub(&s, &r, CW), &ye, CW), CW)
}
fn pq_80_x87_sqrpi(y: f64) -> f64 {
    let ye = ef(y.abs());
    let ysq = ext_div(&ef(1.0), &ext_mul(&ye, &ye, CW), CW);
    let mut xnum = ext_mul(&P80[5], &ysq, CW);
    let mut xden = ysq;
    for i in 0..4 {
        xnum = ext_mul(&ext_add(&xnum, &P80[i], CW), &ysq, CW);
        xden = ext_mul(&ext_add(&xden, &Q80[i], CW), &ysq, CW);
    }
    let r = ext_div(
        &ext_mul(&ysq, &ext_add(&xnum, &P80[4], CW), CW),
        &ext_add(&xden, &Q80[4], CW),
        CW,
    );
    let s = ext_div(&ef(1.0), &ext_sqrt(&ef(std::f64::consts::PI), CW), CW);
    ext_to_f64(&ext_div(&ext_sub(&s, &r, CW), &ye, CW), CW)
}
fn pq_80(y: f64) -> f64 {
    let ye = ef(y.abs());
    let ysq = ext_div(&ef(1.0), &ext_mul(&ye, &ye, CW), CW);
    let mut xnum = ext_mul(&P80[5], &ysq, CW);
    let mut xden = ysq;
    for i in 0..4 {
        xnum = ext_mul(&ext_add(&xnum, &P80[i], CW), &ysq, CW);
        xden = ext_mul(&ext_add(&xden, &Q80[i], CW), &ysq, CW);
    }
    let r = ext_div(
        &ext_mul(&ysq, &ext_add(&xnum, &P80[4], CW), CW),
        &ext_add(&xden, &Q80[4], CW),
        CW,
    );
    ext_to_f64(&ext_div(&ext_sub(&SQRPI80, &r, CW), &ye, CW), CW)
}
fn qw(z: f64, ff: f64) -> f64 {
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
    for (name, ev) in [
        ("P/Q f64-wide", pq_f64 as fn(f64) -> f64),
        ("P/Q 80-bit decimals", pq_80),
        ("P/Q x87 1/sqrt(pi) SQRPI", pq_x87_sqrpi),
        ("P/Q 80-bit + x87 SQRPI", pq_80_x87_sqrpi),
    ] {
        let mut ex = 0usize;
        let mut n = 0usize;
        let mut maxu = 0u64;
        let mut dhit = 0usize;
        let mut dn = 0usize;
        for r in &rows {
            if r.z < 4.0 {
                continue;
            }
            let qg = qw(r.z, ev(r.z));
            if !qg.is_finite() {
                continue;
            }
            let d = ulp_distance(qg, f64::from_bits(r.qbits)).unwrap_or(u64::MAX);
            if d > ULP_CAP {
                continue;
            }
            n += 1;
            if r.direct {
                dn += 1;
            }
            if d == 0 {
                ex += 1;
                if r.direct {
                    dhit += 1;
                }
            } else {
                maxu = maxu.max(d);
            }
        }
        println!("{name:22} tail {ex}/{n} max={maxu} d={dhit}/{dn}");
    }
    println!("P/Q as F_or tail (mixed bar 1370):");
    for (name, ev) in [
        ("P/Q f64-wide", pq_f64 as fn(f64) -> f64),
        ("P/Q 80-bit", pq_80),
        ("P/Q x87 SQRPI", pq_x87_sqrpi),
        ("P/Q 80-bit + x87 SQRPI", pq_80_x87_sqrpi),
    ] {
        let mut ex = 0usize;
        let mut n = 0usize;
        let mut td = 0usize;
        for r in &rows {
            if r.z < 4.0 {
                continue;
            }
            let Some(fo) = f::f_or(r.z, r.qbits) else {
                continue;
            };
            let fg = ev(r.z);
            if !fg.is_finite() {
                continue;
            }
            let d = ulp_distance(fg, fo).unwrap_or(u64::MAX);
            if d > ULP_CAP {
                continue;
            }
            n += 1;
            if d == 0 {
                ex += 1;
                if r.direct {
                    td += 1;
                }
            }
        }
        println!("{name:24} F_or tail {ex}/{n} d={td}");
    }
}
