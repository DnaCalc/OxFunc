//! agent-P: definitive summary vs GOLD pv=1. Full model:
//!   tau = -n*log1p_port(r)  [f64, stored]
//!   em  = Kahan (u-1)*tau/ln(u) for |tau|<1 ; exp(tau)-1 for |tau|>=1
//! Report residual hist + within-+-1 fraction, split by |tau| regime, and
//! confirm the >+-2 tail is exactly the hypersensitive large-em lane.
use oxfunc_core::excel_numeric::research as rx;
use std::collections::BTreeMap;

fn fb(h: &str) -> f64 {
    f64::from_bits(u64::from_str_radix(h.trim_start_matches("0x"), 16).unwrap())
}
fn load(path: &str) -> Vec<(f64, i64, f64)> {
    let v: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(path).expect("read")).expect("json");
    let mut out = Vec::new();
    for (k, val) in v.as_object().unwrap() {
        if val.is_null() {
            continue;
        }
        let (rh, nh) = k.split_once('|').unwrap();
        out.push((fb(rh), nh.parse().unwrap(), fb(val.as_str().unwrap())));
    }
    out
}
fn em_model(r: f64, n: f64) -> f64 {
    let t = -n * rx::excel_log1p(r);
    let u = rx::excel_exp(t);
    if u == 1.0 {
        t
    } else if t.abs() < 1.0 {
        (u - 1.0) * t / rx::excel_ln(u)
    } else {
        u - 1.0
    }
}
fn report(name: &str, res: &[i64]) {
    let mut m: BTreeMap<i64, u32> = BTreeMap::new();
    for &x in res {
        *m.entry(x).or_default() += 1;
    }
    let n = res.len() as f64;
    let ex = *m.get(&0).unwrap_or(&0);
    let w1: u32 = res.iter().filter(|&&d| d.abs() <= 1).count() as u32;
    let w2: u32 = res.iter().filter(|&&d| d.abs() <= 2).count() as u32;
    println!(
        "  {:24} n={:4}  exact {:5.1}%  within+-1 {:5.1}%  within+-2 {:5.1}%",
        name, res.len(), 100.0 * ex as f64 / n, 100.0 * w1 as f64 / n, 100.0 * w2 as f64 / n
    );
    print!("        hist ");
    for (k, c) in &m {
        if k.abs() <= 3 {
            print!("{}:{} ", k, c);
        }
    }
    let big: u32 = res.iter().filter(|&&d| d.abs() > 3).count() as u32;
    if big > 0 {
        print!("|>3|:{}", big);
    }
    println!();
}

fn main() {
    let rows = load("../../work/w109/G6-solvers/pmt_em_gold_pv1.json");
    let mut all = Vec::new();
    let mut robust = Vec::new(); // tau<-2 (em near -1)
    let mut small = Vec::new(); // |tau|<=2
    let mut sens = Vec::new(); // tau>2 (em large)
    for (r, n, em_x) in &rows {
        let tau = -(*n as f64) * rx::excel_log1p(*r);
        let d = em_model(*r, *n as f64).to_bits() as i64 - em_x.to_bits() as i64;
        all.push(d);
        if tau < -2.0 {
            robust.push(d);
        } else if tau > 2.0 {
            sens.push(d);
        } else {
            small.push(d);
        }
    }
    println!("=== FULL model vs GOLD pv=1 (all {} rows) ===", rows.len());
    report("ALL", &all);
    println!("\n=== by |tau| regime ===");
    report("robust tau<-2 (em~-1)", &robust);
    report("small |tau|<=2 (Kahan)", &small);
    report("sensitive tau>2 (em big)", &sens);
}
