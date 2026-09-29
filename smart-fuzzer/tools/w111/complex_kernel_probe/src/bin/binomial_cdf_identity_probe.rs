//! Research on the retained older public BINOM CDF capture; no production edit.
use oxfunc_core::functions::{discrete_dist_family::binom_dist_kernel,special_math_common::regularized_beta};
use oxfunc_core::excel_numeric::research::*;
use serde_json::{json,Value};
use std::collections::BTreeMap;
fn main() {
    let input=std::env::args().nth(1).unwrap();let text=std::fs::read_to_string(&input).unwrap();
    let cw=CW_PC64_RN;let add=|a,b|ext_to_f64(&ext_add(&ext_from_f64(a),&ext_from_f64(b),cw),cw);
    let mut counts=BTreeMap::<String,usize>::new();let mut failures=BTreeMap::<String,Vec<Value>>::new();let mut rows=0;
    for line in text.lines().filter(|l|!l.trim().is_empty()) {
        let row:Value=serde_json::from_str(line).unwrap();let k=row["k"].as_u64().unwrap();let n=row["n"].as_u64().unwrap();let p=row["p"].as_f64().unwrap();
        let expected=row["cdf_bits"].as_str().unwrap();let pmf=|j|binom_dist_kernel(j as f64,n as f64,p,false).unwrap();rows+=1;
        let mut candidates=vec![("production",binom_dist_kernel(k as f64,n as f64,p,true).unwrap()),
            ("reverse-pmf-sum",(0..=k).rev().map(pmf).sum()),
            ("staged-pmf-sum",(0..=k).map(pmf).fold(0.,add)),
            ("reverse-staged-pmf-sum",(0..=k).rev().map(pmf).fold(0.,add)),
            ("tail-sum-subtract",1.-((k+1)..=n).map(pmf).sum::<f64>())];
        if k<n {
            candidates.push(("beta-complement-input",regularized_beta(1.-p,(n-k) as f64,(k+1) as f64)));
            candidates.push(("beta-complement-output",1.-regularized_beta(p,(k+1) as f64,(n-k) as f64)));
        } else {candidates.push(("beta-complement-input",1.));candidates.push(("beta-complement-output",1.));}
        for (name,v) in candidates {
            let actual=format!("0x{:016x}",v.to_bits());*counts.entry(name.to_owned()).or_default()+=usize::from(actual==expected);
            if actual!=expected {failures.entry(name.to_owned()).or_default().push(json!({"row":row,"actual":actual}));}
        }
    }
    println!("{}",json!({"source":input,"qualification":"older_public_capture_for_hypothesis_only_requires_current_excel_validation","rows":rows,"exact":counts,"failures":failures}));
}
