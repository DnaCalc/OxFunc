//! Test candidate log1p op-graphs against the extracted n=1 em (agentP_log1p_em.json).
//! em = Kahan(tau=-log1p(r)). Compare which log1p reproduces em bit-exact per r.
use oxfunc_core::excel_numeric::research as rx;
use std::collections::BTreeMap;
const CW: u16 = rx::CW_PC64_RN;
fn fb(h:&str)->f64{ f64::from_bits(u64::from_str_radix(h.trim_start_matches("0x"),16).unwrap()) }
fn e(x:f64)->rx::Ext80{ rx::ext_from_f64(x) }
fn tf(x:&rx::Ext80)->f64{ rx::ext_to_f64(x,CW) }
fn kahan_f64(tau:f64)->f64{ let u=rx::excel_exp(tau); let l=rx::excel_ln(u); if u==1.0{tau}else if tau.abs()<1.0{(u-1.0)*tau/l}else{u-1.0} }
// Kahan with EXTENDED tau in numerator (u,lnu from f64 tau)
fn kahan_exttau(tau_ext:&rx::Ext80)->f64{
    let tf64=tf(tau_ext); let u=rx::excel_exp(tf64); let l=rx::excel_ln(u);
    if u==1.0 {tf64} else if tf64.abs()<1.0 { let num=tf(&rx::ext_mul(&e(u-1.0),tau_ext,CW)); num/l } else {u-1.0}
}
fn fdlibm_log1p(x:f64)->f64{
    const LP1:f64=6.666666666666735130e-01; const LP2:f64=3.999999999940941908e-01;
    const LP3:f64=2.857142874366239149e-01; const LP4:f64=2.222219843214978396e-01;
    const LP5:f64=1.818357216161805012e-01; const LP6:f64=1.531383769920937332e-01; const LP7:f64=1.479819860511658591e-01;
    if x==0.0{return x;} let f=x; let hfsq=0.5*f*f; let s=f/(2.0+f); let z=s*s;
    let r=z*(LP1+z*(LP2+z*(LP3+z*(LP4+z*(LP5+z*(LP6+z*LP7)))))); f-(hfsq-s*(hfsq+r))
}
fn main(){
    let v:serde_json::Value=serde_json::from_str(&std::fs::read_to_string("../../work/w109/G6-solvers/agentP_log1p_em.json").unwrap()).unwrap();
    let mut rows:Vec<(f64,f64)>=v.as_object().unwrap().iter().map(|(k,val)|(fb(k),fb(val.as_str().unwrap()))).collect();
    rows.sort_by(|a,b|a.0.partial_cmp(&b.0).unwrap());
    let n=rows.len();
    let mut res:BTreeMap<&str,u32>=BTreeMap::new();
    let mut miss_ext=Vec::new();
    for (r,em) in &rows{
        // A portable log1p
        if kahan_f64(-rx::excel_log1p(*r)).to_bits()==em.to_bits(){*res.entry("A portable-log1p f64").or_default()+=1;}
        // B fyl2xp1 stored
        let l_f2=tf(&rx::ext_fyl2xp1(&rx::ext_ln2(),&e(*r),CW));
        if kahan_f64(-l_f2).to_bits()==em.to_bits(){*res.entry("B fyl2xp1-stored f64").or_default()+=1;}
        // C fyl2xp1 EXTENDED into Kahan numerator
        let lext=rx::ext_fyl2xp1(&rx::ext_ln2(),&e(*r),CW);
        let tau_ext=rx::ext_sub(&e(0.0),&lext,CW); // -log1p extended
        if kahan_exttau(&tau_ext).to_bits()==em.to_bits(){*res.entry("C fyl2xp1-EXT in numer").or_default()+=1;} else {miss_ext.push(*r);}
        // D fdlibm log1p f64
        if kahan_f64(-fdlibm_log1p(*r)).to_bits()==em.to_bits(){*res.entry("D fdlibm-log1p f64").or_default()+=1;}
        // E ln(1+r) x87 (1+r f64)
        if kahan_f64(-rx::excel_ln(1.0+*r)).to_bits()==em.to_bits(){*res.entry("E ln(1+r) x87").or_default()+=1;}
    }
    println!("n=1 em reproduced by log1p candidate ({} r):",n);
    for (k,c) in &res{ println!("  {:26} {:3}/{}",k,c,n); }
}
