//! W109 trig promotion verifier: replay every answered oracle witness for
//! SIN/COS/TAN/COT/SEC/CSC through the PRODUCTION kernels.

use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{WitnessArg, WitnessSet};
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
        let func = ws.function.to_uppercase();
        for w in &ws.witnesses {
            let key = format!("{func}|{}", serde_json::to_string(&w.args).unwrap());
            if !seen.insert(key) {
                continue;
            }
            let x = match &w.args[0] {
                WitnessArg::Scalar(s) => parse_bits_hex(s).unwrap(),
                _ => panic!("scalar arg expected"),
            };
            let Some(expected) = parse_bits_hex(&w.expected_bits) else {
                continue; // error rows (none expected in these pools)
            };
            let got: Result<f64, _> = match func.as_str() {
                "SIN" => Ok(oxfunc_core::functions::sin::sin_kernel(x)),
                "COS" => Ok(oxfunc_core::functions::cos::cos_kernel(x)),
                "TAN" => Ok(oxfunc_core::functions::tan::tan_kernel(x)),
                "COT" => oxfunc_core::functions::cot::cot_kernel(x),
                "SEC" => oxfunc_core::functions::sec::sec_kernel(x),
                "CSC" => oxfunc_core::functions::csc::csc_kernel(x),
                other => panic!("unexpected function {other}"),
            };
            numeric += 1;
            match got {
                Ok(v) if v.to_bits() == expected.to_bits() => exact += 1,
                other => {
                    if failures.len() < 20 {
                        failures.push(format!(
                            "{func} {:?}: expected {} got {:?}",
                            w.id, w.expected_bits, other
                        ));
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
