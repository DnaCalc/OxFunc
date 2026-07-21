//! W109 G6-01 PMT H-DF combine (2026-07-21, Fable breakthrough). The combine is
//! QUOTIENT-FIRST, *rate LAST (VB financial lineage): pmt = RN(RN(num/den)*r),
//! num=pv+fv*(1+em), den=em*tf, tf=1+r*type. Confirmed 256/256 on every
//! consecutive-pv sweep. Remaining unknown = em bit-exact. This bin scores H-DF
//! with the REAL x87 internal-Kahan em on the full corpora, per Fable's 4 spill
//! variants {em f64 / em extended} x {ops RN53 / x87 RN64->RN53}.
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;

#[derive(Clone,Copy)] struct Var{ ext_q:bool, x87_ops:bool }

fn pmt_hdf(r:f64,n:f64,pv:f64,fv:f64,ty:f64,v:Var)->f64{
    if r==0.0 { return -(pv+fv)/n; }
    let l = rx::excel_log1p(r);
    let tau = -n*l;
    let em = rx::excel_expm1_internal(tau);   // (1+r)^-n - 1, x87 internal-Kahan
    let vv = 1.0 + em;                          // (1+r)^-n  (num uses v=1+em)
    let tf = 1.0 + r*ty;
    let num = pv + fv*vv;
    let den = em*tf;
    if v.x87_ops {
        // x87 double-rounded ops (RN64 then RN53 store), quotient optionally held extended
        use rx::{ext_from_f64, ext_div, ext_mul, ext_to_f64, CW_PC64_RN as CW};
        let qext = ext_div(&ext_from_f64(num), &ext_from_f64(den), CW);
        if v.ext_q {
            // keep quotient extended, multiply by r extended, single store
            ext_to_f64(&ext_mul(&qext, &ext_from_f64(r), CW), CW)
        } else {
            let qd = ext_to_f64(&qext, CW);     // store quotient to f64
            rx::x87_mul(qd, r)                  // RN53(RN64(qd*r))
        }
    } else {
        // plain SSE2 ops
        let q = num/den;
        q*r
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

fn main(){
    let corpora=[("heldout","answers-pmt-heldout.json"),("r0","answers-pmt-r0.json"),
                 ("pvladder","answers-pmt-pvladder.json"),("fvty","answers-pmt-fvty.json"),
                 ("em","answers-pmt-em.json"),("combsweep","answers-pmt-combsweep.json")];
    let vars=[("sse",Var{ext_q:false,x87_ops:false}),
              ("x87 q->f64",Var{ext_q:false,x87_ops:true}),
              ("x87 q-ext",Var{ext_q:true,x87_ops:true})];
    for (cn,cf) in corpora{
        let p=format!("../../work/w109/G6-solvers/{}",cf);
        if !std::path::Path::new(&p).exists(){continue}
        let rows=load(&p);
        print!("{:10} ({:5}):",cn,rows.len());
        for (vn,v) in vars{
            let mut ex=0u32; let mut w1=0u32;
            for (a,want) in &rows{
                let g=pmt_hdf(a[0],a[1],a[2],a[3],a[4],v).to_bits();
                let d=(sord(g)-sord(*want)).abs();
                if d==0{ex+=1;} if d<=1{w1+=1;}
            }
            print!("  {}: {}/{} (±1 {}%)",vn,ex,rows.len(),100*w1/rows.len() as u32);
        }
        println!();
    }
}
