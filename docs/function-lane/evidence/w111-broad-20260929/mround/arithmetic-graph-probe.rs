//! MROUND arithmetic candidates and diagnostic half-boundary perturbations.
//! Perturbations measure tolerance; a fitted epsilon is not a production model.
use oxfunc_core::excel_numeric::research::*;
use oxfunc_core::functions::round_fn::{excel_decimal_round,DecimalRoundMode};
use serde_json::{json,Value};
use std::collections::BTreeMap;
const CW:u16=CW_PC64_RN;
fn add(a:f64,b:f64)->f64 {ext_to_f64(&ext_add(&ext_from_f64(a),&ext_from_f64(b),CW),CW)}
fn sub(a:f64,b:f64)->f64 {ext_to_f64(&ext_sub(&ext_from_f64(a),&ext_from_f64(b),CW),CW)}
fn div(a:f64,b:f64)->f64 {ext_to_f64(&ext_div(&ext_from_f64(a),&ext_from_f64(b),CW),CW)}
fn main(){
    let bank:Value=serde_json::from_slice(&std::fs::read(std::env::args().nth(1).unwrap()).unwrap()).unwrap();
    let mut exact=BTreeMap::<String,usize>::new();let mut failures=BTreeMap::<String,Vec<Value>>::new();
    for row in bank["witnesses"].as_array().unwrap(){
        let args:Vec<_>=row["args"].as_array().unwrap().iter().map(|v|f64::from_bits(u64::from_str_radix(v.as_str().unwrap().trim_start_matches("0x"),16).unwrap())).collect();
        let (x,m)=(args[0].abs(),args[1].abs());let q=div(x,m);let k=q.floor();let mut c=Vec::new();
        c.push(("staged_round".to_owned(),q.round()));
        c.push(("decimal15_round".into(),excel_decimal_round(q,0,DecimalRoundMode::HalfAwayFromZero)));
        c.push(("staged_shifted_quotient".into(),add(q,0.5).floor()));
        c.push(("staged_shifted_number".into(),div(add(x,m/2.),m).floor()));
        c.push(("measured-window-decimal-half".into(),k+f64::from(q-k>=0.499999999999995)));
        c.push(("fraction-decimal14".into(),k+f64::from(format!("{:.14}",q-k).parse::<f64>().unwrap_or(f64::NAN)>=0.5)));
        for staged_product in [false,true]{for staged_difference in [false,true]{
            let product=if staged_product{x87_mul(k,m)}else{k*m};
            let remainder=if staged_difference{sub(x,product)}else{x-product};
            c.push((format!("remainder-product{staged_product}-difference{staged_difference}"),k+f64::from(remainder>=m/2.)));
        }}
        for staged in [false,true]{
            let midpoint=if staged{x87_mul(k+0.5,m)}else{(k+0.5)*m};
            c.push((format!("midpoint-product{staged}"),k+f64::from(x>=midpoint)));
        }
        for e in -58..=-42{
            let epsilon=2_f64.powi(e);
            c.push((format!("diagnostic-q-plus-2pow{e}"),add(q,epsilon).round()));
            c.push((format!("diagnostic-shift-plus-2pow{e}"),add(add(q,0.5),epsilon).floor()));
            c.push((format!("diagnostic-fraction-threshold-2pow{e}"),k+f64::from(q-k>=0.5-epsilon)));
            let shifted=ext_add(&ext_add(&ext_from_f64(q),&ext_from_f64(0.5),CW),&ext_from_f64(epsilon),CW);
            c.push((format!("diagnostic-retained-shift-2pow{e}"),ext_to_f64(&ext_rndint(&shifted,CW_PC64_RN|0x0400),CW)));
        }
        for e in [3,4,5,6,7,8,16,31]{
            let bias=2_f64.powi(e);
            c.push((format!("diagnostic-biased-shift-{e}"),(add(add(q,0.5),bias)-bias).floor()));
        }
        for(label,count)in c{
            let actual=if args[0]==0.0||args[1]==0.0{"0x0000000000000000".to_owned()}
            else if args[0].signum()!=args[1].signum(){"error:Num".into()}
            else{let result=x87_mul(count,args[1]);if !result.is_finite(){"error:Num".into()}else{format!("0x{:016x}",if result==0.0{0}else{result.to_bits()})}};
            if actual==row["expected_bits"].as_str().unwrap(){*exact.entry(label).or_default()+=1;}
            else{failures.entry(label).or_default().push(json!({"id":row["id"],"args":row["args"],"expected":row["expected_bits"],"actual":actual}));}
        }
    }
    println!("{}",json!({"rows":bank["witnesses"].as_array().unwrap().len(),"exact":exact,"failures":failures}));
}
