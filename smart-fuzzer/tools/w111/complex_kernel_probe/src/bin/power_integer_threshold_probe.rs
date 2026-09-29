//! Research integer dispatch widths and decimal exponent carrier conversion.
use oxfunc_core::functions::power_fn::power_kernel;
use oxfunc_core::excel_numeric::research::*;
use oxfunc_core::value::WorksheetErrorCode;
use oxfunc_core::value::{CalcValue,ExcelText};
use serde_json::{json,Value};
use std::collections::BTreeMap;
fn outcome(value:Result<f64,WorksheetErrorCode>)->String {match value{Ok(v)=>format!("0x{:016x}",v.to_bits()),Err(e)=>format!("error:{e:?}")}}
fn reciprocal(x:f64)->f64 {ext_to_f64(&ext_div(&ext_one(),&ext_from_f64(x),CW_PC64_RN),CW_PC64_RN)}
// Preserve the pre-width candidate's integer body so alternative limits remain
// reproducible after production adopts one of them. The source is retained as
// power-integer-thresholds/initial-power_fn.rs; unaffected fractional paths and
// small positive-ten decimal scales still delegate to the production primitive.
fn initial_power(base:f64,exponent:f64)->Result<f64,WorksheetErrorCode>{
    if exponent==0.0||base==0.0||!exponent.is_finite()||exponent.fract()!=0.0||
       exponent<i64::MIN as f64||exponent>i64::MAX as f64{return power_kernel(base,exponent);}
    let magnitude=(exponent as i64).unsigned_abs();
    let mut p=if base==10.0 {
        if magnitude>308{f64::INFINITY}else{power_kernel(10.0,magnitude as f64).unwrap()}
    } else {
        let(mut n,mut b,mut result)=(magnitude,base,1.0);
        while n>0 {if n&1==1{result=x87_mul(result,b);}n>>=1;if n>0{b=x87_mul(b,b);}}
        result
    };
    if exponent<0.0{if p.abs()<f64::MIN_POSITIVE{return Err(WorksheetErrorCode::Div0);}p=reciprocal(p);}
    if !p.is_finite(){return Err(WorksheetErrorCode::Num);}
    Ok(if p.abs()<f64::MIN_POSITIVE{0.0}else{p})
}
fn candidate(base:f64,exponent:f64,limit:f64,inclusive:bool,decimal_signed:bool)->Result<f64,WorksheetErrorCode>{
    if exponent==0.0||base==0.0||exponent.fract()!=0.0{return initial_power(base,exponent);}
    let magnitude=exponent.abs();
    if magnitude<limit||(inclusive&&magnitude==limit){
        if decimal_signed&&base==10.0&&magnitude>=2147483648.0&&magnitude<4294967296.0{
            let scale=magnitude as u32 as i32;
            // A one-digit scientific text lexeme reaches the same shared
            // decimal pair converter as the production positive-ten branch.
            let p=if scale < -308 {0.0} else {
                let lexeme=format!("1e{scale}");
                oxfunc_core::coercion::coerce_calc_scalar_to_number(&CalcValue::text(
                    ExcelText::from_utf16_code_units(lexeme.encode_utf16().collect()))).unwrap()
            };
            if exponent<0.0 {
                if p<f64::MIN_POSITIVE{return Err(WorksheetErrorCode::Div0);}
                return Ok(reciprocal(p));
            }
            return Ok(p);
        }
        return initial_power(base,exponent);
    }
    // Existing noninteger body, now reached by an out-of-width integer.
    if base<0.0{return Err(WorksheetErrorCode::Num);}
    let p=excel_pow_positive(base,magnitude);
    if exponent>0.0{
        if p.is_infinite(){return Err(WorksheetErrorCode::Num);}
        return Ok(if p==0.0{0.0}else{p});
    }
    if p<f64::MIN_POSITIVE{return Err(WorksheetErrorCode::Div0);}
    if p.is_infinite(){return Ok(0.0);}
    let r=reciprocal(p);
    if r<f64::MIN_POSITIVE{return Ok(0.0);}
    if r.is_infinite(){return Err(WorksheetErrorCode::Num);}
    Ok(r)
}
fn main(){
    let data:Value=serde_json::from_slice(&std::fs::read(std::env::args().nth(1).unwrap()).unwrap()).unwrap();
    let mut exact=BTreeMap::<String,usize>::new();let mut failures=BTreeMap::<String,Vec<Value>>::new();
    for row in data["witnesses"].as_array().unwrap(){
        let args:Vec<_>=row["args"].as_array().unwrap().iter().map(|v|f64::from_bits(u64::from_str_radix(v.as_str().unwrap().trim_start_matches("0x"),16).unwrap())).collect();
        let mut values=vec![("production".into(),initial_power(args[0],args[1]))];
        for limit in [2147483647.,2147483648.,4294967295.,4294967296.]{for inclusive in [false,true]{for decimal_signed in [false,true]{
            let label=format!("limit{limit}-inclusive{inclusive}-decimalSigned{decimal_signed}");
            values.push((label,candidate(args[0],args[1],limit,inclusive,decimal_signed)));
        }}}
        for(label,value)in values{let actual=outcome(value);if actual==row["expected_bits"].as_str().unwrap(){*exact.entry(label).or_default()+=1;}else{failures.entry(label).or_default().push(json!({"id":row["id"],"args":row["args"],"expected":row["expected_bits"],"actual":actual}));}}
    }
    println!("{}",json!({"rows":data["witnesses"].as_array().unwrap().len(),"exact":exact,"failures":failures}));
}
