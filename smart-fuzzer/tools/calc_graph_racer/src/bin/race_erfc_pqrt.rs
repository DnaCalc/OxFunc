//! PQR80 t-map association: (x-3.75)/(x+3.75) vs (4x-15)/(4x+15) vs 1-7.5/(x+3.75). Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_sub, ext_to_f64, Ext80, CW_PC64_RN};
use std::env;

const CW: u16 = CW_PC64_RN;
const ULP_CAP: u64 = 1 << 20;
const PP80: [Ext80; 8] = [
    Ext80([0x33, 0x14, 0x59, 0x30, 0x90, 0x5a, 0x14, 0xad, 0xf2, 0x3f]),
    Ext80([0xb7, 0x80, 0xa3, 0xb9, 0x2d, 0xdf, 0x3a, 0xa2, 0xf2, 0x3f]),
    Ext80([0x65, 0x54, 0x59, 0x48, 0x8a, 0x0e, 0x20, 0xbc, 0xf0, 0x3f]),
    Ext80([0xd8, 0xd3, 0x28, 0x82, 0x3d, 0xf6, 0x02, 0xa5, 0xed, 0xbf]),
    Ext80([0x7e, 0x1f, 0x5d, 0xfa, 0x61, 0x4a, 0xb6, 0xe8, 0xed, 0xbf]),
    Ext80([0x4c, 0xb0, 0x42, 0x2b, 0x7b, 0x9f, 0xa1, 0x89, 0xec, 0xbf]),
    Ext80([0x66, 0x36, 0x86, 0x46, 0x30, 0xb7, 0x99, 0x9b, 0xe9, 0xbf]),
    Ext80([0x6b, 0xb3, 0x83, 0xd7, 0xa8, 0x7b, 0x5e, 0x94, 0xe5, 0xbf]),
];
const QQ80: [Ext80; 8] = [
    Ext80([0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0xff, 0x3f]),
    Ext80([0x8b, 0xc7, 0x50, 0xb6, 0xe1, 0x3d, 0x4a, 0xd0, 0xff, 0x3f]),
    Ext80([0x60, 0x0b, 0xf1, 0x85, 0x91, 0xef, 0x1f, 0x9a, 0xff, 0x3f]),
    Ext80([0x93, 0x30, 0x40, 0xd9, 0x83, 0x06, 0x25, 0x86, 0xfe, 0x3f]),
    Ext80([0x31, 0xd3, 0x69, 0x95, 0xb5, 0xeb, 0x73, 0x94, 0xfc, 0x3f]),
    Ext80([0xa8, 0x82, 0xb8, 0x74, 0x00, 0x66, 0xa7, 0xd1, 0xf9, 0x3f]),
    Ext80([0x01, 0x31, 0xc7, 0xb2, 0x8c, 0xd9, 0x16, 0xb0, 0xf6, 0x3f]),
    Ext80([0xa8, 0x91, 0xa2, 0x37, 0x89, 0xb0, 0xb7, 0x89, 0xf2, 0x3f]),
];
const RR80: [Ext80; 9] = [
    Ext80([0x0c, 0x7c, 0x49, 0x0a, 0xce, 0x78, 0x15, 0x95, 0xfc, 0x3f]),
    Ext80([0xc3, 0x7f, 0x63, 0x90, 0x9b, 0xf5, 0xfd, 0x8b, 0xfd, 0xbf]),
    Ext80([0x8b, 0xaf, 0x01, 0x28, 0xa0, 0xa8, 0x6e, 0xe7, 0xfc, 0x3f]),
    Ext80([0x09, 0x3d, 0xa9, 0x47, 0x17, 0x64, 0x7f, 0xa7, 0xfc, 0xbf]),
    Ext80([0x64, 0x73, 0xd5, 0xda, 0xa4, 0x35, 0x22, 0xd2, 0xfb, 0x3f]),
    Ext80([0x2c, 0x1c, 0x04, 0x65, 0x8e, 0x67, 0x78, 0xe0, 0xfa, 0xbf]),
    Ext80([0xbd, 0x3a, 0xd2, 0x39, 0xac, 0x02, 0xc8, 0xc5, 0xf9, 0x3f]),
    Ext80([0xad, 0xf3, 0x12, 0x9a, 0xc0, 0xc8, 0xaf, 0x86, 0xf8, 0xbf]),
    Ext80([0xfd, 0x51, 0xc8, 0xe4, 0x38, 0x61, 0x51, 0xec, 0xf5, 0x3f]),
];

