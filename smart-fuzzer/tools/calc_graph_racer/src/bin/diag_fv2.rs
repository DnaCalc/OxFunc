//! W109 G6-01: large-fv PMT assembly. KEY: v=1+em cancels catastrophically when
//! em~=-1 (large n -> v tiny). Use v=exp(tau) directly. Cross v-delivery with
//! combine op-orders on all fv corpora + isolate the accurate-em (|tau|>=1) subset.
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;

fn main(){
    for cn in ["fvsweep","fv1sweep","fvty","heldout","pvladder","combsweep"]{
        let p=format!("../../work/w109/G6-solvers/answers-pmt-{}.json",cn);
        if !std::path::Path::new(&p).exists(){continue}
        let ws:WitnessSet=serde_json::from_str(&std::fs::read_to_string(&p).unwrap()).unwrap();
        let names:[&str;8]=[
            "v1e:(num/em)/tf*r","vex:(num/em)/tf*r",
            "v1e:(num*r)/(tf*em)","vex:(num*r)/(tf*em)",
            "v1e:num*r/tf/em","vex:num*r/tf/em",
            "vex:-num*r/(tf*(1-v))","vex:(num/(em*tf))*r",
        ];
        let mut sc=[0u32;8]; let mut tot=0u32;
        // accurate-em subset (|tau|>=1)
        let mut scA=[0u32;8]; let mut totA=0u32;
        for w in &ws.witnesses{
            let a:Vec<f64>=w.args.iter().filter_map(|x|match x{WitnessArg::Scalar(s)=>parse_bits_hex(s),_=>None}).collect();
            if a.len()!=5{continue}
            let want=parse_bits_hex(&w.expected_bits).unwrap().to_bits();
            let (r,n,pv,fv,ty)=(a[0],a[1],a[2],a[3],a[4]);
            if r==0.0{continue}
            let tau=-(n*rx::excel_log1p(r));
            let em=rx::excel_expm1_internal(tau);
            let v1=1.0+em; let ve=rx::excel_exp(tau);
            let tf=1.0+r*ty;
            let n1=pv+fv*v1; let ne=pv+fv*ve;
            let g=[
                (n1/em)/tf*r, (ne/em)/tf*r,
                (n1*r)/(tf*em), (ne*r)/(tf*em),
                n1*r/tf/em, ne*r/tf/em,
                -ne*r/(tf*(1.0-ve)), (ne/(em*tf))*r,
            ];
            tot+=1;
            let acc = tau.abs()>=1.0;
            if acc{totA+=1;}
            for i in 0..8{ if g[i].to_bits()==want{sc[i]+=1; if acc{scA[i]+=1;}} }
        }
        println!("=== {} (N={}, accurate-em subset={}) ===",cn,tot,totA);
        for i in 0..8{ println!("  [{}] {:<26} {:>6}/{}   acc {:>5}/{}",i,names[i],sc[i],tot,scA[i],totA); }
    }
}
