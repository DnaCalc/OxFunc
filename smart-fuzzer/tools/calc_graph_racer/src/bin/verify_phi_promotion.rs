//! W109 PHI promotion verifier over answered oracle witness sets.

use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{WitnessArg, WitnessSet};
use oxfunc_core::functions::normal_dist_common::phi_kernel;
use std::collections::HashSet;

fn main() {
    let files: Vec<String> = std::env::args().skip(1).collect();
    let (mut numeric, mut exact) = (0u32, 0u32);
    let mut seen: HashSet<String> = HashSet::new();
    let mut failures = Vec::new();
    for file in &files {
        let ws: WitnessSet =
            serde_json::from_str(&std::fs::read_to_string(file).expect("read")).expect("parse");
        assert_eq!(ws.function.to_uppercase(), "PHI");
        for w in &ws.witnesses {
            let key = serde_json::to_string(&w.args).unwrap();
            if !seen.insert(key) {
                continue;
            }
            let x = match &w.args[0] {
                WitnessArg::Scalar(s) => parse_bits_hex(s).unwrap(),
                _ => continue,
            };
            let Some(expected) = parse_bits_hex(&w.expected_bits) else {
                continue;
            };
            numeric += 1;
            let v = phi_kernel(x);
            if v.to_bits() == expected.to_bits() {
                exact += 1;
            } else if failures.len() < 15 {
                failures.push(format!(
                    "{:?}: x bits {:?} expected {} got 0x{:016x}",
                    w.id,
                    w.args[0],
                    w.expected_bits,
                    v.to_bits()
                ));
            }
        }
    }
    println!("numeric: {exact}/{numeric} bit-exact");
    for f in &failures {
        println!("  {f}");
    }
    if !failures.is_empty() {
        std::process::exit(1);
    }
    println!("PROMOTION VERIFIED");
}
