//! Cephes P/Q and NSWC CCDD with 80-bit RN of printed decimals. Q=w*F. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_sub, ext_to_f64, Ext80, CW_PC64_RN};
use std::env;

const CW: u16 = CW_PC64_RN;
const ULP_CAP: u64 = 1 << 20;
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
const CC: [f64; 9] = [
    -0.7040906288250128001000086e-04,
    -0.3858822461760510359506941e-02,
    -0.7708202127512212359395078e-01,
    -0.6713655014557429480440263e+00,
    -0.2081992124162995545731882e+01,
    0.2898831421475282558867888e+01,
    0.2199509380600429331650192e+02,
    0.2907064664404115316722996e+01,
    -0.4766208741588182425380950e+02,
];
const DD: [f64; 10] = [
    1.0,
    0.5238852785508439144747174e+02,
    0.9646843357714742409535148e+03,
    0.7007152775135939601804416e+04,
    0.8515386792259821780601162e+04,
    -0.1002360095177164564992134e+06,
    -0.2065250031331232815791912e+06,
    0.5695324805290370358175984e+06,
    0.6589752493461331195697873e+06,
    -0.1192930193156561957631462e+07,
];
const E: [f64; 4] = [
    0.540464821348814822409610122136,
    -0.261515522487415653487049835220e-01,
    -0.288573438386338758794591212600e-02,
    -0.529353396945788057720258856000e-03,
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

fn ef(x: f64) -> Ext80 {
    ext_from_f64(x)
}
fn polevl_f(x: Ext80, c: &[f64]) -> Ext80 {
    let mut a = ef(c[0]);
    for &k in &c[1..] {
        a = ext_add(&ext_mul(&a, &x, CW), &ef(k), CW);
    }
    a
}
fn p1evl_f(x: Ext80, c: &[f64]) -> Ext80 {
    let mut a = ext_add(&x, &ef(c[0]), CW);
    for &k in &c[1..] {
        a = ext_add(&ext_mul(&a, &x, CW), &ef(k), CW);
    }
    a
}
fn polevl_80(x: Ext80, c: &[Ext80]) -> Ext80 {
    let mut a = c[0];
    for k in &c[1..] {
        a = ext_add(&ext_mul(&a, &x, CW), k, CW);
    }
    a
}
fn p1evl_80(x: Ext80, c: &[Ext80]) -> Ext80 {
    let mut a = ext_add(&x, &c[0], CW);
    for k in &c[1..] {
        a = ext_add(&ext_mul(&a, &x, CW), k, CW);
    }
    a
}
fn cephes_f64(z: f64) -> f64 {
    let xe = ef(z.abs());
    ext_to_f64(&ext_div(&polevl_f(xe, &CEPHES_P), &p1evl_f(xe, &CEPHES_Q), CW), CW)
}
fn cephes_80(z: f64) -> f64 {
    let xe = ef(z.abs());
    ext_to_f64(&ext_div(&polevl_80(xe, &P80), &p1evl_80(xe, &Q80), CW), CW)
}
fn horner_f(cs: &[f64], x: Ext80) -> Ext80 {
    let mut acc = ef(0.0);
    for &c in cs.iter().rev() {
        acc = ext_add(&ext_mul(&acc, &x, CW), &ef(c), CW);
    }
    acc
}
fn horner_80(cs: &[Ext80], x: Ext80) -> Ext80 {
    let mut acc = ef(0.0);
    for c in cs.iter().rev() {
        acc = ext_add(&ext_mul(&acc, &x, CW), c, CW);
    }
    acc
}
fn ccdd_f64(x: f64) -> f64 {
    let xe = ef(x.abs());
    let z = ext_div(&ef(1.0), &ext_add(&ef(2.5), &ext_mul(&xe, &xe, CW), CW), CW);
    let t = ext_sub(&ext_mul(&ef(13.0), &z, CW), &ef(1.0), CW);
    let n = horner_f(&CC, z);
    let d = horner_f(&DD, z);
    let mut acc = ext_div(&n, &d, CW);
    acc = ext_add(&ext_mul(&acc, &t, CW), &ef(E[3]), CW);
    acc = ext_add(&ext_mul(&acc, &t, CW), &ef(E[2]), CW);
    acc = ext_add(&ext_mul(&acc, &t, CW), &ef(E[1]), CW);
    acc = ext_add(&ext_mul(&acc, &t, CW), &ef(E[0]), CW);
    ext_to_f64(&ext_div(&acc, &xe, CW), CW)
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
fn maybe(x: Ext80, mask: u32, bit: u32) -> Ext80 {
    if bit < 32 && mask & (1u32 << bit) != 0 {
        ef(ext_to_f64(&x, CW))
    } else {
        x
    }
}
fn horner_80_m(cs: &[Ext80], x: Ext80, mask: u32, mut bit: u32) -> (Ext80, u32) {
    let mut acc = ef(0.0);
    for c in cs.iter().rev() {
        acc = ext_add(&ext_mul(&acc, &x, CW), c, CW);
        acc = maybe(acc, mask, bit);
        bit += 1;
    }
    (acc, bit)
}
fn ccdd_80_m(x: f64, mask: u32) -> f64 {
    let xe = ef(x.abs());
    let mut z = ext_div(&ef(1.0), &ext_add(&ef(2.5), &ext_mul(&xe, &xe, CW), CW), CW);
    z = maybe(z, mask, 0);
    let mut t = ext_sub(&ext_mul(&ef(13.0), &z, CW), &ef(1.0), CW);
    t = maybe(t, mask, 1);
    let (n, b) = horner_80_m(&CC80, z, mask, 2);
    let (d, b) = horner_80_m(&DD80, z, mask, b);
    let mut acc = ext_div(&n, &d, CW);
    acc = maybe(acc, mask, b);
    let mut bit = b + 1;
    acc = ext_add(&ext_mul(&acc, &t, CW), &E80[3], CW);
    acc = maybe(acc, mask, bit);
    bit += 1;
    acc = ext_add(&ext_mul(&acc, &t, CW), &E80[2], CW);
    acc = maybe(acc, mask, bit);
    bit += 1;
    acc = ext_add(&ext_mul(&acc, &t, CW), &E80[1], CW);
    acc = maybe(acc, mask, bit);
    bit += 1;
    acc = ext_add(&ext_mul(&acc, &t, CW), &E80[0], CW);
    acc = maybe(acc, mask, bit);
    bit += 1;
    acc = ext_div(&acc, &xe, CW);
    acc = maybe(acc, mask, bit);
    ext_to_f64(&acc, CW)
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    let graphs: [(&str, fn(f64) -> f64); 4] = [
        ("Cephes P/Q f64-wide", cephes_f64),
        ("Cephes P/Q 80-bit", cephes_80),
        ("NSWC CCDD f64-wide", ccdd_f64),
        ("NSWC CCDD 80-bit", ccdd_80),
    ];
    for (name, ev) in graphs {
        let mut mex = 0usize;
        let mut mn = 0usize;
        let mut mmax = 0u64;
        let mut md = 0usize;
        let mut mdn = 0usize;
        let mut tex = 0usize;
        let mut tn = 0usize;
        let mut tmax = 0u64;
        let mut td = 0usize;
        let mut tdn = 0usize;
        for r in &rows {
            if r.z < 0.5 {
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
            if r.z < 4.0 {
                mn += 1;
                if r.direct {
                    mdn += 1;
                }
                if d == 0 {
                    mex += 1;
                    if r.direct {
                        md += 1;
                    }
                } else {
                    mmax = mmax.max(d);
                }
            } else {
                tn += 1;
                if r.direct {
                    tdn += 1;
                }
                if d == 0 {
                    tex += 1;
                    if r.direct {
                        td += 1;
                    }
                } else {
                    tmax = tmax.max(d);
                }
            }
        }
        println!(
            "{name:22} mid {mex}/{mn} max={mmax} d={md}/{mdn}  tail {tex}/{tn} max={tmax} d={td}/{tdn}"
        );
    }
    let score_tail = |mask: u32| {
        let mut ex = 0usize;
        let mut n = 0usize;
        let mut maxu = 0u64;
        let mut dhit = 0usize;
        for r in &rows {
            if r.z < 4.0 {
                continue;
            }
            let qg = qw(r.z, ccdd_80_m(r.z, mask));
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
                if r.direct {
                    dhit += 1;
                }
            } else {
                maxu = maxu.max(d);
            }
        }
        (ex, n, maxu, dhit)
    };
    let base = score_tail(0);
    println!("CCDD80 mask0 tail {}/{} max={} d={}", base.0, base.1, base.2, base.3);
    let mut best = base.0;
    let mut best_bit = None;
    for b in 0..28u32 {
        let sc = score_tail(1u32 << b);
        if sc.0 != base.0 {
            println!("  bit{b} {}/{} max={} d={}  dlt={}", sc.0, sc.1, sc.2, sc.3, sc.0 as i32 - base.0 as i32);
        }
        if sc.0 > best {
            best = sc.0;
            best_bit = Some(b);
        }
    }
    println!("CCDD80 HW1 best tail {best} bit={best_bit:?} (bar 1572)");
    println!("CCDD as F_or tail (mixed bar 1370):");
    for (name, ev) in [
        ("CCDD f64-wide", ccdd_f64 as fn(f64) -> f64),
        ("CCDD 80-bit", ccdd_80),
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
        println!("{name:16} F_or tail {ex}/{n} d={td}");
    }
}
