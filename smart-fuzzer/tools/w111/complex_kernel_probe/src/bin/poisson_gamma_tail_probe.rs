//! Research only: isolate the inherited GRATIO conservative underflow cutoff.
use oxfunc_core::{excel_numeric::research as excel_numeric, functions::discrete_dist_family::poisson_dist_kernel};
use serde_json::{json, Value};
use std::collections::BTreeMap;
#[allow(dead_code)]
mod no_cutoff {
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../../docs/function-lane/evidence/w111-broad-20260929/distribution-surface/poisson-publication/research/no-cutoff-gratio.rs"));
}
fn main() {
    let data: Value = serde_json::from_slice(&std::fs::read(std::env::args().nth(1).unwrap()).unwrap()).unwrap();
    let mut exact = BTreeMap::<&str,usize>::new();
    let mut residuals = BTreeMap::<&str,Vec<Value>>::new();
    let mut rows = 0;
    for row in data["witnesses"].as_array().unwrap() {
        let args:Vec<_> = row["args"].as_array().unwrap().iter().map(|s| f64::from_bits(u64::from_str_radix(s.as_str().unwrap().trim_start_matches("0x"),16).unwrap())).collect();
        if args[0] < 1.0 || args[1] < 0.0 || args[2] == 0.0 {continue;}
        rows += 1;
        let old = poisson_dist_kernel(args[0],args[1],true).unwrap();
        let raw = no_cutoff::regularized_gamma_q(args[0].trunc()+1.0,args[1]);
        let new = if raw.abs() < f64::MIN_POSITIVE {0.0} else {raw};
        for (label,value) in [("production",old),("no_early_cutoff",new)] {
            let bits = format!("0x{:016x}",value.to_bits());
            if bits == row["expected_bits"].as_str().unwrap() {*exact.entry(label).or_default()+=1;}
            else {residuals.entry(label).or_default().push(json!({"id":row["id"],"args":row["args"],"expected":row["expected_bits"],"actual":bits}));}
        }
    }
    println!("{}",json!({"rows":rows,"exact":exact,"residuals":residuals}));
}
