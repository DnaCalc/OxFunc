//! fdlibm PP/QQ and Cephes T/U 80-bit printed decimals, P-side. Not an identity.
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{ulp_distance, WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC64_RN};
use std::collections::BTreeMap;
use std::env;
use std::fs;

const CW: u16 = CW_PC64_RN;
const ULP_CAP: u64 = 1 << 20;
const NSWC_A: [f64; 5] = [
    0.771058495001320e-04,
    -0.133733772997339e-02,
    0.323076579225834e-01,
    0.479137145607681e-01,
    0.128379167095513e+00,
];
const NSWC_B: [f64; 3] = [
    0.301048631703895e-02,
    0.538971687740286e-01,
    0.375795757275549e+00,
];
const NA80: [Ext80; 5] = [
    Ext80([0x0b, 0x92, 0xc2, 0x09, 0x43, 0xe3, 0xb3, 0xa1, 0xf1, 0x3f]),
    Ext80([0x28, 0x41, 0x15, 0xbd, 0xa0, 0x9b, 0x49, 0xaf, 0xf5, 0xbf]),
    Ext80([0x9b, 0x0d, 0x70, 0x01, 0xe3, 0x08, 0x55, 0x84, 0xfa, 0x3f]),
    Ext80([0x0a, 0x86, 0x13, 0x18, 0xd1, 0x2b, 0x41, 0xc4, 0xfa, 0x3f]),
    Ext80([0x3c, 0xbf, 0xdb, 0xa6, 0x10, 0xd4, 0x75, 0x83, 0xfc, 0x3f]),
];
const NB80: [Ext80; 3] = [
    Ext80([0x41, 0x79, 0x49, 0xd8, 0x46, 0x94, 0x4b, 0xc5, 0xf6, 0x3f]),
    Ext80([0x89, 0xef, 0xff, 0xb3, 0x13, 0x47, 0xc3, 0xdc, 0xfa, 0x3f]),
    Ext80([0xcd, 0xfa, 0xb4, 0xf2, 0x2e, 0x4d, 0x68, 0xc0, 0xfd, 0x3f]),
];
const FDLIBM_PP: [f64; 5] = [
    1.28379167095512558561e-01,
    -3.25042107247001499370e-01,
    -2.84817495755985104766e-02,
    -5.77027029648944159157e-03,
    -2.37630166566501626084e-05,
];
const FDLIBM_QQ: [f64; 5] = [
    3.97917223959155352819e-01,
    6.50222499887672944485e-02,
    5.08130628187576562776e-03,
    1.32494738004321644526e-04,
    -3.96022827877536812320e-06,
];
const AA: [f64; 5] = [
    3.16112374387056560,
    113.864154151050156,
    377.485237685302021,
    3209.37758913846947,
    0.185777706184603153,
];
const BB: [f64; 4] = [
    23.6012909523441209,
    244.024637934444173,
    1282.61652607737228,
    2844.23683343917062,
];
const AA80: [Ext80; 5] = [
    Ext80([0xe6, 0x49, 0x1f, 0xa2, 0xf6, 0xd9, 0x4f, 0xca, 0x00, 0x40]),
    Ext80([0xbc, 0x16, 0xed, 0xb2, 0x69, 0x72, 0xba, 0xe3, 0x05, 0x40]),
    Ext80([0x7e, 0x57, 0x94, 0xba, 0x44, 0x1c, 0xbe, 0xbc, 0x07, 0x40]),
    Ext80([0xac, 0xd1, 0x90, 0xe8, 0x9a, 0x0a, 0x96, 0xc8, 0x0a, 0x40]),
    Ext80([0x00, 0x10, 0x19, 0x8e, 0xd1, 0x82, 0x3c, 0xbe, 0xfc, 0x3f]),
];
const BB80: [Ext80; 4] = [
    Ext80([0xc7, 0xec, 0x96, 0x7d, 0xa1, 0x71, 0xcf, 0xbc, 0x03, 0x40]),
    Ext80([0x44, 0xc0, 0xad, 0xf2, 0xab, 0x4e, 0x06, 0xf4, 0x06, 0x40]),
    Ext80([0x54, 0x3e, 0x6e, 0xe5, 0x94, 0xba, 0x53, 0xa0, 0x09, 0x40]),
    Ext80([0x6d, 0x64, 0x3d, 0xdc, 0x11, 0xca, 0xc3, 0xb1, 0x0a, 0x40]),
];
const CEPHES_T: [f64; 5] = [
    9.60497373987051638749e0,
    9.00260197203842689217e1,
    2.23200534594684319226e3,
    7.00332514112805075473e3,
    5.55923013010394962768e4,
];
const CEPHES_U: [f64; 5] = [
    3.35617141647503099647e1,
    5.21357949780152679795e2,
    4.59432382970980127987e3,
    2.26290000613890934246e4,
    4.92673942608635921086e4,
];
const PP80: [Ext80; 5] = [
    Ext80([0x00, 0x40, 0xdb, 0xa6, 0x10, 0xd4, 0x75, 0x83, 0xfc, 0x3f]),
    Ext80([0x00, 0x98, 0xc8, 0xe5, 0x48, 0xeb, 0x6b, 0xa6, 0xfd, 0xbf]),
    Ext80([0x00, 0x78, 0xca, 0xb8, 0xde, 0x8e, 0x52, 0xe9, 0xf9, 0xbf]),
    Ext80([0x00, 0x20, 0x47, 0x33, 0x1b, 0x89, 0x14, 0xbd, 0xf7, 0xbf]),
    Ext80([0x00, 0x60, 0xb5, 0x00, 0x90, 0xb0, 0x56, 0xc7, 0xef, 0xbf]),
];
const QQ80: [Ext80; 5] = [
    Ext80([0x00, 0x48, 0xe0, 0xd6, 0x6e, 0xce, 0xbb, 0xcb, 0xfd, 0x3f]),
    Ext80([0x00, 0xd0, 0x75, 0xb6, 0xa9, 0x62, 0x2a, 0x85, 0xfb, 0x3f]),
    Ext80([0x00, 0x78, 0x58, 0x9b, 0x26, 0x16, 0x81, 0xa6, 0xf7, 0x3f]),
    Ext80([0x00, 0x80, 0xd0, 0xe0, 0x10, 0x49, 0xee, 0x8a, 0xf2, 0x3f]),
    Ext80([0x00, 0x00, 0x09, 0x13, 0x15, 0x1a, 0xe2, 0x84, 0xed, 0xbf]),
];
const T80: [Ext80; 5] = [
    Ext80([0xfb, 0x40, 0xec, 0xba, 0xf1, 0xf8, 0xad, 0x99, 0x02, 0x40]),
    Ext80([0xc0, 0xf7, 0x33, 0xf0, 0x74, 0x52, 0x0d, 0xb4, 0x05, 0x40]),
    Ext80([0xff, 0xb8, 0xad, 0xa1, 0xe5, 0x15, 0x80, 0x8b, 0x0a, 0x40]),
    Ext80([0x15, 0x80, 0x7c, 0x97, 0xe3, 0x99, 0xda, 0xda, 0x0b, 0x40]),
    Ext80([0x27, 0xe3, 0x9e, 0x10, 0x22, 0x4d, 0x28, 0xd9, 0x0e, 0x40]),
];
const U80: [Ext80; 5] = [
    Ext80([0x57, 0xcc, 0x35, 0x7d, 0xff, 0x31, 0x3f, 0x86, 0x04, 0x40]),
    Ext80([0xff, 0x6e, 0xd7, 0x31, 0xa6, 0xe8, 0x56, 0x82, 0x08, 0x40]),
    Ext80([0xae, 0x8e, 0xe8, 0x07, 0x34, 0x97, 0x92, 0x8f, 0x0b, 0x40]),
    Ext80([0x14, 0x4c, 0xe0, 0x0b, 0x08, 0x00, 0xca, 0xb0, 0x0d, 0x40]),
    Ext80([0x88, 0x38, 0xab, 0x47, 0xee, 0x64, 0x73, 0xc0, 0x0e, 0x40]),
];

