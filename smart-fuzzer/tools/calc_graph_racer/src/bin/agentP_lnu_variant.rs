//! agent-P: test the Kahan denominator variant log1p(u-1) vs log(u), and the
//! numerator using log1p on extended (u-1). Score vs pinned em for n=1..several.
use oxfunc_core::excel_numeric::research as rx;
use std::collections::BTreeMap;

const CW: u16 = rx::CW_PC64_RN;

fn fb(h: &str) -> f64 {
    f64::from_bits(u64::from_str_radix(h.trim_start_matches("0x"), 16).unwrap())
}
fn load_pinned(path: &str) -> Vec<(f64, i64, f64)> {
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
fn hist(name: &str, v: &[i64]) {
    let mut m: BTreeMap<i64, u32> = BTreeMap::new();
    for &x in v {
        *m.entry(x).or_default() += 1;
    }
    let n = v.len();
    let exact = *m.get(&0).unwrap_or(&0);
    print!(
        "  {:34} {:4}/{:4} ({:5.1}%)  ",
        name,
        exact,
        n,
        100.0 * exact as f64 / n as f64
    );
    let mut big = 0;
    for (k, c) in &m {
        if k.abs() <= 3 {
            print!("{}:{} ", k, c);
        } else {
            big += c;
        }
    }
    if big > 0 {
        print!("|>3|:{}", big);
    }
    println!();
}

// log1p(x) via x87 fyl2xp1, stored to f64
fn log1p_x87(x: f64) -> f64 {
    rx::ext_to_f64(
        &rx::ext_fyl2xp1(&rx::ext_ln2(), &rx::ext_from_f64(x), CW),
        CW,
    )
}

fn main() {
    let rows = load_pinned("../../work/w109/G6-solvers/pmt_em_pinned.json");
    for tn in [1i64, 2, 3, 4] {
        let sub: Vec<_> = rows.iter().filter(|(_, n, _)| *n == tn).cloned().collect();
        println!("\n=== n={}  ({} pinned) ===", tn, sub.len());
        let mut r_lnu = Vec::new();
        let mut r_l1p = Vec::new();
        let mut r_l1p_port = Vec::new();
        for (r, _, em_x) in &sub {
            let t = rx::x87_mul(-(tn as f64), rx::excel_log1p(*r));
            let u = rx::excel_exp(t);
            let em = |lnu: f64| -> f64 {
                if u == 1.0 {
                    t
                } else if t.abs() < 1.0 {
                    (u - 1.0) * t / lnu
                } else {
                    u - 1.0
                }
            };
            let um1 = u - 1.0;
            r_lnu.push(em(rx::excel_ln(u)).to_bits() as i64 - em_x.to_bits() as i64);
            r_l1p.push(em(log1p_x87(um1)).to_bits() as i64 - em_x.to_bits() as i64);
            r_l1p_port.push(em(rx::excel_log1p(um1)).to_bits() as i64 - em_x.to_bits() as i64);
        }
        hist("denom=ln(u) [ref]", &r_lnu);
        hist("denom=log1p_x87(u-1)", &r_l1p);
        hist("denom=log1p_port(u-1)", &r_l1p_port);
    }
}
