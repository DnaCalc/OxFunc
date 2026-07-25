//! W109 G6-01: dump the model-free pinned em for every COLLISION config as CSV
//! (r_bits, n, tau0_bits, em_pinned_bits) so python+mpmath can analyse the
//! within-group dependence of em on the EXACT tau (the sub-ULP tail).
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;

fn pin_gen(rows:&[(f64,u64)], r:f64, center:f64)->Option<f64>{
    let cb=center.to_bits() as i64;
    for d in -20..=20i64{ let em=f64::from_bits((cb+d) as u64); if em>=0.0{continue}
        if rows.iter().all(|(pv,want)|((pv/em)*r).to_bits()==*want){ return Some(em); } }
    None
}
fn main(){
    let ws:WitnessSet=serde_json::from_str(&std::fs::read_to_string(
        "../../work/w109/G6-solvers/answers-pmt-collide.json").unwrap()).unwrap();
    let wl=&ws.witnesses;
    let ncfg=wl.len()/128;
    println!("r_bits,n,tau0_bits,em_pinned_bits");
    for ci in 0..ncfg{
        let mut rows=Vec::new(); let mut rn=(0.0f64,0.0f64);
        for j in 0..128{
            let w=&wl[ci*128+j];
            let a:Vec<f64>=w.args.iter().filter_map(|x|match x{WitnessArg::Scalar(s)=>parse_bits_hex(s),_=>None}).collect();
            rn=(a[0],a[1]);
            rows.push((a[2], parse_bits_hex(&w.expected_bits).unwrap().to_bits()));
        }
        let (r,n)=rn;
        let tau0=-(n*rx::excel_log1p(r));
        let km=rx::excel_expm1_internal(tau0);
        if let Some(e)=pin_gen(&rows,r,km){
            println!("{:016x},{},{:016x},{:016x}", r.to_bits(), n as i64, tau0.to_bits(), e.to_bits());
        }
    }
}
