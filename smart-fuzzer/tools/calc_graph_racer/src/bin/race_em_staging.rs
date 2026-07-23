//! W109 G6-01: pin the exact em=(1+r)^-n - 1 staging at po2 rates (model-free em via
//! ·r-exact pv sweep). At po2, log1p(2^-k)=ln(1+2^-k) with 1+r EXACT, and Excel LN is
//! CR there (verified 148/148). So the residual is the expm1/tau staging. Compare
//! candidate stagings to Excel's pinned em (loaded from the pv-sweep oracle).
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_chs, ext_from_f64, ext_fyl2x, ext_l2e, ext_ln2, ext_mul, ext_one,
         ext_sub, ext_to_f64, ext_f2xm1, ext_rndint, ext_scale, ext_div, CW_PC64_RN};
use std::collections::BTreeMap;

// ---- log1p deliveries ----
fn log1p_cr(r:f64)->f64{ rx::excel_log1p(r) }
// ---- extended log1p: fyl2x on exact ext(1+r), stays extended (returns Ext80 as f64 via caller) ----

// ---- expm1 forms on a DOUBLE tau ----
fn expm1_kahan_internal(tau:f64)->f64{ rx::excel_expm1_internal(tau) }
fn expm1_cr(tau:f64)->f64{
    // CR expm1 via mpmath-equivalent: use ext exp then subtract 1 in ext (single round).
    // exp(tau) in ext (raw x87 exp chain), minus 1 ext, round once.
    let cw=CW_PC64_RN;
    let xe=ext_from_f64(tau);
    let t=ext_mul(&xe,&ext_l2e(),cw);
    let k=ext_rndint(&t,cw);
    let f=ext_sub(&t,&k,cw);
    let neg=ext_to_f64(&f,cw)<0.0;
    let w=ext_f2xm1(&ext_abs_local(&f,cw),cw);
    let mut m=ext_add(&w,&ext_one(),cw);
    if neg{ m=ext_div(&ext_one(),&m,cw); }
    let ex=ext_scale(&m,&k,cw);           // exp(tau) ext
    let em=ext_sub(&ex,&ext_one(),cw);    // -1 ext
    ext_to_f64(&em,cw)
}
fn ext_abs_local(x:&rx::Ext80,cw:u16)->rx::Ext80{ rx::ext_abs(x,cw) }

// pure x87 F2XM1 expm1 on double tau, all ext internally, round once
fn expm1_f2xm1(tau:f64)->f64{
    let cw=CW_PC64_RN;
    let t=ext_mul(&ext_from_f64(tau),&ext_l2e(),cw);   // tau*log2e
    let k=ext_rndint(&t,cw);
    let f=ext_sub(&t,&k,cw);
    let neg=ext_to_f64(&f,cw)<0.0;
    let w=ext_f2xm1(&rx::ext_abs(&f,cw),cw);            // 2^|f|-1
    let mut m=ext_add(&w,&ext_one(),cw);
    if neg{ m=ext_div(&ext_one(),&m,cw); }
    let ex=ext_scale(&m,&k,cw);                          // 2^t = exp(tau)
    let em=ext_sub(&ex,&ext_one(),cw);
    ext_to_f64(&em,cw)
}

// ---- full em stagings ----
// tau stored as DOUBLE, then expm1 form:
fn em_dbl_kahan(r:f64,n:f64)->f64{ let tau=-(n*log1p_cr(r)); expm1_kahan_internal(tau) }
fn em_dbl_cr(r:f64,n:f64)->f64{ let tau=-(n*log1p_cr(r)); expm1_cr(tau) }
fn em_dbl_f2xm1(r:f64,n:f64)->f64{ let tau=-(n*log1p_cr(r)); expm1_f2xm1(tau) }
// tau kept EXTENDED (log1p ext via fyl2x on exact 1+r, *n ext, expm1 ext), single store:
fn em_ext_all(r:f64,n:f64)->f64{
    let cw=CW_PC64_RN;
    let u=ext_add(&ext_one(),&ext_from_f64(r),cw);      // exact 1+r
    let l=ext_fyl2x(&ext_ln2(),&u,cw);                  // ln(1+r) ext
    let tau=ext_chs(&ext_mul(&ext_from_f64(n),&l,cw),cw); // -n*ln(1+r) ext
    // expm1 ext via F2XM1 path
    let t=ext_mul(&tau,&ext_l2e(),cw);
    let k=ext_rndint(&t,cw);
    let f=ext_sub(&t,&k,cw);
    let neg=ext_to_f64(&f,cw)<0.0;
    let w=ext_f2xm1(&rx::ext_abs(&f,cw),cw);
    let mut m=ext_add(&w,&ext_one(),cw);
    if neg{ m=ext_div(&ext_one(),&m,cw); }
    let ex=ext_scale(&m,&k,cw);
    let em=ext_sub(&ex,&ext_one(),cw);
    ext_to_f64(&em,cw)
}
// tau stored double from ext-computed -n*ln(1+r), then internal-Kahan double expm1:
fn em_dbltau_extln_kahan(r:f64,n:f64)->f64{
    let cw=CW_PC64_RN;
    let u=ext_add(&ext_one(),&ext_from_f64(r),cw);
    let l=ext_fyl2x(&ext_ln2(),&u,cw);
    let tau=ext_to_f64(&ext_chs(&ext_mul(&ext_from_f64(n),&l,cw),cw),cw); // store double
    expm1_kahan_internal(tau)
}

