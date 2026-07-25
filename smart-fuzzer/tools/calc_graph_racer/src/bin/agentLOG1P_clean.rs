//! W109 G6-01 LOG1P LANE: split the log1p discrimination by (a) expm1 branch
//! (|tau|<1 = contaminated by the Kahan expm1 wall; |tau|>=1 = CLEAN, inversion
//! goes only through the proven-exact x87 exp + exact subtract) and (b) rate
//! class (exact-(1+r) = non-discriminating vs INEXACT-(1+r) = discriminating).
//! Also recovers, on the clean |tau|>=1 rows, the effective log1p deviation.
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_from_f64 as ef, ext_fyl2xp1, ext_fyl2x, ext_ln2, ext_add, ext_one, ext_to_f64, CW_PC64_RN as CW};
use std::collections::BTreeMap;

fn l1p_cr(r:f64)->f64{ rx::excel_log1p(r) }
fn l1p_std(r:f64)->f64{ r.ln_1p() }
fn l1p_lnfl(r:f64)->f64{ rx::excel_ln(1.0+r) }
fn l1p_fyl(r:f64)->f64{
    if r.abs()<0.292893218813452 { ext_to_f64(&ext_fyl2xp1(&ext_ln2(),&ef(r),CW),CW) }
    else { ext_to_f64(&ext_fyl2x(&ext_ln2(),&ext_add(&ext_one(),&ef(r),CW),CW),CW) }
}
fn em_for(r:f64,n:f64,l1p:fn(f64)->f64)->f64{ rx::excel_expm1_internal(-(n*l1p(r))) }
fn exact_1pr(r:f64)->bool{ (1.0+r)-1.0==r }
fn na(x:f64,d:i64)->f64{ f64::from_bits((x.to_bits() as i64 + d) as u64) }

fn pin_gen(rows:&[(f64,u64)], center:f64, r:f64)->Option<f64>{
    let cb=center.to_bits() as i64;
    for d in -24..=24i64{ let em=f64::from_bits((cb+d) as u64); if em>=0.0{continue}
        if rows.iter().all(|(pv,want)|(( (pv/em) *r)).to_bits()==*want){ return Some(em); } }
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
    let cands:[(&str,fn(f64)->f64);4]=[("CR",l1p_cr),("FYL2XP1",l1p_fyl),("std_ln1p",l1p_std),("lnfl",l1p_lnfl)];
    // 4 buckets: {clean|tau|>=1, contaminated|tau|<1} x {exactR, inexactR}
    for (nm,path) in [("collide","../../work/w109/G6-solvers/answers-pmt-collide.json"),
                      ("genrate","../../work/w109/G6-solvers/answers-pmt-genrate.json"),
                      ("po2n","../../work/w109/G6-solvers/answers-pmt-po2n.json")]{
        let data=load(path);
        // bucket -> vec of (r,n,em_pinned)
        let mut buckets:BTreeMap<&str,Vec<(f64,f64,f64)>>=BTreeMap::new();
        for ((rb,nb),rows) in &data{
            let r=f64::from_bits(*rb); let n=f64::from_bits(*nb);
            if rows.len()<8 { continue; }
            let tau=-(n*rx::excel_log1p(r));
            let center=em_for(r,n,l1p_cr);
            if let Some(e)=pin_gen(rows,center,r){
                let key= match (tau.abs()>=1.0, exact_1pr(r)) {
                    (true,true)=>"CLEAN_exactR", (true,false)=>"CLEAN_inexactR",
                    (false,true)=>"cont_exactR", (false,false)=>"cont_inexactR" };
                buckets.entry(key).or_default().push((r,n,e));
            }
        }
        println!("=== {} ===", nm);
        for (bk,v) in &buckets{
            print!("  {:16} ({:3} pinned):", bk, v.len());
            for (cn,cf) in cands{
                let ok=v.iter().filter(|(r,n,ep)| em_for(*r,*n,cf).to_bits()==ep.to_bits()).count();
                print!("  {}:{}/{}", cn, ok, v.len());
            }
            println!();
            // On CLEAN buckets, recover effective log1p deviation (invert em via monotone x87 exp).
            if bk.starts_with("CLEAN"){
                let mut hist:BTreeMap<i64,u32>=BTreeMap::new();
                for (r,n,ep) in v{
                    let lcr=l1p_cr(*r);
                    // effective log1p = na(lcr,d); model em clean = expm1_internal(-(n*L)). scan d.
                    let mut hits=Vec::new();
                    for d in -8..=8i64{ let l=na(lcr,d);
                        if rx::excel_expm1_internal(-(n*l)).to_bits()==ep.to_bits(){ hits.push(d); } }
                    if hits.is_empty(){ *hist.entry(99).or_insert(0)+=1; }
                    else { for d in hits { *hist.entry(d).or_insert(0)+=1; } }
                }
                println!("      recovered log1p-dev(ulp) hist (99=no-hit): {:?}", hist);
            }
        }
    }
}
