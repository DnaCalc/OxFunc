//! Test chop/RZ publication hypotheses on the clean 46-oracle, per-case on the 6.
use oxfunc_core::excel_numeric::research as rx;
const CW: u16 = rx::CW_PC64_RN;
const CW_RZ: u16 = rx::CW_PC64_RN | 0x0C00;
fn fb(h:&str)->f64{ f64::from_bits(u64::from_str_radix(h.trim_start_matches("0x"),16).unwrap()) }
fn e(x:f64)->rx::Ext80{ rx::ext_from_f64(x) }
fn tf(x:&rx::Ext80)->f64{ rx::ext_to_f64(x,CW) }
fn tfr(x:&rx::Ext80,cw:u16)->f64{ rx::ext_to_f64(x,cw) }
fn load(path:&str)->Vec<(f64,i64,f64)>{
    let v:serde_json::Value=serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let mut o=Vec::new();
    for (k,val) in v.as_object().unwrap(){ let (rh,nh)=k.split_once('|').unwrap(); o.push((fb(rh),nh.parse().unwrap(),fb(val.as_str().unwrap()))); }
    o.sort_by(|a,b|(a.0,a.1).partial_cmp(&(b.0,b.1)).unwrap()); o
}
fn main(){
    let rows=load("../../work/w109/G6-solvers/pmt_em_hdf_oracle.json");
    // variants: (name, em fn)
    // baseline u=exp RN
    let variants:Vec<(&str, Box<dyn Fn(f64)->f64>)>=vec![
        ("RN u, RN kahan (ref)", Box::new(|t:f64|{ let u=rx::excel_exp(t); if u==1.0 {t} else if t.abs()<1.0 {(u-1.0)*t/rx::excel_ln(u)} else {u-1.0} })),
        ("RZ u (exp_rz), RN kahan", Box::new(|t:f64|{ let u=rx::excel_exp_rz(t); if u==1.0 {t} else if t.abs()<1.0 {(u-1.0)*t/rx::excel_ln(u)} else {u-1.0} })),
        ("RN u, kahan final divide RZ", Box::new(|t:f64|{ let u=rx::excel_exp(t); if u==1.0 {return t;} if t.abs()<1.0 { let p=e((u-1.0)*t); tfr(&rx::ext_div(&p,&e(rx::excel_ln(u)),CW),CW_RZ) } else {u-1.0} })),
        ("RN u, kahan all-ext RZ store", Box::new(|t:f64|{ let u=rx::excel_exp(t); if u==1.0 {return t;} if t.abs()<1.0 { let q=rx::ext_div(&rx::ext_mul(&e(u-1.0),&e(t),CW),&e(rx::excel_ln(u)),CW); tfr(&q,CW_RZ) } else {u-1.0} })),
        ("RN u, whole em RZ from ext exp", Box::new(|t:f64|{
            // u extended, kahan extended, store RZ
            let te=e(t); let z=rx::ext_mul(&te,&rx::ext_l2e(),CW); let k=rx::ext_rndint(&z,CW); let f=rx::ext_sub(&z,&k,CW);
            let neg=tf(&f)<0.0; let w=rx::ext_f2xm1(&rx::ext_abs(&f,CW),CW); let mut m=rx::ext_add(&w,&rx::ext_one(),CW); if neg{m=rx::ext_div(&rx::ext_one(),&m,CW);} 
            let ue=rx::ext_scale(&m,&k,CW); let u=tf(&ue); if u==1.0 {return t;}
            if t.abs()<1.0 { let lnu=rx::ext_fyl2x(&rx::ext_ln2(),&ue,CW); let q=rx::ext_div(&rx::ext_mul(&rx::ext_sub(&ue,&rx::ext_one(),CW),&te,CW),&lnu,CW); tfr(&q,CW_RZ)} else {u-1.0}
        })),
    ];
    let miss=[(-5,2),(-5,3),(-5,5),(-5,12),(-5,24),(-4,2)];
    for (name,f) in &variants{
        let mut ok=0; let mut d6=Vec::new();
        for (r,n,em) in &rows{
            let t=-(*n as f64)*rx::excel_log1p(*r);
            let val=f(t);
            let d=val.to_bits() as i64 - em.to_bits() as i64;
            if d==0 {ok+=1;}
            let k=(r.log2().round()) as i32;
            if miss.contains(&(k,*n)){ d6.push(((k,*n),d)); }
        }
        print!("  {:32} {:2}/46   on-6:", name, ok);
        for ((k,n),d) in &d6{ print!(" (2^{},{}):{:+}",k,n,d); }
        println!();
    }
}
