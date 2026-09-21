//! Boost53 / Boost1.35 ±1 keep-and-hit vs 4 LOW. Not an identity.
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{ulp_distance, WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC64_RN};
use std::collections::BTreeMap;
use std::env;
use std::fs;

const CW: u16 = CW_PC64_RN;
const Y53: f64 = 1.044948577880859375;
const P53: [f64; 5] = [
    0.0834305892146531832907,
    -0.338165134459360935041,
    -0.0509990735146777432841,
    -0.00772758345802133288487,
    -0.000322780120964605683831,
];
const Q53: [f64; 5] = [
    1.0,
    0.455004033050794024546,
    0.0875222600142252549554,
    0.00858571925074406212772,
    0.000370900071787748000569,
];
const N135: [f64; 7] = [
    0.00337916709551257778174,
    -0.000147024115786688745475,
    -0.37463022236812520164,
    0.0163061594494816999803,
    -0.0534354147807331748737,
    0.00161898096813581982844,
    -0.0059528010489182840404,
];
const D135: [f64; 7] = [
    1.0,
    -0.0435089806536379531594,
    0.442761965043509204727,
    -0.017375974533016704678,
    0.0772756490303260060769,
    -0.00210552465858669941879,
    0.00544772980263244037286,
];
const LOWS: [f64; 4] = [
    0.23046875,
    0.37109375,
    0.4524739583333333,
    0.4716796875,
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
fn horner(cs: &[f64], x: Ext80) -> Ext80 {
    let mut acc = ef(0.0);
    for &c in cs.iter().rev() {
        acc = ext_add(&ext_mul(&acc, &x, CW), &ef(c), CW);
    }
    acc
}
fn b53(z: f64, y: f64, p: &[f64; 5], q: &[f64; 5]) -> f64 {
    let ze = ef(z);
    let zz = ext_mul(&ze, &ze, CW);
    let r = ext_div(&horner(p, zz), &horner(q, zz), CW);
    ext_to_f64(&ext_mul(&ze, &ext_add(&ef(y), &r, CW), CW), CW)
}
fn b135(z: f64, n: &[f64; 7], d: &[f64; 7]) -> f64 {
    let ze = ef(z);
    let r = ext_div(&horner(n, ze), &horner(d, ze), CW);
    ext_to_f64(&ext_mul(&ze, &ext_add(&ef(1.125), &r, CW), CW), CW)
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
            let Some(e) = parse_bits_hex(&w.expected_bits) else {
                continue;
            };
            if x > 0.0 && x < 0.5 {
                map.entry(x.to_bits()).or_insert(e.to_bits());
            }
        }
    }
    let rows: Vec<_> = map
        .into_iter()
        .map(|(b, e)| (f64::from_bits(b), e))
        .collect();
    let keep53 = |y: f64, p: &[f64; 5], q: &[f64; 5]| -> usize {
        rows.iter()
            .filter(|&&(z, bits)| ulp_distance(b53(z, y, p, q), f64::from_bits(bits)).unwrap_or(99) == 0)
            .count()
    };
    let hit53 = |y: f64, p: &[f64; 5], q: &[f64; 5]| -> usize {
        LOWS.iter()
            .filter(|&&lz| {
                rows.iter().any(|(z, bits)| {
                    (*z - lz).abs() < 1e-14
                        && ulp_distance(b53(lz, y, p, q), f64::from_bits(*bits)).unwrap_or(99) == 0
                })
            })
            .count()
    };
    let keep135 = |n: &[f64; 7], d: &[f64; 7]| -> usize {
        rows.iter()
            .filter(|&&(z, bits)| ulp_distance(b135(z, n, d), f64::from_bits(bits)).unwrap_or(99) == 0)
            .count()
    };
    let hit135 = |n: &[f64; 7], d: &[f64; 7]| -> usize {
        LOWS.iter()
            .filter(|&&lz| {
                rows.iter().any(|(z, bits)| {
                    (*z - lz).abs() < 1e-14
                        && ulp_distance(b135(lz, n, d), f64::from_bits(*bits)).unwrap_or(99) == 0
                })
            })
            .count()
    };
    println!("4 LOW Boost53 / 135 CR:");
    for &lz in &LOWS {
        let Some((_, bits)) = rows.iter().find(|(z, _)| (*z - lz).abs() < 1e-14) else {
            continue;
        };
        let t = f64::from_bits(*bits);
        let g1 = b53(lz, Y53, &P53, &Q53);
        let g2 = b135(lz, &N135, &D135);
        let d1 = ulp_distance(g1, t).unwrap_or(99);
        let d2 = ulp_distance(g2, t).unwrap_or(99);
        let s1 = if g1 < t { "L" } else if g1 > t { "H" } else { "=" };
        let s2 = if g2 < t { "L" } else if g2 > t { "H" } else { "=" };
        println!("  z={lz:.16} b53={d1}{s1} b135={d2}{s2}");
    }
    let k53 = keep53(Y53, &P53, &Q53);
    let h53 = hit53(Y53, &P53, &Q53);
    println!("Boost53 CR keep={k53} hit={h53}/4");
    println!("Boost53 ±1 (print hit>0 or keep>=k53):");
    let mut best = format!("CR53 {k53}");
    let mut best_k = k53;
    let mut best_h = h53;
    for k in [-1i32, 1] {
        let y = poke(Y53, k);
        let kk = keep53(y, &P53, &Q53);
        let h = hit53(y, &P53, &Q53);
        if h > 0 || kk >= k53 {
            println!("  Y {k:+} keep={kk} hit={h}/4");
        }
        if h > best_h || (h == best_h && kk > best_k) {
            best_h = h;
            best_k = kk;
            best = format!("Y {k:+}");
        }
    }
    for i in 0..5 {
        for k in [-1i32, 1] {
            let mut p = P53;
            p[i] = poke(P53[i], k);
            let kk = keep53(Y53, &p, &Q53);
            let h = hit53(Y53, &p, &Q53);
            if h > 0 || kk >= k53 {
                println!("  P[{i}] {k:+} keep={kk} hit={h}/4");
            }
            if h > best_h || (h == best_h && kk > best_k) {
                best_h = h;
                best_k = kk;
                best = format!("P[{i}] {k:+}");
            }
        }
        for k in [-1i32, 1] {
            let mut q = Q53;
            q[i] = poke(Q53[i], k);
            let kk = keep53(Y53, &P53, &q);
            let h = hit53(Y53, &P53, &q);
            if h > 0 || kk >= k53 {
                println!("  Q[{i}] {k:+} keep={kk} hit={h}/4");
            }
            if h > best_h || (h == best_h && kk > best_k) {
                best_h = h;
                best_k = kk;
                best = format!("Q[{i}] {k:+}");
            }
        }
    }
    let k135 = keep135(&N135, &D135);
    let h135 = hit135(&N135, &D135);
    println!("Boost135 CR keep={k135} hit={h135}/4");
    println!("Boost135 ±1 (print hit>0 or keep>=k135):");
    for i in 0..7 {
        for k in [-1i32, 1] {
            let mut n = N135;
            n[i] = poke(N135[i], k);
            let kk = keep135(&n, &D135);
            let h = hit135(&n, &D135);
            if h > 0 || kk >= k135 {
                println!("  N[{i}] {k:+} keep={kk} hit={h}/4");
            }
            if h > best_h || (h == best_h && kk > best_k) {
                best_h = h;
                best_k = kk;
                best = format!("N[{i}] {k:+}");
            }
        }
        for k in [-1i32, 1] {
            let mut d = D135;
            d[i] = poke(D135[i], k);
            let kk = keep135(&N135, &d);
            let h = hit135(&N135, &d);
            if h > 0 || kk >= k135 {
                println!("  D[{i}] {k:+} keep={kk} hit={h}/4");
            }
            if h > best_h || (h == best_h && kk > best_k) {
                best_h = h;
                best_k = kk;
                best = format!("D[{i}] {k:+}");
            }
        }
    }
    println!("best-by-hit {best} keep={best_k} hit={best_h}");
}
