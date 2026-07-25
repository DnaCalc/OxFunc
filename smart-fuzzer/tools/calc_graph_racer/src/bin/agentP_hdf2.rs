//! Test log1p delivery = ln(1+r) [x87 fyl2x, 1+r exact for po2 r] vs fyl2xp1/portable,
//! through the full internal-Kahan model, on the clean 46-row oracle.
use oxfunc_core::excel_numeric::research as rx;
const CW: u16 = rx::CW_PC64_RN;
fn fb(h:&str)->f64{ f64::from_bits(u64::from_str_radix(h.trim_start_matches("0x"),16).unwrap()) }
fn load(path:&str)->Vec<(f64,i64,f64)>{
    let v:serde_json::Value=serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let mut o=Vec::new();
    for (k,val) in v.as_object().unwrap(){ let (rh,nh)=k.split_once('|').unwrap(); o.push((fb(rh),nh.parse().unwrap(),fb(val.as_str().unwrap()))); }
    o.sort_by(|a,b|(a.0,a.1).partial_cmp(&(b.0,b.1)).unwrap()); o
}
fn em_model(logp:f64, r:f64, n:i64)->f64{
    let t=-(n as f64)*logp;
    let u=rx::excel_exp(t);
    if u==1.0 { t } else if t.abs()<1.0 { (u-1.0)*t/rx::excel_ln(u) } else { u-1.0 }
}
fn main(){
    let rows=load("../../work/w109/G6-solvers/pmt_em_hdf_oracle.json");
    let logs:[(&str, fn(f64)->f64);3]=[
        ("log1p_port", |r| rx::excel_log1p(r)),
        ("fyl2xp1",    |r| rx::ext_to_f64(&rx::ext_fyl2xp1(&rx::ext_ln2(),&rx::ext_from_f64(r),CW),CW)),
        ("ln(1+r)",    |r| rx::excel_ln(1.0+r)),
    ];
    for (ln,lf) in logs{
        let mut ok=0; let mut miss=Vec::new();
        for (r,n,em) in &rows{
            let e=em_model(lf(*r),*r,*n);
            if e.to_bits()==em.to_bits(){ok+=1;} else { miss.push((*r,*n, e.to_bits() as i64 - em.to_bits() as i64)); }
        }
        print!("  {:12} {:2}/46", ln, ok);
        if !miss.is_empty(){ print!("  miss:"); for (r,n,d) in &miss{ print!(" (2^{},{}):{:+}",(r.log2().round()) as i32,n,d);} }
        println!();
    }
    // also compare the three log1p f64 values per po2 r
    println!("\n per-r log1p f64 bit-compare (port vs fyl2xp1 vs ln(1+r)):");
    let mut seen=std::collections::BTreeSet::new();
    for (r,_,_) in &rows{
        let k=(r.log2().round()) as i32;
        if !seen.insert(k){continue;}
        let a=rx::excel_log1p(*r);
        let b=rx::ext_to_f64(&rx::ext_fyl2xp1(&rx::ext_ln2(),&rx::ext_from_f64(*r),CW),CW);
        let c=rx::excel_ln(1.0+*r);
        println!("  2^{:<3}: port=0x{:016x} fyl2xp1=0x{:016x} ln(1+r)=0x{:016x}  d(ln(1+r)-fyl2xp1)={:+}",
            k, a.to_bits(), b.to_bits(), c.to_bits(), c.to_bits() as i64 - b.to_bits() as i64);
    }
}
