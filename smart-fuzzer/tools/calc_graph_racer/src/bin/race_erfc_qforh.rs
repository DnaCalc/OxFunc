//! F_or mid: 0x74 submasks, C/D leader×mask, named F bands. Not a cube. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC64_RN};
use std::env;

const CW: u16 = CW_PC64_RN;
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
    let ev_j = |z: f64| cody(z, &jc, &jd, 0);
    let ev_h = |z: f64| cody(z, &hamc, &hamd, 0);
    let ev_74 = |z: f64| cody(z, &C0, &D0, 0x74);

    let mut only_j_d = 0usize;
    let mut only_h_d = 0usize;
    let mut only_j = 0usize;
    let mut only_h = 0usize;
    let mut j_on_h_ulp = [0usize; 5];
    let mut h_on_j_ulp = [0usize; 5];
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
            (true, false) => {
                only_j += 1;
                if r.direct {
                    only_j_d += 1;
                }
                let b = dh.min(4) as usize;
                h_on_j_ulp[b] += 1;
            }
            (false, true) => {
                only_h += 1;
                if r.direct {
                    only_h_d += 1;
                }
                let b = dj.min(4) as usize;
                j_on_h_ulp[b] += 1;
            }
            _ => {}
        }
    }
    println!(
        "j vs ham2 exclusives: only_j={only_j} d={only_j_d} only_h={only_h} d={only_h_d}"
    );
    println!("  ham2 ulp on only_j (0..4+): {h_on_j_ulp:?}");
    println!("  j3432 ulp on only_h (0..4+): {j_on_h_ulp:?}");

    println!("0x74 submasks F_or mid (bar 2976):");
    const SB: [u32; 4] = [2, 4, 5, 6];
    let mut best = 0usize;
    let mut best_m = 0u32;
    for s in 0u32..16 {
        let mut mask = 0u32;
        let mut pop = 0u32;
        for i in 0..4 {
            if s & (1 << i) != 0 {
                mask |= 1 << SB[i];
                pop += 1;
            }
        }
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
            if ulp_distance(cody(r.z, &C0, &D0, mask), fo).unwrap_or(99) == 0 {
                ex += 1;
                if r.direct {
                    dd += 1;
                }
            }
        }
        if ex >= best {
            best = ex;
            best_m = mask;
        }
        if ex >= 2960 || pop <= 1 || mask == 0x74 {
            println!("  mask={mask:#06x} pop={pop} F_or mid {ex}/{n} d={dd}");
        }
    }
    println!("  best submask {best_m:#06x} {best}");

    println!("C/D leader × mask F_or mid:");
    for (name, c, d) in [
        ("CR", C0, D0),
        ("ham2", hamc, hamd),
        ("j3432", jc, jd),
    ] {
        for &mask in &[0u32, 0x04, 0x74, 0x200, 0x210] {
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
                if ulp_distance(cody(r.z, &c, &d, mask), fo).unwrap_or(99) == 0 {
                    ex += 1;
                    if r.direct {
                        dd += 1;
                    }
                }
            }
            println!("  {name:6} mask={mask:#06x} F_or mid {ex}/{n} d={dd}");
        }
    }

    println!("named F as F_or mid (bars 2976/3014):");
    let named: [(&str, fn(f64) -> f64); 5] = [
        ("cephes_f", f::cephes_f),
        ("nswc_derfc0", f::nswc_derfc0),
        ("nswc_ccdd_f", f::nswc_ccdd_f),
        ("nswc_pqr_f", f::nswc_pqr_f),
        ("cephes_5005", cephes_5005),
    ];
    for (name, ev) in named {
        let mut band = [0usize; 7];
        let mut bn = [0usize; 7];
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
            let b = ((r.z - 0.5) / 0.5).floor() as usize;
            let b = b.min(6);
            bn[b] += 1;
            let fg = ev(r.z);
            if ulp_distance(fg, fo).unwrap_or(99) == 0 {
                ex += 1;
                band[b] += 1;
                if r.direct {
                    dd += 1;
                }
            }
        }
        println!("  {name:14} F_or mid {ex}/{n} d={dd}");
        for i in 0..7 {
            let lo = 0.5 + i as f64 * 0.5;
            println!(
                "    [{:.1},{:.1}) n={} named={} j={} ham={} 74={}",
                lo,
                lo + 0.5,
                bn[i],
                band[i],
                {
                    let mut k = 0usize;
                    for r in &qrows {
                        if r.z < lo || r.z >= lo + 0.5 {
                            continue;
                        }
                        let Some(fo) = f::f_or(r.z, r.qbits) else {
                            continue;
                        };
                        if ulp_distance(ev_j(r.z), fo).unwrap_or(99) == 0 {
                            k += 1;
                        }
                    }
                    k
                },
                {
                    let mut k = 0usize;
                    for r in &qrows {
                        if r.z < lo || r.z >= lo + 0.5 {
                            continue;
                        }
                        let Some(fo) = f::f_or(r.z, r.qbits) else {
                            continue;
                        };
                        if ulp_distance(ev_h(r.z), fo).unwrap_or(99) == 0 {
                            k += 1;
                        }
                    }
                    k
                },
                {
                    let mut k = 0usize;
                    for r in &qrows {
                        if r.z < lo || r.z >= lo + 0.5 {
                            continue;
                        }
                        let Some(fo) = f::f_or(r.z, r.qbits) else {
                            continue;
                        };
                        if ulp_distance(ev_74(r.z), fo).unwrap_or(99) == 0 {
                            k += 1;
                        }
                    }
                    k
                }
            );
        }
    }

    println!("derfc0-then-0x5005 F_or tail (mixed bar 1370 @ 4.9):");
    for &c in &[4.5, 4.8, 4.9, 5.0, 5.6, 6.0] {
        let mut ex = 0usize;
        let mut dd = 0usize;
        let mut n = 0usize;
        for r in &qrows {
            if r.z < 4.0 {
                continue;
            }
            let Some(fo) = f::f_or(r.z, r.qbits) else {
                continue;
            };
            n += 1;
            let fg = if r.z < c {
                f::nswc_derfc0(r.z)
            } else {
                cephes_5005(r.z)
            };
            if ulp_distance(fg, fo).unwrap_or(99) == 0 {
                ex += 1;
                if r.direct {
                    dd += 1;
                }
            }
        }
        println!("  cut={c:.1} F_or tail {ex}/{n} d={dd}");
    }
}
