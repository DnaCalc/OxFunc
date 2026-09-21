//! ERF z≥4 vs 1−Q of cephes_f fused ∪ F±. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{ulp_distance, WitnessArg, WitnessSet};
use std::collections::BTreeMap;
use std::env;
use std::fs;

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
fn qmul(w: f64, ff: f64) -> f64 {
    let v = w * ff;
    if v.abs() < f64::MIN_POSITIVE {
        0.0
    } else {
        v
    }
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
            if x >= 4.0 {
                map.entry(x.to_bits()).or_insert(e.to_bits());
            }
        }
    }
    let rows: Vec<_> = map
        .into_iter()
        .map(|(b, e)| (f64::from_bits(b), e))
        .collect();
    let mut n0 = 0usize;
    let mut nls = 0usize;
    let mut n_u = 0usize;
    let mut miss = 0usize;
    println!("ERF z>=4 1−Q cephes_f ∪ F± n={}", rows.len());
    for &(z, bits) in &rows {
        let t = f64::from_bits(bits);
        let w = f::w_rn53(z);
        let ff = f::cephes_f(z);
        let p0 = 1.0 - qmul(w, ff);
        let e0 = ulp_distance(p0, t).unwrap_or(99) == 0;
        let els = (-5i32..=5).filter(|&k| k != 0).any(|k| {
            ulp_distance(1.0 - qmul(w, poke(ff, k)), t).unwrap_or(99) == 0
        });
        if e0 {
            n0 += 1;
        }
        if els {
            nls += 1;
        }
        if e0 || els {
            n_u += 1;
        } else {
            miss += 1;
            if miss <= 8 {
                println!(
                    "  MISS z={z:.16} 1-Q={}{}",
                    ulp_distance(p0, t).unwrap_or(99),
                    if p0 < t { "L" } else { "H" }
                );
            }
        }
    }
    println!(
        "fused-1Q={n0} last-store-1Q={nls} union={n_u}/{} miss={miss}",
        rows.len()
    );
}
