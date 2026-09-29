//! Arithmetic graphs motivated by the retained broad sweep, tested offline.
#[cfg(not(all(feature="research-x87",target_arch="x86_64")))]
fn main(){eprintln!("Requires research-x87 on x86_64");}
#[cfg(all(feature="research-x87",target_arch="x86_64"))]
fn main(){
 use oxfunc_core::excel_numeric::research::*;
 use oxfunc_core::functions::{power_fn::power_kernel,permutationa_fn::permutationa_kernel};
 let add=|a,b|ext_to_f64(&ext_add(&ext_from_f64(a),&ext_from_f64(b),CW_PC64_RN),CW_PC64_RN);
 let sub=|a,b|ext_to_f64(&ext_sub(&ext_from_f64(a),&ext_from_f64(b),CW_PC64_RN),CW_PC64_RN);
 let div=|a,b|ext_to_f64(&ext_div(&ext_from_f64(a),&ext_from_f64(b),CW_PC64_RN),CW_PC64_RN);
 let mut reports=Vec::new();
 for path in std::env::args().skip(1){
  let bank:serde_json::Value=serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();let function=bank["function"].as_str().unwrap();
  let mut counts=std::collections::BTreeMap::<String,usize>::new();let mut misses=std::collections::BTreeMap::<String,Vec<serde_json::Value>>::new();let mut rows=0;
  for row in bank["witnesses"].as_array().unwrap(){
   let args:Vec<_>=row["args"].as_array().unwrap().iter().map(|b|f64::from_bits(u64::from_str_radix(b.as_str().unwrap().trim_start_matches("0x"),16).unwrap())).collect();
   if args.iter().any(|x|x.to_bits()==1u64<<63 || (x.abs()<f64::MIN_POSITIVE && *x!=0.)){continue;}rows+=1;
   let mut candidates=Vec::new();
   match function{
    "FISHER"=>{
     let x=args[0];
     for m in 0..16{
      let u=if m&1==0{1.+x}else{add(1.,x)};
      let d=match (m/2)%4 {0=>1.-x,1=>2.-u,2=>sub(2.,u),_=>sub(1.,x)};
      let ratio=if m<8{u/d}else{div(u,d)};
      let value=if x.abs()>=1.{Err(oxfunc_core::value::WorksheetErrorCode::Num)}else{Ok(excel_ln(ratio)/2.)};
      candidates.push((format!("fisher-{m}"),value));
     }
    },
    "MEDIAN"=>{
     let mut values=args.clone();values.sort_by(f64::total_cmp);let n=values.len();let low=values[(n-1)/2];let high=values[n/2];
     for m in 0..7{
      let v=if n%2==1{high}else{match m {0=>(low+high)/2.,1=>low+(high-low)/2.,2=>add(low,(high-low)/2.),3=>low+sub(high,low)/2.,4=>add(low,sub(high,low)/2.),5=>high+(low-high)/2.,_=>{let d=(high-low)/2.;low+if d.abs()<f64::MIN_POSITIVE{0.}else{d}}}};
      candidates.push((format!("median-{m}"),if v.is_finite(){Ok(v)}else{Err(oxfunc_core::value::WorksheetErrorCode::Num)}));
     }
    },
    "LOG"=>{
     let result=oxfunc_core::functions::log_fn::log_kernel(args[0],args[1]);
     candidates.push(("baseline".into(),result));candidates.push(("zero-positive".into(),result.map(|v|if v==0.{0.}else{v})));
     candidates.push(("staged-zero-positive".into(),result.map(|_|{let v=div(excel_ln(args[0]),excel_ln(args[1]));if v==0.{0.}else{v}})));
    },
    "POWER"=>{
     let x=args[0];let p=args[1];
     candidates.push(("baseline".into(),power_kernel(x,p)));
     for m in 0..8{
      let value=if x==0. || p==0. || p.fract()!=0. || p.abs()>1000000. {power_kernel(x,p)}else{
       let mut n=p.abs() as u64;let mut acc=1.;let mut b=x;
       while n>0{if n&1!=0{acc=if m&1==0{acc*b}else{x87_mul(acc,b)};}n>>=1;if n>0{b=if m&1==0{b*b}else{x87_mul(b,b)};}}
       if m&4!=0 && x==10.{acc=permutationa_kernel(10.,p.abs()).unwrap_or(f64::INFINITY);}
       if p<0. && acc.abs()<f64::MIN_POSITIVE{Err(oxfunc_core::value::WorksheetErrorCode::Div0)}else{
        if p<0.{acc=if m&2==0{1./acc}else{x87_recip(acc)};}
        if acc.is_finite(){Ok(if acc.abs()<f64::MIN_POSITIVE{0.}else{acc})}else{Err(oxfunc_core::value::WorksheetErrorCode::Num)}
       }
      };
      candidates.push((format!("power-{m}"),value));
     }
    },_=>panic!("unknown function")
   }
   for (key,v) in candidates{
    let actual=match v{Ok(v)=>format!("0x{:016x}",v.to_bits()),Err(e)=>format!("error:{e:?}")};
    if actual==row["expected_bits"].as_str().unwrap(){*counts.entry(key).or_default()+=1;}
    else {counts.entry(key.clone()).or_default();if misses.entry(key.clone()).or_default().len()<20{misses.entry(key).or_default().push(serde_json::json!({"row":row,"actual":actual}));}}
   }
  }
  reports.push(serde_json::json!({"source":path,"function":function,"rows":rows,"matches":counts,"sample_misses":misses}));
 }
 println!("{}",serde_json::to_string(&reports).unwrap());
}
