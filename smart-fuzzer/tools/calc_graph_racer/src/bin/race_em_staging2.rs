//! W109 G6-01: exhaustive em staging race on the po2xn model-free oracle.
//! log1p is CR (LN 148/148 exact-1+r). Residual = expm1/tau staging. Enumerate
//! {tau: dbl vs ext} x {expm1 Kahan ops: dbl vs ext vs mixed} and score vs pinned em.
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use rx::{Ext80, ext_abs, ext_add, ext_chs, ext_div, ext_f2xm1, ext_from_f64, ext_fyl2x, ext_l2e,
         ext_ln2, ext_mul, ext_one, ext_rndint, ext_scale, ext_sub, ext_to_f64, CW_PC64_RN};
use std::collections::BTreeMap;

const CW:u16=CW_PC64_RN;
fn e(x:f64)->Ext80{ ext_from_f64(x) }
fn tf(x:&Ext80)->f64{ ext_to_f64(x,CW) }

// exp(tau_ext) -> Ext80 (x87 fFEXP chain, all extended)
fn exp_ext(tau:&Ext80)->Ext80{
    let t=ext_mul(tau,&ext_l2e(),CW);
    let k=ext_rndint(&t,CW);
    let f=ext_sub(&t,&k,CW);
    let neg=tf(&f)<0.0;
    let w=ext_f2xm1(&ext_abs(&f,CW),CW);
    let mut m=ext_add(&w,&ext_one(),CW);
    if neg{ m=ext_div(&ext_one(),&m,CW); }
    ext_scale(&m,&k,CW)
}
// ln(u_ext) -> Ext80 via fyl2x
fn ln_ext(u:&Ext80)->Ext80{ ext_fyl2x(&ext_ln2(),u,CW) }

// ---- tau builders ----
fn tau_dbl(r:f64,n:f64)->f64{ -(n*rx::excel_log1p(r)) }         // double n*log1p_cr, negate
fn tau_ext_val(r:f64,n:f64)->Ext80{                              // -n*ln(1+r) all ext (1+r exact @ po2)
    let u=ext_add(&ext_one(),&e(r),CW);
    ext_chs(&ext_mul(&e(n),&ln_ext(&u),CW),CW)
}

// ---- em stagings ----
// S_A: current production = all-double Kahan on double tau
fn s_dbl_kahan(r:f64,n:f64)->f64{
    let t=tau_dbl(r,n); let u=rx::excel_exp(t);
    if u==1.0{return t;}
    if t.abs()<1.0{ (u-1.0)*t/rx::excel_ln(u) } else { u-1.0 }
}
// S_B: full extended Kahan, extended tau, single store
fn s_ext_kahan(r:f64,n:f64)->f64{
    let tau=tau_ext_val(r,n);
    let u=exp_ext(&tau);
    if tf(&u)==1.0{ return tf(&tau); }
    if tf(&tau).abs()<1.0{
        let num=ext_mul(&ext_sub(&u,&ext_one(),CW),&tau,CW);
        tf(&ext_div(&num,&ln_ext(&u),CW))
    } else { tf(&ext_sub(&u,&ext_one(),CW)) }
}
// S_C: extended tau but STORED to double, then all-double Kahan (tau spill only)
fn s_exttau_dblkahan(r:f64,n:f64)->f64{
    let t=tf(&tau_ext_val(r,n));
    let u=rx::excel_exp(t);
    if u==1.0{return t;}
    if t.abs()<1.0{ (u-1.0)*t/rx::excel_ln(u) } else { u-1.0 }
}
// S_D: double tau, extended Kahan internals (u,ln in ext, num/den ext, store once)
fn s_dbltau_extkahan(r:f64,n:f64)->f64{
    let t=tau_dbl(r,n); let tau=e(t);
    let u=exp_ext(&tau);
    if tf(&u)==1.0{ return t; }
    if t.abs()<1.0{
        let num=ext_mul(&ext_sub(&u,&ext_one(),CW),&tau,CW);
        tf(&ext_div(&num,&ln_ext(&u),CW))
    } else { tf(&ext_sub(&u,&ext_one(),CW)) }
}
// S_E: ext tau, ext u, but num/den built with DOUBLE u (u spilled), ext divide
fn s_ext_uspill(r:f64,n:f64)->f64{
    let tau=tau_ext_val(r,n); let t=tf(&tau);
    let ud=ext_to_f64(&exp_ext(&tau),CW);       // u spilled to double
    if ud==1.0{ return t; }
    if t.abs()<1.0{
        let u=e(ud);
        let num=ext_mul(&ext_sub(&u,&ext_one(),CW),&tau,CW);
        tf(&ext_div(&num,&ln_ext(&u),CW))
    } else { ud-1.0 }
}
// S_F: ext everything but tau via double n*log1p, expm1 form (u-1)*(t/ln u) reassoc
fn s_ext_reassoc(r:f64,n:f64)->f64{
    let tau=tau_ext_val(r,n);
    let u=exp_ext(&tau);
    if tf(&u)==1.0{ return tf(&tau); }
    if tf(&tau).abs()<1.0{
        let q=ext_div(&tau,&ln_ext(&u),CW);
        tf(&ext_mul(&ext_sub(&u,&ext_one(),CW),&q,CW))
    } else { tf(&ext_sub(&u,&ext_one(),CW)) }
}

fn sordi(u:u64)->i64{ if u<1<<63 {u as i64} else { -((u ^ (1u64<<63)) as i64) } }
fn pin_em(rows:&[(f64,u64)], r:f64)->Option<f64>{
    let (pv,pmtb)=rows[rows.len()/2]; let pmt=f64::from_bits(pmtb);
    let center=pv/(pmt/r); let cb=center.to_bits() as i64;
    for d in -8..=8i64{
        let em=f64::from_bits((cb+d) as u64);
        if em==-1.0{continue}
        if rows.iter().all(|(pv,want)|((pv/em)*r).to_bits()==*want){ return Some(em); }
    }
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
    let cands:[(&str,fn(f64,f64)->f64);6]=[
        ("dbl_kahan(prod)",s_dbl_kahan),("ext_kahan",s_ext_kahan),("exttau_dblkahan",s_exttau_dblkahan),
        ("dbltau_extkahan",s_dbltau_extkahan),("ext_uspill",s_ext_uspill),("ext_reassoc",s_ext_reassoc),
    ];
    let mut score=[0u32;6]; let mut tot=0u32;
    // per-n breakdown for the best variants
    let mut miss_by_n:BTreeMap<u64,[u32;6]>=BTreeMap::new();
    for ((rb,nb),rows) in &byrn{
        let r=f64::from_bits(*rb); let n=f64::from_bits(*nb);
        let em_ex=match pin_em(rows,r){Some(e)=>e,None=>continue}; tot+=1;
        let ent=miss_by_n.entry(*nb).or_insert([0;6]);
        for (i,(_,f)) in cands.iter().enumerate(){
            let d=sordi(f(r,n).to_bits())-sordi(em_ex.to_bits());
            if d==0{score[i]+=1;} else {ent[i]+=1;}
        }
        let _=n;
    }
    println!("pinned (r,n): {}", tot);
    for (i,(nm,_)) in cands.iter().enumerate(){ println!("  {:<18} {}/{}", nm, score[i], tot); }
    println!("\nmisses per n (n : {}):", cands.iter().map(|c|c.0).collect::<Vec<_>>().join(" "));
    for (nb,arr) in &miss_by_n{
        println!("  n={:<4} {:?}", f64::from_bits(*nb) as i64, arr);
    }
}
