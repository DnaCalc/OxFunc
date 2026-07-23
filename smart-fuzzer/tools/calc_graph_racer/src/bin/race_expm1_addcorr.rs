//! W109 G6-01: expm1 |tau|<1 additive-correction forms. tau, exp, ln all proven
//! EXACT doubles -> Excel uses a different double-op SEQUENCE. Test base+correction
//! variants: em=(u-1)+(u-1)*(t-lnu)/lnu and relatives, all double.
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use std::collections::BTreeMap;

fn tau_d(r:f64,n:f64)->f64{ -(n*rx::excel_log1p(r)) }

fn forms(r:f64,n:f64)->[f64;12]{
    let t=tau_d(r,n); let u=rx::excel_exp(t); let lnu=rx::excel_ln(u); let b=u-1.0;
    [
        b*t/lnu,                       // 0 prod-first (production)
        b*(t/lnu),                     // 1 kahan
        b + b*(t-lnu)/lnu,             // 2 additive, (t-lnu) first
        b + b*((t-lnu)/lnu),           // 3 additive assoc
        b + b*(t/lnu - 1.0),           // 4 additive via t/lnu-1
        b*(1.0 + (t-lnu)/lnu),         // 5 factored
        b + (b/lnu)*(t-lnu),           // 6 (b/lnu) first
        b + b*t/lnu - b,               // 7 (silly, = prod but rounded)
        t + t*(b - lnu)/lnu,           // 8 t-based: t*u form? em~t
        t*b/lnu,                       // 9 t*(u-1)/lnu (numerator t*b)
        t*(b/lnu),                     // 10
        (t/lnu)*b,                     // 11 div-first
    ]
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
    let labels=["0 b*t/lnu","1 b*(t/lnu)","2 b+b*(t-lnu)/lnu","3 b+b*((t-lnu)/lnu)",
        "4 b+b*(t/lnu-1)","5 b*(1+(t-lnu)/lnu)","6 b+(b/lnu)*(t-lnu)","7 b+b*t/lnu-b",
        "8 t+t*(b-lnu)/lnu","9 t*b/lnu","10 t*(b/lnu)","11 (t/lnu)*b"];
    let mut sc=[0u32;12]; let mut tot=0u32;
    for ((rb,nb),rows) in &byrn{
        let r=f64::from_bits(*rb); let n=f64::from_bits(*nb);
        if tau_d(r,n).abs()>=1.0{continue}
        let em_ex=match pin_em(rows,r){Some(e)=>e,None=>continue}; tot+=1;
        let g=forms(r,n);
        for i in 0..12{ if g[i].to_bits()==em_ex.to_bits(){sc[i]+=1;} }
    }
    println!("|tau|<1 pinned: {}", tot);
    for i in 0..12{ println!("  {:<24} {}/{}", labels[i], sc[i], tot); }
}
