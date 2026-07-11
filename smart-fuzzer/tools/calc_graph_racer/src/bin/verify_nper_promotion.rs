//! W109 NPER promotion verifier: replay every answered oracle witness through
//! the PRODUCTION `oxfunc_core` NPER kernel. Pass answered WitnessSet JSONs.

use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{WitnessArg, WitnessSet};
use oxfunc_core::functions::financial_time_value_family::{FinancialError, PaymentTiming, nper};
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
        assert_eq!(ws.function.to_uppercase(), "NPER");
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
            let timing = if vals[4] == 0.0 {
                PaymentTiming::EndOfPeriod
            } else {
                PaymentTiming::BeginningOfPeriod
            };
            let got = nper(vals[0], vals[1], vals[2], vals[3], timing);
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
            } else if let Some(code) = w.expected_bits.strip_prefix("error:") {
                errors += 1;
                let matched = matches!(
                    (&got, code),
                    (Err(FinancialError::Div0), "Div0") | (Err(FinancialError::Num), "Num")
                );
                if matched {
                    errors_matched += 1;
                } else if failures.len() < 20 {
                    failures.push(format!(
                        "{:?}: expected {} got {:?}",
                        w.id, w.expected_bits, got
                    ));
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
