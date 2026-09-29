//! Candidate arithmetic for integer powers, against black-box observations.
#[cfg(not(all(feature="research-x87",target_arch="x86_64")))]
fn main(){eprintln!("Requires research-x87 on x86_64");}
#[cfg(all(feature="research-x87",target_arch="x86_64"))]
fn main(){
 use oxfunc_core::excel_numeric::research::excel_pow_positive;
 use oxfunc_core::excel_numeric::research::{ext_to_f64,ext_from_f64,ext_mul,CW_PC64_RN};
 let mul=|a,b|ext_to_f64(&ext_mul(&ext_from_f64(a),&ext_from_f64(b),CW_PC64_RN),CW_PC64_RN);
 use oxfunc_core::functions::power_fn::power_kernel;
 let names=["worksheet_power","internal_positive_power","standard_powf","binary64_repeated_squaring","staged_repeated_squaring","staged_linear_multiplication"];
 let mut reports=Vec::new();
 for path in std::env::args().skip(1){
  let bank:serde_json::Value=serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();let mut counts=[0;6];let mut misses=Vec::new();
  for row in bank["witnesses"].as_array().unwrap(){
   let args:Vec<_>=row["args"].as_array().unwrap().iter().map(|x|f64::from_bits(u64::from_str_radix(x.as_str().unwrap().trim_start_matches("0x"),16).unwrap())).collect();
   let n=args[0].trunc();let k=args[1].trunc();let expected=row["expected_bits"].as_str().unwrap();
   let mut results=Vec::new();
   for mode in 0..6{
    let value=if args[0]<0.0||k<0.0||args[0]>=2147483647.0||args[1]>=2147483647.0 {None} else if k==0.0 {Some(1.0)} else if n==0.0 {Some(0.0)} else if n==1.0 {Some(1.0)} else{
     Some(match mode{0=>power_kernel(n,k).unwrap_or(f64::INFINITY),1=>excel_pow_positive(n,k),2=>n.powf(k),_=>{
      if k>=1024.0 {f64::INFINITY} else if mode==5{let mut acc=1.0;for _ in 0..k as usize{acc=mul(acc,n);}acc} else {let mut e=k as u64;let mut acc=1.0;let mut b=n;while e>0{if e&1!=0{acc=if mode==4{mul(acc,b)}else{acc*b};}e>>=1;if e>0{b=if mode==4{mul(b,b)}else{b*b};}}acc}
     }})
    };
    let actual=match value{Some(v) if v.is_finite()=>format!("0x{:016x}",if v==0.0{0}else{v.to_bits()}),_=>"error:Num".to_owned()};counts[mode]+=usize::from(actual==expected);results.push(actual);
   }
   if results.iter().any(|v|v!=expected){misses.push(serde_json::json!({"row":row,"candidates":results}));}
  }
  reports.push(serde_json::json!({"path":path,"rows":bank["witnesses"].as_array().unwrap().len(),"candidate_names":names,"exact":counts,"differences":misses}));
 }
 println!("{}",serde_json::to_string_pretty(&reports).unwrap());
}
