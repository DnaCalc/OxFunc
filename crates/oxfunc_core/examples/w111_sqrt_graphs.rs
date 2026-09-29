//! Public FSQRT arithmetic candidates, compared only with black-box Value2 data.
#[cfg(all(feature = "research-x87", target_arch = "x86_64"))]
use oxfunc_core::excel_numeric::research::{CW_PC64_RN, ext_from_f64, ext_sqrt, ext_to_f64};
#[cfg(not(all(feature = "research-x87", target_arch = "x86_64")))]
fn main() { eprintln!("This research example requires research-x87 on x86_64."); }
#[cfg(all(feature = "research-x87", target_arch = "x86_64"))]
fn main() {
    let mut reports=Vec::new();
    for path in std::env::args().skip(1) {
        let bank:serde_json::Value=serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        let mut rows=0; let mut standard=0; let mut staged=0; let mut misses=Vec::new();
        for row in bank["witnesses"].as_array().unwrap() {
            let x=f64::from_bits(u64::from_str_radix(row["args"][0].as_str().unwrap().trim_start_matches("0x"),16).unwrap());
            let expected=row["expected_bits"].as_str().unwrap();
            let actual=|y:f64| if x<0.0 {"error:Num".to_owned()} else {format!("0x{:016x}",y.to_bits())};
            let a=actual(x.sqrt());
            let b=actual(ext_to_f64(&ext_sqrt(&ext_from_f64(x),CW_PC64_RN),CW_PC64_RN));
            rows+=1;standard+=usize::from(a==expected);staged+=usize::from(b==expected);
            if b!=expected { misses.push(serde_json::json!({"row":row,"staged":b,"standard":a})); }
        }
        reports.push(serde_json::json!({"path":path,"rows":rows,"standard_exact":standard,"staged_exact":staged,"staged_misses":misses}));
    }
    println!("{}",serde_json::to_string_pretty(&reports).unwrap());
}
