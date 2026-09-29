//! General density operation graphs against retained public Excel captures.
use oxfunc_core::excel_numeric::research::{ext_from_f64 as from,ext_to_f64 as to,ext_sub,ext_div,ext_mul,excel_exp,CW_PC64_RN as CW};
use oxfunc_core::functions::normal_log_family::{norm_dist_kernel,norm_s_dist_kernel};
use serde_json::{Value,json};

fn op(a:f64,b:f64,kind:char,staged:bool)->f64 {
    if staged {return to(&match kind {'-'=>ext_sub(&from(a),&from(b),CW),'/'=>ext_div(&from(a),&from(b),CW),'*'=>ext_mul(&from(a),&from(b),CW),_=>unreachable!()},CW);}
    match kind {'-'=>a-b,'/'=>a/b,'*'=>a*b,_=>unreachable!()}
}
fn main() {
    let path=std::env::args().nth(1).unwrap();
    let data:Value=serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    for mode in -1..24 {
        let mut rows=0;let mut misses=Vec::new();let mut by_flag=[(0,0),(0,0)];
        for row in data["witnesses"].as_array().unwrap() {
            let a:Vec<f64>=row["args"].as_array().unwrap().iter().map(|a|f64::from_bits(u64::from_str_radix(a.as_str().unwrap().trim_start_matches("0x"),16).unwrap())).collect();
            if a.iter().any(|x|x.is_subnormal()){continue;}
            let standard=a.len()==2;let flag=if standard {a[1]!=0.0} else {a[3]!=0.0};
            if mode>=0 && (standard||flag){continue;}
            let result=if mode<0 {if standard {norm_s_dist_kernel(a[0],flag)} else {norm_dist_kernel(a[0],a[1],a[2],flag)}}
            else if a[2]<=0.0 || !a[2].is_finite() {Err(oxfunc_core::value::WorksheetErrorCode::Num)}
            else {
                let z=op(op(a[0],a[1],'-',mode&1!=0),a[2],'/',mode&2!=0);
                let square=op(z,z,'*',true);
                if !square.is_finite(){Err(oxfunc_core::value::WorksheetErrorCode::Num)} else {
                    let ex=excel_exp(-square/2.0);let inv=f64::from_bits(0x3fd9884533d43651);
                    let staged=mode&4!=0;
                    let value=match mode/8 {
                        0=>op(op(ex,inv,'*',true),a[2],'/',staged),
                        1=>op(ex,op(inv,a[2],'/',staged),'*',staged),
                        _=>op(op(ex,a[2],'/',staged),inv,'*',staged),
                    };
                    Ok(if value.abs()<f64::MIN_POSITIVE {0.0} else {value})
                }
            };
            let actual=match result {Ok(n)=>format!("0x{:016x}",n.to_bits()),Err(e)=>format!("error:{e:?}")};
            rows+=1;by_flag[flag as usize].0+=1;
            if actual!=row["expected_bits"].as_str().unwrap(){misses.push(json!({"id":row["id"],"args":row["args"],"expected":row["expected_bits"],"actual":actual}));}
            else {by_flag[flag as usize].1+=1;}
        }
        println!("{}",json!({"mode":mode,"rows":rows,"matches":rows-misses.len(),"by_flag":by_flag,"misses":misses}));
    }
}
