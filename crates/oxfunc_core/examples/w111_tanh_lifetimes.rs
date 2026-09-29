//! Read-only TANH composition research from retained public-interface captures.
#[cfg(not(all(feature = "research-x87", target_arch = "x86_64")))]
fn main() { panic!("research-x87 on x86_64 required"); }

#[cfg(all(feature = "research-x87", target_arch = "x86_64"))]
fn main() {
    use oxfunc_core::excel_numeric::research::*;
    use std::collections::BTreeMap;
    let cw = CW_PC64_RN;
    let stored = |v: &Ext80| ext_from_f64(ext_to_f64(v, cw));
    let mut reports = Vec::new();
    for path in std::env::args().skip(1) {
        let data: serde_json::Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        let name = data["function"].as_str().unwrap();
        let mut scores = BTreeMap::<String, (usize, usize)>::new();
        let mut failures = BTreeMap::<String, Vec<serde_json::Value>>::new();
        let mut admitted = 0;
        let mut small = 0;
        for row in data["witnesses"].as_array().unwrap() {
            let x = f64::from_bits(u64::from_str_radix(row["args"][0].as_str().unwrap().trim_start_matches("0x"), 16).unwrap());
            if x.to_bits() == 1u64 << 63 || (x != 0. && x.abs() < f64::MIN_POSITIVE) { continue; }
            admitted += 1;
            small += usize::from(x.abs() < 1.);
            let expected = row["expected_bits"].as_str().unwrap();
            let base = oxfunc_core::functions::tanh::tanh_kernel(x);
            let staged_base = ext_to_f64(&ext_div(
                &ext_from_f64(oxfunc_core::functions::sinh::sinh_kernel(x)),
                &ext_from_f64(oxfunc_core::functions::cosh::cosh_kernel(x)), cw), cw);
            let mut candidates = vec![("production".to_owned(), base), ("std".to_owned(), x.tanh())];
            if x.abs() < 1. {
                let ep = ext_from_f64(excel_exp(x));
                let en = ext_from_f64(excel_exp(-x));
                let mp = ext_from_f64(excel_expm1_internal(x));
                let mn = ext_from_f64(excel_expm1_internal(-x));
                let two = ext_from_f64(2.);
                let nums = [ext_sub(&mp, &mn, cw), ext_sub(&ep, &en, cw),
                    ext_from_f64(2. * x.sinh()), ext_from_f64(2. * oxfunc_core::functions::sinh::sinh_kernel(x))];
                let dens = [ext_add(&ep, &en, cw),
                    ext_add(&ext_add(&mp, &mn, cw), &two, cw),
                    ext_from_f64(2. * x.cosh()),
                    ext_add(&stored(&ext_add(&mp, &mn, cw)), &two, cw),
                    ext_from_f64(excel_expm1_internal(x) + excel_expm1_internal(-x) + 2.)];
                for (ni, n) in nums.iter().enumerate() {
                    for (di, d) in dens.iter().enumerate() {
                        for ns in 0..2 { for ds in 0..2 {
                            let n = if ns == 1 { stored(n) } else { *n };
                            let d = if ds == 1 { stored(d) } else { *d };
                            candidates.push((format!("n{ni}d{di}-store{ns}{ds}"), ext_to_f64(&ext_div(&n, &d, cw), cw)));
                        }}
                    }
                }
            } else {
                for ni in 0..4 { for di in 0..5 { for ns in 0..2 { for ds in 0..2 {
                    candidates.push((format!("n{ni}d{di}-store{ns}{ds}"), staged_base));
                }}}}
            }
            for (key, mut v) in candidates {
                let zero_coth = name == "COTH" && v == 0.;
                if name == "COTH" && !zero_coth {
                    v = ext_to_f64(&ext_div(&ext_one(), &ext_from_f64(v), cw), cw);
                }
                if !v.is_finite() { v = x.signum(); }
                if x == 0. { v = 0.; }
                let actual = if zero_coth { "error:Div0".to_owned() } else { format!("0x{:016x}", v.to_bits()) };
                let matched = actual == expected;
                let score = scores.entry(key.clone()).or_default();
                score.0 += usize::from(matched);
                score.1 += usize::from(matched && x.abs() < 1.);
                if !matched { failures.entry(key).or_default().push(serde_json::json!({"row":row,"actual":actual})); }
            }
        }
        let best = scores.iter().max_by_key(|(_, score)| score.0).unwrap().0;
        reports.push(serde_json::json!({"source":path,"function":name,"rows":admitted,"small_rows":small,"scores":scores,
            "best":best,"best_misses":failures.get(best),"production_misses":failures.get("production")}));
    }
    println!("{}", serde_json::to_string(&reports).unwrap());
}
