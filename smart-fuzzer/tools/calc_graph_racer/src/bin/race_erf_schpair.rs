//! Schonfelder P-side pair grid and descent from a1-1 / a2+5 basins.
//! Not an identity.
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{ulp_distance, WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_from_f64, ext_mul, ext_sub, ext_to_f64, Ext80, CW_PC64_RN};
use std::collections::BTreeMap;
use std::env;
use std::fs;

const CW: u16 = CW_PC64_RN;
const ULP_CAP: u64 = 1 << 20;
const A0: [f64; 18] = [
    1.4831105640848035818894480790578,
    -0.3010710733865949424707310463118,
    0.0689948306898315662466031807188,
    -0.0139162712647221876825465256678,
    0.0024207995224334636628916782398,
    -0.0003658639685848086446493825778,
    4.86209844323190482828875688e-5,
    -5.7492565580356848350542158e-6,
    6.113243578434764697067588e-7,
    -5.89910153129584343908468e-8,
    5.2070090920686482404558e-9,
    -4.232975879965543268108e-10,
    3.18811350664917497488e-11,
    -2.2361550188326842738e-12,
    1.467329847991084928e-13,
    -9.0440019853817478e-15,
    5.254813715470928e-16,
    -2.88742612228498e-17,
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
fn erf_sch(z: f64, a: &[f64; 18]) -> f64 {
    let xe = ef(z.abs());
    let t = ext_sub(&ext_mul(&ef(0.5), &ext_mul(&xe, &xe, CW), CW), &ef(1.0), CW);
    let two = ef(2.0);
    let mut d1 = ef(0.0);
    let mut d2 = ef(0.0);
    for k in (1..a.len()).rev() {
        let dk = ext_add(
            &ext_sub(&ext_mul(&ext_mul(&two, &t, CW), &d1, CW), &d2, CW),
            &ef(a[k]),
            CW,
        );
        d2 = d1;
        d1 = dk;
    }
    let y = ext_add(
        &ext_sub(&ext_mul(&t, &d1, CW), &d2, CW),
        &ext_mul(&ef(0.5), &ef(a[0]), CW),
        CW,
    );
    ext_to_f64(&ext_mul(&xe, &y, CW), CW)
}
fn score(rows: &[(f64, u64)], a: &[f64; 18]) -> (usize, u64, u128) {
    let mut ex = 0usize;
    let mut mx = 0u64;
    let mut sum = 0u128;
    for &(z, pbits) in rows {
        let pg = erf_sch(z, a);
        let d = ulp_distance(pg, f64::from_bits(pbits)).unwrap_or(u64::MAX);
        if d > ULP_CAP {
            continue;
        }
        if d == 0 {
            ex += 1;
        } else {
            mx = mx.max(d);
            sum += d as u128;
        }
    }
    (ex, mx, sum)
}
fn better(a: (usize, u64, u128), b: (usize, u64, u128)) -> bool {
    a.0 > b.0 || (a.0 == b.0 && a.1 < b.1) || (a.0 == b.0 && a.1 == b.1 && a.2 < b.2)
}

fn load_p(dir: &str) -> Vec<(f64, u64)> {
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
    map.into_values().collect()
}

fn descent(rows: &[(f64, u64)], mut a: [f64; 18], mut best: (usize, u64, u128)) -> ([f64; 18], (usize, u64, u128)) {
    let mut deltas = Vec::new();
    for k in 1..=8i32 {
        deltas.push(k);
        deltas.push(-k);
    }
    loop {
        let mut moved = false;
        for i in 0..18 {
            for &k in &deltas {
                let old = a[i];
                a[i] = poke(old, k);
                let sc = score(rows, &a);
                if better(sc, best) {
                    best = sc;
                    moved = true;
                    println!("  keep a[{i}] {k:+} {} max={} sum={}", sc.0, sc.1, sc.2);
                } else {
                    a[i] = old;
                }
            }
        }
        if !moved {
            break;
        }
    }
    (a, best)
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = load_p(&dir);
    let base = score(&rows, &A0);
    println!("mask0 {} max={} sum={}", base.0, base.1, base.2);

    println!("## a[1] x a[2] k=-8..=8");
    let mut bp = base;
    let mut lab = String::from("CR");
    for k1 in -8i32..=8 {
        for k2 in -8i32..=8 {
            if k1 == 0 && k2 == 0 {
                continue;
            }
            let mut a = A0;
            a[1] = poke(A0[1], k1);
            a[2] = poke(A0[2], k2);
            let sc = score(&rows, &a);
            if better(sc, bp) {
                bp = sc;
                lab = format!("a1{k1:+} a2{k2:+}");
                println!("HIT {lab} {} max={}", sc.0, sc.1);
            }
        }
    }
    println!("best pair {lab} {} max={}", bp.0, bp.1);

    println!("## descent from a[1]-1");
    let mut a = A0;
    a[1] = poke(A0[1], -1);
    let s = score(&rows, &a);
    println!("start a1-1 {} max={}", s.0, s.1);
    let (_, b1) = descent(&rows, a, s);
    println!("from a1-1 {} max={} sum={}", b1.0, b1.1, b1.2);

    println!("## descent from a[2]+5");
    let mut a = A0;
    a[2] = poke(A0[2], 5);
    let s = score(&rows, &a);
    println!("start a2+5 {} max={}", s.0, s.1);
    let (_, b2) = descent(&rows, a, s);
    println!("from a2+5 {} max={} sum={}", b2.0, b2.1, b2.2);
}
