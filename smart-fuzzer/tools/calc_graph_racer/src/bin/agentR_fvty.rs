//! Score the UNIFIED discount combine on fvty (family grid), split by |tau|>=1
//! (accurate em) vs |tau|<1 (expm1 wall), and by fv magnitude.
use oxfunc_core::excel_numeric::research as rx;
use serde_json::Value;
use std::collections::BTreeMap;

fn invf(r: f64, n: f64) -> f64 { rx::excel_exp(-(n * rx::excel_log1p(r))) }

#[derive(Clone, Copy)]
struct Row { r: f64, n: f64, pv: f64, fv: f64, ty: f64, want: u64 }
fn load(path: &str) -> Vec<Row> {
    let v: Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let mut o = Vec::new();
    for w in v["witnesses"].as_array().unwrap() {
        let a = w["args"].as_array().unwrap();
        let g = |i: usize| f64::from_bits(u64::from_str_radix(a[i].as_str().unwrap().trim_start_matches("0x"), 16).unwrap());
        o.push(Row { r: g(0), n: g(1), pv: g(2), fv: g(3), ty: g(4),
            want: u64::from_str_radix(w["expected_bits"].as_str().unwrap().trim_start_matches("0x"), 16).unwrap() });
    }
    o
}

// quotient-first discount, em via expm1 (internal) if |tau|<1 else invF-1
fn pmt(r: f64, n: f64, pv: f64, fv: f64, ty: f64) -> f64 {
    let tau = -(n * rx::excel_log1p(r));
    let iv = rx::excel_exp(tau);
    let em = if tau.abs() >= 1.0 { iv - 1.0 } else { rx::excel_expm1(tau) };
    let tf = if ty == 0.0 { 1.0 } else { 1.0 + r * ty };
    let num = pv + fv * iv;
    (num / em) / tf * r
}

fn main() {
    let rows = load("../../work/w109/G6-solvers/answers-pmt-fvty.json");
    // filter valid (r finite, 1+r>0, n>0, r!=0)
    let mut acc_ok = 0; let mut acc_c = 0; let mut acc_h: BTreeMap<i64, usize> = BTreeMap::new();
    let mut wall_ok = 0; let mut wall_c = 0;
    let mut by_fv: BTreeMap<i64, (usize, usize)> = BTreeMap::new();
    for row in &rows {
        if row.r == 0.0 || 1.0 + row.r <= 0.0 || row.n <= 0.0 { continue; }
        let tau = -(row.n * rx::excel_log1p(row.r));
        let got = pmt(row.r, row.n, row.pv, row.fv, row.ty).to_bits();
        let d = got as i64 - row.want as i64;
        let fvk = row.fv as i64;
        let ent = by_fv.entry(fvk).or_insert((0, 0));
        ent.1 += 1; if d == 0 { ent.0 += 1; }
        if tau.abs() >= 1.0 {
            acc_c += 1; if d == 0 { acc_ok += 1; }
            *acc_h.entry(d.clamp(-6, 6)).or_default() += 1;
        } else {
            wall_c += 1; if d == 0 { wall_ok += 1; }
        }
    }
    println!("fvty unified discount combine:");
    println!("  accurate-em (|tau|>=1): {}/{} ({:.1}%)", acc_ok, acc_c, 100.0 * acc_ok as f64 / acc_c as f64);
    println!("  expm1-wall (|tau|<1):   {}/{} ({:.1}%)", wall_ok, wall_c, 100.0 * wall_ok as f64 / wall_c as f64);
    println!("  accurate-em residual hist (clamped +-6): {:?}", acc_h);
    println!("  by fv (ok/total):");
    for (fv, (ok, c)) in &by_fv {
        println!("    fv={:6}: {}/{} ({:.1}%)", fv, ok, c, 100.0 * *ok as f64 / *c as f64);
    }
}
