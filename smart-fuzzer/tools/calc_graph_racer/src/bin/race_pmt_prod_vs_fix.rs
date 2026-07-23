//! W109 G6-01: measure production PMT (reciprocal-multiply combine + portable exp/expm1)
//! vs the corrected op-graph (quotient-first SSE2 combine + x87 excel_exp/excel_expm1_internal,
//! CR log1p kept). Workflow: production uses a REFUTED combine + wrong substrate.
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;

// production op-graph (transcribed from financial_time_value_family.rs::pmt, but with the
// SAME portable core it calls: excel_log1p + portable exp/expm1). We approximate portable
// exp/expm1 by the x87 ones' CR-ish behavior? No: to isolate the COMBINE+SUBSTRATE effect,
// compare these concrete variants:
fn v_prod(r:f64,n:f64,pv:f64,fv:f64,ty:f64)->f64{  // recip-mult combine + x87 substrate
    let nl=-(n*rx::excel_log1p(r)); let invf=rx::excel_exp(nl); let denom=-rx::excel_expm1_internal(nl);
    let tf=1.0+r*ty; let num=pv+fv*invf; let recip=1.0/(tf*denom); -num*r*recip
}
fn v_qf_div(r:f64,n:f64,pv:f64,fv:f64,ty:f64)->f64{ // quotient-first, tf=divide, v=1+em
    let nl=-(n*rx::excel_log1p(r)); let em=rx::excel_expm1_internal(nl); let v=1.0+em;
    let tf=1.0+r*ty; let num=pv+fv*v; ((num/em)/tf)*r
}
fn v_qf_recip(r:f64,n:f64,pv:f64,fv:f64,ty:f64)->f64{ // quotient-first, tf=reciprocal-multiply
    let nl=-(n*rx::excel_log1p(r)); let em=rx::excel_expm1_internal(nl); let v=1.0+em;
    let tf=1.0+r*ty; let num=pv+fv*v; ((num/em)*(1.0/tf))*r
}
fn v_qf_vexp(r:f64,n:f64,pv:f64,fv:f64,ty:f64)->f64{ // quotient-first, v=exp(tau) directly
    let nl=-(n*rx::excel_log1p(r)); let em=rx::excel_expm1_internal(nl); let v=rx::excel_exp(nl);
    let tf=1.0+r*ty; let num=pv+fv*v; ((num/em)/tf)*r
}
fn v_qf_num165(r:f64,n:f64,pv:f64,fv:f64,ty:f64)->f64{ // qf_div but em via x87 double-rounded numerator (165)
    let nl=-(n*rx::excel_log1p(r)); let t=nl;
    let em = if t==0.0 {0.0} else { let u=rx::excel_exp(t); if u==1.0 {t} else if t.abs()>=1.0 {u-1.0}
        else { let num=rx::x87_mul(u-1.0,t); num/rx::excel_ln(u) } };
    let v=1.0+em; let tf=1.0+r*ty; let num=pv+fv*v; ((num/em)/tf)*r
}
fn main(){
    let corpora=["heldout","combsweep","po2","r25","fvty","fv1sweep","genrate"];
    let variants:[(&str,fn(f64,f64,f64,f64,f64)->f64);5]=[
        ("prod(recip-mult)",v_prod),("qf_div",v_qf_div),("qf_recip",v_qf_recip),
        ("qf_vexp",v_qf_vexp),("qf_num165",v_qf_num165)];
    println!("{:12} {:>6}  {}", "corpus","N", variants.iter().map(|v|format!("{:>16}",v.0)).collect::<String>());
    for cn in corpora{
        let p=format!("../../work/w109/G6-solvers/answers-pmt-{}.json",cn);
        if !std::path::Path::new(&p).exists(){continue}
        let ws:WitnessSet=serde_json::from_str(&std::fs::read_to_string(&p).unwrap()).unwrap();
        let mut sc=[0u32;5]; let mut tot=0u32;
        for w in &ws.witnesses{
            let a:Vec<f64>=w.args.iter().filter_map(|x|match x{WitnessArg::Scalar(s)=>parse_bits_hex(s),_=>None}).collect();
            if a.len()!=5{continue}
            let want=parse_bits_hex(&w.expected_bits).unwrap().to_bits(); tot+=1;
            for (i,(_,f)) in variants.iter().enumerate(){ if f(a[0],a[1],a[2],a[3],a[4]).to_bits()==want{sc[i]+=1;} }
        }
        print!("{:12} {:>6}  ", cn, tot);
        for i in 0..5{ print!("{:>16}", format!("{}({:.0}%)",sc[i],100.0*sc[i] as f64/tot as f64)); }
        println!();
    }
}
