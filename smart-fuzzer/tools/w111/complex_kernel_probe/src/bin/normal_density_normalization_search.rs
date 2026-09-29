//! Select fresh numeric inputs where public RN64-vs-RN53 operation graphs differ.
//! This search never consults Excel outcomes.
use oxfunc_core::excel_numeric::research::{ext_from_f64 as from,ext_to_f64 as to,ext_sub,ext_div,ext_mul,excel_exp,CW_PC64_RN as CW};
use serde_json::json;
fn op(a:f64,b:f64,kind:char,staged:bool)->f64 {
    if staged {to(&match kind {'-'=>ext_sub(&from(a),&from(b),CW),'/'=>ext_div(&from(a),&from(b),CW),_=>ext_mul(&from(a),&from(b),CW)},CW)}
    else {if kind=='-' {a-b} else {a/b}}
}
fn main(){
    let initial_seed=std::env::args().nth(1).map(|s|s.parse::<u64>().unwrap()).unwrap_or(2026092927);
    let mut seed=initial_seed;let mut selected=Vec::new();let mut draws=0;
    let next=|state:&mut u64|{*state^=*state<<13;*state^=*state>>7;*state^=*state<<17;*state};
    while draws<1_000_000 && selected.len()<128 {
        draws+=1;
        let x=f64::from_bits(((1023+next(&mut seed)%3)<<52)|(next(&mut seed)&((1<<52)-1))|(next(&mut seed)&(1<<63)));
        let mean=f64::from_bits(((1023+next(&mut seed)%3)<<52)|(next(&mut seed)&((1<<52)-1))|(next(&mut seed)&(1<<63)));
        let sigma=f64::from_bits((1023<<52)|(next(&mut seed)&((1<<52)-1)));
        let outputs:Vec<_>=(0..4).map(|mode|{
            let z=op(op(x,mean,'-',mode&1!=0),sigma,'/',mode&2!=0);
            let square=op(z,z,'*',true);let ex=excel_exp(-square/2.0);
            let result=op(op(ex,sigma,'/',true),f64::from_bits(0x3fd9884533d43651),'*',true);
            format!("0x{:016x}",result.to_bits())
        }).collect();
        if outputs.iter().any(|v|v!=&outputs[0]) {
            selected.push(json!({"id":format!("normal-density-normalization-{:04}",selected.len()),
                "args":[format!("0x{:016x}",x.to_bits()),format!("0x{:016x}",mean.to_bits()),format!("0x{:016x}",sigma.to_bits()),"0x0000000000000000"],"outputs":outputs}));
        }
    }
    println!("{}",json!({"seed":initial_seed,"draws":draws,"mode_order":["native/native","staged/native","native/staged","staged/staged"],"selected":selected}));
}
