//! W109 G6-01: exhaustive x87 spill search scored on the COLLISION oracle (configs
//! sharing a tau_double but differing in exact tau). This ISOLATES the sub-ULP
//! dependence: only op-graphs that keep tau EXTENDED (not spilled) can reproduce the
//! em variation within a group. tau computed FRESH from (r,n) at each config's precision.
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use rx::{Ext80, ext_abs, ext_add, ext_chs, ext_div, ext_f2xm1, ext_from_f64 as ef, ext_fyl2x,
         ext_fyl2xp1, ext_l2e, ext_ln2, ext_mul, ext_one, ext_rndint, ext_scale, ext_sub,
         ext_to_f64, CW_PC53_RN, CW_PC64_RN};
use std::collections::BTreeMap;

const RC_RZ:u16=0x0C00; const RC_RM:u16=0x0400; const RC_RP:u16=0x0800;
fn spill(x:&Ext80, cw:u16)->Ext80{ ef(ext_to_f64(x,cw)) }
fn exp_ext(tau:&Ext80)->Ext80{
    let cw=CW_PC64_RN;
    let t=ext_mul(tau,&ext_l2e(),cw); let k=ext_rndint(&t,cw); let f=ext_sub(&t,&k,cw);
    let neg=ext_to_f64(&f,cw)<0.0; let w=ext_f2xm1(&ext_abs(&f,cw),cw); let mut m=ext_add(&w,&ext_one(),cw);
    if neg{ m=ext_div(&ext_one(),&m,cw); } ext_scale(&m,&k,cw)
}
#[derive(Clone,Copy)] struct Cfg{ arith:u16, spill:u8, lndel:u8, assoc:u8, fin:u16 }
fn em(r:f64,n:f64,c:Cfg)->f64{
    let a=c.arith; let ln2=ext_ln2();
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
    let mut l = if c.lndel==0 { ext_fyl2x(&ln2,&u,a) } else { ext_fyl2xp1(&ln2,&ext_sub(&u,&ext_one(),a),a) };
    if c.spill&32!=0 { l=spill(&l,a); }
    let em = match c.assoc { 0 => ext_div(&p,&l,a), 1 => ext_mul(&b,&ext_div(&tau,&l,a),a), _ => ext_mul(&ext_div(&tau,&l,a),&b,a) };
    ext_to_f64(&em, c.fin)
}
fn pin_gen(rows:&[(f64,u64)], r:f64, center:f64)->Option<f64>{
    let cb=center.to_bits() as i64;
    for d in -14..=14i64{ let em=f64::from_bits((cb+d) as u64); if em>=0.0{continue}
        if rows.iter().all(|(pv,want)|((pv/em)*r).to_bits()==*want){ return Some(em); } }
    None
}
fn main(){
    // load collision: 128 consecutive probes per config, order = meta order
    let ws:WitnessSet=serde_json::from_str(&std::fs::read_to_string(
        "../../work/w109/G6-solvers/answers-pmt-collide.json").unwrap()).unwrap();
    // reconstruct configs of 128
    let mut cfgs:Vec<(f64,f64,Vec<(f64,u64)>)>=Vec::new();
    let wl=&ws.witnesses;
    let ncfg=wl.len()/128;
    for ci in 0..ncfg{
        let mut rows=Vec::new(); let mut rn=(0.0,0.0);
        for j in 0..128{
            let w=&wl[ci*128+j];
            let a:Vec<f64>=w.args.iter().filter_map(|x|match x{WitnessArg::Scalar(s)=>parse_bits_hex(s),_=>None}).collect();
            rn=(a[0],a[1]);
            rows.push((a[2], parse_bits_hex(&w.expected_bits).unwrap().to_bits()));
        }
        cfgs.push((rn.0,rn.1,rows));
    }
    // pin em per config
    let mut pins:Vec<(f64,f64,f64)>=Vec::new();
    for (r,n,rows) in &cfgs{
        let km=rx::excel_expm1_internal(-(n*rx::excel_log1p(*r)));
        if let Some(e)=pin_gen(rows,*r,km){ pins.push((*r,*n,e)); }
    }
    println!("collision configs pinned: {}/{}", pins.len(), cfgs.len());
    let kbase=pins.iter().filter(|(r,n,ep)| rx::excel_expm1_internal(-(n*rx::excel_log1p(*r))).to_bits()==ep.to_bits()).count();
    println!("double-Kahan baseline on collision: {}/{}", kbase, pins.len());

    let ariths=[("PC53_RN",CW_PC53_RN),("PC64_RN",CW_PC64_RN),("PC64_RZ",CW_PC64_RN|RC_RZ),
                ("PC64_RM",CW_PC64_RN|RC_RM),("PC64_RP",CW_PC64_RN|RC_RP),("PC53_RZ",CW_PC53_RN|RC_RZ)];
    let fins=[("RN",CW_PC64_RN),("RZ",CW_PC64_RN|RC_RZ),("RM",CW_PC64_RN|RC_RM),("RP",CW_PC64_RN|RC_RP)];
    let mut best:Vec<(u32,String)>=Vec::new();
    for (an,acw) in ariths{ for (fnm,fcw) in fins{ for spill in 0u8..64{ for lndel in 0..2u8{ for assoc in 0..3u8{
        let c=Cfg{arith:acw,spill,lndel,assoc,fin:fcw};
        let ok=pins.iter().filter(|(r,n,ep)| em(*r,*n,c).to_bits()==ep.to_bits()).count() as u32;
        best.push((ok, format!("arith={} fin={} spill={:06b} ln={} assoc={}",an,fnm,spill,lndel,assoc)));
    }}}}}
    best.sort_by(|a,b| b.0.cmp(&a.0));
    println!("TOP configs on collision oracle ({} pts):", pins.len());
    for (ok,d) in best.iter().take(16){ println!("  {:>3}/{}  {}", ok, pins.len(), d); }
}
