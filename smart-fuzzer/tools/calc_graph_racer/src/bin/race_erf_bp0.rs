//! Boost53 P[0] +1..+8 vs 4 LOW. Not an identity.
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
fn b53(z: f64, p: &[f64; 5]) -> f64 {
    let ze = ef(z);
    let zz = ext_mul(&ze, &ze, CW);
    let r = ext_div(&horner(p, zz), &horner(&Q53, zz), CW);
    ext_to_f64(&ext_mul(&ze, &ext_add(&ef(Y53), &r, CW), CW), CW)
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
    println!("Boost53 P[0] steps (bar CR 563 / joint 866):");
    for k in 0i32..=16 {
        let mut p = P53;
        p[0] = poke(P53[0], k);
        let mut keep = 0usize;
        let mut hit = 0usize;
        for &(z, bits) in &rows {
            if ulp_distance(b53(z, &p), f64::from_bits(bits)).unwrap_or(99) == 0 {
                keep += 1;
            }
        }
        for &lz in &LOWS {
            let Some((_, bits)) = rows.iter().find(|(z, _)| (*z - lz).abs() < 1e-14) else {
                continue;
            };
            if ulp_distance(b53(lz, &p), f64::from_bits(*bits)).unwrap_or(99) == 0 {
                hit += 1;
            }
        }
        println!("  P[0] {k:+} keep={keep} hit={hit}/4");
    }
}