fn ef(x: f64) -> Ext80 {
    ext_from_f64(x)
}
fn horner_f(cs: &[f64], x: Ext80) -> Ext80 {
    let mut a = ef(0.0);
    for &c in cs.iter().rev() {
        a = ext_add(&ext_mul(&a, &x, CW), &ef(c), CW);
    }
    a
}
fn horner_80(cs: &[Ext80], x: Ext80) -> Ext80 {
    let mut a = ef(0.0);
    for c in cs.iter().rev() {
        a = ext_add(&ext_mul(&a, &x, CW), c, CW);
    }
    a
}
fn polevl_f(cs: &[f64], x: Ext80) -> Ext80 {
    let mut a = ef(cs[0]);
    for &c in &cs[1..] {
        a = ext_add(&ext_mul(&a, &x, CW), &ef(c), CW);
    }
    a
}
fn polevl_80(cs: &[Ext80], x: Ext80) -> Ext80 {
    let mut a = cs[0];
    for c in &cs[1..] {
        a = ext_add(&ext_mul(&a, &x, CW), c, CW);
    }
    a
}
fn p1evl_f(cs: &[f64], x: Ext80) -> Ext80 {
    let mut a = ext_add(&x, &ef(cs[0]), CW);
    for &c in &cs[1..] {
        a = ext_add(&ext_mul(&a, &x, CW), &ef(c), CW);
    }
    a
}
fn p1evl_80(cs: &[Ext80], x: Ext80) -> Ext80 {
    let mut a = ext_add(&x, &cs[0], CW);
    for c in &cs[1..] {
        a = ext_add(&ext_mul(&a, &x, CW), c, CW);
    }
    a
}
fn nswc_f64(z: f64) -> f64 {
    let ze = ef(z.abs());
    let t = ext_mul(&ze, &ze, CW);
    let num = ext_add(&ef(1.0), &horner_f(&NSWC_A, t), CW);
    let den = ext_add(&ef(1.0), &ext_mul(&t, &horner_f(&NSWC_B, t), CW), CW);
    ext_to_f64(&ext_div(&ext_mul(&ze, &num, CW), &den, CW), CW)
}
fn nswc_80(z: f64) -> f64 {
    let ze = ef(z.abs());
    let t = ext_mul(&ze, &ze, CW);
    let num = ext_add(&ef(1.0), &horner_80(&NA80, t), CW);
    let den = ext_add(&ef(1.0), &ext_mul(&t, &horner_80(&NB80, t), CW), CW);
    ext_to_f64(&ext_div(&ext_mul(&ze, &num, CW), &den, CW), CW)
}
fn fdlibm_f64(z: f64) -> f64 {
    let ze = ef(z.abs());
    let zz = ext_mul(&ze, &ze, CW);
    let r = horner_f(&FDLIBM_PP, zz);
    let q = horner_f(&FDLIBM_QQ, zz);
    let s = ext_add(&ef(1.0), &ext_mul(&zz, &q, CW), CW);
    let y = ext_div(&r, &s, CW);
    ext_to_f64(&ext_add(&ze, &ext_mul(&ze, &y, CW), CW), CW)
}
fn fdlibm_80(z: f64) -> f64 {
    let ze = ef(z.abs());
    let zz = ext_mul(&ze, &ze, CW);
    let r = horner_80(&PP80, zz);
    let q = horner_80(&QQ80, zz);
    let s = ext_add(&ef(1.0), &ext_mul(&zz, &q, CW), CW);
    let y = ext_div(&r, &s, CW);
    ext_to_f64(&ext_add(&ze, &ext_mul(&ze, &y, CW), CW), CW)
}
fn cephes_f64(z: f64) -> f64 {
    let ze = ef(z.abs());
    let zz = ext_mul(&ze, &ze, CW);
    ext_to_f64(
        &ext_div(
            &ext_mul(&ze, &polevl_f(&CEPHES_T, zz), CW),
            &p1evl_f(&CEPHES_U, zz),
            CW,
        ),
        CW,
    )
}
fn cody_f64(z: f64) -> f64 {
    let ye = ef(z.abs());
    let ysq = ext_mul(&ye, &ye, CW);
    let mut xnum = ext_mul(&ef(AA[4]), &ysq, CW);
    let mut xden = ysq;
    for i in 0..3 {
        xnum = ext_mul(&ext_add(&xnum, &ef(AA[i]), CW), &ysq, CW);
        xden = ext_mul(&ext_add(&xden, &ef(BB[i]), CW), &ysq, CW);
    }
    ext_to_f64(
        &ext_div(
            &ext_mul(&ye, &ext_add(&xnum, &ef(AA[3]), CW), CW),
            &ext_add(&xden, &ef(BB[3]), CW),
            CW,
        ),
        CW,
    )
}
fn cody_80(z: f64) -> f64 {
    let ye = ef(z.abs());
    let ysq = ext_mul(&ye, &ye, CW);
    let mut xnum = ext_mul(&AA80[4], &ysq, CW);
    let mut xden = ysq;
    for i in 0..3 {
        xnum = ext_mul(&ext_add(&xnum, &AA80[i], CW), &ysq, CW);
        xden = ext_mul(&ext_add(&xden, &BB80[i], CW), &ysq, CW);
    }
    ext_to_f64(
        &ext_div(
            &ext_mul(&ye, &ext_add(&xnum, &AA80[3], CW), CW),
            &ext_add(&xden, &BB80[3], CW),
            CW,
        ),
        CW,
    )
}
fn cephes_80(z: f64) -> f64 {
    let ze = ef(z.abs());
    let zz = ext_mul(&ze, &ze, CW);
    ext_to_f64(
        &ext_div(
            &ext_mul(&ze, &polevl_80(&T80, zz), CW),
            &p1evl_80(&U80, zz),
            CW,
        ),
        CW,
    )
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    const BANKS: [&str; 7] = [
        "answers-b9train.json",
        "answers-erfp.json",
        "answers-erfm.json",
        "answers-b8erf.json",
        "answers-b7erf.json",
        "answers-b11.json",
        "answers-b10.json",
    ];
    let mut map = BTreeMap::new();
    for name in BANKS {
        let path = format!("{dir}/{name}");
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        let bank: WitnessSet = serde_json::from_str(&text).unwrap();
        for w in &bank.witnesses {
            let x = match &w.args[0] {
                WitnessArg::Scalar(s) => parse_bits_hex(s).unwrap(),
                _ => continue,
            };
            if x < 0.0 || x >= 0.5 {
                continue;
            }
            let Some(p) = parse_bits_hex(&w.expected_bits) else {
                continue;
            };
            map.insert(x.to_bits(), (x, p.to_bits()));
        }
    }
    let rows: Vec<_> = map.into_values().collect();
    println!(
        "PP[0] f64-wide {:016x} 80store {:016x}",
        ext_to_f64(&ef(FDLIBM_PP[0]), CW).to_bits(),
        ext_to_f64(&PP80[0], CW).to_bits()
    );
    let graphs: [(&str, fn(f64) -> f64); 8] = [
        ("fdlibm PP/QQ f64-wide x87", fdlibm_f64),
        ("fdlibm PP/QQ 80-bit", fdlibm_80),
        ("Cephes T/U f64-wide x87", cephes_f64),
        ("Cephes T/U 80-bit", cephes_80),
        ("Cody A/B f64-wide x87", cody_f64),
        ("Cody A/B 80-bit", cody_80),
        ("NSWC 5+3 f64-wide x87", nswc_f64),
        ("NSWC 5+3 80-bit", nswc_80),
    ];
    println!("P-side rows={}", rows.len());
    for (name, ev) in graphs {
        let mut ex = 0usize;
        let mut n = 0usize;
        let mut maxu = 0u64;
        for &(z, pbits) in &rows {
            let pg = ev(z);
            if !pg.is_finite() {
                continue;
            }
            let d = ulp_distance(pg, f64::from_bits(pbits)).unwrap_or(u64::MAX);
            if d > ULP_CAP {
                continue;
            }
            n += 1;
            if d == 0 {
                ex += 1;
            } else {
                maxu = maxu.max(d);
            }
        }
        println!("{name:28} {ex}/{n} max={maxu}");
    }
    fn maybe(x: Ext80, mask: u32, bit: u32) -> Ext80 {
        if bit < 32 && mask & (1u32 << bit) != 0 {
            ef(ext_to_f64(&x, CW))
        } else {
            x
        }
    }
    let cephes_m = |z: f64, mask: u32| {
        let ze = ef(z.abs());
        let mut zz = ext_mul(&ze, &ze, CW);
        zz = maybe(zz, mask, 0);
        let mut t = T80[0];
        let mut bit = 1u32;
        for c in &T80[1..] {
            t = ext_add(&ext_mul(&t, &zz, CW), c, CW);
            t = maybe(t, mask, bit);
            bit += 1;
        }
        let mut u = ext_add(&zz, &U80[0], CW);
        u = maybe(u, mask, bit);
        bit += 1;
        for c in &U80[1..] {
            u = ext_add(&ext_mul(&u, &zz, CW), c, CW);
            u = maybe(u, mask, bit);
            bit += 1;
        }
        let mut q = ext_div(&ext_mul(&ze, &t, CW), &u, CW);
        q = maybe(q, mask, bit);
        ext_to_f64(&q, CW)
    };
    let score_m = |mask: u32| {
        let mut ex = 0usize;
        let mut n = 0usize;
        for &(z, pbits) in &rows {
            let pg = cephes_m(z, mask);
            if !pg.is_finite() {
                continue;
            }
            let d = ulp_distance(pg, f64::from_bits(pbits)).unwrap_or(u64::MAX);
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
    println!("Cephes80 HW0 {}/{}", base.0, base.1);
    let mut best = base.0;
    let mut best_b = None;
    for b in 0..16u32 {
        let sc = score_m(1u32 << b);
        if sc.0 != base.0 {
            println!(
                "  bit{b} {}/{} dlt={}",
                sc.0,
                sc.1,
                sc.0 as i32 - base.0 as i32
            );
        }
        if sc.0 > best {
            best = sc.0;
            best_b = Some(b);
        }
    }
    println!("Cephes80 HW1 best {best} bit={best_b:?} (bar 866)");
    let cody_m = |z: f64, mask: u32| {
        let ye = ef(z.abs());
        let mut ysq = ext_mul(&ye, &ye, CW);
        ysq = maybe(ysq, mask, 0);
        let mut xnum = ext_mul(&AA80[4], &ysq, CW);
        let mut xden = ysq;
        xnum = maybe(xnum, mask, 1);
        xden = maybe(xden, mask, 2);
        let mut bit = 3u32;
        for i in 0..3 {
            xnum = ext_mul(&ext_add(&xnum, &AA80[i], CW), &ysq, CW);
            xden = ext_mul(&ext_add(&xden, &BB80[i], CW), &ysq, CW);
            xnum = maybe(xnum, mask, bit);
            bit += 1;
            xden = maybe(xden, mask, bit);
            bit += 1;
        }
        let mut q = ext_div(
            &ext_mul(&ye, &ext_add(&xnum, &AA80[3], CW), CW),
            &ext_add(&xden, &BB80[3], CW),
            CW,
        );
        q = maybe(q, mask, bit);
        ext_to_f64(&q, CW)
    };
    let score_c = |mask: u32| {
        let mut ex = 0usize;
        let mut n = 0usize;
        for &(z, pbits) in &rows {
            let pg = cody_m(z, mask);
            if !pg.is_finite() {
                continue;
            }
            let d = ulp_distance(pg, f64::from_bits(pbits)).unwrap_or(u64::MAX);
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
    let basec = score_c(0);
    println!("Cody80 HW0 {}/{}", basec.0, basec.1);
    let mut bestc = basec.0;
    let mut bestcb = None;
    for b in 0..16u32 {
        let sc = score_c(1u32 << b);
        if sc.0 != basec.0 {
            println!(
                "  bit{b} {}/{} dlt={}",
                sc.0,
                sc.1,
                sc.0 as i32 - basec.0 as i32
            );
        }
        if sc.0 > bestc {
            bestc = sc.0;
            bestcb = Some(b);
        }
    }
    println!("Cody80 HW1 best {bestc} bit={bestcb:?} (bar 866)");
    let fd_m = |z: f64, mask: u32| {
        let ze = ef(z.abs());
        let mut zz = ext_mul(&ze, &ze, CW);
        zz = maybe(zz, mask, 0);
        let mut r = horner_80(&PP80, zz);
        r = maybe(r, mask, 1);
        let mut q = horner_80(&QQ80, zz);
        q = maybe(q, mask, 2);
        let mut s = ext_add(&ef(1.0), &ext_mul(&zz, &q, CW), CW);
        s = maybe(s, mask, 3);
        let mut y = ext_div(&r, &s, CW);
        y = maybe(y, mask, 4);
        let mut t = ext_mul(&ze, &y, CW);
        t = maybe(t, mask, 5);
        let mut a = ext_add(&ze, &t, CW);
        a = maybe(a, mask, 6);
        ext_to_f64(&a, CW)
    };
    let fd_zy = |z: f64| {
        let ze = ef(z.abs());
        let zz = ext_mul(&ze, &ze, CW);
        let r = horner_80(&PP80, zz);
        let q = horner_80(&QQ80, zz);
        let s = ext_add(&ef(1.0), &ext_mul(&zz, &q, CW), CW);
        let y = ext_div(&r, &s, CW);
        ext_to_f64(&ext_mul(&ze, &ext_add(&ef(1.0), &y, CW), CW), CW)
    };
    let score_f = |mask: u32| {
        let mut ex = 0usize;
        let mut n = 0usize;
        for &(z, pbits) in &rows {
            let pg = fd_m(z, mask);
            if !pg.is_finite() {
                continue;
            }
            let d = ulp_distance(pg, f64::from_bits(pbits)).unwrap_or(u64::MAX);
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
    let basef = score_f(0);
    println!("fdlibm80 HW0 {}/{} (z+z*y)", basef.0, basef.1);
    let mut bestf = basef.0;
    let mut bestfb = None;
    for b in 0..8u32 {
        let sc = score_f(1u32 << b);
        if sc.0 != basef.0 {
            println!(
                "  bit{b} {}/{} dlt={}",
                sc.0,
                sc.1,
                sc.0 as i32 - basef.0 as i32
            );
        }
        if sc.0 > bestf {
            bestf = sc.0;
            bestfb = Some(b);
        }
    }
    println!("fdlibm80 HW1 best {bestf} bit={bestfb:?} (bar 866)");
    let mut exy = 0usize;
    let mut ny = 0usize;
    let mut maxy = 0u64;
    for &(z, pbits) in &rows {
        let pg = fd_zy(z);
        if !pg.is_finite() {
            continue;
        }
        let d = ulp_distance(pg, f64::from_bits(pbits)).unwrap_or(u64::MAX);
        if d > ULP_CAP {
            continue;
        }
        ny += 1;
        if d == 0 {
            exy += 1;
        } else {
            maxy = maxy.max(d);
        }
    }
    println!("fdlibm80 z*(1+y) {exy}/{ny} max={maxy}");
}
