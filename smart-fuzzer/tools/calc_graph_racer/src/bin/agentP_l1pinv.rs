//! Invert n=1 em -> effective log1p. For each r with pinned em, scan tau near
//! -CR_log1p(r); find the set of tau (=-effective log1p) reproducing em via the
//! confirmed n=1 Kahan. Output deviation (effective log1p - CR log1p) in ulp.
use oxfunc_core::excel_numeric::research as rx;
fn fb(h:&str)->f64{ f64::from_bits(u64::from_str_radix(h.trim_start_matches("0x"),16).unwrap()) }
fn na(x:f64,d:i64)->f64{ f64::from_bits((x.to_bits() as i64 + d) as u64) }
fn kahan_n1(tau:f64)->f64{
    let u=rx::excel_exp(tau); let lnu=rx::excel_ln(u);
    if u==1.0 {tau} else if tau.abs()<1.0 {(u-1.0)*tau/lnu} else {u-1.0}
}
fn main(){
    let v:serde_json::Value=serde_json::from_str(&std::fs::read_to_string("../../work/w109/G6-solvers/agentP_log1p_em.json").unwrap()).unwrap();
    let mut rows:Vec<(f64,f64)>=v.as_object().unwrap().iter().map(|(k,val)|(fb(k),fb(val.as_str().unwrap()))).collect();
    rows.sort_by(|a,b|a.0.partial_cmp(&b.0).unwrap());
    println!("r,r_log2,log1p_cr,dev_ulps");
    for (r,em) in &rows{
        let lcr=rx::excel_log1p(*r);
        // scan log1p deviation d: effective log1p = na(lcr,d), tau=-that
        let mut hits=Vec::new();
        for d in -12..=12 {
            let l=na(lcr,d);
            if kahan_n1(-l).to_bits()==em.to_bits(){ hits.push(d); }
        }
        let devs= if hits.is_empty(){ "NONE".to_string() } else { hits.iter().map(|d|d.to_string()).collect::<Vec<_>>().join(" ") };
        println!("{},{:.4},0x{:016x},{}", format!("0x{:016x}",r.to_bits()), r.log2(), lcr.to_bits(), devs);
    }
}
