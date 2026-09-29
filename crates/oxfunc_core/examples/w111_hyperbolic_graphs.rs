//! Candidate arithmetic graphs compared with public-interface Excel captures.
#[cfg(not(all(feature="research-x87",target_arch="x86_64")))]
fn main(){eprintln!("Requires research-x87 on x86_64");}
#[cfg(all(feature="research-x87",target_arch="x86_64"))]
fn main(){
 use oxfunc_core::excel_numeric::research::*;
 let add=|a,b|ext_to_f64(&ext_add(&ext_from_f64(a),&ext_from_f64(b),CW_PC64_RN),CW_PC64_RN);
 let sub=|a,b|ext_to_f64(&ext_sub(&ext_from_f64(a),&ext_from_f64(b),CW_PC64_RN),CW_PC64_RN);
 let div=|a,b|ext_to_f64(&ext_div(&ext_from_f64(a),&ext_from_f64(b),CW_PC64_RN),CW_PC64_RN);
 let mul=|a,b|ext_to_f64(&ext_mul(&ext_from_f64(a),&ext_from_f64(b),CW_PC64_RN),CW_PC64_RN);
 let em1=|t:f64,mode:usize|{
  if mode==0{return excel_expm1_internal(t);}
  let u=excel_exp(t);if u==1.0{return t;}if u==0.0{return -1.0;}if !u.is_finite(){return u;}
  let delta=sub(u,1.0);let log=excel_ln(u);
  if mode==8{return ext_to_f64(&ext_div(&ext_mul(&ext_from_f64(delta),&ext_from_f64(t),CW_PC64_RN),&ext_from_f64(log),CW_PC64_RN),CW_PC64_RN);}
  if mode==9{return ext_to_f64(&ext_mul(&ext_from_f64(delta),&ext_div(&ext_from_f64(t),&ext_from_f64(log),CW_PC64_RN),CW_PC64_RN),CW_PC64_RN);}
  if mode==10 || mode==11{
   let l=ext_fyl2x(&ext_ln2(),&ext_from_f64(u),CW_PC64_RN);
   let product=if mode==10{ext_from_f64(delta*t)}else{ext_mul(&ext_from_f64(delta),&ext_from_f64(t),CW_PC64_RN)};
   return ext_to_f64(&ext_div(&product,&l,CW_PC64_RN),CW_PC64_RN);
  }
  match mode{1=>delta*t/log,2=>div(mul(delta,t),log),3=>if t.abs()<1.0{div(mul(delta,t),log)}else{delta},4=>delta*(t/log),5=>mul(delta,div(t,log)),6=>if t.abs()<2.0{div(mul(delta,t),log)}else{delta},_=>delta}
 };
 let mut reports=Vec::new();
 for path in std::env::args().skip(1){
  let bank:serde_json::Value=serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();let name=bank["function"].as_str().unwrap();let mut counts=std::collections::BTreeMap::<String,usize>::new();let mut misses=Vec::new();
  for row in bank["witnesses"].as_array().unwrap(){
   let x=f64::from_bits(u64::from_str_radix(row["args"][0].as_str().unwrap().trim_start_matches("0x"),16).unwrap());
   if x.to_bits()==1u64<<63 || (x!=0.0 && x.abs()<f64::MIN_POSITIVE){continue;}
   let ep=excel_exp(x);let en=excel_exp(-x);let mp=excel_expm1_internal(x);let mn=excel_expm1_internal(-x);
   let mut ss=vec![(mp-mn)/2.0,sub(mp,mn)/2.0,(ep-en)/2.0,sub(ep,en)/2.0];
   for mode in 1..8{ss.push(sub(em1(x,mode),em1(-x,mode))/2.0);}
   ss.push(if x.abs()<1.0{ss[0]}else{ss[3]});
   for mode in [8,9,10,11]{ss.push(if x.abs()<1.0{sub(em1(x,mode),em1(-x,mode))/2.0}else{ss[3]});}
   let cs=[(ep+en)/2.0,add(ep,en)/2.0];
   let mut candidates=Vec::new();
   match name{
    "SINH"=>for (s,&v) in ss.iter().enumerate(){candidates.push((format!("s{s}"),v));},
    "COSH"=>for (c,&v) in cs.iter().enumerate(){candidates.push((format!("c{c}"),v));},
    "CSCH"=>for (s,&v) in ss.iter().enumerate(){for d in 0..2{candidates.push((format!("s{s}d{d}"),if d==0{1.0/v}else{div(1.0,v)}));}},
    "TANH"|"COTH"=>for (s,&sv) in ss.iter().enumerate(){for(c,&cv)in cs.iter().enumerate(){for d in 0..2{let t=if d==0{sv/cv}else{div(sv,cv)};if name=="TANH"{candidates.push((format!("s{s}c{c}d{d}"),t));}else{for r in 0..2{candidates.push((format!("s{s}c{c}d{d}r{r}"),if r==0{1.0/t}else{div(1.0,t)}));}}}}},
    _=>panic!("Unknown function")
   }
   if name=="TANH"{
    for mode in 0..8{for d in 0..2{
     let m=em1(2.0*x,mode);let raw=if d==0{m/(m+2.0)}else{div(m,add(m,2.0))};candidates.push((format!("m2mode{mode}d{d}"),raw));
     let m=em1(-2.0*x.abs(),mode);let raw=if d==0{-m/(m+2.0)}else{div(-m,add(m,2.0))};candidates.push((format!("m2negmode{mode}d{d}"),raw.copysign(x)));
    }}
   }
   let mut results=Vec::new();let expected=row["expected_bits"].as_str().unwrap();
   for (key,mut v) in candidates{
    let output=if x==0.0 && matches!(name,"CSCH"|"COTH"){"error:Div0".to_owned()}else{
     if matches!(name,"TANH"|"COTH") && !v.is_finite(){v=x.signum();}
     if name=="CSCH" && v.abs()<f64::MIN_POSITIVE{v=0.0;}
     if !v.is_finite(){"error:Num".to_owned()}else{format!("0x{:016x}",v.to_bits())}
    };
    *counts.entry(key.clone()).or_default()+=usize::from(output==expected);
    if output!=expected{results.push((key,output));}
   }
   if !results.is_empty(){misses.push(serde_json::json!({"row":row,"differences":results}));}
  }
  reports.push(serde_json::json!({"source":path,"function":name,"rows":bank["witnesses"].as_array().unwrap().len(),"exact":counts,"misses":misses}));
 }
 println!("{}",serde_json::to_string(&reports).unwrap());
}