fn ef(x: f64) -> Ext80 {
    ext_from_f64(x)
}
fn horner_80(cs: &[Ext80], x: Ext80) -> Ext80 {
    let mut acc = ef(0.0);
    for c in cs.iter().rev() {
        acc = ext_add(&ext_mul(&acc, &x, CW), c, CW);
    }
    acc
}
fn finish(xe: Ext80, t: Ext80) -> f64 {
    let u = horner_80(&PP80, xe);
    let v = horner_80(&QQ80, xe);
    let mut acc = ext_div(&u, &v, CW);
    for r in RR80.iter().rev() {
        acc = ext_add(&ext_mul(&acc, &t, CW), r, CW);
    }
    ext_to_f64(&acc, CW)
}
fn t_std(z: f64) -> f64 {
    let xe = ef(z.abs());
    let t = ext_div(
        &ext_sub(&xe, &ef(3.75), CW),
        &ext_add(&xe, &ef(3.75), CW),
        CW,
    );
    finish(xe, t)
}
fn t_4x(z: f64) -> f64 {
    let xe = ef(z.abs());
    let four = ef(4.0);
    let t = ext_div(
        &ext_sub(&ext_mul(&four, &xe, CW), &ef(15.0), CW),
        &ext_add(&ext_mul(&four, &xe, CW), &ef(15.0), CW),
        CW,
    );
    finish(xe, t)
}
fn t_one_minus(z: f64) -> f64 {
    let xe = ef(z.abs());
    let t = ext_sub(
        &ef(1.0),
        &ext_div(&ef(7.5), &ext_add(&xe, &ef(3.75), CW), CW),
        CW,
    );
    finish(xe, t)
}
fn t_15q(z: f64) -> f64 {
    let xe = ef(z.abs());
    let c = ext_div(&ef(15.0), &ef(4.0), CW);
    let t = ext_div(&ext_sub(&xe, &c, CW), &ext_add(&xe, &c, CW), CW);
    finish(xe, t)
}
fn pq_in_x2(z: f64) -> f64 {
    let xe = ef(z.abs());
    let xx = ext_mul(&xe, &xe, CW);
    let u = horner_80(&PP80, xx);
    let v = horner_80(&QQ80, xx);
    let t = ext_div(
        &ext_sub(&xe, &ef(3.75), CW),
        &ext_add(&xe, &ef(3.75), CW),
        CW,
    );
    let mut acc = ext_div(&u, &v, CW);
    for r in RR80.iter().rev() {
        acc = ext_add(&ext_mul(&acc, &t, CW), r, CW);
    }
    ext_to_f64(&acc, CW)
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
    let graphs: [(&str, fn(f64) -> f64); 5] = [
        ("t=(x-3.75)/(x+3.75)", t_std),
        ("t=(4x-15)/(4x+15)", t_4x),
        ("t=1-7.5/(x+3.75)", t_one_minus),
        ("t 15/4 x87", t_15q),
        ("P/Q Horner in x2", pq_in_x2),
    ];
    for (name, ev) in graphs {
        let mut ex = 0usize;
        let mut n = 0usize;
        let mut maxu = 0u64;
        let mut dmid = 0usize;
        let mut dn = 0usize;
        for r in &rows {
            if r.z < 0.5 || r.z >= 4.0 {
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
                    dmid += 1;
                }
            } else {
                maxu = maxu.max(d);
            }
        }
        println!("{name:24} {ex}/{n} max={maxu} dmid={dmid}/{dn}");
    }
    fn maybe(x: Ext80, mask: u32, bit: u32) -> Ext80 {
        if bit < 32 && mask & (1u32 << bit) != 0 {
            ef(ext_to_f64(&x, CW))
        } else {
            x
        }
    }
    fn pqr80_m(z: f64, mask: u32) -> f64 {
        let xe = ef(z.abs());
        let mut t = ext_div(
            &ext_sub(&xe, &ef(3.75), CW),
            &ext_add(&xe, &ef(3.75), CW),
            CW,
        );
        t = maybe(t, mask, 0);
        let mut u = ef(0.0);
        let mut bit = 1u32;
        for c in PP80.iter().rev() {
            u = ext_add(&ext_mul(&u, &xe, CW), c, CW);
            u = maybe(u, mask, bit);
            bit += 1;
        }
        let mut v = ef(0.0);
        for c in QQ80.iter().rev() {
            v = ext_add(&ext_mul(&v, &xe, CW), c, CW);
            v = maybe(v, mask, bit);
            bit += 1;
        }
        let mut acc = ext_div(&u, &v, CW);
        acc = maybe(acc, mask, bit);
        bit += 1;
        for r in RR80.iter().rev() {
            acc = ext_add(&ext_mul(&acc, &t, CW), r, CW);
            acc = maybe(acc, mask, bit);
            bit += 1;
        }
        ext_to_f64(&acc, CW)
    }
    let score_m = |mask: u32| {
        let mut ex = 0usize;
        let mut n = 0usize;
        for r in &rows {
            if r.z < 0.5 || r.z >= 4.0 {
                continue;
            }
            let qg = qw(r.z, pqr80_m(r.z, mask));
            if !qg.is_finite() {
                continue;
            }
            let d = ulp_distance(qg, f64::from_bits(r.qbits)).unwrap_or(u64::MAX);
            if d > ULP_CAP {
                continue;
            }
            n += 1;
            if d == 0 {
                ex += 1;
            }
        }
        (ex, n)
    };
    let base = score_m(0);
    println!("PQR80 HW0 {}/{}", base.0, base.1);
    let mut best = base.0;
    let mut best_b = None;
    for b in 0..28u32 {
        let sc = score_m(1u32 << b);
        if sc.0 != base.0 {
            println!("  bit{b} {}/{} dlt={}", sc.0, sc.1, sc.0 as i32 - base.0 as i32);
        }
        if sc.0 > best {
            best = sc.0;
            best_b = Some(b);
        }
    }
    println!("PQR80 HW1 best {best} bit={best_b:?} (bars 3386/3432)");
}
