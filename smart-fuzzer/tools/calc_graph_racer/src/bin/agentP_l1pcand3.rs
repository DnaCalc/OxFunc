use oxfunc_core::excel_numeric::research as rx;
use std::collections::BTreeMap;
const CW: u16 = rx::CW_PC64_RN;
fn fb(h:&str)->f64{ f64::from_bits(u64::from_str_radix(h.trim_start_matches("0x"),16).unwrap()) }
fn e(x:f64)->rx::Ext80{ rx::ext_from_f64(x) }
fn tf(x:&rx::Ext80)->f64{ rx::ext_to_f64(x,CW) }
fn kahan_f64(tau:f64)->f64{ let u=rx::excel_exp(tau); let l=rx::excel_ln(u); if u==1.0{tau}else if tau.abs()<1.0{(u-1.0)*tau/l}else{u-1.0} }
fn main(){
    let v:serde_json::Value=serde_json::from_str(&std::fs::read_to_string("../../work/w109/G6-solvers/agentP_log1p_em.json").unwrap()).unwrap();
    let mut rows:Vec<(f64,f64)>=v.as_object().unwrap().iter().map(|(k,val)|(fb(k),fb(val.as_str().unwrap()))).collect();
    rows.sort_by(|a,b|a.0.partial_cmp(&b.0).unwrap());
    let n=rows.len();
    let mut res:BTreeMap<&str,u32>=BTreeMap::new();
    for (r,em) in &rows{
        // fyl2x on extended-exact (1+r)
        let onepr=rx::ext_add(&rx::ext_one(),&e(*r),CW); // exact for these r
        let l1=tf(&rx::ext_fyl2x(&rx::ext_ln2(),&onepr,CW));
        if kahan_f64(-l1).to_bits()==em.to_bits(){*res.entry("fyl2x(ext 1+r) stored").or_default()+=1;}
        // fyl2x on extended (1+r), kept EXTENDED into em numerator
        let tau_ext=rx::ext_sub(&e(0.0),&rx::ext_fyl2x(&rx::ext_ln2(),&onepr,CW),CW);
        let tf64=tf(&tau_ext); let u=rx::excel_exp(tf64); let ln=rx::excel_ln(u);
        let emc= if u==1.0{tf64} else if tf64.abs()<1.0 { tf(&rx::ext_mul(&e(u-1.0),&tau_ext,CW))/ln } else {u-1.0};
        if emc.to_bits()==em.to_bits(){*res.entry("fyl2x(ext 1+r) EXT numer").or_default()+=1;}
        // fyl2xp1 extended kept, exp from extended tau (u also from ext), full ext em
        let le=rx::ext_fyl2xp1(&rx::ext_ln2(),&e(*r),CW);
        let te=rx::ext_sub(&e(0.0),&le,CW);
        // exp on extended tau
        let z=rx::ext_mul(&te,&rx::ext_l2e(),CW); let k=rx::ext_rndint(&z,CW); let f=rx::ext_sub(&z,&k,CW);
        let neg=tf(&f)<0.0; let w=rx::ext_f2xm1(&rx::ext_abs(&f,CW),CW); let mut m=rx::ext_add(&w,&rx::ext_one(),CW); if neg{m=rx::ext_div(&rx::ext_one(),&m,CW);} 
        let ue=rx::ext_scale(&m,&k,CW); let u2=tf(&ue);
        let lnu2=rx::ext_fyl2x(&rx::ext_ln2(),&ue,CW);
        let emf= if u2==1.0{tf(&te)} else if tf(&te).abs()<1.0 { tf(&rx::ext_div(&rx::ext_mul(&rx::ext_sub(&ue,&rx::ext_one(),CW),&te,CW),&lnu2,CW)) } else {u2-1.0};
        if emf.to_bits()==em.to_bits(){*res.entry("FULL extended (fyl2xp1)").or_default()+=1;}
    }
    println!("candidates vs extracted em ({} r):",n);
    for (k,c) in &res{ println!("  {:28} {:3}/{}",k,c,n); }
}