fn sordi(u:u64)->i64{ if u<1<<63 {u as i64} else { -((u ^ (1u64<<63)) as i64) } }

fn pin_em(rows:&[(f64,u64)], r:f64)->Option<f64>{
    // r po2 -> pmt = RN(RN(pv/em)*r) = r*RN(pv/em). Find em double s.t. all pv reproduce.
    // Search around the algebraic true em = (1+r)^-1... but n varies; instead brute search
    // near CR of true. Caller supplies candidate center via first-guess; do a local bit scan.
    // We scan a window around em_dbl_kahan as center.
    let center = {
        // crude center: take pv=rows median, q_obs=pmt/r, em ~ pv/q_obs
        let (pv,pmtbits)=rows[rows.len()/2];
        let pmt=f64::from_bits(pmtbits);
        pv/(pmt/r)
    };
    let cb=center.to_bits() as i64;
    let mut best:Option<(f64,u32)>=None;
    for d in -6..=6i64{
        let em=f64::from_bits((cb+d) as u64);
        let mut ok=0u32;
        for (pv,want) in rows{
            let q=pv/em; let g=q*r;
            if g.to_bits()==*want{ok+=1;}
        }
        if best.map_or(true,|b|ok>b.1){ best=Some((em,ok)); }
        if ok as usize==rows.len(){ return Some(em); }
    }
    best.filter(|b|b.1 as usize==rows.len()).map(|b|b.0)
}

fn main(){
    // capture-driven: expect an oracle file po2 x n. If absent, fall back to denselog1p n=1.
    let path="../../work/w109/G6-solvers/answers-pmt-po2n.json";
    let path=if std::path::Path::new(path).exists(){path}else{"../../work/w109/G6-solvers/answers-pmt-denselog1p.json"};
    let ws:WitnessSet=serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    // group by (r,n)
    let mut byrn:BTreeMap<(u64,u64),Vec<(f64,u64)>>=BTreeMap::new();
    for w in &ws.witnesses{
        let a:Vec<f64>=w.args.iter().filter_map(|x|match x{WitnessArg::Scalar(s)=>parse_bits_hex(s),_=>None}).collect();
        if a.len()!=5{continue}
        // only po2 r, fv=0, ty=0
        if a[3]!=0.0||a[4]!=0.0{continue}
        if (a[0].to_bits() & 0x000f_ffff_ffff_ffff)!=0{continue}
        let want=parse_bits_hex(&w.expected_bits).unwrap().to_bits();
        byrn.entry((a[0].to_bits(),a[1].to_bits())).or_default().push((a[2],want));
    }
    println!("po2 (r,n) groups: {}", byrn.len());
    let cands:[(&str, fn(f64,f64)->f64);5]=[
        ("dbl_kahan",em_dbl_kahan),("dbl_cr",em_dbl_cr),("dbl_f2xm1",em_dbl_f2xm1),
        ("ext_all",em_ext_all),("dbltau_extln_kahan",em_dbltau_extln_kahan),
    ];
    let mut score=[0u32;5]; let mut tot=0u32;
    let mut detail:Vec<String>=Vec::new();
    for ((rb,nb),rows) in &byrn{
        let r=f64::from_bits(*rb); let n=f64::from_bits(*nb);
        let em_ex=match pin_em(rows,r){Some(e)=>e,None=>continue};
        tot+=1;
        let mut line=format!("r=2^{:>3} n={:>4}: emEx={:016x}", r.log2() as i64, n, em_ex.to_bits());
        for (i,(nm,f)) in cands.iter().enumerate(){
            let g=f(r,n);
            let d=sordi(g.to_bits())-sordi(em_ex.to_bits());
            if d==0{score[i]+=1;}
            line.push_str(&format!("  {}:{:+}", nm, d));
        }
        if detail.len()<40{detail.push(line);}
    }
    println!("pinned (r,n): {}", tot);
    for (i,(nm,_)) in cands.iter().enumerate(){ println!("  {:<20} {}/{}", nm, score[i], tot); }
    println!("--- detail (offset of each staging from Excel em, in ulps) ---");
    for l in &detail{ println!("{}",l); }
}
