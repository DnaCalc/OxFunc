//! Distinguish: is the r=2^-5/2^-4 +1 due to (a) non-CR log1p, or (b) Excel keeping
//! tau EXTENDED (fyl2xp1_ext * -n) into the Kahan NUMERATOR? Test extended-tau in
//! the numerator (u-1)*tau_ext, u & lnu as f64, on the pox oracle. Report per-r.
use oxfunc_core::excel_numeric::research as rx;
use std::collections::BTreeMap;
const CW: u16 = rx::CW_PC64_RN;
fn fb(h:&str)->f64{ f64::from_bits(u64::from_str_radix(h.trim_start_matches("0x"),16).unwrap()) }
fn e(x:f64)->rx::Ext80{ rx::ext_from_f64(x) }
fn tf(x:&rx::Ext80)->f64{ rx::ext_to_f64(x,CW) }
fn load(path:&str)->Vec<(f64,i64,f64)>{
    let v:serde_json::Value=serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let mut o=Vec::new();
    for (k,val) in v.as_object().unwrap(){ let (rh,nh)=k.split_once('|').unwrap(); o.push((fb(rh),nh.parse().unwrap(),fb(val.as_str().unwrap()))); }
    o.sort_by(|a,b|(a.0,a.1).partial_cmp(&(b.0,b.1)).unwrap()); o
}
fn main(){
    for (label,path) in [("HDF46","../../work/w109/G6-solvers/pmt_em_hdf_oracle.json"),
                         ("POX","../../work/w109/G6-solvers/agentP_pox_em.json")]{
        let rows=load(path);
        // variants
        let mut res:BTreeMap<&str,BTreeMap<i32,(u32,u32)>>=BTreeMap::new(); // name-> k -> (ok,tot)
        for (r,n,em) in &rows{
            let k=(r.log2().round()) as i32;
            let lp_ext=rx::ext_fyl2xp1(&rx::ext_ln2(),&e(*r),CW);
            let tau_ext=rx::ext_mul(&e(-(*n as f64)),&lp_ext,CW); // extended tau
            let tau_f64=-(*n as f64)*rx::excel_log1p(*r);
            let u=rx::excel_exp(tau_f64); // u from f64 tau (insensitive anyway)
            let lnu=rx::excel_ln(u);
            // (a) ref f64 kahan
            let a= if u==1.0 {tau_f64} else if tau_f64.abs()<1.0 {(u-1.0)*tau_f64/lnu} else {u-1.0};
            // (b) extended tau in numerator: num=RN53((u-1)*tau_ext), em=num/lnu
            let b= if u==1.0 {tau_f64} else if tau_f64.abs()<1.0 {
                let num=tf(&rx::ext_mul(&e(u-1.0),&tau_ext,CW)); num/lnu
            } else {u-1.0};
            // (c) fully extended numerator+divide with tau_ext, single store
            let c= if u==1.0 {tau_f64} else if tau_f64.abs()<1.0 {
                tf(&rx::ext_div(&rx::ext_mul(&e(u-1.0),&tau_ext,CW),&e(lnu),CW))
            } else {u-1.0};
            for (nm,val) in [("a f64kahan",a),("b extTau num",b),("c extTau num+div",c)]{
                let ent=res.entry(nm).or_default().entry(k).or_insert((0,0));
                ent.1+=1; if val.to_bits()==em.to_bits(){ent.0+=1;}
            }
        }
        println!("=== {} ===",label);
        for (nm,km) in &res{
            let tot:u32=km.values().map(|x|x.1).sum();
            let ok:u32=km.values().map(|x|x.0).sum();
            print!("  {:18} {:3}/{:3}  per-r:", nm, ok, tot);
            for (k,(o,t)) in km{ print!(" 2^{}:{}/{}",k,o,t); }
            println!();
        }
    }
}
