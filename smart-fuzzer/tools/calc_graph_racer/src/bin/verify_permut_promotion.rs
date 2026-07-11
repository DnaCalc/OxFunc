//! W109 PERMUT promotion verifier: replay answered oracle witnesses through
//! the PRODUCTION `permut_kernel`. Pass answered WitnessSet JSONs.

use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{WitnessArg, WitnessSet};
use oxfunc_core::functions::permut_fn::permut_kernel;
use std::collections::HashSet;

fn main() {
    let files: Vec<String> = std::env::args().skip(1).collect();
    assert!(!files.is_empty(), "pass answered WitnessSet JSON files");
    let (mut numeric, mut exact) = (0u32, 0u32);
    let mut seen: HashSet<String> = HashSet::new();
    let mut failures: Vec<String> = Vec::new();
    for file in &files {
        let ws: WitnessSet =
            serde_json::from_str(&std::fs::read_to_string(file).expect("read")).expect("parse");
        assert_eq!(ws.function.to_uppercase(), "PERMUT");
        for w in &ws.witnesses {
            let key = serde_json::to_string(&w.args).unwrap();
            if !seen.insert(key) {
                continue;
            }
            let vals: Vec<f64> = w
                .args
                .iter()
                .map(|a| match a {
                    WitnessArg::Scalar(s) => parse_bits_hex(s).unwrap(),
                    _ => panic!("scalar args expected"),
                })
                .collect();
            if let Some(expected) = parse_bits_hex(&w.expected_bits) {
                numeric += 1;
                match permut_kernel(vals[0], vals[1]) {
                    Ok(v) if v.to_bits() == expected.to_bits() => exact += 1,
                    other => {
                        if failures.len() < 20 {
                            failures.push(format!(
                                "{:?}: expected {} got {:?}",
                                w.id, w.expected_bits, other
                            ));
                        }
                    }
                }
            }
        }
    }
    println!("numeric: {exact}/{numeric} bit-exact");
    if !failures.is_empty() {
        for f in &failures {
            println!("  {f}");
        }
        std::process::exit(1);
    }
    println!("PROMOTION VERIFIED");
}
