//! W109 G6-01 EXPM1 LANE (x87 microcode realism). Task: does PMT's internal
//! expm1 for |tau|<1 keep u in 80-bit (fFEXP), use the reciprocal branch for
//! negative tau, and/or take ln via FYL2XP1(u-1)? And does a ROUND-TOWARD-ZERO
//! final store explain the uniform toward-zero residual?  Baselines to beat:
//! double Kahan 163, num-double-rounded (NPER-style) 165.
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use rx::{Ext80, ext_abs, ext_add, ext_chs, ext_div, ext_f2xm1, ext_from_f64 as e, ext_fyl2x,
         ext_fyl2xp1, ext_l2e, ext_ln2, ext_mul, ext_one, ext_rndint, ext_scale, ext_sub,
         ext_to_f64, CW_PC64_RN as CW};
use std::collections::BTreeMap;

const CWZ: u16 = 0x1F3F; // PC64, round-toward-zero (RC=11), for the store barrier
fn tfz(x:&Ext80)->f64{ ext_to_f64(x,CWZ) } // store toward zero
fn tf(x:&Ext80)->f64{ ext_to_f64(x,CW) }   // store round-nearest

fn tau_sse(r:f64,n:f64)->f64{ -(n*rx::excel_log1p(r)) }
// x87 per-op double-rounded product RN53(RN64(a*b))
fn dmul(a:f64,b:f64)->f64{ ext_to_f64(&ext_mul(&e(a),&e(b),CW),CW) }

// the 80-bit fFEXP result as Ext80 (u kept on the stack, NOT spilled to double)
fn exp_ext(tau:&Ext80)->Ext80{
    let t=ext_mul(tau,&ext_l2e(),CW); let k=ext_rndint(&t,CW); let f=ext_sub(&t,&k,CW);
    let neg=tf(&f)<0.0; let w=ext_f2xm1(&ext_abs(&f,CW),CW); let mut m=ext_add(&w,&ext_one(),CW);
    if neg{ m=ext_div(&ext_one(),&m,CW); } ext_scale(&m,&k,CW)
}

// ============ candidate em forms, all take (r,n), assume |tau|<1 ============

// --- baselines ---
fn v_base(r:f64,n:f64)->f64{ // all-double Kahan = 163
    let t=tau_sse(r,n); let u=rx::excel_exp(t); let b=u-1.0; let lnu=rx::excel_ln(u);
    (b*t)/lnu
}
fn v_numdr(r:f64,n:f64)->f64{ // num double-rounded = 165
    let t=tau_sse(r,n); let u=rx::excel_exp(t); let b=u-1.0; let lnu=rx::excel_ln(u);
    dmul(b,t)/lnu
}

// --- TASK 1: u kept in 80-bit (fFEXP), Kahan in ext, various spill points ---
// u80 fully extended; b=u80-1 ext; num=b*t ext; lnu=fyl2x(u80) ext; div ext; store RN
fn v_u80_fyl2x_rn(r:f64,n:f64)->f64{
    let t=e(tau_sse(r,n)); let u=exp_ext(&t);
    let b=ext_sub(&u,&ext_one(),CW); let num=ext_mul(&b,&t,CW);
    let lnu=ext_fyl2x(&ext_ln2(),&u,CW);
    tf(&ext_div(&num,&lnu,CW))
}
// same but store toward zero
fn v_u80_fyl2x_rz(r:f64,n:f64)->f64{
    let t=e(tau_sse(r,n)); let u=exp_ext(&t);
    let b=ext_sub(&u,&ext_one(),CW); let num=ext_mul(&b,&t,CW);
    let lnu=ext_fyl2x(&ext_ln2(),&u,CW);
    tfz(&ext_div(&num,&lnu,CW))
}

// --- TASK 3: ln via FYL2XP1(u-1) instead of FYL2X(u) (accurate near 1) ---
// u80 extended, b=u80-1, lnu = ln2*log2(1+b) via fyl2xp1(1,b); Kahan ext; RN
fn v_u80_fyl2xp1_rn(r:f64,n:f64)->f64{
    let t=e(tau_sse(r,n)); let u=exp_ext(&t);
    let b=ext_sub(&u,&ext_one(),CW); let num=ext_mul(&b,&t,CW);
    let lnu=ext_mul(&ext_ln2(),&ext_fyl2xp1(&ext_one(),&b,CW),CW);
    tf(&ext_div(&num,&lnu,CW))
}
// store toward zero
fn v_u80_fyl2xp1_rz(r:f64,n:f64)->f64{
    let t=e(tau_sse(r,n)); let u=exp_ext(&t);
    let b=ext_sub(&u,&ext_one(),CW); let num=ext_mul(&b,&t,CW);
    let lnu=ext_mul(&ext_ln2(),&ext_fyl2xp1(&ext_one(),&b,CW),CW);
    tfz(&ext_div(&num,&lnu,CW))
}
// u spilled to DOUBLE (confirmed exp store), then Kahan with fyl2xp1 den on (u_d-1)
fn v_ud_fyl2xp1_rn(r:f64,n:f64)->f64{
    let t=tau_sse(r,n); let u=rx::excel_exp(t); let b=u-1.0;
    // ln(u) = ln2*log2(1+b) with b a double
    let lnu=tf(&ext_mul(&ext_ln2(),&ext_fyl2xp1(&ext_one(),&e(b),CW),CW));
    (b*t)/lnu
}

