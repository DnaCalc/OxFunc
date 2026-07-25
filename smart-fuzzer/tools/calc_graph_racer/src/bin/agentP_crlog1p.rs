//! agent-P: test CR log1p (from agentP_crlog1p.json) vs portable log1p through
//! the x87 exp chain on the sensitive lane, and invert for the tau-ulp offset
//! Excel actually used. Pins whether the tau tail is a log1p or product issue.
use oxfunc_core::excel_numeric::research as rx;
use std::collections::{BTreeMap, HashMap};

fn fb(h: &str) -> f64 {
    f64::from_bits(u64::from_str_radix(h.trim_start_matches("0x"), 16).unwrap())
}
fn na(x: f64, d: i64) -> f64 {
    f64::from_bits((x.to_bits() as i64 + d) as u64)
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
fn load_map(path: &str) -> HashMap<u64, f64> {
    let v: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(path).expect("read")).expect("json");
    let mut m = HashMap::new();
    for (k, val) in v.as_object().unwrap() {
        m.insert(fb(k).to_bits(), fb(val.as_str().unwrap()));
    }
    m
}
fn hist(name: &str, v: &[i64]) {
    let mut m: BTreeMap<i64, u32> = BTreeMap::new();
    for &x in v {
        *m.entry(x).or_default() += 1;
    }
    let n = v.len();
    let exact = *m.get(&0).unwrap_or(&0);
    print!("  {:34} {:4}/{:4} ({:5.1}%)  ", name, exact, n, 100.0 * exact as f64 / n as f64);
    let mut big = 0;
    for (k, c) in &m {
        if k.abs() <= 2 {
            print!("{}:{} ", k, c);
        } else {
            big += c;
        }
    }
    if big > 0 {
        print!("|>2|:{}", big);
    }
    println!();
}

fn main() {
    let rows = load("../../work/w109/G6-solvers/pmt_em_gold_pv1.json");
    let cr = load_map("../../work/w109/G6-solvers/agentP_crlog1p.json");
    let sens: Vec<_> = rows
        .iter()
        .filter(|(r, n, _)| -(*n as f64) * rx::excel_log1p(*r) > 2.0)
        .cloned()
        .collect();
    println!("sensitive rows: {}", sens.len());

    let mut r_port = Vec::new();
    let mut r_cr = Vec::new();
    // also invert: tau offset (from portable tau) reproducing em_gold
    let mut off = BTreeMap::<i64, u32>::new();
    let mut log1p_differs = 0u32;
    for (r, n, em_x) in &sens {
        let lp = rx::excel_log1p(*r);
        let lcr = cr[&r.to_bits()];
        if lp.to_bits() != lcr.to_bits() {
            log1p_differs += 1;
        }
        let tau_p = -(*n as f64) * lp;
        let tau_c = -(*n as f64) * lcr;
        r_port.push((rx::excel_exp(tau_p) - 1.0).to_bits() as i64 - em_x.to_bits() as i64);
        r_cr.push((rx::excel_exp(tau_c) - 1.0).to_bits() as i64 - em_x.to_bits() as i64);
        // invert tau
        let mut found = None;
        for d in -12..=12 {
            let tau = na(tau_p, d);
            if (rx::excel_exp(tau) - 1.0).to_bits() == em_x.to_bits() {
                found = Some(d);
                break;
            }
        }
        *off.entry(found.unwrap_or(999)).or_default() += 1;
    }
    hist("tau=-n*log1p_port", &r_port);
    hist("tau=-n*log1p_CR", &r_cr);
    println!("portable log1p != CR log1p on sensitive: {}/{}", log1p_differs, sens.len());
    println!("tau-ulp offset (from portable tau) reproducing em_gold (999=none in +-12):");
    for (k, c) in &off {
        println!("    d={:+}: {}", k, c);
    }
}
