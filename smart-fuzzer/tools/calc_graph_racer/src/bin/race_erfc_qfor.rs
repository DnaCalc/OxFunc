//! Q-metric C/D leaders scored as F vs F_or. Mixed bar 2976/1370. Not a cube. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{
    ext_add, ext_div, ext_from_f64, ext_mul, ext_sqrt, ext_sub, ext_to_f64, Ext80, CW_PC64_RN,
};
use std::env;

const CW: u16 = CW_PC64_RN;
const ULP_CAP: u64 = 1 << 20;
const C0: [f64; 9] = [
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
const SP: [f64; 6] = [
    0.305326634961232344,
    0.360344899949804439,
    0.125781726111229246,
    0.0160837851487422766,
    6.58749161529837803e-4,
    0.0163153871373020978,
];
const SQ: [f64; 5] = [
    2.56852019228982242,
    1.87295284992346047,
    0.527905102951428412,
    0.0605183413124413191,
    0.00233520497626869185,
];
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
const D0: [f64; 8] = [
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
fn poke(x: f64, k: i32) -> f64 {
    let mut v = x;
    if k > 0 {
        for _ in 0..k {
            v = v.next_up();
        }
    } else {
        for _ in 0..(-k) {
            v = v.next_down();
        }
    }
    v
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
fn pq_x87(y: f64) -> f64 {
    let ye = ef(y.abs());
    let ysq = ext_div(&ef(1.0), &ext_mul(&ye, &ye, CW), CW);
    let mut xnum = ext_mul(&ef(SP[5]), &ysq, CW);
    let mut xden = ysq;
    for i in 0..4 {
        xnum = ext_mul(&ext_add(&xnum, &ef(SP[i]), CW), &ysq, CW);
        xden = ext_mul(&ext_add(&xden, &ef(SQ[i]), CW), &ysq, CW);
    }
    let r = ext_div(
        &ext_mul(&ysq, &ext_add(&xnum, &ef(SP[4]), CW), CW),
        &ext_add(&xden, &ef(SQ[4]), CW),
        CW,
    );
    let s = ext_div(&ef(1.0), &ext_sqrt(&ef(std::f64::consts::PI), CW), CW);
    ext_to_f64(&ext_div(&ext_sub(&s, &r, CW), &ye, CW), CW)
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
fn cody(y: f64, c: &[f64; 9], d: &[f64; 8], mask: u32) -> f64 {
    let ye = ef(y.abs());
    let mut xnum = ext_mul(&ef(c[8]), &ye, CW);
    let mut xden = ye;
    xnum = maybe(xnum, mask, 0);
    xden = maybe(xden, mask, 1);
    for i in 0..7 {
        xnum = ext_mul(&ext_add(&xnum, &ef(c[i]), CW), &ye, CW);
        xden = ext_mul(&ext_add(&xden, &ef(d[i]), CW), &ye, CW);
        xnum = maybe(xnum, mask, 2 + 2 * i as u32);
        xden = maybe(xden, mask, 3 + 2 * i as u32);
    }
    let mut q = ext_div(
        &ext_add(&xnum, &ef(c[7]), CW),
        &ext_add(&xden, &ef(d[7]), CW),
        CW,
    );
    q = maybe(q, mask, 16);
    ext_to_f64(&q, CW)
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let qrows = f::load_q_rows_tagged(&dir);
    let mut jc = C0;
    jc[0] = poke(C0[0], 4);
    jc[1] = poke(C0[1], 1);
    jc[4] = poke(C0[4], -1);
    let mut jd = D0;
    jd[0] = poke(D0[0], -1);
    jd[4] = poke(D0[4], -1);
    let mut hamc = C0;
    hamc[4] = poke(C0[4], -1);
    let mut hamd = D0;
    hamd[4] = poke(D0[4], -1);
    let graphs: [(&str, Box<dyn Fn(f64) -> f64>); 5] = [
        ("C/D CR mask0", Box::new(|z| cody(z, &C0, &D0, 0))),
        (
            "ham2 C4-1 D4-1",
            Box::new({
                let c = hamc;
                let d = hamd;
                move |z| cody(z, &c, &d, 0)
            }),
        ),
        (
            "j3432",
            Box::new({
                let c = jc;
                let d = jd;
                move |z| cody(z, &c, &d, 0)
            }),
        ),
        ("C/D 0x74", Box::new(|z| cody(z, &C0, &D0, 0x74))),
        ("C/D 0x210", Box::new(|z| cody(z, &C0, &D0, 0x210))),
    ];
    println!("F vs F_or (mixed bar mid 2976 tail 1370)");
    for (name, ev) in &graphs {
        let mut mid = 0usize;
        let mut md = 0usize;
        let mut tail = 0usize;
        let mut td = 0usize;
        let mut mn = 0usize;
        let mut tn = 0usize;
        for r in &qrows {
            if r.z < 0.5 {
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
            if r.z < 4.0 {
                mn += 1;
                if d == 0 {
                    mid += 1;
                    if r.direct {
                        md += 1;
                    }
                }
            } else {
                tn += 1;
                if d == 0 {
                    tail += 1;
                    if r.direct {
                        td += 1;
                    }
                }
            }
        }
        println!("{name:18} F_or mid {mid}/{mn} d={md}  tail {tail}/{tn} d={td}  (Q-bars 3386/3432)");
    }
    let ev_j = |z: f64| cody(z, &jc, &jd, 0);
    let ev_74 = |z: f64| cody(z, &C0, &D0, 0x74);
    let mut both = 0usize;
    let mut only_j = 0usize;
    let mut only_74 = 0usize;
    let mut shown = 0usize;
    println!("F_or exact j3432 vs 0x74 (mid):");
    for r in &qrows {
        if r.z < 0.5 || r.z >= 4.0 {
            continue;
        }
        let Some(fo) = f::f_or(r.z, r.qbits) else {
            continue;
        };
        let dj = ulp_distance(ev_j(r.z), fo).unwrap_or(99);
        let d7 = ulp_distance(ev_74(r.z), fo).unwrap_or(99);
        match (dj == 0, d7 == 0) {
            (true, true) => both += 1,
            (true, false) => {
                only_j += 1;
                if shown < 8 {
                    println!("  only_j3432 z={:.16} direct={}", r.z, r.direct);
                    shown += 1;
                }
            }
            (false, true) => only_74 += 1,
            _ => {}
        }
    }
    println!(
        "F_or overlap both={} only_j3432={} only_0x74={} union={}",
        both,
        only_j,
        only_74,
        both + only_j + only_74
    );
    println!("named F as F_or tail (mixed bar 1370):");
    for (name, ev) in [
        ("cephes_f", f::cephes_f as fn(f64) -> f64),
        ("cephes 0x5005", cephes_5005),
        ("nswc_ccdd_f", f::nswc_ccdd_f),
        ("nswc_derfc0", f::nswc_derfc0),
        ("nswc_pqr_f", f::nswc_pqr_f),
    ] {
        let mut ex = 0usize;
        let mut n = 0usize;
        let mut td = 0usize;
        for r in &qrows {
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
    println!("pq_x87 then 0x5005 as F_or tail:");
    for &c in &[5.0, 5.8, 6.0, 6.5, 8.0] {
        let mut ex = 0usize;
        let mut n = 0usize;
        let mut td = 0usize;
        for r in &qrows {
            if r.z < 4.0 {
                continue;
            }
            let Some(fo) = f::f_or(r.z, r.qbits) else {
                continue;
            };
            let fg = if r.z < c {
                pq_x87(r.z)
            } else {
                cephes_5005(r.z)
            };
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
        println!("  cut={c:.1} F_or tail {ex}/{n} d={td} (bar 1370)");
    }
    println!("CCDD f64 then 0x5005 as F_or tail:");
    for &c in &[4.5, 5.0, 5.6, 6.0, 8.0] {
        let mut ex = 0usize;
        let mut n = 0usize;
        let mut td = 0usize;
        for r in &qrows {
            if r.z < 4.0 {
                continue;
            }
            let Some(fo) = f::f_or(r.z, r.qbits) else {
                continue;
            };
            let fg = if r.z < c {
                f::nswc_ccdd_f(r.z)
            } else {
                cephes_5005(r.z)
            };
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
        println!("  cut={c:.1} F_or tail {ex}/{n} d={td} (bar 1370)");
    }
    println!("CCDD80 then 0x5005 as F_or tail:");
    for &c in &[4.5, 5.0, 5.6, 6.0, 6.5, 8.0] {
        let mut ex = 0usize;
        let mut n = 0usize;
        let mut td = 0usize;
        for r in &qrows {
            if r.z < 4.0 {
                continue;
            }
            let Some(fo) = f::f_or(r.z, r.qbits) else {
                continue;
            };
            let fg = if r.z < c {
                ccdd_80(r.z)
            } else {
                cephes_5005(r.z)
            };
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
        println!("  cut={c:.1} F_or tail {ex}/{n} d={td} (bar 1370)");
    }
    println!("cephes mask0 then 0x5005 as F_or tail:");
    for &c in &[4.5, 5.0, 8.0] {
        let mut ex = 0usize;
        let mut n = 0usize;
        let mut td = 0usize;
        for r in &qrows {
            if r.z < 4.0 {
                continue;
            }
            let Some(fo) = f::f_or(r.z, r.qbits) else {
                continue;
            };
            let fg = if r.z < c {
                f::cephes_f(r.z)
            } else {
                cephes_5005(r.z)
            };
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
        println!("  cut={c:.1} F_or tail {ex}/{n} d={td} (bar 1370)");
    }
    println!("j3432 vs 0x74 F_or mid cuts (bars 3014/2976):");
    let mut band_j = [0usize; 7];
    let mut band_74 = [0usize; 7];
    let mut band_n = [0usize; 7];
    for r in &qrows {
        if r.z < 0.5 || r.z >= 4.0 {
            continue;
        }
        let Some(fo) = f::f_or(r.z, r.qbits) else {
            continue;
        };
        let b = ((r.z - 0.5) / 0.5).floor() as usize;
        let b = b.min(6);
        band_n[b] += 1;
        if ulp_distance(ev_j(r.z), fo).unwrap_or(99) == 0 {
            band_j[b] += 1;
        }
        if ulp_distance(ev_74(r.z), fo).unwrap_or(99) == 0 {
            band_74[b] += 1;
        }
    }
    for i in 0..7 {
        let lo = 0.5 + i as f64 * 0.5;
        println!(
            "  [{:.1},{:.1}) n={} j3432={} 0x74={}",
            lo,
            lo + 0.5,
            band_n[i],
            band_j[i],
            band_74[i]
        );
    }
    for &c in &[0.75, 1.0, 1.5, 2.0, 2.5, 3.0, 3.5] {
        let mut j7 = 0usize;
        let mut seven_j = 0usize;
        let mut n = 0usize;
        for r in &qrows {
            if r.z < 0.5 || r.z >= 4.0 {
                continue;
            }
            let Some(fo) = f::f_or(r.z, r.qbits) else {
                continue;
            };
            n += 1;
            let g = if r.z < c { ev_j(r.z) } else { ev_74(r.z) };
            let h = if r.z < c { ev_74(r.z) } else { ev_j(r.z) };
            if ulp_distance(g, fo).unwrap_or(99) == 0 {
                j7 += 1;
            }
            if ulp_distance(h, fo).unwrap_or(99) == 0 {
                seven_j += 1;
            }
        }
        println!("  cut={c:.2} j-then-74={j7}/{n} 74-then-j={seven_j}/{n}");
    }
    let ev_h = |z: f64| cody(z, &hamc, &hamd, 0);
    let mut both_h = 0usize;
    let mut only_jh = 0usize;
    let mut only_h = 0usize;
    let mut shown_h = 0usize;
    let mut shown_j = 0usize;
    println!("F_or exact j3432 vs ham2 (mid):");
    for r in &qrows {
        if r.z < 0.5 || r.z >= 4.0 {
            continue;
        }
        let Some(fo) = f::f_or(r.z, r.qbits) else {
            continue;
        };
        let dj = ulp_distance(ev_j(r.z), fo).unwrap_or(99);
        let dh = ulp_distance(ev_h(r.z), fo).unwrap_or(99);
        match (dj == 0, dh == 0) {
            (true, true) => both_h += 1,
            (true, false) => {
                only_jh += 1;
                if shown_j < 8 {
                    println!("  only_j3432 z={:.16} direct={}", r.z, r.direct);
                    shown_j += 1;
                }
            }
            (false, true) => {
                only_h += 1;
                if shown_h < 8 {
                    println!("  only_ham2 z={:.16} direct={}", r.z, r.direct);
                    shown_h += 1;
                }
            }
            _ => {}
        }
    }
    println!(
        "F_or overlap both={} only_j3432={} only_ham2={} union={}",
        both_h,
        only_jh,
        only_h,
        both_h + only_jh + only_h
    );
    println!("j3432 vs ham2 F_or mid 0.5-bands (bars 3014/3000):");
    let mut band_j2 = [0usize; 7];
    let mut band_h = [0usize; 7];
    let mut band_n2 = [0usize; 7];
    for r in &qrows {
        if r.z < 0.5 || r.z >= 4.0 {
            continue;
        }
        let Some(fo) = f::f_or(r.z, r.qbits) else {
            continue;
        };
        let b = ((r.z - 0.5) / 0.5).floor() as usize;
        let b = b.min(6);
        band_n2[b] += 1;
        if ulp_distance(ev_j(r.z), fo).unwrap_or(99) == 0 {
            band_j2[b] += 1;
        }
        if ulp_distance(ev_h(r.z), fo).unwrap_or(99) == 0 {
            band_h[b] += 1;
        }
    }
    for i in 0..7 {
        let lo = 0.5 + i as f64 * 0.5;
        println!(
            "  [{:.1},{:.1}) n={} j3432={} ham2={}",
            lo,
            lo + 0.5,
            band_n2[i],
            band_j2[i],
            band_h[i]
        );
    }
    println!("j3432 vs ham2 F_or mid cuts:");
    for &c in &[0.75, 1.0, 1.5, 2.0, 2.5, 3.0, 3.5] {
        let mut jh = 0usize;
        let mut hj = 0usize;
        let mut n = 0usize;
        for r in &qrows {
            if r.z < 0.5 || r.z >= 4.0 {
                continue;
            }
            let Some(fo) = f::f_or(r.z, r.qbits) else {
                continue;
            };
            n += 1;
            let g = if r.z < c { ev_j(r.z) } else { ev_h(r.z) };
            let h = if r.z < c { ev_h(r.z) } else { ev_j(r.z) };
            if ulp_distance(g, fo).unwrap_or(99) == 0 {
                jh += 1;
            }
            if ulp_distance(h, fo).unwrap_or(99) == 0 {
                hj += 1;
            }
        }
        println!("  cut={c:.2} j-then-ham2={jh}/{n} ham2-then-j={hj}/{n}");
    }
    // Isolate which of the 3 extra pokes (C0+4, C1+1, D0-1) drive j vs ham2.
    let mut c0 = hamc;
    c0[0] = poke(C0[0], 4);
    let mut c1 = hamc;
    c1[1] = poke(C0[1], 1);
    let mut d0p = hamd;
    d0p[0] = poke(D0[0], -1);
    let mut c0c1 = hamc;
    c0c1[0] = poke(C0[0], 4);
    c0c1[1] = poke(C0[1], 1);
    let mut c0d0 = hamc;
    c0d0[0] = poke(C0[0], 4);
    let mut d0c0 = hamd;
    d0c0[0] = poke(D0[0], -1);
    let mut c1d0c = hamc;
    c1d0c[1] = poke(C0[1], 1);
    let mut c1d0d = hamd;
    c1d0d[0] = poke(D0[0], -1);
    let extras: [(&str, [f64; 9], [f64; 8]); 6] = [
        ("ham2+C0+4", c0, hamd),
        ("ham2+C1+1", c1, hamd),
        ("ham2+D0-1", hamc, d0p),
        ("ham2+C0+4+C1+1", c0c1, hamd),
        ("ham2+C0+4+D0-1", c0d0, d0c0),
        ("ham2+C1+1+D0-1", c1d0c, c1d0d),
    ];
    println!("ham2 + extra poke F_or mid (bars ham2 3000 / j3432 3014):");
    for (name, c, d) in extras {
        let mut ex = 0usize;
        let mut dd = 0usize;
        let mut n = 0usize;
        for r in &qrows {
            if r.z < 0.5 || r.z >= 4.0 {
                continue;
            }
            let Some(fo) = f::f_or(r.z, r.qbits) else {
                continue;
            };
            n += 1;
            let fg = cody(r.z, &c, &d, 0);
            if ulp_distance(fg, fo).unwrap_or(99) == 0 {
                ex += 1;
                if r.direct {
                    dd += 1;
                }
            }
        }
        println!("  {name:22} F_or mid {ex}/{n} d={dd}");
    }
}
