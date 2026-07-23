//! W109 G6-01: nail Excel's expm1 for |tau|<1 (the SOLE failing branch). po2xn
//! model-free em oracle, restricted to |tau|<1. Candidates: Kahan associations,
//! and the x87-native F2XM1 path in the log2 domain (em = 2^(-n*log2(1+r)) - 1
//! via FYL2XP1 + F2XM1, the legacy financial-runtime primitive).
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use rx::{Ext80, ext_abs, ext_add, ext_chs, ext_div, ext_f2xm1, ext_from_f64, ext_fyl2x,
         ext_fyl2xp1, ext_l2e, ext_ln2, ext_mul, ext_one, ext_rndint, ext_scale, ext_sub,
         ext_to_f64, CW_PC64_RN};
use std::collections::BTreeMap;

const CW:u16=CW_PC64_RN;
fn e(x:f64)->Ext80{ ext_from_f64(x) }
fn tf(x:&Ext80)->f64{ ext_to_f64(x,CW) }

// current tau (double): -n*log1p_cr(r)
fn tau_d(r:f64,n:f64)->f64{ -(n*rx::excel_log1p(r)) }

// --- candidates: given (r,n) return em, ASSUMING |tau|<1 ---
// A: production ((u-1)*t)/ln(u), all double
fn a_prod(r:f64,n:f64)->f64{ let t=tau_d(r,n); let u=rx::excel_exp(t); (u-1.0)*t/rx::excel_ln(u) }
// B: Kahan canonical (u-1)*(t/ln u), all double
fn b_kahan(r:f64,n:f64)->f64{ let t=tau_d(r,n); let u=rx::excel_exp(t); (u-1.0)*(t/rx::excel_ln(u)) }
// C: t/ln(u)*(u-1)  (divide first, mult last) all double
fn c_divfirst(r:f64,n:f64)->f64{ let t=tau_d(r,n); let u=rx::excel_exp(t); (t/rx::excel_ln(u))*(u-1.0) }
// D: F2XM1 direct in log2 domain: y=-n*log2(1+r) via fyl2xp1; em=F2XM1_ext(y), store.
//    valid only |y|<1 (F2XM1 domain). y = -n * log2(1+r); log2(1+r)=fyl2xp1(1,r).
fn d_f2xm1_log2(r:f64,n:f64)->Option<f64>{
    let y=ext_chs(&ext_mul(&e(n),&ext_fyl2xp1(&ext_one(),&e(r),CW),CW),CW); // -n*log2(1+r) ext
    if tf(&y).abs()>=1.0{ return None; }
    Some(tf(&ext_f2xm1(&y,CW)))
}
// E: F2XM1 with y computed from double tau (tau*log2e): em=F2XM1(tau_d*log2e) ext store
fn e_f2xm1_taud(r:f64,n:f64)->Option<f64>{
    let t=tau_d(r,n);
    let y=ext_mul(&e(t),&ext_l2e(),CW);
    if tf(&y).abs()>=1.0{ return None; }
    Some(tf(&ext_f2xm1(&y,CW)))
}
// F: general 2^y-1 via k-split (ext), y=-n*log2(1+r) fyl2xp1, single store. handles |y|>=1 too
fn f_pow2m1(r:f64,n:f64)->f64{
    let y=ext_chs(&ext_mul(&e(n),&ext_fyl2xp1(&ext_one(),&e(r),CW),CW),CW);
    let k=ext_rndint(&y,CW);
    let f=ext_sub(&y,&k,CW);
    let w=ext_f2xm1(&ext_abs(&f,CW),CW);
    let neg=tf(&f)<0.0;
    let mut m=ext_add(&w,&ext_one(),CW);
    if neg{ m=ext_div(&ext_one(),&m,CW); }
    // 2^y = 2^k * m ; em = 2^k*m - 1 = scale(m,k)-1
    tf(&ext_sub(&ext_scale(&m,&k,CW),&ext_one(),CW))
}
// G: Kahan but u from EXT exp on double tau (u extended), (u-1)*t/ln(u) mixed? all double after
fn g_extu(r:f64,n:f64)->f64{
    let t=tau_d(r,n);
    // u via ext exp of double tau, store double
    let xe=e(t); let tt=ext_mul(&xe,&ext_l2e(),CW); let k=ext_rndint(&tt,CW); let ff=ext_sub(&tt,&k,CW);
    let neg=tf(&ff)<0.0; let w=ext_f2xm1(&ext_abs(&ff,CW),CW); let mut m=ext_add(&w,&ext_one(),CW);
    if neg{ m=ext_div(&ext_one(),&m,CW); }
    let u=tf(&ext_scale(&m,&k,CW));
    (u-1.0)*t/rx::excel_ln(u)
}

fn sordi(u:u64)->i64{ if u<1<<63 {u as i64} else { -((u ^ (1u64<<63)) as i64) } }
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
    // restrict to |tau|<1
    let names=["A_prod","B_kahan","C_divfirst","D_f2xm1log2","E_f2xm1taud","F_pow2m1","G_extu"];
    let mut score=[0u32;7]; let mut tot=0u32; let mut detail=Vec::new();
    for ((rb,nb),rows) in &byrn{
        let r=f64::from_bits(*rb); let n=f64::from_bits(*nb);
        if tau_d(r,n).abs()>=1.0{ continue; }
        let em_ex=match pin_em(rows,r){Some(e)=>e,None=>continue}; tot+=1;
        let gs=[Some(a_prod(r,n)),Some(b_kahan(r,n)),Some(c_divfirst(r,n)),
                d_f2xm1_log2(r,n),e_f2xm1_taud(r,n),Some(f_pow2m1(r,n)),Some(g_extu(r,n))];
        let mut ds=[99i64;7];
        for i in 0..7{ if let Some(g)=gs[i]{ let d=sordi(g.to_bits())-sordi(em_ex.to_bits()); ds[i]=d; if d==0{score[i]+=1;} } }
        if detail.len()<32{ detail.push(format!("r=2^{:>3} n={:>4}: {:?}", r.log2() as i64, n as i64, ds)); }
    }
    println!("|tau|<1 pinned points: {}", tot);
    for i in 0..7{ println!("  {:<14} {}/{}", names[i], score[i], tot); }
    println!("\ndetail (offset from Excel em; 99=out-of-domain) [{}]:", names.join(","));
    for d in &detail{ println!("  {}", d); }
}