// --- TASK 2: reciprocal decomposition, no rounded u ---
// tau<0: T=tau*log2e, k=rndint(T), f=T-k (|f|<=.5). 2^tau = 2^k*m, m=1/(1+w) if f<0.
// em = 2^k*m - 1 formed directly in ext, single store (RN / RZ).
fn recip_em(r:f64,n:f64, rz:bool)->f64{
    let tau=e(tau_sse(r,n));
    let t=ext_mul(&tau,&ext_l2e(),CW); let k=ext_rndint(&t,CW); let f=ext_sub(&t,&k,CW);
    let neg=tf(&f)<0.0; let w=ext_f2xm1(&ext_abs(&f,CW),CW); let mut m=ext_add(&w,&ext_one(),CW);
    if neg{ m=ext_div(&ext_one(),&m,CW); }
    let em=ext_sub(&ext_scale(&m,&k,CW),&ext_one(),CW);
    if rz{ tfz(&em) } else { tf(&em) }
}
fn v_recip_rn(r:f64,n:f64)->f64{ recip_em(r,n,false) }
fn v_recip_rz(r:f64,n:f64)->f64{ recip_em(r,n,true) }
// tiny-arg exact form: when k==0, em = m-1 = -w/(1+w) (neg tau). general: fold.
fn v_negw_over_1pw(r:f64,n:f64)->f64{
    let tau=e(tau_sse(r,n));
    let t=ext_mul(&tau,&ext_l2e(),CW); let k=ext_rndint(&t,CW); let f=ext_sub(&t,&k,CW);
    let neg=tf(&f)<0.0; let w=ext_f2xm1(&ext_abs(&f,CW),CW);
    if tf(&k)==0.0 {
        // em = m-1. neg: 1/(1+w)-1 = -w/(1+w); pos: w
        if neg { return tf(&ext_div(&ext_chs(&w,CW),&ext_add(&w,&ext_one(),CW),CW)); }
        return tf(&w);
    }
    // fall back to full decomposition
    let mut m=ext_add(&w,&ext_one(),CW); if neg{ m=ext_div(&ext_one(),&m,CW); }
    tf(&ext_sub(&ext_scale(&m,&k,CW),&ext_one(),CW))
}

// --- RZ store of the per-op double-rounded Kahan (chop hypothesis) ---
// num double-rounded, div in ext80 stored RZ.
fn v_numdr_divrz(r:f64,n:f64)->f64{
    let t=tau_sse(r,n); let u=rx::excel_exp(t); let b=u-1.0; let lnu=rx::excel_ln(u);
    let num=dmul(b,t);
    tfz(&ext_div(&e(num),&e(lnu),CW))
}
// pure double Kahan but final divide ext80 stored RZ
fn v_base_divrz(r:f64,n:f64)->f64{
    let t=tau_sse(r,n); let u=rx::excel_exp(t); let b=u-1.0; let lnu=rx::excel_ln(u);
    let num=b*t;
    tfz(&ext_div(&e(num),&e(lnu),CW))
}

// --- CHOPPED-EXP hypothesis: internal exp published round-toward-zero ---
// u = excel_exp_rz(tau) (F2XM1 chain stored RZ), then Kahan / u-1.
fn v_chopexp_kahan(r:f64,n:f64)->f64{
    let t=tau_sse(r,n); let u=rx::excel_exp_rz(t);
    if u==1.0{return t;} let b=u-1.0; let lnu=rx::excel_ln(u);
    (b*t)/lnu
}
fn v_chopexp_kahan_numdr(r:f64,n:f64)->f64{
    let t=tau_sse(r,n); let u=rx::excel_exp_rz(t);
    if u==1.0{return t;} let b=u-1.0; let lnu=rx::excel_ln(u);
    dmul(b,t)/lnu
}
fn v_chopexp_um1(r:f64,n:f64)->f64{
    let t=tau_sse(r,n); let u=rx::excel_exp_rz(t); u-1.0
}
// chopped exp, ln via fyl2xp1(u-1) (accurate near 1)
fn v_chopexp_fyl2xp1(r:f64,n:f64)->f64{
    let t=tau_sse(r,n); let u=rx::excel_exp_rz(t);
    if u==1.0{return t;} let b=u-1.0;
    let lnu=tf(&ext_mul(&ext_ln2(),&ext_fyl2xp1(&ext_one(),&e(b),CW),CW));
    (b*t)/lnu
}

