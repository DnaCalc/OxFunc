//! Read-only finite PMF identities; no fitted constants or production changes.
use oxfunc_core::functions::discrete_dist_family as d;
use oxfunc_core::excel_numeric::research::*;
use serde_json::{Value,json};
use std::collections::BTreeMap;
fn main() {
    let cw=CW_PC64_RN;
    let mul=|a,b|ext_to_f64(&ext_mul(&ext_from_f64(a),&ext_from_f64(b),cw),cw);
    let div=|a,b|ext_to_f64(&ext_div(&ext_from_f64(a),&ext_from_f64(b),cw),cw);
    let mut reports=Vec::new();
    for path in std::env::args().skip(1) {
        let bank:Value=serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        let mut counts=BTreeMap::<String,usize>::new(); let mut failures=BTreeMap::<String,Vec<Value>>::new();let mut rows=0;
        for row in bank["witnesses"].as_array().unwrap() {
            let a:Vec<f64>=row["args"].as_array().unwrap().iter().map(|v|f64::from_bits(u64::from_str_radix(v.as_str().unwrap().trim_start_matches("0x"),16).unwrap())).collect();
            if a[3]!=0. || row["expected_bits"].as_str().unwrap().starts_with("error") {continue;}
            rows+=1;let f=a[0].trunc();let r=a[1].trunc();let p=a[2];let n=f+r;
            let old=d::negbinom_dist_kernel(a[0],a[1],p,false).unwrap();
            let b=d::binom_dist_kernel(r,n,p,false).unwrap();
            let c=d::binom_dist_kernel(r-1.,n-1.,p,false).unwrap();
            let variants=[("production",old),("binom-ratio-native",b*(r/n)),("binom-ratio-staged",mul(b,div(r,n))),
                ("binom-product-div-native",(b*r)/n),("binom-product-div-staged",div(mul(b,r),n)),
                ("binom-last-success-native",c*p),("binom-last-success-staged",mul(c,p)),
                ("binom-ratio-retained",ext_to_f64(&ext_mul(&ext_from_f64(b),&ext_div(&ext_from_f64(r),&ext_from_f64(n),cw),cw),cw))];
            for (name,value) in variants {
                let actual=format!("0x{:016x}",value.to_bits());let expected=row["expected_bits"].as_str().unwrap();
                *counts.entry(name.to_owned()).or_default()+=usize::from(actual==expected);
                if actual!=expected {failures.entry(name.to_owned()).or_default().push(json!({"row":row,"actual":actual}));}
            }
        }
        reports.push(json!({"source":path,"rows":rows,"exact":counts,"failures":failures}));
    }
    println!("{}",serde_json::to_string(&reports).unwrap());
}
