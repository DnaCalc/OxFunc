//! W109 G6-01: final expm1 |tau|<1 staging test - mixed extended-tau in the Kahan
//! correction. If none beats the all-double 163/234, the expm1 double-rounding is a
//! genuine op-graph wall. tau_ext=-n*ln(1+r) [ext, 1+r exact @ po2].
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use rx::{Ext80, ext_abs, ext_add, ext_chs, ext_div, ext_f2xm1, ext_from_f64 as e, ext_fyl2x,
         ext_l2e, ext_ln2, ext_mul, ext_one, ext_rndint, ext_scale, ext_sub, ext_to_f64, CW_PC64_RN as CW};
use std::collections::BTreeMap;

fn tf(x:&Ext80)->f64{ ext_to_f64(x,CW) }
fn tau_ext(r:f64,n:f64)->Ext80{
    let u=ext_add(&ext_one(),&e(r),CW);
    ext_chs(&ext_mul(&e(n),&ext_fyl2x(&ext_ln2(),&u,CW),CW),CW)
}
fn exp_ext(t:&Ext80)->Ext80{
    let tt=ext_mul(t,&ext_l2e(),CW); let k=ext_rndint(&tt,CW); let f=ext_sub(&tt,&k,CW);
    let neg=tf(&f)<0.0; let w=ext_f2xm1(&ext_abs(&f,CW),CW); let mut m=ext_add(&w,&ext_one(),CW);
    if neg{ m=ext_div(&ext_one(),&m,CW); } ext_scale(&m,&k,CW)
}
fn tau_d(r:f64,n:f64)->f64{ -(n*rx::excel_log1p(r)) }

// M0: all double (production) = 163 baseline
fn m0(r:f64,n:f64)->f64{ let t=tau_d(r,n); let u=rx::excel_exp(t); (u-1.0)*t/rx::excel_ln(u) }
// M1: tau ext, u=RN_dbl(exp_ext), correction ext with tau_ext, round once
fn m1(r:f64,n:f64)->f64{
    let te=tau_ext(r,n); let ud=ext_to_f64(&exp_ext(&te),CW);
    let u=e(ud); let num=ext_mul(&ext_sub(&u,&ext_one(),CW),&te,CW);
    let den=ext_fyl2x(&ext_ln2(),&u,CW);   // ln(u) ext (u exact double)
    tf(&ext_div(&num,&den,CW))
}
// M2: tau_d double, but numerator (u-1)*tau_d and /ln(u) in EXTENDED, round once
fn m2(r:f64,n:f64)->f64{
    let t=tau_d(r,n); let u=rx::excel_exp(t);
    let num=ext_mul(&ext_sub(&e(u),&ext_one(),CW),&e(t),CW);
    let den=e(rx::excel_ln(u));
    tf(&ext_div(&num,&den,CW))
}
// M3: fully ext (tau_ext, u ext, all ext, round once) = ext_kahan
fn m3(r:f64,n:f64)->f64{
    let te=tau_ext(r,n); let u=exp_ext(&te);
    let num=ext_mul(&ext_sub(&u,&ext_one(),CW),&te,CW);
    let den=ext_fyl2x(&ext_ln2(),&u,CW);
    tf(&ext_div(&num,&den,CW))
}
// M4: tau_d, u double, num double (u-1)*t, den=ln(u) double, divide in ext (only final divide ext)
fn m4(r:f64,n:f64)->f64{
    let t=tau_d(r,n); let u=rx::excel_exp(t);
    let num=(u-1.0)*t; tf(&ext_div(&e(num),&e(rx::excel_ln(u)),CW))
}

fn pin_em(rows:&[(f64,u64)], r:f64)->Option<f64>{
    let (pv,pmtb)=rows[rows.len()/2]; let pmt=f64::from_bits(pmtb);
    let center=pv/(pmt/r); let cb=center.to_bits() as i64;
    for d in -8..=8i64{ let em=f64::from_bits((cb+d) as u64); if em==-1.0{continue}
        if rows.iter().all(|(pv,want)|((pv/em)*r).to_bits()==*want){ return Some(em); } }
    None
}
fn main(){
    let ws:WitnessSet=serde_json::from_str(&std::fs::read_to_string(
        "../../work/w109/G6-solvers/answers-pmt-po2n.json").unwrap()).unwrap();
    let mut byrn:BTreeMap<(u64,u64),Vec<(f64,u64)>>=BTreeMap::new();
    for w in &ws.witnesses{
        let a:Vec<f64>=w.args.iter().filter_map(|x|match x{WitnessArg::Scalar(s)=>parse_bits_hex(s),_=>None}).collect();
        if a.len()!=5||a[3]!=0.0||a[4]!=0.0{continue}
        let want=parse_bits_hex(&w.expected_bits).unwrap().to_bits();
        byrn.entry((a[0].to_bits(),a[1].to_bits())).or_default().push((a[2],want));
    }
    let cands:[(&str,fn(f64,f64)->f64);5]=[("M0_alldouble",m0),("M1_exttau_dblu",m1),
        ("M2_dbltau_extcorr",m2),("M3_fullext",m3),("M4_extdiv",m4)];
    let mut sc=[0u32;5]; let mut tot=0u32;
    for ((rb,nb),rows) in &byrn{
        let r=f64::from_bits(*rb); let n=f64::from_bits(*nb);
        if tau_d(r,n).abs()>=1.0{continue}
        let em_ex=match pin_em(rows,r){Some(e)=>e,None=>continue}; tot+=1;
        for (i,(_,f)) in cands.iter().enumerate(){ if f(r,n).to_bits()==em_ex.to_bits(){sc[i]+=1;} }
    }
    println!("|tau|<1 pinned: {}", tot);
    for (i,(nm,_)) in cands.iter().enumerate(){ println!("  {:<20} {}/{}", nm, sc[i], tot); }
}
