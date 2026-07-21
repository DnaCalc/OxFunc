//! W109 G6-01 PMT: the untested cell (2026-07-21). Findings so far:
//!  * FV/PV close bit-exact as naive FORWARD binexp (plain double).
//!  * Metamorphic proof: PMT does NOT share FV's forward factor (feeding Excel's
//!    own P + tf·q into -(pv·P+fv)/(tf·q) is 0/109 on small rate). PMT is a
//!    numerically-STABLE (discount) form.
//!  * Standing prior: legacy financial bodies are per-op x87 double-rounded
//!    spill loops (XNPV, NPER closed that way).
//! Never raced: the STABLE DISCOUNT form with the whole body in x87 DR AND x87
//! log1p/expm1. This bin races exactly that, held-out ranked (overfit killer).
//!
//! Stable identity: v=(1+r)^-n; em=v-1=expm1(-n·log1p r);
//!   pmt = -(pv + fv·v)·r / (tf·(1-v)),  tf = 1 + r·type.  (1-v = -em)

use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use rx::Ext80;

const CW: u16 = rx::CW_PC64_RN;

// x87 extended temp with explicit store-to-double control (mirrors fit_pmt_stores).
#[derive(Clone, Copy)]
struct V(Ext80);
impl V {
    fn n(x: f64) -> V { V(rx::ext_from_f64(x)) }
    fn st(self, yes: bool) -> V { if yes { V::n(self.f()) } else { self } }
    fn f(self) -> f64 { rx::ext_to_f64(&self.0, CW) }
    fn add(self, o: V) -> V { V(rx::ext_add(&self.0, &o.0, CW)) }
    fn sub(self, o: V) -> V { V(rx::ext_sub(&self.0, &o.0, CW)) }
    fn mul(self, o: V) -> V { V(rx::ext_mul(&self.0, &o.0, CW)) }
    fn div(self, o: V) -> V { V(rx::ext_div(&self.0, &o.0, CW)) }
    fn neg(self) -> V { V::n(0.0).sub(self) }
}

#[derive(Clone, Copy)] enum LP { Log1pCR, Fyl2xp1Nat, LnOf1PlusR }
#[derive(Clone, Copy)] enum EP { Expm1Int, Expm1CR, ExpMinus1 }
#[derive(Clone, Copy)] enum AR { StableEm, Stable1mv, VIndep, ForwardX87 }

fn lname(l:LP)->&'static str{match l{LP::Log1pCR=>"log1pCR",LP::Fyl2xp1Nat=>"fyl2xp1",LP::LnOf1PlusR=>"ln(1+r)"}}
fn ename(e:EP)->&'static str{match e{EP::Expm1Int=>"expm1Int",EP::Expm1CR=>"expm1CR",EP::ExpMinus1=>"exp-1"}}
fn aname(a:AR)->&'static str{match a{AR::StableEm=>"stableEm",AR::Stable1mv=>"stable1mv",AR::VIndep=>"vIndep",AR::ForwardX87=>"fwdX87"}}

// natural log1p(r)
fn logp(lp: LP, r: f64) -> f64 {
    match lp {
        LP::Log1pCR => rx::excel_log1p(r),
        LP::Fyl2xp1Nat => rx::ext_to_f64(&rx::ext_fyl2xp1(&rx::ext_ln2(), &rx::ext_from_f64(r), CW), CW),
        LP::LnOf1PlusR => rx::excel_ln(1.0 + r),
    }
}
fn expm1p(ep: EP, t: f64) -> f64 {
    match ep { EP::Expm1Int => rx::excel_expm1_internal(t), EP::Expm1CR => rx::excel_expm1(t), EP::ExpMinus1 => rx::excel_exp(t) - 1.0 }
}

// mask bits: b0 L, b1 t, b2 v, b3 em, b4 num, b5 tf, b6 den, b7 quotient
fn model(lp:LP, ep:EP, ar:AR, m:u8, rate:f64, n:f64, pv:f64, fv:f64, ty:f64) -> f64 {
    let bit=|i:u8| m & (1<<i) != 0;
    if rate == 0.0 { return -(pv + fv) / n; }
    if 1.0 + rate <= 0.0 { return f64::NAN; }
    let l = logp(lp, rate);
    let t = V::n(-n).mul(V::n(l)).st(bit(1)).f();     // -n·log1p(r)  (x87 mul, stored per mask)
    let v = rx::excel_exp(t);                          // (1+r)^-n via x87 exp
    let em = expm1p(ep, t);                            // (1+r)^-n - 1
    let tf = V::n(1.0).add(V::n(rate).mul(V::n(ty))).st(bit(5)).f();
    match ar {
        AR::StableEm => {
            // pmt = (pv + fv·v)·r / (tf·em)   [em<0 gives the sign]
            let num = V::n(pv).add(V::n(fv).mul(V::n(v)).st(bit(2))).st(bit(4));
            let den = V::n(tf).mul(V::n(em)).st(bit(6));
            num.mul(V::n(rate)).st(bit(3)).div(den).st(bit(7)).f()
        }
        AR::Stable1mv => {
            // pmt = -(pv + fv·v)·r / (tf·(1-v))
            let onemv = V::n(1.0).sub(V::n(v)).st(bit(3));
            let num = V::n(pv).add(V::n(fv).mul(V::n(v)).st(bit(2))).st(bit(4));
            let den = V::n(tf).mul(onemv).st(bit(6));
            num.mul(V::n(rate)).div(den).st(bit(7)).neg().f()
        }
        AR::VIndep => {
            // production-style: v and em independent; num=pv+fv·v; den=tf·em
            let num = V::n(pv).add(V::n(fv).mul(V::n(v)).st(bit(2))).st(bit(4));
            let den = V::n(tf).mul(V::n(em)).st(bit(6));
            V::n(-1.0).mul(num).mul(V::n(rate)).st(bit(3)).div(den.neg()).st(bit(7)).f()
        }
        AR::ForwardX87 => {
            // control: forward P=exp(+n·L) x87, q=(P-1)/r, pmt=-(pv·P+fv)/(tf·q)
            let p = rx::excel_exp(V::n(n).mul(V::n(l)).st(bit(1)).f());
            let q = V::n(p).sub(V::n(1.0)).st(bit(3)).div(V::n(rate)).st(bit(2));
            let num = V::n(pv).mul(V::n(p)).add(V::n(fv)).st(bit(4));
            let den = V::n(tf).mul(q).st(bit(6));
            num.div(den).st(bit(7)).neg().f()
        }
    }
}