// --- PC=53 hypothesis: legacy body runs at the Windows-default CW (PC=53) ---
// exp F2XM1 chain and fyl2x ln computed at PC=53 (single-round to 53-bit on stack).
const CW53: u16 = 0x123F; // PC=53, RN
fn exp_pc53(t:f64)->f64{
    let tau=e(t);
    let tt=ext_mul(&tau,&ext_l2e(),CW53); let k=ext_rndint(&tt,CW53); let f=ext_sub(&tt,&k,CW53);
    let neg=ext_to_f64(&f,CW53)<0.0; let w=ext_f2xm1(&ext_abs(&f,CW53),CW53);
    let mut m=ext_add(&w,&ext_one(),CW53); if neg{ m=ext_div(&ext_one(),&m,CW53); }
    ext_to_f64(&ext_scale(&m,&k,CW53),CW53)
}
fn ln_pc53(u:f64)->f64{ ext_to_f64(&ext_fyl2x(&ext_ln2(),&e(u),CW53),CW53) }
fn v_pc53_kahan(r:f64,n:f64)->f64{
    let t=tau_sse(r,n); let u=exp_pc53(t); if u==1.0{return t;}
    let b=u-1.0; let lnu=ln_pc53(u); (b*t)/lnu
}
fn v_pc53_kahan_numdr(r:f64,n:f64)->f64{
    let t=tau_sse(r,n); let u=exp_pc53(t); if u==1.0{return t;}
    let b=u-1.0; let lnu=ln_pc53(u); dmul(b,t)/lnu
}
// PC=53 exp, worksheet(PC64) ln
fn v_pc53exp_pc64ln(r:f64,n:f64)->f64{
    let t=tau_sse(r,n); let u=exp_pc53(t); if u==1.0{return t;}
    let b=u-1.0; let lnu=rx::excel_ln(u); (b*t)/lnu
}
// PC=53 fully double-rounded body: tau also x87 dr at PC53, num/div dr
fn v_pc53_allbody(r:f64,n:f64)->f64{
    let lp=rx::excel_log1p(r);
    let t=ext_to_f64(&ext_mul(&e(n),&e(lp),CW53),CW53); let t=-t;
    let u=exp_pc53(t); if u==1.0{return t;}
    let b=u-1.0; let lnu=ln_pc53(u);
    let num=ext_to_f64(&ext_mul(&e(b),&e(t),CW53),CW53);
    ext_to_f64(&ext_div(&e(num),&e(lnu),CW53),CW53)
}

fn sordi(u:u64)->i64{ if u<1<<63 {u as i64} else { -((u ^ (1u64<<63)) as i64) } }
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
    let cands:[(&str,fn(f64,f64)->f64);20]=[
        ("base_kahan(163)",v_base),("numdr(165)",v_numdr),
        ("u80_fyl2x_RN",v_u80_fyl2x_rn),("u80_fyl2x_RZ",v_u80_fyl2x_rz),
        ("u80_fyl2xp1_RN",v_u80_fyl2xp1_rn),("u80_fyl2xp1_RZ",v_u80_fyl2xp1_rz),
        ("ud_fyl2xp1_RN",v_ud_fyl2xp1_rn),
        ("recip_RN",v_recip_rn),("recip_RZ",v_recip_rz),("negw_1pw",v_negw_over_1pw),
        ("numdr_divRZ",v_numdr_divrz),("base_divRZ",v_base_divrz),
        ("chopexp_kahan",v_chopexp_kahan),("chopexp_kahan_ndr",v_chopexp_kahan_numdr),
        ("chopexp_um1",v_chopexp_um1),("chopexp_fyl2xp1",v_chopexp_fyl2xp1),
        ("pc53_kahan",v_pc53_kahan),("pc53_kahan_ndr",v_pc53_kahan_numdr),
        ("pc53exp_pc64ln",v_pc53exp_pc64ln),("pc53_allbody",v_pc53_allbody),
    ];
    for (src,path,po2) in [("po2n","../../work/w109/G6-solvers/answers-pmt-po2n.json",true),
                            ("gen","../../work/w109/G6-solvers/answers-pmt-genrate.json",false)]{
        let data=load(path);
        let mut pins:Vec<(f64,f64,f64)>=Vec::new();
        for ((rb,nb),rows) in &data{
            let r=f64::from_bits(*rb); let n=f64::from_bits(*nb);
            if tau_sse(r,n).abs()>=1.0{continue}
            let km=v_base(r,n);
            let p=if po2{pin_po2(rows,r)}else{pin_gen(rows,r,km)};
            if let Some(e)=p{ pins.push((r,n,e)); }
        }
        let ln2=0.6931471805599453_f64;
        let tot_lt=pins.iter().filter(|(r,n,_)|tau_sse(*r,*n).abs()<ln2).count();
        println!("=== {} ({} |tau|<1 pinned; |tau|<ln2 subset {}) ===", src, pins.len(), tot_lt);
        for (nm,f) in cands{
            let mut ok=0; let mut plus=0; let mut minus=0; let mut ok_lt=0;
            for (r,n,ep) in &pins{
                let g=f(*r,*n).to_bits(); let d=sordi(g)-sordi(ep.to_bits());
                if d==0{ok+=1; if tau_sse(*r,*n).abs()<ln2{ok_lt+=1;}}
                else if d>0{plus+=1}else{minus+=1}
            }
            println!("  {:<18} {:>3}/{}  (miss +{}/-{})  [<ln2: {}/{}]", nm, ok, pins.len(), plus, minus, ok_lt, tot_lt);
        }
    }
}
