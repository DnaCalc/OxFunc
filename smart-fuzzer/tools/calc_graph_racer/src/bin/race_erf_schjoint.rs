//! Schonfelder Table 1 P-side: one-coeff ±16, keep-descent, Clenshaw HW=1,2.
//! Not an identity. Do not land 680.
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
const NSITE: u32 = 20;

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
fn erf_sch(z: f64, a: &[f64; 18], mask: u32) -> f64 {
    let xe = ef(z.abs());
    let mut t = ext_sub(&ext_mul(&ef(0.5), &ext_mul(&xe, &xe, CW), CW), &ef(1.0), CW);
    t = maybe(t, mask, 0);
    let two = ef(2.0);
    let mut d1 = ef(0.0);
    let mut d2 = ef(0.0);
    for k in (1..a.len()).rev() {
        let mut dk = ext_add(
            &ext_sub(&ext_mul(&ext_mul(&two, &t, CW), &d1, CW), &d2, CW),
            &ef(a[k]),
            CW,
        );
        let bit = 1 + (a.len() - 1 - k) as u32;
        dk = maybe(dk, mask, bit);
        d2 = d1;
        d1 = dk;
    }
    let mut y = ext_add(
        &ext_sub(&ext_mul(&t, &d1, CW), &d2, CW),
        &ext_mul(&ef(0.5), &ef(a[0]), CW),
        CW,
    );
    y = maybe(y, mask, 18);
    let mut p = ext_mul(&xe, &y, CW);
    p = maybe(p, mask, 19);
    ext_to_f64(&p, CW)
}

#[derive(Clone, Copy, Default)]
struct Sc {
    exact: usize,
    n: usize,
    max_ulp: u64,
    sum: u128,
}
fn score(rows: &[(f64, u64)], a: &[f64; 18], mask: u32) -> Sc {
    let mut s = Sc::default();
    for &(z, pbits) in rows {
        let pg = erf_sch(z, a, mask);
        if !pg.is_finite() {
            continue;
        }
        let d = ulp_distance(pg, f64::from_bits(pbits)).unwrap_or(u64::MAX);
        if d > ULP_CAP {
            continue;
        }
        s.n += 1;
        if d == 0 {
            s.exact += 1;
        } else {
            s.max_ulp = s.max_ulp.max(d);
            s.sum += d as u128;
        }
    }
    s
}
fn better(a: Sc, b: Sc) -> bool {
    a.exact > b.exact
        || (a.exact == b.exact && a.max_ulp < b.max_ulp)
        || (a.exact == b.exact && a.max_ulp == b.max_ulp && a.sum < b.sum)
}
fn fmt(s: Sc) -> String {
    format!("{}/{} max={} sum={}", s.exact, s.n, s.max_ulp, s.sum)
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

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = load_p(&dir);
    let base = score(&rows, &A0, 0);
    println!("Schonfelder mask0 {}", fmt(base));

    println!("## one-coeff k=-16..=16");
    let mut best1 = base;
    let mut lab1 = String::from("CR");
    for i in 0..18 {
        for k in -16i32..=16 {
            if k == 0 {
                continue;
            }
            let mut a = A0;
            a[i] = poke(A0[i], k);
            let sc = score(&rows, &a, 0);
            if better(sc, best1) {
                best1 = sc;
                lab1 = format!("a[{i}]{k:+}");
                println!("HIT {lab1} {}", fmt(sc));
            }
        }
    }
    println!("best one-coeff {lab1} {}", fmt(best1));

    println!("## keep-descent |k|<=8");
    let mut a = A0;
    let mut best = base;
    let mut deltas = Vec::new();
    for k in 1..=8 {
        deltas.push(k);
        deltas.push(-k);
    }
    loop {
        let mut moved = false;
        for i in 0..18 {
            for &k in &deltas {
                let old = a[i];
                a[i] = poke(old, k);
                let sc = score(&rows, &a, 0);
                if better(sc, best) {
                    best = sc;
                    moved = true;
                    println!("  keep a[{i}] {k:+} {}", fmt(sc));
                } else {
                    a[i] = old;
                }
            }
        }
        if !moved {
            break;
        }
    }
    println!("joint {}", fmt(best));
    for i in 0..18 {
        if a[i].to_bits() != A0[i].to_bits() {
            println!("  a[{i}] {:016x}", a[i].to_bits());
        }
    }

    println!("## HW=1 stores");
    let mut besth = base;
    let mut labh = 0u32;
    for b in 0..NSITE {
        let m = 1u32 << b;
        let sc = score(&rows, &A0, m);
        if better(sc, besth) {
            besth = sc;
            labh = m;
            println!("HIT bit={b} mask={m:#x} {}", fmt(sc));
        }
    }
    println!("best HW1 mask={labh:#x} {}", fmt(besth));

    println!("## HW=2 stores");
    let mut best2 = besth;
    let mut lab2 = labh;
    for i in 0..NSITE {
        for j in (i + 1)..NSITE {
            let m = (1u32 << i) | (1u32 << j);
            let sc = score(&rows, &A0, m);
            if better(sc, best2) {
                best2 = sc;
                lab2 = m;
                println!("HIT bits={i},{j} mask={m:#x} {}", fmt(sc));
            }
        }
    }
    println!("best HW2 mask={lab2:#x} {}", fmt(best2));
}
