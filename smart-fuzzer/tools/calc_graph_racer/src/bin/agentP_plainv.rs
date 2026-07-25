//! Test em = exp(tau)-1 (plain v-1, NO Kahan) vs Kahan, on clean 46-oracle.
//! Also a hybrid: plain v-1 when v-1 is exact (Sterbenz, |tau| moderate), Kahan for tiny tau.
use oxfunc_core::excel_numeric::research as rx;
const CW: u16 = rx::CW_PC64_RN;
fn fb(h:&str)->f64{ f64::from_bits(u64::from_str_radix(h.trim_start_matches("0x"),16).unwrap()) }
fn e(x:f64)->rx::Ext80{ rx::ext_from_f64(x) }
fn tf(x:&rx::Ext80)->f64{ rx::ext_to_f64(x,CW) }
fn load(path:&str)->Vec<(f64,i64,f64)>{
    let v:serde_json::Value=serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let mut o=Vec::new();
    for (k,val) in v.as_object().unwrap(){ let (rh,nh)=k.split_once('|').unwrap(); o.push((fb(rh),nh.parse().unwrap(),fb(val.as_str().unwrap()))); }
    o.sort_by(|a,b|(a.0,a.1).partial_cmp(&(b.0,b.1)).unwrap()); o
}
fn exp_ext(t:f64)->rx::Ext80{
    let te=e(t); let z=rx::ext_mul(&te,&rx::ext_l2e(),CW); let k=rx::ext_rndint(&z,CW); let f=rx::ext_sub(&z,&k,CW);
    let neg=tf(&f)<0.0; let w=rx::ext_f2xm1(&rx::ext_abs(&f,CW),CW); let mut m=rx::ext_add(&w,&rx::ext_one(),CW); if neg{m=rx::ext_div(&rx::ext_one(),&m,CW);} 
    rx::ext_scale(&m,&k,CW)
}
fn main(){
    let rows=load("../../work/w109/G6-solvers/pmt_em_hdf_oracle.json");
    let variants:Vec<(&str, Box<dyn Fn(f64)->f64>)>=vec![
        ("plain u-1 (f64 exp)", Box::new(|t:f64| rx::excel_exp(t)-1.0)),
        ("plain RN53(u_ext-1)", Box::new(|t:f64| tf(&rx::ext_sub(&exp_ext(t),&rx::ext_one(),CW)))),
        ("Kahan (ref)", Box::new(|t:f64|{ let u=rx::excel_exp(t); if u==1.0 {t} else if t.abs()<1.0 {(u-1.0)*t/rx::excel_ln(u)} else {u-1.0} })),
    ];
    let miss=[(-5,2),(-5,3),(-5,5),(-5,12),(-5,24),(-4,2)];
    for (name,f) in &variants{
        let mut ok=0; let mut d6=Vec::new(); let mut allmiss=Vec::new();
        for (r,n,em) in &rows{
            let t=-(*n as f64)*rx::excel_log1p(*r);
            let d=f(t).to_bits() as i64 - em.to_bits() as i64;
            if d==0 {ok+=1;} else { allmiss.push((( r.log2().round()) as i32,*n,d)); }
            let k=(r.log2().round()) as i32;
            if miss.contains(&(k,*n)){ d6.push(((k,*n),d)); }
        }
        print!("  {:22} {:2}/46  on-6:", name, ok);
        for ((k,n),d) in &d6{ print!(" (2^{},{}):{:+}",k,n,d); }
        println!();
        if *name!="Kahan (ref)" && ok>=38 { print!("      all misses:"); for (k,n,d) in &allmiss{ print!(" (2^{},{}):{:+}",k,n,d);} println!(); }
    }

    // HYBRID: if |v-1| is Sterbenz-exact (u in [0.5,2)) use plain u-1 else Kahan.
    // Actually test: plain u-1 for u>=0.5, Kahan for u<0.5 (small tau) -- and vice versa.
    for cut in ["u>=0.5 plain","tau: plain if |t|<thr not tiny"]{
        let _=cut;
    }
    // Sweep: plain u-1 for |t| in [lo,1), Kahan for |t|<lo
    println!("\n  hybrid: plain u-1 for |t|>=lo, Kahan for |t|<lo:");
    for lo in [0.02f64,0.03,0.05,0.06,0.1]{
        let mut ok=0;
        for (r,n,em) in &rows{
            let t=-(*n as f64)*rx::excel_log1p(*r);
            let u=rx::excel_exp(t);
            let val= if u==1.0 {t} else if t.abs()>=1.0 {u-1.0}
                     else if t.abs()>=lo { u-1.0 }  // plain
                     else { (u-1.0)*t/rx::excel_ln(u) }; // kahan tiny
            if val.to_bits()==em.to_bits(){ok+=1;}
        }
        println!("    lo={:.2}: {}/46", lo, ok);
    }
}
