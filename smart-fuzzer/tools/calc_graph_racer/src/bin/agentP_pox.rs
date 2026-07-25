use oxfunc_core::excel_numeric::research as rx;
fn fb(h:&str)->f64{ f64::from_bits(u64::from_str_radix(h.trim_start_matches("0x"),16).unwrap()) }
fn hx(x:f64)->String{ format!("0x{:016x}", x.to_bits()) }
fn load(path:&str)->Vec<(f64,i64,f64)>{
    let v:serde_json::Value=serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let mut o=Vec::new();
    for (k,val) in v.as_object().unwrap(){ let (rh,nh)=k.split_once('|').unwrap(); o.push((fb(rh),nh.parse().unwrap(),fb(val.as_str().unwrap()))); }
    o.sort_by(|a,b|(a.0,a.1).partial_cmp(&(b.0,b.1)).unwrap()); o
}
fn main(){
    let rows=load("../../work/w109/G6-solvers/agentP_pox_em.json");
    println!("r,n,em_excel,tau,u,lnu,kahan,uminus1");
    for (r,n,em) in &rows{
        let t=-(*n as f64)*rx::excel_log1p(*r);
        let u=rx::excel_exp(t); let lnu=rx::excel_ln(u);
        let kah= if u==1.0 {t} else if t.abs()<1.0 {(u-1.0)*t/lnu} else {u-1.0};
        println!("{},{},{},{},{},{},{},{}", hx(*r),n,hx(*em),hx(t),hx(u),hx(lnu),hx(kah),hx(u-1.0));
    }
}
