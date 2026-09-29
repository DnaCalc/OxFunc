//! Offline dependency audit: replay unchanged XNPV with frozen POWER outcomes.
use oxfunc_core::excel_numeric::research::*;
use oxfunc_core::functions::{cashflow_rate_family::xnpv_kernel, power_fn::power_kernel};
use oxfunc_core::value::WorksheetErrorCode;
use serde_json::{json,Value};
use std::collections::BTreeMap;
const CW:u16=CW_PC64_RN;
fn add(a:f64,b:f64)->f64 {ext_to_f64(&ext_add(&ext_from_f64(a),&ext_from_f64(b),CW),CW)}
fn div(a:f64,b:f64)->f64 {ext_to_f64(&ext_div(&ext_from_f64(a),&ext_from_f64(b),CW),CW)}
fn bits(x:f64)->String {format!("0x{:016x}",x.to_bits())}
fn float(v:&Value)->f64 {f64::from_bits(u64::from_str_radix(v.as_str().unwrap().trim_start_matches("0x"),16).unwrap())}
fn outcome(value:Result<f64,WorksheetErrorCode>)->String {match value {Ok(v)=>bits(v),Err(e)=>format!("error:{e:?}")}}
fn args(row:&Value)->(f64,Vec<f64>,Vec<i64>){
    (float(&row["args"][0]),row["args"][1].as_array().unwrap().iter().map(float).collect(),
     row["args"][2].as_array().unwrap().iter().map(|v|float(v) as i64).collect())
}
fn admitted(rate:f64,values:&[f64],dates:&[i64])->bool {
    rate.is_finite()&&rate>0.0&&values.len()==dates.len()&&!dates.is_empty()&&
    values.iter().all(|v|v.is_finite())&&dates.iter().all(|v|*v>=dates[0])
}
fn key(base:f64,years:f64)->String {format!("{}:{}",bits(base),bits(years))}
fn compose(rate:f64,values:&[f64],dates:&[i64],power:&BTreeMap<String,String>)->String {
    // Non-admitted records never reach POWER; the consumer's unchanged
    // validation is evaluated by the production kernel.
    if !admitted(rate,values,dates){return outcome(xnpv_kernel(rate,values,dates));}
    let base=add(1.0,rate);let mut total=0.0;
    for (value,date) in values.iter().zip(dates){
        let years=(*date-dates[0]) as f64/365.0;
        let stored=&power[&key(base,years)];
        if stored.starts_with("error:"){return stored.clone();}
        total=add(total,div(*value,float(&Value::String(stored.clone()))));
    }
    outcome(if total.is_finite(){Ok(total)}else{Err(WorksheetErrorCode::Num)})
}
fn main(){
    let argv:Vec<_>=std::env::args().skip(1).collect();
    let data:Value=serde_json::from_slice(&std::fs::read(&argv[1]).unwrap()).unwrap();
    let mut powers=BTreeMap::new();
    for row in data["witnesses"].as_array().unwrap(){
        let (rate,values,dates)=args(row);if !admitted(rate,&values,&dates){continue;}
        let base=add(1.0,rate);
        for date in &dates{let years=(*date-dates[0]) as f64/365.0;
            powers.entry(key(base,years)).or_insert_with(||json!({"id":key(base,years),"args":[bits(base),bits(years)],"expected_bits":outcome(power_kernel(base,years))}));}
    }
    if argv[0]=="factors"{println!("{}",json!({"function":"POWER","witnesses":powers.into_values().collect::<Vec<_>>()}));return;}
    let prior:BTreeMap<String,String>=serde_json::from_slice(&std::fs::read(&argv[2]).unwrap()).unwrap();
    let current:BTreeMap<_,_>=powers.iter().map(|(k,v)|(k.clone(),v["expected_bits"].as_str().unwrap().to_owned())).collect();
    let mut changed=Vec::new();let mut old_exact=0;let mut new_exact=0;let mut invalid=0;
    let mut retained_misses=Vec::new();
    for row in data["witnesses"].as_array().unwrap(){
        let(rate,values,dates)=args(row);let new=outcome(xnpv_kernel(rate,&values,&dates));
        let reconstructed=compose(rate,&values,&dates,&current);assert_eq!(new,reconstructed,"current composition differs {}",row["id"]);
        let old=compose(rate,&values,&dates,&prior);let expected=row["expected_bits"].as_str().unwrap();
        old_exact+=usize::from(old==expected);new_exact+=usize::from(new==expected);
        invalid+=usize::from(!admitted(rate,&values,&dates));
        if old!=new{changed.push(json!({"id":row["id"],"args":row["args"],"expected":expected,"old":old,"new":new}));}
        if new!=expected{retained_misses.push(json!({"id":row["id"],"args":row["args"],"expected":expected,"old":old,"new":new}));}
    }
    println!("{}",json!({"rows":data["witnesses"].as_array().unwrap().len(),"dependency_not_reached":invalid,
        "distinct_power_calls":current.len(),"old_exact":old_exact,"new_exact":new_exact,"changed":changed,"retained_misses":retained_misses}));
}
