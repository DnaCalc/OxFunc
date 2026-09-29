//! Fresh public input selectors that distinguish RN64/RN53 operation stores.
//! Selects using arithmetic graph disagreement only, without oracle outcomes.
use oxfunc_core::excel_numeric::research::*;
use serde_json::json;
use std::collections::BTreeSet;
fn next(s: &mut u64) -> u64 {
    *s ^= *s << 13; *s ^= *s >> 7; *s ^= *s << 17; *s
}
fn count(q: f64) -> f64 {
    let k=q.floor(); k+f64::from(q-k>=f64::from_bits(0x3fdf_ffff_ffff_ffa6))
}
fn main() {
    let mut seed=2026092968_u64; let mut rows=BTreeSet::new();
    let mut divisions=0;let mut products=0;let mut draws=0;
    while divisions<160 || products<160 {
        draws+=1;
        assert!(draws<20_000_000,"selector limit");
        let exponent=523+(next(&mut seed)%1000);
        let m=f64::from_bits((exponent<<52)|(next(&mut seed)&0x000f_ffff_ffff_ffff));
        let k=(1_u64<<(40+next(&mut seed)%14)) as f64
            +(next(&mut seed)&0x000f_ffff_ffff_ffff) as f64;
        let center=(k*m).to_bits();
        let x=f64::from_bits(center.wrapping_add_signed((next(&mut seed)%3) as i64-1));
        if !x.is_normal(){continue;}
        let q=ext_to_f64(&ext_div(&ext_from_f64(x),&ext_from_f64(m),CW_PC64_RN),CW_PC64_RN);
        let n=count(q); let result=x87_mul(n,m);
        let division_disagrees=x87_mul(count(x/m),m).to_bits()!=result.to_bits();
        let product_disagrees=(n*m).to_bits()!=result.to_bits();
        let admit=(divisions<160&&division_disagrees)||(products<160&&product_disagrees);
        if !admit{continue;}
        if division_disagrees{divisions+=1;}
        if product_disagrees{products+=1;}
        for offset in -2_i64..=2 {
            let xb=x.to_bits().wrapping_add_signed(offset);
            for sign in [0,1_u64<<63]{rows.insert((xb|sign,m.to_bits()|sign));}
        }
    }
    let probes:Vec<_>=rows.iter().enumerate().map(|(i,(x,m))|json!({"probe":{
        "id":format!("mround-staged-heldout-{i:05}"),
        "args":[format!("0x{x:016x}"),format!("0x{m:016x}")]}})).collect();
    let out=std::path::PathBuf::from(std::env::args().nth(1).unwrap());
    std::fs::create_dir_all(&out).unwrap();
    std::fs::write(out.join("batch-mround.json"),serde_json::to_vec_pretty(&json!({"function":"MROUND","probes":probes})).unwrap()).unwrap();
    let manifest=json!({"generator":"mround_staged_holdout.rs","seed":2026092968_u64,
        "selection":"fresh random graph disagreements, five source-ULP neighbors and both signs; no oracle selection",
        "draws":draws,"division_centers":divisions,"product_centers":products,
        "batches":[{"function":"MROUND","path":"batch-mround.json","rows":rows.len()}]});
    std::fs::write(out.join("manifest.json"),serde_json::to_vec_pretty(&manifest).unwrap()).unwrap();
    println!("{manifest}");
}
