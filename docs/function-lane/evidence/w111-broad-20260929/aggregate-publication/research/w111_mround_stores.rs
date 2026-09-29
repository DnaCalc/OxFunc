#[cfg(not(all(feature="research-x87",target_arch="x86_64")))]fn main(){}
#[cfg(all(feature="research-x87",target_arch="x86_64"))]
fn main(){
 use oxfunc_core::excel_numeric::research::*;
 let divide=|a,b|ext_to_f64(&ext_div(&ext_from_f64(a),&ext_from_f64(b),CW_PC64_RN),CW_PC64_RN);
 let mut reports=Vec::new();
 for path in std::env::args().skip(1){
  let bank:serde_json::Value=serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();let mut counts=vec![0;64];let mut misses=Vec::new();
  for row in bank["witnesses"].as_array().unwrap(){
   let xs:Vec<_>=row["args"].as_array().unwrap().iter().map(|v|f64::from_bits(u64::from_str_radix(v.as_str().unwrap().trim_start_matches("0x"),16).unwrap())).collect();let x=xs[0];let m=xs[1];
   for mode in 0..64{
    let result=if x==0. || m==0.{Ok(0.)}else if x.signum()!=m.signum(){Err("Num")}else{
     let q=if mode<32{if mode&1==0{x/m}else{divide(x,m)}}else{let r=if mode&1==0{1./m}else{x87_recip(m)};if mode&8==0{x*r}else{x87_mul(x,r)}};
     let k=if mode>=32{if mode&16==0{q.round()}else{(q+0.5).floor()}}else{match mode/8 {0=>q.round(),1=>(q+0.5).floor(),2=>{let t=x+m/2.;if mode&1==0{(t/m).floor()}else{divide(t,m).floor()}},_=>ext_to_f64(&ext_add(&ext_div(&ext_from_f64(x),&ext_from_f64(m),CW_PC64_RN),&ext_from_f64(0.5),CW_PC64_RN),CW_PC64_RN).floor()}};
     let v=if mode&2==0{k*m}else{x87_mul(k,m)};
     if mode&4!=0 && !v.is_finite(){Err("Num")}else{Ok(if mode&4!=0 && v==0.{0.}else{v})}
    };
    let actual=match result{Ok(v)=>format!("0x{:016x}",v.to_bits()),Err(e)=>format!("error:{e}")};
    if actual==row["expected_bits"].as_str().unwrap(){counts[mode]+=1;}
    else if mode==7{misses.push(serde_json::json!({"row":row,"actual":actual}));}
   }
  }
  reports.push(serde_json::json!({"source":path,"rows":bank["witnesses"].as_array().unwrap().len(),"matches":counts,"selected_misses":misses}));
 }
 println!("{}",serde_json::to_string(&reports).unwrap());
}
