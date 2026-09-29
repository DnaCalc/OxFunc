//! General ACOT arithmetic graphs, compared only with public-interface captures.
use serde_json::{Value,json};
use std::f64::consts::{PI,FRAC_PI_2};

fn fpatan2(y:f64,x:f64)->f64 {
    let mut old=0_u16;let control=0x133f_u16;let mut out=0.0_f64;
    unsafe {std::arch::asm!(
        "fnstcw word ptr [{old}]", "fldcw word ptr [{control}]",
        "fld qword ptr [{y}]", "fld qword ptr [{x}]", "fpatan", "fstp qword ptr [{out}]",
        "fldcw word ptr [{old}]",
        old=in(reg)&mut old,control=in(reg)&control,y=in(reg)&y,x=in(reg)&x,out=in(reg)&mut out,
        options(nostack,preserves_flags));}
    out
}
fn extended_recip(x:f64)->f64 {
    use oxfunc_core::excel_numeric::research::{ext_from_f64 as from,ext_to_f64 as to,ext_div as div,CW_PC64_RN as cw};
    to(&div(&from(1.0),&from(x),cw),cw)
}
fn main() {
    let path=std::env::args().nth(1).unwrap();
    let capture:Value=serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    for mode in ["current-public-kernel","std-atan2","fpatan2","std-recip-angle","fpatan-recip-angle","fpatan-extended-recip-angle","half-pi-minus-fpatan"] {
        let mut misses=Vec::new();let mut rows=0;let mut excluded=0;
        for row in capture["witnesses"].as_array().unwrap() {
            let x=f64::from_bits(u64::from_str_radix(row["args"][0].as_str().unwrap().trim_start_matches("0x"),16).unwrap());
            if x.is_subnormal(){excluded+=1;continue;}
            let value=match mode {
                "current-public-kernel"=>oxfunc_core::functions::acot::acot_kernel(x).unwrap(),
                "std-atan2"=>1.0_f64.atan2(x),
                "fpatan2"=>fpatan2(1.0,x),
                "half-pi-minus-fpatan"=>FRAC_PI_2-fpatan2(x,1.0),
                _=>{let reciprocal=if mode=="fpatan-extended-recip-angle" {extended_recip(x)} else {1.0/x};
                    let angle=if mode=="std-recip-angle" {reciprocal.atan()} else {fpatan2(reciprocal,1.0)};
                    if x<0.0 {angle+PI} else if x==0.0 {FRAC_PI_2} else {angle}}
            };
            let value=if value.abs()<f64::MIN_POSITIVE {0.0} else {value};
            let actual=format!("0x{:016x}",value.to_bits());rows+=1;
            if actual!=row["expected_bits"].as_str().unwrap(){misses.push(json!({"id":row["id"],"args":row["args"],"expected":row["expected_bits"],"actual":actual}));}
        }
        println!("{}",json!({"candidate":mode,"rows":rows,"matches":rows-misses.len(),"input_subnormals_excluded":excluded,"misses":misses}));
    }
}
