//! Direct-mid dmid search: HW=3 stores, one-coeff ±16, ham2 pokes on 0x210.
//! Not a cube. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC64_RN};
use std::env;

const CW: u16 = CW_PC64_RN;
const ULP_CAP: u64 = 1 << 20;
const NSITE: u32 = 17;
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
fn maybe(x: Ext80, mask: u32, bit: u32) -> Ext80 {
    if bit < 32 && mask & (1u32 << bit) != 0 {
        ef(ext_to_f64(&x, CW))
    } else {
        x
    }
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
    let mut q = ext_div(&ext_add(&xnum, &ef(c[7]), CW), &ext_add(&xden, &ef(d[7]), CW), CW);
    q = maybe(q, mask, 16);
    ext_to_f64(&q, CW)
}

#[derive(Clone, Copy, Default)]
struct Sc {
    exact: usize,
    dmid: usize,
    max_d: u64,
    m1d: usize,
}
fn score(rows: &[f::QRow], c: &[f64; 9], d: &[f64; 8], mask: u32) -> Sc {
    let mut s = Sc::default();
    for r in rows {
        if r.z < 0.5 || r.z >= 4.0 {
            continue;
        }
        let qg = f::w_rn53(r.z) * cody(r.z, c, d, mask);
        if !qg.is_finite() {
            continue;
        }
        let dist = ulp_distance(qg, f64::from_bits(r.qbits)).unwrap_or(u64::MAX);
        if dist > ULP_CAP {
            continue;
        }
        if dist == 0 {
            s.exact += 1;
        }
        if r.direct {
            s.max_d = s.max_d.max(dist);
            if dist == 0 {
                s.dmid += 1;
            } else if dist == 1 {
                s.m1d += 1;
            }
        }
    }
    s
}
fn better(a: Sc, b: Sc) -> bool {
    a.dmid > b.dmid
        || (a.dmid == b.dmid && a.m1d > b.m1d)
        || (a.dmid == b.dmid && a.m1d == b.m1d && a.max_d < b.max_d)
        || (a.dmid == b.dmid && a.m1d == b.m1d && a.max_d == b.max_d && a.exact > b.exact)
}
fn fmt(s: Sc) -> String {
    format!(
        "exact={} dmid={} maxd={} m1d={}",
        s.exact, s.dmid, s.max_d, s.m1d
    )
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    let m210 = 0x210u32;
    let base0 = score(&rows, &C0, &D0, 0);
    let base210 = score(&rows, &C0, &D0, m210);
    println!("mask0 {}", fmt(base0));
    println!("0x210 {}", fmt(base210));

    println!("## HW=3 by dmid from mask0");
    let mut best3 = base210;
    let mut lab3 = m210;
    for i in 0..NSITE {
        for j in (i + 1)..NSITE {
            for k in (j + 1)..NSITE {
                let m = (1u32 << i) | (1u32 << j) | (1u32 << k);
                let sc = score(&rows, &C0, &D0, m);
                if better(sc, best3) {
                    best3 = sc;
                    lab3 = m;
                    println!("HIT bits={i},{j},{k} mask={m:#x} {}", fmt(sc));
                }
            }
        }
    }
    println!("best HW3 mask={lab3:#x} {}", fmt(best3));

    println!("## one-coeff k=-16..=16 on mask0 by dmid");
    let mut bestc = base0;
    let mut labc = String::from("CR");
    for i in 0..9 {
        for k in -16i32..=16 {
            if k == 0 {
                continue;
            }
            let mut c = C0;
            c[i] = poke(C0[i], k);
            let sc = score(&rows, &c, &D0, 0);
            if better(sc, bestc) {
                bestc = sc;
                labc = format!("C[{i}]{k:+}");
                println!("HIT {labc} {}", fmt(sc));
            }
        }
    }
    for i in 0..8 {
        for k in -16i32..=16 {
            if k == 0 {
                continue;
            }
            let mut d = D0;
            d[i] = poke(D0[i], k);
            let sc = score(&rows, &C0, &d, 0);
            if better(sc, bestc) {
                bestc = sc;
                labc = format!("D[{i}]{k:+}");
                println!("HIT {labc} {}", fmt(sc));
            }
        }
    }
    println!("best one-coeff {labc} {}", fmt(bestc));

    println!("## ham2 ±1 pokes on 0x210 by dmid");
    let mut besth = base210;
    let mut labh = String::from("0x210");
    let deltas = [1i32, -1];
    for i in 0..9 {
        for &k in &deltas {
            let mut c = C0;
            c[i] = poke(C0[i], k);
            let sc = score(&rows, &c, &D0, m210);
            if better(sc, besth) {
                besth = sc;
                labh = format!("0x210 C[{i}]{k:+}");
                println!("HIT {labh} {}", fmt(sc));
            }
        }
    }
    for i in 0..8 {
        for &k in &deltas {
            let mut d = D0;
            d[i] = poke(D0[i], k);
            let sc = score(&rows, &C0, &d, m210);
            if better(sc, besth) {
                besth = sc;
                labh = format!("0x210 D[{i}]{k:+}");
                println!("HIT {labh} {}", fmt(sc));
            }
        }
    }
    for i in 0..9 {
        for j in (i + 1)..9 {
            for &k1 in &deltas {
                for &k2 in &deltas {
                    let mut c = C0;
                    c[i] = poke(C0[i], k1);
                    c[j] = poke(C0[j], k2);
                    let sc = score(&rows, &c, &D0, m210);
                    if better(sc, besth) {
                        besth = sc;
                        labh = format!("0x210 C[{i}]{k1:+} C[{j}]{k2:+}");
                        println!("HIT {labh} {}", fmt(sc));
                    }
                }
            }
        }
    }
    for i in 0..8 {
        for j in (i + 1)..8 {
            for &k1 in &deltas {
                for &k2 in &deltas {
                    let mut d = D0;
                    d[i] = poke(D0[i], k1);
                    d[j] = poke(D0[j], k2);
                    let sc = score(&rows, &C0, &d, m210);
                    if better(sc, besth) {
                        besth = sc;
                        labh = format!("0x210 D[{i}]{k1:+} D[{j}]{k2:+}");
                        println!("HIT {labh} {}", fmt(sc));
                    }
                }
            }
        }
    }
    for i in 0..9 {
        for j in 0..8 {
            for &k1 in &deltas {
                for &k2 in &deltas {
                    let mut c = C0;
                    let mut d = D0;
                    c[i] = poke(C0[i], k1);
                    d[j] = poke(D0[j], k2);
                    let sc = score(&rows, &c, &d, m210);
                    if better(sc, besth) {
                        besth = sc;
                        labh = format!("0x210 C[{i}]{k1:+} D[{j}]{k2:+}");
                        println!("HIT {labh} {}", fmt(sc));
                    }
                }
            }
        }
    }
    println!("best 0x210+ham {labh} {}", fmt(besth));
}
