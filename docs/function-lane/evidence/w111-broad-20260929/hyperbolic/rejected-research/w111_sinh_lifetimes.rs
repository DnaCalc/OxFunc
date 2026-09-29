//! Offline arithmetic-lifetime hypotheses against public Excel observations.
#[cfg(not(all(feature = "research-x87", target_arch = "x86_64")))]
fn main() { eprintln!("Requires research-x87 on x86_64"); }

#[cfg(all(feature = "research-x87", target_arch = "x86_64"))]
fn main() {
    use oxfunc_core::excel_numeric::research::*;
    use oxfunc_core::functions::sinh::sinh_kernel;
    let cw = CW_PC64_RN;
    let store = |v: &Ext80| ext_from_f64(ext_to_f64(v, cw));
    // Product and quotient are varied independently. Earlier graphs changed
    // both at once, which cannot identify a mixed SSE/x87 lifetime.
    let em = |t: f64, mode: usize| -> Ext80 {
        let u = excel_exp(t);
        if u == 1.0 { return ext_from_f64(t); }
        let delta = ext_sub(&ext_from_f64(u), &ext_one(), cw);
        let log = if mode & 4 == 0 { ext_from_f64(excel_ln(u)) }
            else { ext_fyl2x(&ext_ln2(), &ext_from_f64(u), cw) };
        let product = if mode & 2 == 0 { ext_from_f64((u-1.0)*t) }
            else { store(&ext_mul(&delta, &ext_from_f64(t), cw)) };
        let quotient = if mode & 1 == 0 {
            ext_from_f64(ext_to_f64(&product,cw)/ext_to_f64(&log,cw))
        } else { store(&ext_div(&product,&log,cw)) };
        quotient
    };
    let mut output = Vec::new();
    for path in std::env::args().skip(1) {
        let bank: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(bank["function"], "SINH");
        let mut matches = vec![0usize; 128];
        let mut misses = vec![Vec::new(); 128];
        let mut baseline_misses = Vec::new();
        let mut rows = 0usize;
        for row in bank["witnesses"].as_array().unwrap() {
            let x = f64::from_bits(u64::from_str_radix(row["args"][0].as_str().unwrap().trim_start_matches("0x"), 16).unwrap());
            if x == 0.0 || x.abs() >= 1.0 || x.abs() < f64::MIN_POSITIVE { continue; }
            rows += 1;
            let expected = row["expected_bits"].as_str().unwrap();
            let baseline = format!("0x{:016x}", sinh_kernel(x).to_bits());
            if baseline != expected { baseline_misses.push(serde_json::json!({"row":row,"actual":baseline})); }
            let positive: Vec<_> = (0..8).map(|m| em(x,m)).collect();
            let negative: Vec<_> = (0..8).map(|m| em(-x,m)).collect();
            for a in 0..8 { for b in 0..8 { for staged_sub in 0..2 {
                let index = (a*8+b)*2+staged_sub;
                let result = if staged_sub == 1 { ext_to_f64(&ext_sub(&positive[a], &negative[b], cw), cw)/2.0 }
                    else { (ext_to_f64(&positive[a],cw)-ext_to_f64(&negative[b],cw))/2.0 };
                let actual = format!("0x{:016x}", result.to_bits());
                if actual == expected { matches[index] += 1; }
                else if misses[index].len() < 30 { misses[index].push(serde_json::json!({"row":row,"actual":actual})); }
            }}}
        }
        let mut order: Vec<_> = (0..128).collect();
        order.sort_by_key(|&i| std::cmp::Reverse(matches[i]));
        let ranked: Vec<_> = order.iter().take(16).map(|&i| serde_json::json!({"positive_mode":i/16,"negative_mode":(i/2)%8,"staged_sub":i%2,"matches":matches[i],"misses":misses[i]})).collect();
        output.push(serde_json::json!({"source":path,"rows":rows,"baseline_matches":rows-baseline_misses.len(),"baseline_misses":baseline_misses,"ranked":ranked,"all_counts":matches}));
    }
    println!("{}", serde_json::to_string(&output).unwrap());
}
