//! Cephes P/Q+R/S cut=8, f64 vs 80-bit decimals, mask0 and 0x5005. Q-tail. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_sub, ext_to_f64, Ext80, CW_PC64_RN};
use std::env;

const CW: u16 = CW_PC64_RN;
const ULP_CAP: u64 = 1 << 20;
const P: [f64; 9] = [
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
const Q: [f64; 8] = [
    1.32281951154744992508e1,
    8.67072140885989742329e1,
    3.54937778887819891062e2,
    9.75708501743205489753e2,
    1.82390916687909736289e3,
    2.24633760818710981792e3,
    1.65666309194161350182e3,
    5.57535340817727675546e2,
];
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
const P80: [Ext80; 9] = [
    Ext80([0x4d, 0xc6, 0x23, 0x7b, 0x12, 0x25, 0x59, 0x87, 0xdf, 0x3f]),
    Ext80([0x78, 0x3d, 0xa4, 0xb1, 0x31, 0xba, 0x6e, 0x90, 0xfe, 0x3f]),
    Ext80([0xd7, 0x21, 0x6e, 0x2b, 0xf6, 0x9e, 0xd2, 0xee, 0x01, 0x40]),
    Ext80([0x5f, 0x2f, 0xd3, 0x6d, 0x65, 0x7d, 0x8c, 0xc2, 0x04, 0x40]),
    Ext80([0x59, 0xb7, 0x07, 0x01, 0x4f, 0x55, 0x85, 0xc4, 0x06, 0x40]),
    Ext80([0x88, 0xc2, 0x7a, 0x26, 0x13, 0x7e, 0x9c, 0x83, 0x08, 0x40]),
    Ext80([0x1e, 0xbd, 0xa6, 0xa1, 0x63, 0xd3, 0xa1, 0xe9, 0x08, 0x40]),
    Ext80([0xb8, 0x4d, 0xb9, 0xb4, 0x0e, 0xa9, 0x71, 0x80, 0x09, 0x40]),
    Ext80([0x89, 0x94, 0xfd, 0x47, 0xef, 0x42, 0x62, 0x8b, 0x08, 0x40]),
];
const Q80: [Ext80; 8] = [
    Ext80([0x22, 0x62, 0x1e, 0xe1, 0xeb, 0xaf, 0xa6, 0xd3, 0x02, 0x40]),
    Ext80([0x2a, 0x28, 0x9b, 0x0b, 0xf7, 0x17, 0x6a, 0xad, 0x05, 0x40]),
    Ext80([0x5c, 0x68, 0x08, 0x7b, 0x23, 0x09, 0x78, 0xb1, 0x07, 0x40]),
    Ext80([0x1a, 0x80, 0x0e, 0xb2, 0x17, 0x58, 0xed, 0xf3, 0x08, 0x40]),
    Ext80([0xc1, 0x8b, 0x8a, 0x23, 0xe5, 0x17, 0xfd, 0xe3, 0x09, 0x40]),
    Ext80([0xed, 0xf9, 0xa7, 0xd7, 0xd7, 0x66, 0x65, 0x8c, 0x0a, 0x40]),
    Ext80([0x83, 0x13, 0x6f, 0x97, 0x0c, 0x38, 0x15, 0xcf, 0x09, 0x40]),
    Ext80([0x41, 0xac, 0x16, 0x22, 0x06, 0x43, 0x62, 0x8b, 0x08, 0x40]),
];
const R80: [Ext80; 6] = [
    Ext80([0x26, 0x11, 0xdb, 0x14, 0x82, 0xba, 0x6e, 0x90, 0xfe, 0x3f]),
    Ext80([0xea, 0x31, 0x2d, 0xc6, 0x5d, 0x37, 0x3f, 0xa3, 0xff, 0x3f]),
    Ext80([0xfa, 0xcc, 0x38, 0xb5, 0xa1, 0x0f, 0x9c, 0xa0, 0x01, 0x40]),
    Ext80([0x8d, 0x5e, 0x60, 0xe8, 0xc6, 0x72, 0x20, 0xc5, 0x01, 0x40]),
    Ext80([0x14, 0x6f, 0xb0, 0xc4, 0xb8, 0x9c, 0x1c, 0xed, 0x01, 0x40]),
    Ext80([0x1f, 0x3b, 0x1b, 0x38, 0x52, 0xc0, 0xa5, 0xbe, 0x00, 0x40]),
];
const CC80: [Ext80; 9] = [
    Ext80([0xb2, 0xf4, 0x46, 0xe2, 0xea, 0x93, 0xa8, 0x93, 0xf1, 0xbf]),
    Ext80([0x55, 0x8d, 0x98, 0x3a, 0x46, 0x4c, 0xe4, 0xfc, 0xf6, 0xbf]),
    Ext80([0x04, 0x52, 0xca, 0xe3, 0xc3, 0x2d, 0xdd, 0x9d, 0xfb, 0xbf]),
    Ext80([0x17, 0x22, 0x41, 0x6a, 0x08, 0x9c, 0xde, 0xab, 0xfe, 0xbf]),
    Ext80([0xb9, 0x18, 0xd1, 0xf3, 0xe4, 0x5b, 0x3f, 0x85, 0x00, 0xc0]),
    Ext80([0x13, 0x3f, 0xa0, 0xf6, 0x39, 0x74, 0x86, 0xb9, 0x00, 0x40]),
    Ext80([0x02, 0xc4, 0xec, 0xc9, 0xbd, 0xf3, 0xf5, 0xaf, 0x03, 0x40]),
    Ext80([0xfe, 0xd6, 0x43, 0x3e, 0xf3, 0x58, 0x0d, 0xba, 0x00, 0x40]),
    Ext80([0x95, 0xeb, 0x38, 0x59, 0x3e, 0xfa, 0xa5, 0xbe, 0x04, 0xc0]),
];
const DD80: [Ext80; 10] = [
    Ext80([0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0xff, 0x3f]),
    Ext80([0xd9, 0xa0, 0xb0, 0xfc, 0x3e, 0xda, 0x8d, 0xd1, 0x04, 0x40]),
    Ext80([0x32, 0xbf, 0x7d, 0x43, 0x28, 0xcc, 0x2b, 0xf1, 0x08, 0x40]),
    Ext80([0x36, 0x05, 0xa4, 0x2b, 0xe2, 0x38, 0xf9, 0xda, 0x0b, 0x40]),
    Ext80([0x37, 0x1f, 0x29, 0x45, 0x13, 0x8c, 0x0d, 0x85, 0x0c, 0x40]),
    Ext80([0xe6, 0x74, 0x64, 0xe0, 0x37, 0x01, 0xc6, 0xc3, 0x0f, 0xc0]),
    Ext80([0xd7, 0x81, 0x45, 0x55, 0x33, 0x40, 0xaf, 0xc9, 0x10, 0xc0]),
    Ext80([0x8e, 0x2d, 0x37, 0x3f, 0xb0, 0xc7, 0x0b, 0x8b, 0x12, 0x40]),
    Ext80([0x1f, 0xf2, 0x5e, 0x52, 0xfd, 0xf3, 0xe1, 0xa0, 0x12, 0x40]),
    Ext80([0xed, 0xe4, 0xaa, 0x95, 0x8b, 0x11, 0x9f, 0x91, 0x13, 0xc0]),
];
const E80: [Ext80; 4] = [
    Ext80([0x02, 0x82, 0xe6, 0x54, 0x0c, 0xe7, 0x5b, 0x8a, 0xfe, 0x3f]),
    Ext80([0x0f, 0x24, 0x41, 0xbc, 0xb4, 0xc7, 0x3b, 0xd6, 0xf9, 0xbf]),
    Ext80([0x53, 0x15, 0x16, 0xbb, 0xcd, 0x96, 0x1e, 0xbd, 0xf6, 0xbf]),
    Ext80([0xdb, 0xde, 0x16, 0x94, 0x1c, 0x4e, 0xc4, 0x8a, 0xf4, 0xbf]),
];
const S80: [Ext80; 6] = [
    Ext80([0xb2, 0x0a, 0x71, 0xbe, 0x48, 0x80, 0xac, 0x90, 0x00, 0x40]),
    Ext80([0x12, 0xe6, 0x67, 0xc2, 0x0e, 0x29, 0x56, 0x96, 0x02, 0x40]),
    Ext80([0xae, 0x68, 0x62, 0x29, 0xf8, 0x83, 0xc8, 0xc0, 0x02, 0x40]),
    Ext80([0xd5, 0xa5, 0xff, 0xe3, 0xac, 0xcc, 0xa6, 0x88, 0x03, 0x40]),
    Ext80([0x3f, 0x09, 0x36, 0x19, 0x53, 0x55, 0xbe, 0x99, 0x02, 0x40]),
    Ext80([0x21, 0x81, 0x6c, 0xb1, 0xd5, 0xf2, 0x9e, 0xd7, 0x00, 0x40]),
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
fn polevl_f(x: Ext80, c: &[f64], mask: u32, mut bit: u32) -> (Ext80, u32) {
    let mut a = ef(c[0]);
    for &k in &c[1..] {
        a = ext_add(&ext_mul(&a, &x, CW), &ef(k), CW);
        a = maybe(a, mask, bit);
        bit += 1;
    }
    (a, bit)
}
fn p1evl_f(x: Ext80, c: &[f64], mask: u32, mut bit: u32) -> (Ext80, u32) {
    let mut a = ext_add(&x, &ef(c[0]), CW);
    a = maybe(a, mask, bit);
    bit += 1;
    for &k in &c[1..] {
        a = ext_add(&ext_mul(&a, &x, CW), &ef(k), CW);
        a = maybe(a, mask, bit);
        bit += 1;
    }
    (a, bit)
}
fn polevl_80(x: Ext80, c: &[Ext80], mask: u32, mut bit: u32) -> (Ext80, u32) {
    let mut a = c[0];
    for k in &c[1..] {
        a = ext_add(&ext_mul(&a, &x, CW), k, CW);
        a = maybe(a, mask, bit);
        bit += 1;
    }
    (a, bit)
}
fn p1evl_80(x: Ext80, c: &[Ext80], mask: u32, mut bit: u32) -> (Ext80, u32) {
    let mut a = ext_add(&x, &c[0], CW);
    a = maybe(a, mask, bit);
    bit += 1;
    for k in &c[1..] {
        a = ext_add(&ext_mul(&a, &x, CW), k, CW);
        a = maybe(a, mask, bit);
        bit += 1;
    }
    (a, bit)
}
fn cephes_f64(z: f64, mask: u32) -> f64 {
    let ax = z.abs();
    let xe = ef(ax);
    let (num, den, b) = if ax < 8.0 {
        let (p, b) = polevl_f(xe, &P, mask, 0);
        let (q, b) = p1evl_f(xe, &Q, mask, b);
        (p, q, b)
    } else {
        let (r, b) = polevl_f(xe, &R, mask, 0);
        let (s, b) = p1evl_f(xe, &S, mask, b);
        (r, s, b)
    };
    let mut q = ext_div(&num, &den, CW);
    q = maybe(q, mask, b);
    ext_to_f64(&q, CW)
}
fn cephes_80(z: f64, mask: u32) -> f64 {
    let ax = z.abs();
    let xe = ef(ax);
    let (num, den, b) = if ax < 8.0 {
        let (p, b) = polevl_80(xe, &P80, mask, 0);
        let (q, b) = p1evl_80(xe, &Q80, mask, b);
        (p, q, b)
    } else {
        let (r, b) = polevl_80(xe, &R80, mask, 0);
        let (s, b) = p1evl_80(xe, &S80, mask, b);
        (r, s, b)
    };
    let mut q = ext_div(&num, &den, CW);
    q = maybe(q, mask, b);
    ext_to_f64(&q, CW)
}
fn horner_80(cs: &[Ext80], x: Ext80) -> Ext80 {
    let mut acc = ef(0.0);
    for c in cs.iter().rev() {
        acc = ext_add(&ext_mul(&acc, &x, CW), c, CW);
    }
    acc
}
fn ccdd_80(x: f64) -> f64 {
    let xe = ef(x.abs());
    let z = ext_div(&ef(1.0), &ext_add(&ef(2.5), &ext_mul(&xe, &xe, CW), CW), CW);
    let t = ext_sub(&ext_mul(&ef(13.0), &z, CW), &ef(1.0), CW);
    let n = horner_80(&CC80, z);
    let d = horner_80(&DD80, z);
    let mut acc = ext_div(&n, &d, CW);
    acc = ext_add(&ext_mul(&acc, &t, CW), &E80[3], CW);
    acc = ext_add(&ext_mul(&acc, &t, CW), &E80[2], CW);
    acc = ext_add(&ext_mul(&acc, &t, CW), &E80[1], CW);
    acc = ext_add(&ext_mul(&acc, &t, CW), &E80[0], CW);
    ext_to_f64(&ext_div(&acc, &xe, CW), CW)
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
    let score = |ev: fn(f64, u32) -> f64, mask: u32| {
        let mut ex = 0usize;
        let mut n = 0usize;
        let mut maxu = 0u64;
        let mut dhit = 0usize;
        let mut dn = 0usize;
        for r in &rows {
            if r.z < 4.0 {
                continue;
            }
            let qg = qw(r.z, ev(r.z, mask));
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
        (ex, n, maxu, dhit, dn)
    };
    for (name, ev, mask) in [
        ("Cephes f64 mask0 cut8", cephes_f64 as fn(f64, u32) -> f64, 0u32),
        ("Cephes f64 0x5005 cut8", cephes_f64, 0x5005),
        ("Cephes 80 mask0 cut8", cephes_80, 0),
        ("Cephes 80 0x5005 cut8", cephes_80, 0x5005),
    ] {
        let (ex, n, maxu, dhit, dn) = score(ev, mask);
        println!("{name:24} tail {ex}/{n} max={maxu} d={dhit}/{dn}");
    }
    let mut both = 0usize;
    let mut only_f = 0usize;
    let mut only_80 = 0usize;
    for r in &rows {
        if r.z < 4.0 {
            continue;
        }
        let want = f64::from_bits(r.qbits);
        let df = ulp_distance(qw(r.z, cephes_f64(r.z, 0x5005)), want).unwrap_or(99);
        let d8 = ulp_distance(qw(r.z, cephes_80(r.z, 0x5005)), want).unwrap_or(99);
        match (df == 0, d8 == 0) {
            (true, true) => both += 1,
            (true, false) => only_f += 1,
            (false, true) => only_80 += 1,
            _ => {}
        }
    }
    println!(
        "overlap f64-0x5005 vs 80-0x5005 both={} only_f64={} only_80={} union={}",
        both,
        only_f,
        only_80,
        both + only_f + only_80
    );
    let mut both2 = 0usize;
    let mut only_c = 0usize;
    let mut only_n = 0usize;
    for r in &rows {
        if r.z < 4.0 {
            continue;
        }
        let want = f64::from_bits(r.qbits);
        let dc = ulp_distance(qw(r.z, cephes_f64(r.z, 0x5005)), want).unwrap_or(99);
        let dn = ulp_distance(qw(r.z, ccdd_80(r.z)), want).unwrap_or(99);
        match (dc == 0, dn == 0) {
            (true, true) => both2 += 1,
            (true, false) => only_c += 1,
            (false, true) => only_n += 1,
            _ => {}
        }
    }
    println!(
        "overlap 0x5005 vs CCDD80 both={} only_5005={} only_ccdd80={} union={}",
        both2,
        only_c,
        only_n,
        both2 + only_c + only_n
    );
    let cuts = [
        4.5, 4.7, 4.8, 4.85, 4.9, 4.95, 5.0, 5.05, 5.1, 5.2, 5.3, 5.6, 6.0, 7.0, 8.0, 8.5, 10.0,
        12.0,
    ];
    for &c in &cuts {
        let mut cn = 0usize;
        let mut nc = 0usize;
        for r in &rows {
            if r.z < 4.0 {
                continue;
            }
            let want = f64::from_bits(r.qbits);
            let g = if r.z < c {
                qw(r.z, ccdd_80(r.z))
            } else {
                qw(r.z, cephes_f64(r.z, 0x5005))
            };
            let h = if r.z < c {
                qw(r.z, cephes_f64(r.z, 0x5005))
            } else {
                qw(r.z, ccdd_80(r.z))
            };
            if ulp_distance(g, want).unwrap_or(99) == 0 {
                cn += 1;
            }
            if ulp_distance(h, want).unwrap_or(99) == 0 {
                nc += 1;
            }
        }
        println!("cut={c:.1} CCDD80-then-5005={cn} 5005-then-CCDD80={nc}");
    }
}
