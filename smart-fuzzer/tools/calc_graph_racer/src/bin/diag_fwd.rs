//! W109 G6-01: FORWARD PMT form with x87 exp power (matching RATE's (1+r)^n).
//! P=exp(n*log1p r); Pm1=expm1(n*log1p r) [accurate P-1]; pmt=-(pv*P+fv)/((Pm1/r))/tf.
//! The metamorphic "forward refuted" used FV's BINEXP P; the exp-based P is untested.
//! Race forward orderings vs discount across ALL corpora to find one unifying form.
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;

fn main(){
    let names:[&str;10]=[
        "D:(num/em)/tf*r          [disc,v1]",
        "F1:-(pv*P+fv)*r/(tf*Pm1)",
        "F2:-(pv*P+fv)/(tf*(Pm1/r))",
        "F3:num=pvP+fv; -(num/Pm1)/tf*r",
        "F4:num=pvP+fv; -(num/Pm1)*r/tf",
        "F5:num=pvP+fv; -num*r/(tf*Pm1)",
        "F6:num=pvP+fv; -(num/(Pm1*tf))*r",
        "F7:fvifa=Pm1/r; -(pvP+fv)/fvifa/tf",
        "F8:fvifa=(P-1)/r subtractive",
        "F9:P via 1/v (v=exp(tau)); Pm1=-em/v",
    ];
    for cn in ["combsweep","pvladder","fvsweep","fv1sweep","fvty","heldout","po2","r25"]{
        let p=format!("../../work/w109/G6-solvers/answers-pmt-{}.json",cn);
        if !std::path::Path::new(&p).exists(){continue}
        let ws:WitnessSet=serde_json::from_str(&std::fs::read_to_string(&p).unwrap()).unwrap();
        let mut sc=[0u32;10]; let mut tot=0u32;
        for w in &ws.witnesses{
            let a:Vec<f64>=w.args.iter().filter_map(|x|match x{WitnessArg::Scalar(s)=>parse_bits_hex(s),_=>None}).collect();
            if a.len()!=5{continue}
            let want=parse_bits_hex(&w.expected_bits).unwrap().to_bits();
            let (r,n,pv,fv,ty)=(a[0],a[1],a[2],a[3],a[4]);
            if r==0.0{continue}
            let tau=-(n*rx::excel_log1p(r));          // = -n*log1p(r)
            let em=rx::excel_expm1_internal(tau);      // (1+r)^-n - 1
            let v1=1.0+em;
            let p_=rx::excel_exp(-tau);                // (1+r)^n
            let pm1=rx::excel_expm1_internal(-tau);    // (1+r)^n - 1
            let vexp=rx::excel_exp(tau);
            let tf=1.0+r*ty;
            let g=[
                { let num=pv+fv*v1; (num/em)/tf*r },
                -(pv*p_+fv)*r/(tf*pm1),
                -(pv*p_+fv)/(tf*(pm1/r)),
                { let num=pv*p_+fv; -(num/pm1)/tf*r },
                { let num=pv*p_+fv; -(num/pm1)*r/tf },
                { let num=pv*p_+fv; -num*r/(tf*pm1) },
                { let num=pv*p_+fv; -(num/(pm1*tf))*r },
                { let fvifa=pm1/r; -(pv*p_+fv)/fvifa/tf },
                { let fvifa=(p_-1.0)/r; -(pv*p_+fv)/fvifa/tf },
                { let pp=1.0/vexp; let pm=(-em)/vexp; let num=pv*pp+fv; -(num/pm)/tf*r },
            ];
            tot+=1;
            for i in 0..10{ if g[i].to_bits()==want{sc[i]+=1;} }
        }
        print!("{:10} N={:6}", cn, tot);
        for i in 0..10{ print!(" [{}]{:>5}", i, sc[i]); }
        println!();
    }
    println!("\nlegend:");
    for i in 0..10{ println!("  [{}] {}", i, names[i]); }
}
