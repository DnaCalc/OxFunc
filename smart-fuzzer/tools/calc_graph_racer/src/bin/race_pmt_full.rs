//! W109 G6-01: full pinned PMT op-graph on ALL corpora, with failure categorization.
//! Combine (quotient-first H-DF): num=pv+fv*v; q1=num/em; q2=q1/tf; pmt=q2*r.
//! v=1+em; em=expm1_internal(tau); tau=-n*log1p_cr(r); tf=1+r*ty. Categorize misses
//! by {rate sign, |tau| regime, fv!=0, ty!=0} to localize the residual.
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use std::collections::BTreeMap;

fn expm1i(t:f64)->f64{ rx::excel_expm1_internal(t) }

fn pmt(r:f64,n:f64,pv:f64,fv:f64,ty:f64)->f64{
    if r==0.0{ return -(pv+fv)/n; }
    let tau=-(n*rx::excel_log1p(r));
    let em=expm1i(tau);
    let v=1.0+em;
    let tf=1.0+r*ty;
    let num=pv+fv*v;
    let q1=num/em;
    let q2=q1/tf;
    q2*r
}

fn main(){
    let corpora=["heldout","combsweep","po2","r25","pvladder","fvty"];
    for cn in corpora{
        let p=format!("../../work/w109/G6-solvers/answers-pmt-{}.json",cn);
        if !std::path::Path::new(&p).exists(){ continue; }
        let ws:WitnessSet=serde_json::from_str(&std::fs::read_to_string(&p).unwrap()).unwrap();
        let mut ok=0u32; let mut tot=0u32;
        // categories
        let mut cat:BTreeMap<&str,(u32,u32)>=BTreeMap::new(); // name -> (ok,tot)
        let mut off_hist:BTreeMap<i64,u32>=BTreeMap::new();
        for w in &ws.witnesses{
            let a:Vec<f64>=w.args.iter().filter_map(|x|match x{WitnessArg::Scalar(s)=>parse_bits_hex(s),_=>None}).collect();
            if a.len()!=5{continue}
            let want=parse_bits_hex(&w.expected_bits).unwrap();
            let (r,n,pv,fv,ty)=(a[0],a[1],a[2],a[3],a[4]);
            let g=pmt(r,n,pv,fv,ty);
            let hit=g.to_bits()==want.to_bits();
            tot+=1; if hit{ok+=1;}
            // categorize
            let tau=-(n*rx::excel_log1p(r));
            let cats:[&str;5]=[
                if r<0.0{"r<0"}else if r>0.1{"r>0.1"}else{"0<r<=0.1"},
                if fv!=0.0{"fv!=0"}else{"fv=0"},
                if ty!=0.0{"ty!=0"}else{"ty=0"},
                if tau.abs()<1.0{"|tau|<1"}else{"|tau|>=1"},
                "ALL",
            ];
            for c in cats{ let e=cat.entry(c).or_insert((0,0)); e.1+=1; if hit{e.0+=1;} }
            if !hit{
                // signed ulp offset (ordered-int)
                let so=|u:u64|->i64{ if u<1<<63 {u as i64} else {-((u^(1u64<<63))as i64)} };
                let d=(so(g.to_bits())-so(want.to_bits())).clamp(-9,9);
                *off_hist.entry(d).or_default()+=1;
            }
        }
        println!("=== {:10} {}/{} ({:.1}%) ===", cn, ok, tot, 100.0*ok as f64/tot as f64);
        for (c,(o,t)) in &cat{
            if *c=="ALL"{continue}
            println!("    {:12} {}/{} ({:.0}%)", c, o, t, 100.0*(*o as f64)/(*t as f64));
        }
        print!("    off-hist:");
        for (d,c) in &off_hist{ print!(" {:+}:{}", d, c); }
        println!("\n");
    }
}
