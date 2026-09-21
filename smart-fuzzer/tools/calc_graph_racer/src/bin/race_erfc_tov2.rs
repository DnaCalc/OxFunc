//! CCDD80-then-0x5005 vs mask0-then-0x5005 Q-tail. Not an identity.
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
fn cephes_mask(x: f64, mask: u32) -> f64 {
    let ax = x.abs();
    let xe = ef(ax);
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
    let ev5 = |z: f64| qw(z, cephes_mask(z, 0x5005));
    let ev0 = |z: f64| qw(z, cephes_mask(z, 0));
    let evn = |z: f64| qw(z, ccdd_80(z));
    let score = |pick: &dyn Fn(f64) -> f64| {
        let mut ex = 0usize;
        let mut n = 0usize;
        let mut dhit = 0usize;
        let mut dn = 0usize;
        let mut maxu = 0u64;
        for r in &rows {
            if r.z < 4.0 {
                continue;
            }
            let qg = pick(r.z);
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
    for (name, ev) in [
        ("0x5005", &ev5 as &dyn Fn(f64) -> f64),
        ("mask0", &ev0),
        ("CCDD80", &evn),
    ] {
        let (ex, n, maxu, dhit, dn) = score(ev);
        println!("{name:10} {ex}/{n} max={maxu} d={dhit}/{dn}");
    }
    let cuts = [
        4.2, 4.5, 4.7, 4.9, 5.0, 5.2, 5.5, 5.6, 6.0, 7.0, 8.0,
    ];
    println!("## one-cut");
    for &c in &cuts {
        let m0 = score(&|z| if z < c { ev0(z) } else { ev5(z) });
        let nc = score(&|z| if z < c { evn(z) } else { ev5(z) });
        println!(
            "cut={c:.1} mask0-then-5005={}/{n} d={}  CCDD80-then-5005={}/{n} d={}",
            m0.0, m0.3, nc.0, nc.3, n = m0.1
        );
    }
    let g_m = |z: f64| if z < 4.5 { ev0(z) } else { ev5(z) };
    let g_n = |z: f64| if z < 5.0 { evn(z) } else { ev5(z) };
    let mut both = 0usize;
    let mut only_m = 0usize;
    let mut only_n = 0usize;
    let mut both_d = 0usize;
    let mut only_md = 0usize;
    let mut only_nd = 0usize;
    for r in &rows {
        if r.z < 4.0 {
            continue;
        }
        let want = f64::from_bits(r.qbits);
        let dm = ulp_distance(g_m(r.z), want).unwrap_or(99);
        let dn = ulp_distance(g_n(r.z), want).unwrap_or(99);
        match (dm == 0, dn == 0) {
            (true, true) => {
                both += 1;
                if r.direct {
                    both_d += 1;
                }
            }
            (true, false) => {
                only_m += 1;
                if r.direct {
                    only_md += 1;
                }
            }
            (false, true) => {
                only_n += 1;
                if r.direct {
                    only_nd += 1;
                }
            }
            _ => {}
        }
    }
    println!(
        "best-cut overlap mask0@4.5 vs CCDD80@5.0 both={} only_m0={} only_ccdd={} union={}",
        both,
        only_m,
        only_n,
        both + only_m + only_n
    );
    println!("  DIRECT both={both_d} only_m0={only_md} only_ccdd={only_nd} union={}", both_d + only_md + only_nd);
    println!("## two-cut CCDD80 [4,c1) mask0 [c1,c2) 5005 [c2,)");
    let c1s = [4.2, 4.5, 4.9, 5.0];
    let c2s = [4.5, 4.9, 5.0, 5.6, 8.0];
    let mut best = (0usize, 0.0, 0.0);
    for &c1 in &c1s {
        for &c2 in &c2s {
            if c2 <= c1 {
                continue;
            }
            let sc = score(&|z| {
                if z < c1 {
                    evn(z)
                } else if z < c2 {
                    ev0(z)
                } else {
                    ev5(z)
                }
            });
            println!("  [{c1:.1},{c2:.1}) {ex} d={d}", ex = sc.0, d = sc.3);
            if sc.0 > best.0 {
                best = (sc.0, c1, c2);
            }
        }
    }
    println!("two-cut best {} @ [{},{}] (bars 1572/1580/1582)", best.0, best.1, best.2);
    println!("DIRECT only mask0@4.5 vs 0x5005:");
    for r in &rows {
        if r.z < 4.0 || !r.direct {
            continue;
        }
        let want = f64::from_bits(r.qbits);
        let dm = ulp_distance(g_m(r.z), want).unwrap_or(99);
        let d5 = ulp_distance(ev5(r.z), want).unwrap_or(99);
        if dm == 0 && d5 != 0 {
            println!("  only_m0 z={:.16} 5005={}", r.z, d5);
        }
        if d5 == 0 && dm != 0 {
            println!("  only_5005 z={:.16} m0={}", r.z, dm);
        }
    }
}