fn load(path:&str)->Vec<(Vec<f64>,u64)>{
    let ws:WitnessSet=serde_json::from_str(&std::fs::read_to_string(path).expect("read")).expect("parse");
    let mut o=Vec::new();
    for w in &ws.witnesses{
        let a:Vec<f64>=w.args.iter().filter_map(|x|match x{WitnessArg::Scalar(s)=>parse_bits_hex(s),_=>None}).collect();
        if a.len()!=5{continue}
        if let Some(want)=parse_bits_hex(&w.expected_bits){o.push((a,want.to_bits()))}
    }
    o
}
fn sord(u:u64)->i128{if u<1<<63{u as i128}else{((1u128<<63)as i128)-(u as i128-(1i128<<63))}}

const LPS:[LP;3]=[LP::Log1pCR,LP::Fyl2xp1Nat,LP::LnOf1PlusR];
const EPS:[EP;3]=[EP::Expm1Int,EP::Expm1CR,EP::ExpMinus1];
const ARS:[AR;4]=[AR::StableEm,AR::Stable1mv,AR::VIndep,AR::ForwardX87];

fn score(o:&[(Vec<f64>,u64)],lp:LP,ep:EP,ar:AR,m:u8)->u32{
    o.iter().filter(|(a,w)| model(lp,ep,ar,m,a[0],a[1],a[2],a[3],a[4]).to_bits()==*w).count() as u32
}

fn main(){
    let av:Vec<String>=std::env::args().collect();
    let tr=av.get(1).cloned().unwrap_or_else(||"../../work/w109/G6-solvers/answers-pmt-r0.json".into());
    let ho=av.get(2).cloned().unwrap_or_else(||"../../work/w109/G6-solvers/answers-pmt-heldout.json".into());
    let tro=load(&tr); let hoo=load(&ho);
    println!("train {} held {}", tro.len(), hoo.len());
    let mut cand:Vec<(u32,u32,LP,EP,AR,u8)>=Vec::new();
    for &lp in &LPS{for &ep in &EPS{for &ar in &ARS{for m in 0u8..=255u8{
        cand.push((score(&hoo,lp,ep,ar,m),score(&tro,lp,ep,ar,m),lp,ep,ar,m));
    }}}}
    cand.sort_by(|a,b|b.0.cmp(&a.0).then(b.1.cmp(&a.1)));
    println!("\nTOP 20 by HELD-OUT:");
    for (h,t,lp,ep,ar,m) in cand.iter().take(20){
        println!("  held {:4}/{}  train {:2}/{}  L={:8} E={:8} arr={:9} mask={:08b}",h,hoo.len(),t,tro.len(),lname(*lp),ename(*ep),aname(*ar),m);
    }
    // champion residual + n=1 + regime
    let (_,_,lp,ep,ar,m)=cand[0];
    println!("\n=== champion residual: L={} E={} arr={} mask={:08b} ===",lname(lp),ename(ep),aname(ar),m);
    use std::collections::BTreeMap;
    let mut hist:BTreeMap<i64,u32>=BTreeMap::new();
    let mut byn:BTreeMap<i64,(u32,u32)>=BTreeMap::new();
    let mut reg:BTreeMap<&str,(u32,u32)>=BTreeMap::new();
    for (a,w) in &hoo{
        let g=model(lp,ep,ar,m,a[0],a[1],a[2],a[3],a[4]).to_bits();
        let d=(sord(g)-sord(*w))as i64;
        *hist.entry(d.clamp(-6,6)).or_insert(0)+=1;
        let e=byn.entry(a[1]as i64).or_insert((0,0)); if d==0{e.0+=1}else{e.1+=1}
        let rr=a[0];
        let rk = if rr<0.0 {"neg"} else if rr.abs()<1e-4 {"tiny"} else if rr.abs()<1e-2 {"small"} else {"normal"};
        let e2=reg.entry(rk).or_insert((0,0)); if d==0{e2.0+=1}else{e2.1+=1}
    }
    println!("ulp hist: {:?}",hist);
    println!("by regime: {:?}",reg);
    println!("by n:");
    for (n,(ex,off)) in &byn{println!("  n={:5} exact {:4} off {:4}",n,ex,off)}
}
