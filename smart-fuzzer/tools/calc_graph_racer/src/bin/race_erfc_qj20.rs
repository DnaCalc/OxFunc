//! j3432+0x20 vs 0x74 overlap/cut; 3439 vs 3432 extra exacts. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC64_RN};
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
    let mut jc = C0;
    jc[0] = poke(C0[0], 4);
    jc[1] = poke(C0[1], 1);
    jc[4] = poke(C0[4], -1);
    let mut jd = D0;
    jd[0] = poke(D0[0], -1);
    jd[4] = poke(D0[4], -1);
    let ev_j = |z: f64| cody(z, &jc, &jd, 0);
    let ev_j20 = |z: f64| cody(z, &jc, &jd, 0x20);
    let ev_j24 = |z: f64| cody(z, &jc, &jd, 0x24);
    let ev_74 = |z: f64| cody(z, &C0, &D0, 0x74);

    println!("Q extra exacts j3432+0x24 vs mask0:");
    let mut n_extra = 0usize;
    let mut n_extra_d = 0usize;
    let mut n_lost = 0usize;
    let mut n_lost_d = 0usize;
    for r in &rows {
        if r.z < 0.5 || r.z >= 4.0 {
            continue;
        }
        let t = f64::from_bits(r.qbits);
        let d0 = ulp_distance(qw(r.z, ev_j(r.z)), t).unwrap_or(99);
        let d24 = ulp_distance(qw(r.z, ev_j24(r.z)), t).unwrap_or(99);
        match (d0 == 0, d24 == 0) {
            (false, true) => {
                n_extra += 1;
                if r.direct {
                    n_extra_d += 1;
                }
                println!(
                    "  extra z={:.16} direct={} j_ulp={} 0x74_ulp={}",
                    r.z,
                    r.direct,
                    d0,
                    ulp_distance(qw(r.z, ev_74(r.z)), t).unwrap_or(99)
                );
            }
            (true, false) => {
                n_lost += 1;
                if r.direct {
                    n_lost_d += 1;
                }
                if n_lost <= 8 {
                    println!(
                        "  lost z={:.16} direct={} j24_ulp={}",
                        r.z, r.direct, d24
                    );
                }
            }
            _ => {}
        }
    }
    println!("  extra={n_extra} d={n_extra_d} lost={n_lost} d={n_lost_d} net=+{}", n_extra as i32 - n_lost as i32);

    println!("F_or overlap j+0x20 vs 0x74:");
    let mut both = 0usize;
    let mut only_j = 0usize;
    let mut only_74 = 0usize;
    let mut only_j_d = 0usize;
    let mut only_74_d = 0usize;
    for r in &rows {
        if r.z < 0.5 || r.z >= 4.0 {
            continue;
        }
        let Some(fo) = f::f_or(r.z, r.qbits) else {
            continue;
        };
        let dj = ulp_distance(ev_j20(r.z), fo).unwrap_or(99);
        let d7 = ulp_distance(ev_74(r.z), fo).unwrap_or(99);
        match (dj == 0, d7 == 0) {
            (true, true) => both += 1,
            (true, false) => {
                only_j += 1;
                if r.direct {
                    only_j_d += 1;
                }
            }
            (false, true) => {
                only_74 += 1;
                if r.direct {
                    only_74_d += 1;
                }
            }
            _ => {}
        }
    }
    println!(
        "  both={both} only_j20={only_j} d={only_j_d} only_74={only_74} d={only_74_d} union={}",
        both + only_j + only_74
    );

    println!("Q overlap j+0x20 vs 0x74:");
    let mut qboth = 0usize;
    let mut qj = 0usize;
    let mut q7 = 0usize;
    for r in &rows {
        if r.z < 0.5 || r.z >= 4.0 {
            continue;
        }
        let t = f64::from_bits(r.qbits);
        let dj = ulp_distance(qw(r.z, ev_j20(r.z)), t).unwrap_or(99);
        let d7 = ulp_distance(qw(r.z, ev_74(r.z)), t).unwrap_or(99);
        match (dj == 0, d7 == 0) {
            (true, true) => qboth += 1,
            (true, false) => qj += 1,
            (false, true) => q7 += 1,
            _ => {}
        }
    }
    println!(
        "  both={qboth} only_j20={qj} only_74={q7} union={}",
        qboth + qj + q7
    );

    println!("F_or 0.5-bands j+0x20 vs 0x74:");
    let mut bj = [0usize; 7];
    let mut b7 = [0usize; 7];
    let mut bn = [0usize; 7];
    for r in &rows {
        if r.z < 0.5 || r.z >= 4.0 {
            continue;
        }
        let Some(fo) = f::f_or(r.z, r.qbits) else {
            continue;
        };
        let b = ((r.z - 0.5) / 0.5).floor() as usize;
        let b = b.min(6);
        bn[b] += 1;
        if ulp_distance(ev_j20(r.z), fo).unwrap_or(99) == 0 {
            bj[b] += 1;
        }
        if ulp_distance(ev_74(r.z), fo).unwrap_or(99) == 0 {
            b7[b] += 1;
        }
    }
    for i in 0..7 {
        let lo = 0.5 + i as f64 * 0.5;
        println!(
            "  [{:.1},{:.1}) n={} j20={} 74={}",
            lo,
            lo + 0.5,
            bn[i],
            bj[i],
            b7[i]
        );
    }
    println!("F_or cuts j+0x20 vs 0x74:");
    for &c in &[0.75, 1.0, 1.5, 2.0, 2.5, 3.0, 3.5] {
        let mut a = 0usize;
        let mut b = 0usize;
        let mut n = 0usize;
        for r in &rows {
            if r.z < 0.5 || r.z >= 4.0 {
                continue;
            }
            let Some(fo) = f::f_or(r.z, r.qbits) else {
                continue;
            };
            n += 1;
            let g = if r.z < c { ev_j20(r.z) } else { ev_74(r.z) };
            let h = if r.z < c { ev_74(r.z) } else { ev_j20(r.z) };
            if ulp_distance(g, fo).unwrap_or(99) == 0 {
                a += 1;
            }
            if ulp_distance(h, fo).unwrap_or(99) == 0 {
                b += 1;
            }
        }
        println!("  cut={c:.2} j20-then-74={a}/{n} 74-then-j20={b}/{n} (bar 3021)");
    }
    println!("Q cuts j+0x20 vs 0x74:");
    for &c in &[0.75, 1.0, 1.5, 2.0, 2.5, 3.0, 3.5] {
        let mut a = 0usize;
        let mut b = 0usize;
        let mut n = 0usize;
        for r in &rows {
            if r.z < 0.5 || r.z >= 4.0 {
                continue;
            }
            n += 1;
            let t = f64::from_bits(r.qbits);
            let g = if r.z < c {
                qw(r.z, ev_j20(r.z))
            } else {
                qw(r.z, ev_74(r.z))
            };
            let h = if r.z < c {
                qw(r.z, ev_74(r.z))
            } else {
                qw(r.z, ev_j20(r.z))
            };
            if ulp_distance(g, t).unwrap_or(99) == 0 {
                a += 1;
            }
            if ulp_distance(h, t).unwrap_or(99) == 0 {
                b += 1;
            }
        }
        println!("  cut={c:.2} j20-then-74={a}/{n} 74-then-j20={b}/{n} (bar 3438)");
    }
}
