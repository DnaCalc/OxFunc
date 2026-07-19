//! agentV G6-03d gate: run the PRODUCTION price_kernel against a captured
//! answers-*.json (args = 7 hex f64, expected_bits = Excel result bits).
//! Usage: agentV_gate_price <answers.json> [<answers2.json> ...]
use oxfunc_core::functions::bond_core_family::price_kernel;
use std::collections::BTreeMap;

fn h2f(s: &str) -> f64 {
    f64::from_bits(u64::from_str_radix(s.trim_start_matches("0x"), 16).unwrap())
}

fn main() {
    for path in std::env::args().skip(1) {
        let txt = std::fs::read_to_string(&path).unwrap();
        let v: serde_json::Value = serde_json::from_str(&txt).unwrap();
        let ws = v["witnesses"].as_array().unwrap();
        let mut exact: BTreeMap<i64, usize> = BTreeMap::new();
        let mut total: BTreeMap<i64, usize> = BTreeMap::new();
        let mut errrows = 0usize;
        let mut misses: Vec<(String, i64, i64)> = Vec::new();
        for w in ws {
            let eb = w["expected_bits"].as_str();
            let eb = match eb {
                Some(s) if s.starts_with("0x") => s,
                _ => {
                    errrows += 1;
                    continue;
                }
            };
            let a: Vec<f64> = w["args"]
                .as_array()
                .unwrap()
                .iter()
                .map(|x| h2f(x.as_str().unwrap()))
                .collect();
            let basis = a[6] as i64;
            let exp_bits = u64::from_str_radix(eb.trim_start_matches("0x"), 16).unwrap();
            let got = price_kernel(a[0], a[1], a[2], a[3], a[4], a[5], Some(a[6]));
            *total.entry(basis).or_default() += 1;
            match got {
                Ok(val) => {
                    let gb = val.to_bits();
                    if gb == exp_bits {
                        *exact.entry(basis).or_default() += 1;
                    } else {
                        let d = gb as i128 - exp_bits as i128;
                        if misses.len() < 40 {
                            misses.push((
                                w["id"].as_str().unwrap_or("?").to_string(),
                                basis,
                                d as i64,
                            ));
                        }
                    }
                }
                Err(_) => {
                    if misses.len() < 40 {
                        misses.push((w["id"].as_str().unwrap_or("?").to_string(), basis, i64::MAX));
                    }
                }
            }
        }
        println!("== {path}  (err rows {errrows})");
        let mut te = 0usize;
        let mut tt = 0usize;
        for (b, t) in &total {
            let e = *exact.get(b).unwrap_or(&0);
            te += e;
            tt += t;
            println!("   b{b} exact {e:5} / {t:5}");
        }
        println!("   ALL exact {te:5} / {tt:5}");
        for (id, b, d) in misses.iter().take(40) {
            println!("     miss {id} b{b} d={d}");
        }
    }
}
