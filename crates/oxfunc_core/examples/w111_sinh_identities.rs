//! General cancellation-safe sinh identities; no input-specific correction.
#[cfg(not(all(feature="research-x87",target_arch="x86_64")))]
fn main(){eprintln!("Requires research-x87 on x86_64");}
#[cfg(all(feature="research-x87",target_arch="x86_64"))]
fn main(){
 use oxfunc_core::excel_numeric::research::*;
 let add=|a,b|ext_to_f64(&ext_add(&ext_from_f64(a),&ext_from_f64(b),CW_PC64_RN),CW_PC64_RN);
 let div=|a,b|ext_to_f64(&ext_div(&ext_from_f64(a),&ext_from_f64(b),CW_PC64_RN),CW_PC64_RN);
 let mut reports=Vec::new();
 for path in std::env::args().skip(1){
  let bank:serde_json::Value=serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
  let mut counts=std::collections::BTreeMap::<String,usize>::new();let mut misses=std::collections::BTreeMap::<String,Vec<serde_json::Value>>::new();let mut rows=0;
  for row in bank["witnesses"].as_array().unwrap(){
   let x=f64::from_bits(u64::from_str_radix(row["args"][0].as_str().unwrap().trim_start_matches("0x"),16).unwrap());
   if x==0. || x.abs()>=1. || x.abs()<f64::MIN_POSITIVE{continue;} rows+=1;
   let a=x.abs();let p=excel_expm1_internal(a);let n=excel_expm1_internal(-a);let e=excel_exp(a);
   let mut candidates=vec![("baseline".to_owned(),oxfunc_core::functions::sinh::sinh_kernel(x))];
   for (label,t,sign) in [("positive",p,1.),("negative",n,-1.)]{
    for m in 0..4{
     let denom=if m&1==0{t+1.}else{add(t,1.)};
     let q=if m&2==0{t/denom}else{div(t,denom)};
     candidates.push((format!("{label}-{m}-add53"),((t+q)/2.*sign).copysign(x)));
     candidates.push((format!("{label}-{m}-add64"),(add(t,q)/2.*sign).copysign(x)));
    }
   }
   let t2=excel_expm1_internal(2.*a);let n2=excel_expm1_internal(-2.*a);
   candidates.push(("double-positive".to_owned(),(t2/(2.*e)).copysign(x)));
   candidates.push(("double-positive64".to_owned(),div(t2,2.*e).copysign(x)));
   candidates.push(("double-negative".to_owned(),(-n2*e/2.).copysign(x)));
   candidates.push(("expm1-product".to_owned(),(p*(p+2.)/(p+1.)/2.).copysign(x)));
   for (key,v) in candidates{
    let actual=format!("0x{:016x}",v.to_bits());let expected=row["expected_bits"].as_str().unwrap();
    *counts.entry(key.clone()).or_default()+=usize::from(actual==expected);
    if actual!=expected && misses.entry(key.clone()).or_default().len()<12{misses.entry(key).or_default().push(serde_json::json!({"row":row,"actual":actual}));}
   }
  }
  reports.push(serde_json::json!({"source":path,"rows":rows,"matches":counts,"sample_misses":misses}));
 }
 println!("{}",serde_json::to_string(&reports).unwrap());
}
