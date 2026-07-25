//! dump exact intermediates for the clean oracle (46 rows) for Python analysis.
use oxfunc_core::excel_numeric::research as rx;
const CW: u16 = rx::CW_PC64_RN;
fn fb(h:&str)->f64{ f64::from_bits(u64::from_str_radix(h.trim_start_matches("0x"),16).unwrap()) }
fn hx(x:f64)->String{ format!("0x{:016x}", x.to_bits()) }
fn ehex(e:&rx::Ext80)->String{ let mut s=String::from("0x"); for b in e.0.iter().rev(){ s.push_str(&format!("{:02x}",b)); } s }
fn load(path:&str)->Vec<(f64,i64,f64)>{
    let v:serde_json::Value=serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let mut o=Vec::new();
    for (k,val) in v.as_object().unwrap(){ let (rh,nh)=k.split_once('|').unwrap(); o.push((fb(rh),nh.parse().unwrap(),fb(val.as_str().unwrap()))); }
    o.sort_by(|a,b|(a.0,a.1).partial_cmp(&(b.0,b.1)).unwrap()); o
}
fn main(){
    let rows=load("../../work/w109/G6-solvers/pmt_em_hdf_oracle.json");
    println!("r,n,em_excel,tau,u,lnu,logp_ext,u_ext");
    for (r,n,em) in &rows{
        let logp_ext=rx::ext_fyl2xp1(&rx::ext_ln2(),&rx::ext_from_f64(*r),CW);
        let t=-(*n as f64)*rx::excel_log1p(*r);
        let u=rx::excel_exp(t);
        let lnu=rx::excel_ln(u);
        // u_ext: exp chain kept extended
        let te=rx::ext_from_f64(t);
        let z=rx::ext_mul(&te,&rx::ext_l2e(),CW); let k=rx::ext_rndint(&z,CW); let f=rx::ext_sub(&z,&k,CW);
        let neg=rx::ext_to_f64(&f,CW)<0.0; let w=rx::ext_f2xm1(&rx::ext_abs(&f,CW),CW);
        let mut m=rx::ext_add(&w,&rx::ext_one(),CW); if neg{ m=rx::ext_div(&rx::ext_one(),&m,CW);} 
        let ue=rx::ext_scale(&m,&k,CW);
        println!("{},{},{},{},{},{},{},{}", hx(*r),n,hx(*em),hx(t),hx(u),hx(lnu),ehex(&logp_ext),ehex(&ue));
    }
}
