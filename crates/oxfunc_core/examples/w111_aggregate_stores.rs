//! Offline stored-arithmetic candidates for broad-sweep aggregate discrepancies.
#[cfg(not(all(feature="research-x87",target_arch="x86_64")))]
fn main(){eprintln!("Requires research-x87 on x86_64");}
#[cfg(all(feature="research-x87",target_arch="x86_64"))]
fn main(){
 use oxfunc_core::excel_numeric::research::*;
 let excel_x87_add=|a,b|ext_to_f64(&ext_add(&ext_from_f64(a),&ext_from_f64(b),CW_PC64_RN),CW_PC64_RN);
 let excel_x87_sub=|a,b|ext_to_f64(&ext_sub(&ext_from_f64(a),&ext_from_f64(b),CW_PC64_RN),CW_PC64_RN);
 let excel_x87_mul=|a,b|x87_mul(a,b);
 let excel_x87_div=|a,b|ext_to_f64(&ext_div(&ext_from_f64(a),&ext_from_f64(b),CW_PC64_RN),CW_PC64_RN);
 let mut reports=Vec::new();
 for path in std::env::args().skip(1){
  let bank:serde_json::Value=serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();let function=bank["function"].as_str().unwrap();let mut counts=vec![0usize;256];let mut misses=Vec::new();let mut rows=0;
  for row in bank["witnesses"].as_array().unwrap(){
   let xs:Vec<_>=row["args"].as_array().unwrap().iter().map(|x|f64::from_bits(u64::from_str_radix(x.as_str().unwrap().trim_start_matches("0x"),16).unwrap())).collect();
   if xs.iter().any(|x|x.to_bits()==1u64<<63 || (x.abs()<f64::MIN_POSITIVE && *x!=0.)){continue;}rows+=1;
   for mode in 0..256{
    let add=|a,b|if mode&1==0{a+b}else{excel_x87_add(a,b)};
    let div=|a,b|if mode&2==0{a/b}else{excel_x87_div(a,b)};
    let sub=|a,b|if mode&4==0{a-b}else{excel_x87_sub(a,b)};
    let mul=|a,b|if mode&8==0{a*b}else{excel_x87_mul(a,b)};
    let v=match function{
     "HARMEAN"=>if xs.iter().any(|x|*x<=0.){Err("Num")}else{
      let sum=xs.iter().fold(0.,|s,x|add(s,div(1.,*x)));if mode&128!=0 && !sum.is_finite(){Err("Num")}else{Ok(if mode&64==0{div(xs.len() as f64,sum)}else{div(1.,div(sum,xs.len() as f64))})}
     },
     "DEVSQ"=>{
      let mean=div(xs.iter().fold(0.,|a,x|add(a,*x)),xs.len() as f64);
      Ok(xs.iter().fold(0.,|s,x|{let d=sub(*x,mean);let p=mul(d,d);add(s,if mode&128!=0 && p<f64::MIN_POSITIVE{0.}else{p})}))
     },_=>panic!("unsupported")
    };
    let v=v.and_then(|v|if mode&16!=0 && !v.is_finite(){Err("Num")}else{Ok(if mode&32!=0 && v.abs()<f64::MIN_POSITIVE{0.}else{v})});
    let actual=match v{Ok(v)=>format!("0x{:016x}",v.to_bits()),Err(e)=>format!("error:{e}")};
    if actual==row["expected_bits"].as_str().unwrap(){counts[mode]+=1;}
    else if mode==if function=="HARMEAN"{195}else{184} && misses.len()<50{misses.push(serde_json::json!({"row":row,"actual":actual}));}
   }
  }
  reports.push(serde_json::json!({"source":path,"function":function,"rows":rows,"matches":counts,"selected_misses":misses}));
 }
 println!("{}",serde_json::to_string(&reports).unwrap());
}
