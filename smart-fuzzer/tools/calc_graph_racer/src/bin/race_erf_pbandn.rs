//! ERF witness counts by z band. Not an identity.
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{WitnessArg, WitnessSet};
use std::collections::BTreeMap;
use std::env;
use std::fs;

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
            if x > 0.0 {
                map.entry(x.to_bits()).or_insert(e.to_bits());
            }
        }
    }
    let mut n1 = 0usize;
    let mut n2 = 0usize;
    let mut n3 = 0usize;
    for (b, _) in &map {
        let z = f64::from_bits(*b);
        if z < 0.5 {
            n1 += 1;
        } else if z < 4.0 {
            n2 += 1;
        } else {
            n3 += 1;
        }
    }
    println!(
        "ERF z>0 n={}  [0,0.5)={n1}  [0.5,4)={n2}  z>=4={n3}",
        map.len()
    );
}
