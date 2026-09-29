//! Test whether cancellation-sensitive compositions retain an extended EXP.
#[cfg(not(all(feature="research-x87",target_arch="x86_64")))]
fn main(){eprintln!("Requires research-x87 on x86_64");}
#[cfg(all(feature="research-x87",target_arch="x86_64"))]
fn main(){
 use oxfunc_core::excel_numeric::research::*;
 let cw=CW_PC64_RN;
 let store=|v:&Ext80|ext_from_f64(ext_to_f64(v,cw));
 let exp_ext=|x:f64|{
  let t=ext_mul(&ext_from_f64(x),&ext_l2e(),cw);let k=ext_rndint(&t,cw);let f=ext_sub(&t,&k,cw);
  let mut m=ext_add(&ext_f2xm1(&ext_abs(&f,cw),cw),&ext_one(),cw);
  if ext_to_f64(&f,cw)<0. {m=ext_div(&ext_one(),&m,cw);}
  ext_scale(&m,&k,cw)
 };
 // mode: u store, log argument store, log result store, product choice,
 // quotient store. Stored-u branch is varied separately.
 let em=|t:f64,mode:usize,branch:usize|{
  let ue=exp_ext(t);let uf=ext_to_f64(&ue,cw);
  let u=if mode&1==0{ue}else{store(&ue)};
  let delta=ext_sub(&u,&ext_one(),cw);
  if (branch==0 && uf==1.) || (branch==1 && ext_to_f64(&delta,cw)==0.) {return ext_from_f64(t);}
  let logarg=if mode&2==0{u}else{store(&u)};
  let log=ext_fyl2x(&ext_ln2(),&logarg,cw);
  let log=if mode&4==0{log}else{store(&log)};
  let pmode=(mode>>3)%3;
  let product=match pmode {0=>ext_from_f64(ext_to_f64(&delta,cw)*t),1=>store(&ext_mul(&delta,&ext_from_f64(t),cw)),_=>ext_mul(&delta,&ext_from_f64(t),cw)};
  let quotient=ext_div(&product,&log,cw);
  if mode&32==0{quotient}else{store(&quotient)}
 };
 let mut reports=Vec::new();
 for path in std::env::args().skip(1){
  let bank:serde_json::Value=serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
  let mut counts=vec![0usize;256];let mut misses=vec![Vec::new();256];let mut rows=0;
  for row in bank["witnesses"].as_array().unwrap(){
   let x=f64::from_bits(u64::from_str_radix(row["args"][0].as_str().unwrap().trim_start_matches("0x"),16).unwrap());
   if x==0. || x.abs()>=1. || x.abs()<f64::MIN_POSITIVE{continue;}rows+=1;
   for mode in 0..64{for branch in 0..2{for substore in 0..2{
    let p=em(x,mode,branch);let n=em(-x,mode,branch);let index=mode*4+branch*2+substore;
    let v=if substore==0{(ext_to_f64(&p,cw)-ext_to_f64(&n,cw))/2.}else{ext_to_f64(&ext_sub(&p,&n,cw),cw)/2.};
    let actual=format!("0x{:016x}",v.to_bits());
    if actual==row["expected_bits"].as_str().unwrap(){counts[index]+=1;}
    else if misses[index].len()<8{misses[index].push(serde_json::json!({"row":row,"actual":actual}));}
   }}}
  }
  let mut ranked:Vec<_>=(0..256).collect();ranked.sort_by_key(|&i|std::cmp::Reverse(counts[i]));
  let best:Vec<_>=ranked.iter().take(12).map(|&i|serde_json::json!({"mode":i/4,"branch":(i/2)%2,"substore":i%2,"matches":counts[i],"misses":misses[i]})).collect();
  reports.push(serde_json::json!({"source":path,"rows":rows,"all_counts":counts,"best":best}));
 }
 println!("{}",serde_json::to_string(&reports).unwrap());
}
