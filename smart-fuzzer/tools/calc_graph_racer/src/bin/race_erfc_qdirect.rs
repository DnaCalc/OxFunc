//! Direct-only Q-mid: HW=1,2 by dmid, named poke tables, miss bands.
//! Implied-Q miss-1 is noise. Not an identity.
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
    max_all: u64,
    m1d: usize,
    dn: usize,
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
        s.max_all = s.max_all.max(dist);
        if dist == 0 {
            s.exact += 1;
        }
        if r.direct {
            s.dn += 1;
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
fn better_d(a: Sc, b: Sc) -> bool {
    a.dmid > b.dmid || (a.dmid == b.dmid && a.exact > b.exact) || (a.dmid == b.dmid && a.exact == b.exact && a.max_d < b.max_d)
}
fn fmt(s: Sc) -> String {
    format!(
        "exact={} dmid={}/{} maxd={} maxa={} m1d={}",
        s.exact, s.dmid, s.dn, s.max_d, s.max_all, s.m1d
    )
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);

    let mut ham = C0;
    ham[4] = poke(C0[4], -1);
    let mut hamd = D0;
    hamd[4] = poke(D0[4], -1);
    let mut j = ham;
    j[0] = poke(C0[0], 4);
    j[1] = poke(C0[1], 1);
    let mut jd = hamd;
    jd[0] = poke(D0[0], -1);

    let named: [(&str, [f64; 9], [f64; 8], u32); 6] = [
        ("CR mask0", C0, D0, 0),
        ("CR 0x74", C0, D0, 0x74),
        ("CR 0x210", C0, D0, 0x210),
        ("CR 0x200", C0, D0, 0x200),
        ("ham2 C4-1 D4-1", ham, hamd, 0),
        ("j3432", j, jd, 0),
    ];
    println!("## named");
    for (name, c, d, m) in named {
        let sc = score(&rows, &c, &d, m);
        println!("{name:22} {}", fmt(sc));
    }

    let base = score(&rows, &C0, &D0, 0);
    println!("## HW=1 by dmid");
    let mut best1 = base;
    let mut lab1 = 0u32;
    for b in 0..NSITE {
        let m = 1u32 << b;
        let sc = score(&rows, &C0, &D0, m);
        if better_d(sc, best1) {
            best1 = sc;
            lab1 = m;
            println!("HIT bit={b} mask={m:#x} {}", fmt(sc));
        }
    }
    println!("best HW1 mask={lab1:#x} {}", fmt(best1));

    println!("## HW=2 by dmid");
    let mut best2 = best1;
    let mut lab2 = lab1;
    for i in 0..NSITE {
        for jbit in (i + 1)..NSITE {
            let m = (1u32 << i) | (1u32 << jbit);
            let sc = score(&rows, &C0, &D0, m);
            if better_d(sc, best2) {
                best2 = sc;
                lab2 = m;
                println!("HIT bits={i},{jbit} mask={lab2:#x} {}", fmt(sc));
            }
        }
    }
    println!("best HW2 mask={lab2:#x} {}", fmt(best2));

    println!("## CR mask0 direct misses by band / ulp");
    let mut band_n = [0usize; 3];
    let mut band_m = [0usize; 3];
    let mut ulph = [0usize; 6];
    for r in &rows {
        if !r.direct || r.z < 0.5 || r.z >= 4.0 {
            continue;
        }
        let b = if r.z < 1.0 {
            0
        } else if r.z < 2.0 {
            1
        } else {
            2
        };
        band_n[b] += 1;
        let qg = f::w_rn53(r.z) * cody(r.z, &C0, &D0, 0);
        let d = ulp_distance(qg, f64::from_bits(r.qbits)).unwrap_or(99);
        if d != 0 {
            band_m[b] += 1;
            let u = d.min(5) as usize;
            ulph[u] += 1;
        }
    }
    println!(
        "[0.5,1) miss {}/{}  [1,2) {}/{}  [2,4) {}/{}",
        band_m[0], band_n[0], band_m[1], band_n[1], band_m[2], band_n[2]
    );
    println!(
        "direct-miss ulp1..5+ = {} {} {} {} {}",
        ulph[1], ulph[2], ulph[3], ulph[4], ulph[5]
    );
}
