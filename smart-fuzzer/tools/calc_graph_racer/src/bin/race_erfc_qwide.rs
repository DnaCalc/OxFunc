//! Wide one-coeff ULP landscape and keep-descent on SPECFUN C/D as Q=w*F.
//! Not ham2 restatement. Not an identity.
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
    sum: u128,
    dmid: usize,
    max_ulp: u64,
}
fn score_q(rows: &[f::QRow], c: &[f64; 9], d: &[f64; 8]) -> Mid {
    let mut exact = 0usize;
    let mut sum = 0u128;
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
            sum += dist as u128;
            max_ulp = max_ulp.max(dist);
        }
    }
    Mid {
        exact,
        sum,
        dmid,
        max_ulp,
    }
}
fn better(a: Mid, b: Mid) -> bool {
    a.exact > b.exact
        || (a.exact == b.exact && a.dmid > b.dmid)
        || (a.exact == b.exact && a.dmid == b.dmid && a.sum < b.sum)
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    let base = score_q(&rows, &C0, &D0);
    println!(
        "CR mask0 Q-mid {} dmid={} max={} sum={}",
        base.exact, base.dmid, base.max_ulp, base.sum
    );

    println!("## one-coeff k=-32..=32");
    for i in 0..9 {
        let mut c = C0;
        let mut best_k = 0i32;
        let mut best = base;
        for k in -32i32..=32 {
            if k == 0 {
                continue;
            }
            c[i] = poke(C0[i], k);
            let sc = score_q(&rows, &c, &D0);
            if better(sc, best) {
                best = sc;
                best_k = k;
            }
        }
        println!(
            "C[{i}] best k={best_k:+} exact={} dmid={} max={} (dExact={:+})",
            best.exact,
            best.dmid,
            best.max_ulp,
            best.exact as i32 - base.exact as i32
        );
    }
    for i in 0..8 {
        let mut d = D0;
        let mut best_k = 0i32;
        let mut best = base;
        for k in -32i32..=32 {
            if k == 0 {
                continue;
            }
            d[i] = poke(D0[i], k);
            let sc = score_q(&rows, &C0, &d);
            if better(sc, best) {
                best = sc;
                best_k = k;
            }
        }
        println!(
            "D[{i}] best k={best_k:+} exact={} dmid={} max={} (dExact={:+})",
            best.exact,
            best.dmid,
            best.max_ulp,
            best.exact as i32 - base.exact as i32
        );
    }

    println!("## keep-descent from CR, |k| in 1..=16");
    let mut c = C0;
    let mut d = D0;
    let mut best = base;
    let mut deltas: Vec<i32> = Vec::new();
    for k in 1..=16 {
        deltas.push(k);
        deltas.push(-k);
    }
    loop {
        let mut moved = false;
        for i in 0..9 {
            for &k in &deltas {
                let old = c[i];
                c[i] = poke(old, k);
                let sc = score_q(&rows, &c, &d);
                if better(sc, best) {
                    best = sc;
                    moved = true;
                    println!(
                        "  keep C[{i}] {k:+} exact={} dmid={} max={}",
                        sc.exact, sc.dmid, sc.max_ulp
                    );
                } else {
                    c[i] = old;
                }
            }
        }
        for i in 0..8 {
            for &k in &deltas {
                let old = d[i];
                d[i] = poke(old, k);
                let sc = score_q(&rows, &c, &d);
                if better(sc, best) {
                    best = sc;
                    moved = true;
                    println!(
                        "  keep D[{i}] {k:+} exact={} dmid={} max={}",
                        sc.exact, sc.dmid, sc.max_ulp
                    );
                } else {
                    d[i] = old;
                }
            }
        }
        if !moved {
            break;
        }
    }
    println!(
        "wide joint Q-mid {} dmid={} max={} sum={}",
        best.exact, best.dmid, best.max_ulp, best.sum
    );
    for i in 0..9 {
        if c[i].to_bits() != C0[i].to_bits() {
            println!("  C[{i}] {:016x}", c[i].to_bits());
        }
    }
    for i in 0..8 {
        if d[i].to_bits() != D0[i].to_bits() {
            println!("  D[{i}] {:016x}", d[i].to_bits());
        }
    }
}
