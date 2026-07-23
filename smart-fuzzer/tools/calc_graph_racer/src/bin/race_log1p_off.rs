//! W109 G6-01: per-r effective log1p OFFSET extraction on the dense n=1 sweep.
//! For each r, find the integer ulp offset o applied to CR log1p bits such that
//! pmt_n1(r,pv, log1p=CR+o) reproduces ALL pv. Reads o(r) structure. Also flags
//! rows where NO single offset closes all pv (=> expm1/combine sub-ulp, not log1p),
//! and separately reports the EXACT power-of-two r rows (em is model-free there).
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use std::collections::BTreeMap;

fn ulp_off(x:f64, o:i64)->f64{
    // offset by o ulps toward the direction of increasing magnitude if x>0.
    // Work in ordered-int space so it is monotone across zero for positive x.
    let b=x.to_bits() as i64;
    f64::from_bits((b + o) as u64)
}
fn pmt_off(r:f64, pv:f64, o:i64)->f64{
    let l = ulp_off(rx::excel_log1p(r), o);   // nudge log1p by o ulps
    let tau=-l;
    let em=rx::excel_expm1_internal(tau);
    (pv/em)*r
}
fn is_po2(rb:u64)->bool{ (rb & 0x000f_ffff_ffff_ffff)==0 }

fn main(){
    let path="../../work/w109/G6-solvers/answers-pmt-denselog1p.json";
    let ws:WitnessSet = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let mut byr:BTreeMap<u64,Vec<(f64,u64)>> = BTreeMap::new();
    for w in &ws.witnesses{
        let a:Vec<f64>=w.args.iter().filter_map(|x|match x{WitnessArg::Scalar(s)=>parse_bits_hex(s),_=>None}).collect();
        if a.len()!=5{continue}
        let want=parse_bits_hex(&w.expected_bits).unwrap().to_bits();
        byr.entry(a[0].to_bits()).or_default().push((a[2],want));
    }
    // For each r: best offset in [-4,4] (min failures); record winner+residual.
    let mut hist:BTreeMap<i64,u32>=BTreeMap::new();
    let mut noclose=0u32;         // r where even best offset leaves >0
    let mut po2rows=Vec::new();
    // structure dump keyed by (binade, position-in-binade) — but simplest: emit CSV
    let mut rows_out:Vec<(u64,f64,i64,u32)>=Vec::new(); // r_bits, r, best_o, residual
    for (rb, rows) in &byr{
        let r=f64::from_bits(*rb);
        let mut best=(u32::MAX, 0i64);
        for o in -4..=4{
            let mut fail=0u32;
            for (pv,want) in rows{ if pmt_off(r,*pv,o).to_bits()!=*want{fail+=1;} }
            if fail<best.0{best=(fail,o);}
            if fail==0{break;}
        }
        *hist.entry(best.1).or_default()+=1;
        if best.0>0{noclose+=1;}
        if is_po2(*rb){ po2rows.push((r, best.1, best.0)); }
        rows_out.push((*rb, r, best.1, best.0));
    }
    println!("offset histogram (o : #r):");
    for (o,c) in &hist{ println!("  o={:+}  {}", o, c); }
    println!("r with NO single-offset close (expm1/combine sub-ulp): {}", noclose);
    println!("\nEXACT power-of-two r rows (em model-free): r_log2  best_o  residual");
    for (r,o,res) in &po2rows{ println!("  2^{:+.4}  o={:+}  res={}", r.log2(), o, res); }

    // dump CSV of o(r) for offline structure analysis
    let mut s=String::from("r_bits,r,r_log2,best_o,residual\n");
    for (rb,r,o,res) in &rows_out{
        s.push_str(&format!("0x{:016x},{:.17e},{:.6},{},{}\n", rb, r, r.log2(), o, res));
    }
    std::fs::write("../../work/w109/G6-solvers/log1p_offset_curve.csv", s).unwrap();
    println!("\nwrote log1p_offset_curve.csv ({} rows)", rows_out.len());
}
