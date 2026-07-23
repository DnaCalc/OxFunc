//! Export pinned Excel em for po2n + genrate corpora -> em_consolidated.csv
//! (r_bits, n, tau_bits, em_pinned_bits, kahan_bits) for offline minimax-fit analysis.
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use std::collections::BTreeMap;

fn kahan(r:f64,n:f64)->f64{ rx::excel_expm1_internal(-(n*rx::excel_log1p(r))) }

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
    let mut s=String::from("src,r_bits,n,tau_bits,em_pinned,kahan\n");
    for (src,path,po2) in [("po2n","../../work/w109/G6-solvers/answers-pmt-po2n.json",true),
                            ("gen","../../work/w109/G6-solvers/answers-pmt-genrate.json",false)]{
        for ((rb,nb),rows) in &load(path){
            let r=f64::from_bits(*rb); let n=f64::from_bits(*nb);
            let tau=-(n*rx::excel_log1p(r));
            if tau.abs()>=1.0{continue}
            let km=kahan(r,n);
            let em=if po2{pin_po2(rows,r)}else{pin_gen(rows,r,km)};
            if let Some(em)=em{
                s.push_str(&format!("{},{:016x},{},{:016x},{:016x},{:016x}\n",
                    src,rb,n as i64,tau.to_bits(),em.to_bits(),km.to_bits()));
            }
        }
    }
    std::fs::write("../../work/w109/G6-solvers/em_consolidated.csv",&s).unwrap();
    println!("wrote em_consolidated.csv ({} rows)", s.lines().count()-1);
}
