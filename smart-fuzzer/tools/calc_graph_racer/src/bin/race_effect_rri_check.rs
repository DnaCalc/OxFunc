//! W109 observability byproduct: does OxFunc's EFFECT / RRI match the fresh live-Excel grids?
use oxfunc_core::functions::financial_time_value_family as fin;
use serde_json::Value;

fn bits(s: &str) -> f64 { f64::from_bits(u64::from_str_radix(s.trim_start_matches("0x"), 16).unwrap()) }

fn load(batch: &str, ans: &str) -> Vec<(Vec<f64>, u64)> {
    let b: Value = serde_json::from_str(&std::fs::read_to_string(batch).unwrap()).unwrap();
    let a: Value = serde_json::from_str(&std::fs::read_to_string(ans).unwrap()).unwrap();
    let probes = b["probes"].as_array().unwrap();
    let wits = a["witnesses"].as_array().unwrap();
    probes.iter().zip(wits).map(|(p, w)| {
        let args: Vec<f64> = p["probe"]["args"].as_array().unwrap().iter()
            .map(|x| bits(x.as_str().unwrap())).collect();
        let exp = bits(w["expected_bits"].as_str().unwrap()).to_bits();
        (args, exp)
    }).collect()
}

fn main() {
    let root = "../../work/w109/G6-solvers";
    // EFFECT(nominal, npery)
    let eff = load(&format!("{root}/batch-effect-grid.json"), &format!("{root}/answers-effect-grid.json"));
    let (mut ok, mut tot) = (0u32, 0u32);
    let mut miss: Vec<(f64,f64,i64)> = Vec::new();
    for (args, exp) in &eff {
        if let Ok(v) = fin::effect(args[0], args[1]) {
            tot += 1;
            if v.to_bits() == *exp { ok += 1; } else { miss.push((args[0], args[1], v.to_bits() as i64 - *exp as i64)); }
        }
    }
    println!("OxFunc EFFECT vs Excel grid: {}/{} exact; {} miss", ok, tot, miss.len());
    for m in miss.iter().take(12) { println!("   nominal={:.8} npery={} ulp={:+}", m.0, m.1, m.2); }

    // RRI(nper, pv, fv)
    let rri = load(&format!("{root}/batch-rri-grid.json"), &format!("{root}/answers-rri-grid.json"));
    let (mut ok2, mut tot2) = (0u32, 0u32);
    let mut miss2: Vec<(f64,f64,i64)> = Vec::new();
    for (args, exp) in &rri {
        if let Ok(v) = fin::rri(args[0], args[1], args[2]) {
            tot2 += 1;
            if v.to_bits() == *exp { ok2 += 1; } else { miss2.push((args[0], args[2], v.to_bits() as i64 - *exp as i64)); }
        }
    }
    println!("OxFunc RRI vs Excel grid: {}/{} exact; {} miss", ok2, tot2, miss2.len());
    for m in miss2.iter().take(12) { println!("   nper={} fv={:.10} ulp={:+}", m.0, m.1, m.2); }
}
