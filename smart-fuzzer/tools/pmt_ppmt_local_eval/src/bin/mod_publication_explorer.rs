fn main(){
 let mut reports=Vec::new();
 for path in std::env::args().skip(1){
  let bank:serde_json::Value=serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();let mut counts=vec![0usize;17];let mut misses=Vec::new();
  for row in bank["witnesses"].as_array().unwrap(){
   let xs:Vec<_>=row["args"].as_array().unwrap().iter().map(|v|f64::from_bits(u64::from_str_radix(v.as_str().unwrap().trim_start_matches("0x"),16).unwrap())).collect();let x=xs[0];let d=xs[1];
   for mode in 0..17{
    let result=if d==0.{Err("Div0")}else if (x/d).abs()>=1_125_900_000_000.{Err("Num")}else{
     let r=x%d;let mut v=if r!=0. && r.is_sign_negative()!=d.is_sign_negative(){r+d}else{r};
     if (mode&1!=0 || mode>=8) && (v==0. || v==d){v=0.;}
     if mode<4 && mode&2!=0 && v.abs()<f64::MIN_POSITIVE{v=0.;}
     let threshold=if mode==11{f64::MIN_POSITIVE}else{f64::MIN_POSITIVE/16.};
     if mode>=6 && mode!=10 && r!=0. && (r.abs()<threshold || (mode==9 && r.abs()==threshold)) && x.is_sign_negative()!=d.is_sign_negative(){v=d-r;}
     if mode>=8 && r!=0. && r.abs()<f64::MIN_POSITIVE && (d.to_bits()&0xfffffffffffff!=0 || (mode>=12 && (x/d).abs()>=2f64.powi(24+(mode-12) as i32))) {Err("Num")}
     else if mode>=4 && v!=0. && v.abs()<f64::MIN_POSITIVE{Err("Num")}else{Ok(v)}
    };
    let actual=match result{Ok(v)=>format!("0x{:016x}",v.to_bits()),Err(e)=>format!("error:{e}")};
    if actual==row["expected_bits"].as_str().unwrap(){counts[mode]+=1;}
    else if mode==14 && misses.len()<100{misses.push(serde_json::json!({"row":row,"actual":actual}));}
   }
  }
  reports.push(serde_json::json!({"source":path,"rows":bank["witnesses"].as_array().unwrap().len(),"matches":counts,"selected_misses":misses}));
 }
 println!("{}",serde_json::to_string(&reports).unwrap());
}
