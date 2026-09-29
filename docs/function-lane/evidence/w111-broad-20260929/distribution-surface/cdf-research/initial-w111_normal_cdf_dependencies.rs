//! Public ERFC packet and wrapper staging race; no fitted coefficients or edits.
use calc_graph_racer::erfc_f_packets as f;
use oxfunc_core::excel_numeric::research::*;
use oxfunc_core::functions::{normal_log_family::identified_std_normal_cdf, special_dist_family::erfc_precise_kernel};
use serde_json::{json, Value};
use std::collections::BTreeMap;

fn main() {
    let path = std::env::args().nth(1).unwrap();
    let out = std::env::args().nth(2).unwrap();
    let data: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    let cw = CW_PC64_RN;
    let mul = |a,b| ext_to_f64(&ext_mul(&ext_from_f64(a),&ext_from_f64(b),cw),cw);
    let mut scores = BTreeMap::<String,usize>::new();
    let mut failures = BTreeMap::<String,Vec<Value>>::new();
    let mut dependencies = Vec::new();
    let mut rows = 0;
    for row in data["witnesses"].as_array().unwrap() {
        if row["args"][1] == "0x0000000000000000" { continue; }
        let x = f64::from_bits(u64::from_str_radix(row["args"][0].as_str().unwrap().trim_start_matches("0x"),16).unwrap());
        let expected = row["expected_bits"].as_str().unwrap();
        let old = identified_std_normal_cdf(x);
        rows += 1;
        let mut candidates = vec![("production".to_owned(),old)];
        for zm in 0..2 {
            let z = if zm==0 {x.abs()*f::FRAC_1_SQRT_2} else {mul(x.abs(),f::FRAC_1_SQRT_2)};
            if old.to_bits() != u64::from_str_radix(expected.trim_start_matches("0x"),16).unwrap() {
                dependencies.push(json!({"source_id":row["id"],"x_bits":row["args"][0],"z_mode":zm,
                    "z_bits":format!("0x{:016x}",z.to_bits()),"normal_expected":expected}));
            }
            for mode in 0..6 {
                for sm in 0..2 { for pm in 0..2 {
                    let square=if sm==0 {z*z} else {mul(z,z)};
                    let q=if mode==0 {erfc_precise_kernel(z).unwrap()}
                        else if mode==1 {libm::erfc(z)}
                        else if z<=0.5 {erfc_precise_kernel(z).unwrap()}
                        else if z>30. {0.}
                        else {
                            let factor=match mode {2=>f::nswc_derfc0(z),3=>f::cody_erfcx_f(z),
                                4=>f::cdflib_erfc1_f(z),_=>f::cf_gautschi_f(z)};
                            let e=excel_exp(-square);
                            if pm==0 {e*factor} else {mul(e,factor)}
                        };
                    let value=if x==0. {0.5} else if x<0. {q/2.} else {1.-q/2.};
                    candidates.push((format!("packet{mode}-z{zm}-square{sm}-product{pm}"),value));
                }}
            }
        }
        for (name,mut value) in candidates {
            if value.abs()<f64::MIN_POSITIVE {value=0.;}
            let actual=format!("0x{:016x}",value.to_bits());
            *scores.entry(name.clone()).or_default()+=usize::from(actual==expected);
            if actual!=expected {failures.entry(name).or_default().push(json!({"row":row,"actual":actual}));}
        }
    }
    let best=scores.iter().max_by_key(|(_,v)|**v).unwrap().0;
    let result=json!({"source":path,"rows":rows,"scores":scores,"best":best,
        "best_misses":failures.get(best),"production_misses":failures.get("production"),"dependencies":dependencies});
    std::fs::write(out,serde_json::to_string(&result).unwrap()).unwrap();
    println!("rows={rows}, best={best}, exact={}",scores[best]);
}
