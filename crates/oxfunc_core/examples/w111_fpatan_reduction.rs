//! Own-code arithmetic hypotheses using only the public FPATAN instruction.
use core::arch::asm;
#[path="support/w111_fpatan.rs"] mod primitive;
fn inverse_reduction(x:f64,binary64_pi:bool,stored_angle:bool)->f64 {
    let mut saved=0u16;let cw=0x033fu16;let mut angle=0.0f64;let mut value=0.0f64;
    let half_pi=std::f64::consts::FRAC_PI_2;let two=2.0f64;
    unsafe {
        asm!("fnstcw word ptr [{saved}]","fldcw word ptr [{cw}]","fld1","fld qword ptr [{x}]","fpatan",
            saved=in(reg)&mut saved,cw=in(reg)&cw,x=in(reg)&x,options(nostack,preserves_flags));
        if stored_angle {asm!("fstp qword ptr [{angle}]","fld qword ptr [{angle}]",angle=in(reg)&mut angle,options(nostack,preserves_flags));}
        if binary64_pi {asm!("fld qword ptr [{pi}]",pi=in(reg)&half_pi,options(nostack,preserves_flags));}
        else {asm!("fldpi","fdiv qword ptr [{two}]",two=in(reg)&two,options(nostack,preserves_flags));}
        asm!("fsubrp st(1), st(0)","fstp qword ptr [{value}]","fldcw word ptr [{saved}]",
            value=in(reg)&mut value,saved=in(reg)&saved,options(nostack,preserves_flags));
    }
    value
}
fn main(){
    let mut total=0usize;let mut exact=[0usize;4];let mut failures=Vec::new();
    for path in std::env::args().skip(1){
        let bank:serde_json::Value=serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
        for row in bank["witnesses"].as_array().unwrap(){
            let x=f64::from_bits(u64::from_str_radix(row["args"][0].as_str().unwrap().trim_start_matches("0x"),16).unwrap());
            let expected=u64::from_str_radix(row["expected_bits"].as_str().unwrap().trim_start_matches("0x"),16).unwrap();
            let results:Vec<_>=[(false,false),(false,true),(true,false),(true,true)].iter().map(|(p,s)|
                if x.abs()>1.0 {inverse_reduction(x.abs(),*p,*s).copysign(x).to_bits()} else {primitive::direct(x).to_bits()}).collect();
            for(i,b)in results.iter().enumerate(){exact[i]+=usize::from(*b==expected);}
            if primitive::direct(x).to_bits()!=expected {failures.push(serde_json::json!({"id":row["id"],"args":row["args"],"expected":row["expected_bits"],"alternatives":results.iter().map(|b|format!("0x{b:016x}")).collect::<Vec<_>>()}));}
            total+=1;
        }
    }
    println!("{}",serde_json::to_string_pretty(&serde_json::json!({"rows":total,"exact":exact,"graphs":["extended_pi_extended_angle","extended_pi_stored_angle","f64_pi_extended_angle","f64_pi_stored_angle"],"default_failures":failures})).unwrap());
}
