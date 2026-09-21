//! Pair/triple ULP grids on C/D coeffs that actually move Q-exact.
//! Not a 24-bit cube. Not an identity.
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
fn cody_f(y: f64, c: &[f64; 9], d: &[f64; 8]) -> f64 {
    let ye = ef(y.abs());
    let mut xnum = ext_mul(&ef(c[8]), &ye, CW);
    let mut xden = ye;
    for i in 0..7 {
        xnum = ext_mul(&ext_add(&xnum, &ef(c[i]), CW), &ye, CW);
        xden = ext_mul(&ext_add(&xden, &ef(d[i]), CW), &ye, CW);
    }
    ext_to_f64(
        &ext_div(&ext_add(&xnum, &ef(c[7]), CW), &ext_add(&xden, &ef(d[7]), CW), CW),
        CW,
    )
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
#[derive(Clone, Copy)]
struct Mid {
    exact: usize,
    dmid: usize,
    max_ulp: u64,
}
fn score_q(rows: &[f::QRow], c: &[f64; 9], d: &[f64; 8]) -> Mid {
    let mut exact = 0usize;
    let mut dmid = 0usize;
    let mut max_ulp = 0u64;
    for r in rows {
        if r.z < 0.5 || r.z >= 4.0 {
            continue;
        }
        let qg = f::w_rn53(r.z) * cody_f(r.z, c, d);
        if !qg.is_finite() {
            continue;
        }
        let dist = ulp_distance(qg, f64::from_bits(r.qbits)).unwrap_or(u64::MAX);
        if dist > ULP_CAP {
            continue;
        }
        if dist == 0 {
            exact += 1;
            if r.direct {
                dmid += 1;
            }
        } else {
            max_ulp = max_ulp.max(dist);
        }
    }
    Mid {
        exact,
        dmid,
        max_ulp,
    }
}
fn better(a: Mid, b: Mid) -> bool {
    a.exact > b.exact || (a.exact == b.exact && a.dmid > b.dmid)
}

#[derive(Clone, Copy)]
enum Which {
    C(usize),
    D(usize),
}
fn apply(c: &mut [f64; 9], d: &mut [f64; 8], w: Which, k: i32) {
    match w {
        Which::C(i) => c[i] = poke(C0[i], k),
        Which::D(i) => d[i] = poke(D0[i], k),
    }
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    let base = score_q(&rows, &C0, &D0);
    println!(
        "CR {} dmid={} max={}",
        base.exact, base.dmid, base.max_ulp
    );

    let names: [(&str, Which); 7] = [
        ("C0", Which::C(0)),
        ("C1", Which::C(1)),
        ("C4", Which::C(4)),
        ("D0", Which::D(0)),
        ("D1", Which::D(1)),
        ("D4", Which::D(4)),
        ("D7", Which::D(7)),
    ];
    let lo = -8i32;
    let hi = 8i32;
    let mut best = base;
    let mut lab = String::from("CR");
    println!("## pairs k={lo}..={hi}");
    for i in 0..names.len() {
        for j in (i + 1)..names.len() {
            let mut local = base;
            let mut local_lab = String::new();
            for k1 in lo..=hi {
                for k2 in lo..=hi {
                    if k1 == 0 && k2 == 0 {
                        continue;
                    }
                    let mut c = C0;
                    let mut d = D0;
                    apply(&mut c, &mut d, names[i].1, k1);
                    apply(&mut c, &mut d, names[j].1, k2);
                    let sc = score_q(&rows, &c, &d);
                    if better(sc, local) {
                        local = sc;
                        local_lab = format!("{}{:+} {}{:+}", names[i].0, k1, names[j].0, k2);
                    }
                    if better(sc, best) {
                        best = sc;
                        lab = local_lab.clone();
                        println!(
                            "HIT {lab} exact={} dmid={} max={}",
                            sc.exact, sc.dmid, sc.max_ulp
                        );
                    }
                }
            }
            println!(
                "  pair {} {} best {} exact={} dmid={}",
                names[i].0, names[j].0, local_lab, local.exact, local.dmid
            );
        }
    }

    println!("## triple C0,D0,D7 k=-6..=6");
    for k0 in -6i32..=6 {
        for k1 in -6i32..=6 {
            for k2 in -6i32..=6 {
                if k0 == 0 && k1 == 0 && k2 == 0 {
                    continue;
                }
                let mut c = C0;
                let mut d = D0;
                c[0] = poke(C0[0], k0);
                d[0] = poke(D0[0], k1);
                d[7] = poke(D0[7], k2);
                let sc = score_q(&rows, &c, &d);
                if better(sc, best) {
                    best = sc;
                    lab = format!("C0{k0:+} D0{k1:+} D7{k2:+}");
                    println!(
                        "HIT {lab} exact={} dmid={} max={}",
                        sc.exact, sc.dmid, sc.max_ulp
                    );
                }
            }
        }
    }
    println!(
        "best {lab} exact={} dmid={} max={}",
        best.exact, best.dmid, best.max_ulp
    );
}
