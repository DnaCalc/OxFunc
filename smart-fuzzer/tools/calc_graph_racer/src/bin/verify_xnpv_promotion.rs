//! W109 XNPV promotion verifier: replay every answered oracle witness through
//! the PRODUCTION `oxfunc_core` XNPV kernel (not the DSL candidate) and check
//! bit-exactness on numeric rows and `#NUM!` publication on Excel's error
//! rows. Usage: pass one or more answered WitnessSet JSON files.

use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{WitnessArg, WitnessSet};
use oxfunc_core::functions::cashflow_rate_family::xnpv_kernel;
use std::collections::HashSet;

fn main() {
    let files: Vec<String> = std::env::args().skip(1).collect();
    assert!(!files.is_empty(), "pass answered WitnessSet JSON files");
    let (mut numeric, mut numeric_exact, mut errors, mut errors_matched) = (0u32, 0u32, 0u32, 0u32);
    let mut seen: HashSet<String> = HashSet::new();
    let mut failures: Vec<String> = Vec::new();
    for file in &files {
        let ws: WitnessSet =
            serde_json::from_str(&std::fs::read_to_string(file).expect("read")).expect("parse");
        assert_eq!(ws.function.to_uppercase(), "XNPV");
        for w in &ws.witnesses {
            let key = serde_json::to_string(&w.args).unwrap();
            if !seen.insert(key) {
                continue; // dedupe overlapping answer files
            }
            let rate = match &w.args[0] {
                WitnessArg::Scalar(s) => parse_bits_hex(s).unwrap(),
                _ => panic!("rate must be scalar"),
            };
            let values: Vec<f64> = match &w.args[1] {
                WitnessArg::Array(items) => {
                    items.iter().map(|s| parse_bits_hex(s).unwrap()).collect()
                }
                _ => panic!("values must be array"),
            };
            let dates: Vec<i64> = match &w.args[2] {
                WitnessArg::Array(items) => items
                    .iter()
                    .map(|s| parse_bits_hex(s).unwrap() as i64)
                    .collect(),
                _ => panic!("dates must be array"),
            };
            let got = xnpv_kernel(rate, &values, &dates);
            if let Some(expected) = parse_bits_hex(&w.expected_bits) {
                numeric += 1;
                match got {
                    Ok(v) if v.to_bits() == expected.to_bits() => numeric_exact += 1,
                    other => {
                        if failures.len() < 20 {
                            failures.push(format!(
                                "{:?}: expected {} got {:?}",
                                w.id, w.expected_bits, other
                            ));
                        }
                    }
                }
            } else if w.expected_bits.starts_with("error:") {
                errors += 1;
                let code = w.expected_bits.trim_start_matches("error:");
                match got {
                    Err(e) if format!("{e:?}") == code => errors_matched += 1,
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
    println!("numeric: {numeric_exact}/{numeric} bit-exact");
    println!("errors : {errors_matched}/{errors} matching worksheet error code");
    if !failures.is_empty() {
        println!("failures:");
        for f in &failures {
            println!("  {f}");
        }
        std::process::exit(1);
    }
    println!("PROMOTION VERIFIED: all answered witnesses reproduce through the production kernel");
}
