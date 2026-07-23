//! W109 G6-01: EXHAUSTIVE x87 spill/PC/RC search for PMT's em op-graph. Models the
//! human-code->compiler->x87 pipeline: every intermediate is an 80-bit register at the
//! control-word precision (PC=53|64), rounded to a 53-bit double only when SPILLED to a
//! memory variable. Enumerate spill-schedule (6 nodes) x arith (PC,RC) x ln-delivery x
//! association, computed FRESH from (r,n), scored vs the model-free em_pinned oracle.
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use rx::{Ext80, ext_abs, ext_add, ext_chs, ext_div, ext_f2xm1, ext_from_f64 as ef, ext_fyl2x,
         ext_fyl2xp1, ext_l2e, ext_ln2, ext_mul, ext_one, ext_rndint, ext_scale, ext_sub,
         ext_to_f64, CW_PC53_RN, CW_PC64_RN};
use std::collections::BTreeMap;

const RC_RZ:u16=0x0C00; const RC_RM:u16=0x0400; const RC_RP:u16=0x0800;

fn spill(x:&Ext80, cw:u16)->Ext80{ ef(ext_to_f64(x,cw)) }   // round to double, back to ext
fn exp_ext(tau:&Ext80)->Ext80{
    let cw=CW_PC64_RN;
    let t=ext_mul(tau,&ext_l2e(),cw); let k=ext_rndint(&t,cw); let f=ext_sub(&t,&k,cw);
    let neg=ext_to_f64(&f,cw)<0.0; let w=ext_f2xm1(&ext_abs(&f,cw),cw); let mut m=ext_add(&w,&ext_one(),cw);
    if neg{ m=ext_div(&ext_one(),&m,cw); } ext_scale(&m,&k,cw)
}

#[derive(Clone,Copy)]
struct Cfg{ arith:u16, spill:u8, lndel:u8, assoc:u8, fin:u16 }

// em from (r,n), assuming |tau|<1 (Kahan branch). Fresh x87 computation with spills.
fn em(r:f64,n:f64,c:Cfg)->f64{
    let a=c.arith; let ln2=ext_ln2();
    // L = log2(1+r) [fyl2xp1 for small r, else fyl2x(1+r)]
    // ln(1+r) via SINGLE-instruction fyl2xp1(ln2,r)=ln2*log2(1+r), or fyl2x(ln2,1+r).
    let mut ln1p = if r.abs()<0.292893218813452 { ext_fyl2xp1(&ln2,&ef(r),a) }
             else { ext_fyl2x(&ln2,&ext_add(&ext_one(),&ef(r),a),a) };
    if c.spill&1!=0 { ln1p=spill(&ln1p,a); }
    let mut tau = ext_chs(&ext_mul(&ef(n),&ln1p,a),a);
    if c.spill&2!=0 { tau=spill(&tau,a); }
    let mut u = exp_ext(&tau);
    if c.spill&4!=0 { u=spill(&u,a); }
    let mut b = ext_sub(&u,&ext_one(),a);
    if c.spill&8!=0 { b=spill(&b,a); }
    let mut p = ext_mul(&b,&tau,a);
    if c.spill&16!=0 { p=spill(&p,a); }
    // ln(u): SINGLE-instruction fyl2x(ln2,u), or fyl2xp1(ln2,u-1) [accurate for u~1].
    let mut l = if c.lndel==0 { ext_fyl2x(&ln2,&u,a) }
                else { ext_fyl2xp1(&ln2,&ext_sub(&u,&ext_one(),a),a) };
    if c.spill&32!=0 { l=spill(&l,a); }
    let em = match c.assoc {
        0 => ext_div(&p,&l,a),
        1 => ext_mul(&b,&ext_div(&tau,&l,a),a),
        _ => ext_mul(&ext_div(&tau,&l,a),&b,a),
    };
    ext_to_f64(&em, c.fin)
}

fn pin_po2(rows:&[(f64,u64)], r:f64)->Option<f64>{
    let (pv,pmtb)=rows[rows.len()/2]; let pmt=f64::from_bits(pmtb);
    let center=pv/(pmt/r); let cb=center.to_bits() as i64;
    for d in -8..=8i64{ let em=f64::from_bits((cb+d) as u64); if em==-1.0{continue}
        if rows.iter().all(|(pv,want)|((pv/em)*r).to_bits()==*want){ return Some(em); } }
    None
}
fn pin_gen(rows:&[(f64,u64)], r:f64, center:f64)->Option<f64>{
    let cb=center.to_bits() as i64;
    for d in -12..=12i64{ let em=f64::from_bits((cb+d) as u64); if em>=0.0{continue}
        if rows.iter().all(|(pv,want)|((pv/em)*r).to_bits()==*want){ return Some(em); } }
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
    // pin em for po2n and gen
    let mut pins:Vec<(f64,f64,f64,bool)>=Vec::new(); // r,n,em, is_po2
    for (path,po2) in [("../../work/w109/G6-solvers/answers-pmt-po2n.json",true),
                       ("../../work/w109/G6-solvers/answers-pmt-genrate.json",false)]{
        for ((rb,nb),rows) in &load(path){
            let r=f64::from_bits(*rb); let n=f64::from_bits(*nb);
            if (-(n*rx::excel_log1p(r))).abs()>=1.0{continue}
            let km=rx::excel_expm1_internal(-(n*rx::excel_log1p(r)));
            let p=if po2{pin_po2(rows,r)}else{pin_gen(rows,r,km)};
            if let Some(e)=p{ pins.push((r,n,e,po2)); }
        }
    }
    let npo2=pins.iter().filter(|x|x.3).count();
    let ngen=pins.len()-npo2;
    println!("pinned: po2={} gen={}", npo2, ngen);

    let ariths=[("PC53_RN",CW_PC53_RN),("PC64_RN",CW_PC64_RN),
                ("PC64_RZ",CW_PC64_RN|RC_RZ),("PC64_RM",CW_PC64_RN|RC_RM),("PC64_RP",CW_PC64_RN|RC_RP),
                ("PC53_RZ",CW_PC53_RN|RC_RZ)];
    let fins=[("RN",CW_PC64_RN),("RZ",CW_PC64_RN|RC_RZ),("RM",CW_PC64_RN|RC_RM),("RP",CW_PC64_RN|RC_RP)];
    let mut best:Vec<(u32,u32,String)>=Vec::new(); // (po2_ok, gen_ok, desc)
    for (an,acw) in ariths{
        for (fnm,fcw) in fins{
            for spill in 0u8..64{
                for lndel in 0..2u8{
                    for assoc in 0..3u8{
                        let c=Cfg{arith:acw,spill,lndel,assoc,fin:fcw};
                        let mut po2ok=0u32; let mut genok=0u32;
                        for (r,n,ep,ispo2) in &pins{
                            if em(*r,*n,c).to_bits()==ep.to_bits(){ if *ispo2{po2ok+=1}else{genok+=1} }
                        }
                        best.push((po2ok,genok,format!("arith={} fin={} spill={:06b} ln={} assoc={}",an,fnm,spill,lndel,assoc)));
                    }
                }
            }
        }
    }
    best.sort_by(|a,b| (b.0+b.1).cmp(&(a.0+a.1)));
    println!("TOP configs (po2/{}, gen/{}):", npo2, ngen);
    for (p,g,d) in best.iter().take(20){ println!("  po2 {:>3} gen {:>3} tot {:>3}  {}", p, g, p+g, d); }
}
