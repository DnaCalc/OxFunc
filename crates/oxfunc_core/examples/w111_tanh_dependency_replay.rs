//! Compare public SINH/COSH dependency observations and the TANH denominator.
#[cfg(not(all(feature = "research-x87", target_arch = "x86_64")))]
fn main() { panic!("research-x87 on x86_64 required"); }

#[cfg(all(feature = "research-x87", target_arch = "x86_64"))]
fn main() {
    use oxfunc_core::excel_numeric::research::*;
    let cw = CW_PC64_RN;
    let add = |a, b| ext_to_f64(&ext_add(&ext_from_f64(a), &ext_from_f64(b), cw), cw);
    let mut reports = Vec::new();
    for path in std::env::args().skip(1) {
        let data: serde_json::Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        let mut rows = Vec::new();
        for row in data["witnesses"].as_array().unwrap() {
            let x = f64::from_bits(u64::from_str_radix(row["args"][0].as_str().unwrap().trim_start_matches("0x"),16).unwrap());
            let local = match data["function"].as_str().unwrap() {
                "SINH" => oxfunc_core::functions::sinh::sinh_kernel(x),
                "COSH" => oxfunc_core::functions::cosh::cosh_kernel(x),
                _ => panic!("only the named dependencies are admitted"),
            };
            let denominator = if x.abs() < 1. {
                add(add(excel_expm1_internal(x), excel_expm1_internal(-x)), 2.) / 2.
            } else { oxfunc_core::functions::cosh::cosh_kernel(x) };
            rows.push(serde_json::json!({"row":row,"actual":format!("0x{:016x}",local.to_bits()),
                "tanh_denominator":format!("0x{:016x}",denominator.to_bits())}));
        }
        let matches = rows.iter().filter(|r| r["actual"] == r["row"]["expected_bits"]).count();
        reports.push(serde_json::json!({"source":path,"function":data["function"],"rows":rows.len(),"matches":matches,"details":rows}));
    }
    println!("{}", serde_json::to_string(&reports).unwrap());
}
