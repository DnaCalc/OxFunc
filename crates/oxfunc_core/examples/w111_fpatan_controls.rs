//! Own-code public-ISA experiment; no Excel binary or runtime inspection.
//! Compare control-word arithmetic hypotheses against retained black-box outputs.
use core::arch::asm;
#[cfg(target_arch="x86_64")]
fn fpatan_staged(x:f64, control:u16, store:u16)->f64 {
    let mut saved=0u16;let mut value=0.0;
    // SAFETY: balanced x87 pushes/pops, valid locals and restored caller control.
    unsafe {asm!("fnstcw word ptr [{saved}]", "fldcw word ptr [{control}]",
        "fld qword ptr [{x}]", "fld1", "fpatan", "fldcw word ptr [{store}]",
        "fstp qword ptr [{value}]", "fldcw word ptr [{saved}]",
        saved=in(reg)&mut saved,control=in(reg)&control,store=in(reg)&store,
        x=in(reg)&x,value=in(reg)&mut value,options(nostack,preserves_flags));}
    value
}
fn main(){
    let modes:Vec<_>=[0x003fu16,0x023f,0x033f].into_iter().flat_map(|pc|
        [0x0000u16,0x0400,0x0800,0x0c00].into_iter().map(move|rc|pc|rc)).collect();
    let mut exact=vec![0usize;modes.len()];let mut total=0;let mut failures=Vec::new();
    for path in std::env::args().skip(1){
        let bank:serde_json::Value=serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
        for row in bank["witnesses"].as_array().unwrap(){
            let x=f64::from_bits(u64::from_str_radix(row["args"][0].as_str().unwrap().trim_start_matches("0x"),16).unwrap());
            let expected=u64::from_str_radix(row["expected_bits"].as_str().unwrap().trim_start_matches("0x"),16).unwrap();
            let results:Vec<_>=modes.iter().map(|cw|fpatan_staged(x,*cw,0x033f).to_bits()).collect();
            for(i,b)in results.iter().enumerate(){exact[i]+=usize::from(*b==expected);}
            if results[8]!=expected {failures.push(serde_json::json!({"id":row["id"],"args":row["args"],"expected_bits":row["expected_bits"],"modes":results.iter().map(|b|format!("0x{b:016x}")).collect::<Vec<_>>()}));}
            total+=1;
        }
    }
    println!("{}",serde_json::to_string_pretty(&serde_json::json!({"rows":total,"control_words":modes.iter().map(|cw|format!("0x{cw:04x}")).collect::<Vec<_>>(),"exact":exact,"default_failures":failures})).unwrap());
}
