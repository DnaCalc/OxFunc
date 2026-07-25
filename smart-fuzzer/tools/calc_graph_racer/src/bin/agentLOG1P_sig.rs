//! Characterize the discriminating log1p signatures: FYL2XP1-CR and std_ln1p-CR
//! ULP curves over the financial general-r range, and measure how often the
//! candidates disagree on the collide inexact-(1+r) em rows + oracle preference.
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_from_f64 as ef, ext_fyl2xp1, ext_fyl2x, ext_ln2, ext_add, ext_one, ext_to_f64, CW_PC64_RN as CW};
use std::collections::BTreeMap;

fn l1p_cr(r:f64)->f64{ rx::excel_log1p(r) }
fn l1p_std(r:f64)->f64{ r.ln_1p() }
fn l1p_fyl(r:f64)->f64{
    if r.abs()<0.292893218813452 { ext_to_f64(&ext_fyl2xp1(&ext_ln2(),&ef(r),CW),CW) }
    else { ext_to_f64(&ext_fyl2x(&ext_ln2(),&ext_add(&ext_one(),&ef(r),CW),CW),CW) }
}
fn ulp_diff(a:f64,b:f64)->i64{ a.to_bits() as i64 - b.to_bits() as i64 }
fn exact_1pr(r:f64)->bool{ (1.0+r)-1.0==r }
fn em_for(r:f64,n:f64,l1p:fn(f64)->f64)->f64{ rx::excel_expm1_internal(-(n*l1p(r))) }
fn pin_gen(rows:&[(f64,u64)], center:f64, r:f64)->Option<f64>{
    let cb=center.to_bits() as i64;
    for d in -24..=24i64{ let em=f64::from_bits((cb+d) as u64); if em>=0.0{continue}
        if rows.iter().all(|(pv,want)|(((pv/em)*r)).to_bits()==*want){ return Some(em); } }
    None
}
fn load(p:&str)->BTreeMap<(u64,u64),Vec<(f64,u64)>>{
    let ws:WitnessSet=serde_json::from_str(&std::fs::read_to_string(p).unwrap()).unwrap();
    let mut m=BTreeMap::new();
    for w in &ws.witnesses{
        let a:Vec<f64>=w.args.iter().filter_map(|x|match x{WitnessArg::Scalar(s)=>parse_bits_hex(s),_=>None}).collect();
        if a.len()!=5||a[3]!=0.0||a[4]!=0.0{continue}
        let want=parse_bits_hex(&w.expected_bits).unwrap().to_bits();
        m.entry((a[0].to_bits(),a[1].to_bits())).or_insert_with(Vec::new).push((a[2],want));
    }
    m
}
fn main(){
    // (1) signature curves over a pseudo-random general-r grid in [3e-4, 0.29]
    let mut hf:BTreeMap<i64,u32>=BTreeMap::new();
    let mut hs:BTreeMap<i64,u32>=BTreeMap::new();
    let mut hfs:BTreeMap<i64,u32>=BTreeMap::new(); // fyl vs std
    let mut nz_f=0u32; let mut nz_s=0u32; let mut tot=0u32; let mut both_move=0u32;
    let mut seed=0x1234_5678_9abc_def0u64;
    for _ in 0..200000 {
        // xorshift
        seed^=seed<<13; seed^=seed>>7; seed^=seed<<17;
        let frac=(seed>>11) as f64 / (1u64<<53) as f64; // [0,1)
        let r=3e-4*(0.29f64/3e-4).powf(frac);           // log-uniform financial range
        if exact_1pr(r){continue}
        let cr=l1p_cr(r); let fy=l1p_fyl(r); let sd=l1p_std(r);
        let df=ulp_diff(fy,cr); let ds=ulp_diff(sd,cr); let dfs=ulp_diff(fy,sd);
        *hf.entry(df).or_insert(0)+=1; *hs.entry(ds).or_insert(0)+=1; *hfs.entry(dfs).or_insert(0)+=1;
        if df!=0{nz_f+=1;} if ds!=0{nz_s+=1;} if df!=0&&ds!=0{both_move+=1;}
        tot+=1;
    }
    println!("=== log1p signature over {} general-r pts (log-uniform 3e-4..0.29) ===", tot);
    println!("FYL2XP1-CR ulp hist: {:?}   (nonzero: {}/{} = {:.1}%)", hf, nz_f, tot, 100.0*nz_f as f64/tot as f64);
    println!("std_ln1p-CR ulp hist: {:?}   (nonzero: {}/{} = {:.1}%)", hs, nz_s, tot, 100.0*nz_s as f64/tot as f64);
    println!("FYL2XP1-std ulp hist: {:?}  (this is where CR-class vs std split; both-move {}/{})", hfs, both_move, tot);

    // (2) collide inexact-r: pin em, count CR!=FYL and CR!=std, and oracle preference
    let data=load("../../work/w109/G6-solvers/answers-pmt-collide.json");
    let mut n=0; let mut cr_fy_diff=0; let mut cr_std_diff=0;
    let mut cr_only=0; let mut fy_only=0; let mut neither_crfy=0; let mut both_crfy=0;
    let mut crw=0; let mut stdw=0; let mut tie_crstd=0;
    for ((rb,nb),rows) in &data{
        let r=f64::from_bits(*rb); let nn=f64::from_bits(*nb);
        if rows.len()<8 || exact_1pr(r){continue}
        let center=em_for(r,nn,l1p_cr);
        if let Some(ep)=pin_gen(rows,center,r){
            n+=1;
            let ecr=em_for(r,nn,l1p_cr); let efy=em_for(r,nn,l1p_fyl); let esd=em_for(r,nn,l1p_std);
            if ecr.to_bits()!=efy.to_bits(){cr_fy_diff+=1;
                let c=ecr.to_bits()==ep.to_bits(); let f=efy.to_bits()==ep.to_bits();
                match(c,f){(true,false)=>cr_only+=1,(false,true)=>fy_only+=1,(true,true)=>both_crfy+=1,_=>neither_crfy+=1}
            }
            if ecr.to_bits()!=esd.to_bits(){cr_std_diff+=1;
                let c=ecr.to_bits()==ep.to_bits(); let s=esd.to_bits()==ep.to_bits();
                match(c,s){(true,false)=>crw+=1,(false,true)=>stdw+=1,(true,true)=>tie_crstd+=1,_=>{}}
            }
        }
    }
    println!("\n=== collide inexact-r em rows: {} pinned ===", n);
    println!("CR!=FYL2XP1 on {} rows -> oracle: CR-only:{} FYL-only:{} both:{} neither:{}", cr_fy_diff,cr_only,fy_only,both_crfy,neither_crfy);
    println!("CR!=std_ln1p on {} rows -> oracle: CR-right:{} std-right:{} both-right:{}", cr_std_diff,crw,stdw,tie_crstd);
}
