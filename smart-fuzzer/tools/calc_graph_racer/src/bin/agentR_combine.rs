//! Refine the PMT combine on the RESOLUTION corpus (fv1sweep n=360) with the
//! confirmed invF=exp(-n*log1p). Dump per-group residual histograms; test
//! num-order, em-sign, tf placement (ty=1), and per-op sse/x87.
use oxfunc_core::excel_numeric::research as rx;
use serde_json::Value;
use std::collections::BTreeMap;

const CW: u16 = rx::CW_PC64_RN;
fn e(x: f64) -> rx::Ext80 { rx::ext_from_f64(x) }
fn t64(x: &rx::Ext80) -> f64 { rx::ext_to_f64(x, CW) }
fn xadd(a: f64, b: f64) -> f64 { t64(&rx::ext_add(&e(a), &e(b), CW)) }
fn xsub(a: f64, b: f64) -> f64 { t64(&rx::ext_sub(&e(a), &e(b), CW)) }
fn xmul(a: f64, b: f64) -> f64 { t64(&rx::ext_mul(&e(a), &e(b), CW)) }
fn xdiv(a: f64, b: f64) -> f64 { t64(&rx::ext_div(&e(a), &e(b), CW)) }

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
fn invf(r: f64, n: f64) -> f64 { rx::excel_exp(-(n * rx::excel_log1p(r))) }

#[derive(Clone, Copy, PartialEq)]
enum M { S, X }
fn mul(m: M, a: f64, b: f64) -> f64 { if m == M::S { a * b } else { xmul(a, b) } }
fn div(m: M, a: f64, b: f64) -> f64 { if m == M::S { a / b } else { xdiv(a, b) } }
fn add(m: M, a: f64, b: f64) -> f64 { if m == M::S { a + b } else { xadd(a, b) } }
fn sub(m: M, a: f64, b: f64) -> f64 { if m == M::S { a - b } else { xsub(a, b) } }

// combine variants (m, invF, r, pv, fv, ty)
fn v_qf_tfr(m: M, iv: f64, r: f64, pv: f64, fv: f64, ty: f64) -> f64 {
    // ((num/em)/tf)*r   tf=1+r*ty
    let tf = if ty == 0.0 { 1.0 } else { add(m, 1.0, mul(m, r, ty)) };
    let em = sub(m, iv, 1.0);
    let num = add(m, pv, mul(m, fv, iv));
    mul(m, div(m, div(m, num, em), tf), r)
}
fn v_qf_rtf(m: M, iv: f64, r: f64, pv: f64, fv: f64, ty: f64) -> f64 {
    // ((num/em)*r)/tf
    let tf = if ty == 0.0 { 1.0 } else { add(m, 1.0, mul(m, r, ty)) };
    let em = sub(m, iv, 1.0);
    let num = add(m, pv, mul(m, fv, iv));
    div(m, mul(m, div(m, num, em), r), tf)
}
fn v_adv(m: M, iv: f64, r: f64, pv: f64, fv: f64, ty: f64) -> f64 {
    // pay-in-advance LibreOffice style: end/(1+r) when ty=1
    let em = sub(m, iv, 1.0);
    let num = add(m, pv, mul(m, fv, iv));
    let end = mul(m, div(m, num, em), r); // = (num/em)*r  (ty=0 value)
    if ty == 0.0 { end } else { div(m, end, add(m, 1.0, r)) }
}
fn v_negem(m: M, iv: f64, r: f64, pv: f64, fv: f64, ty: f64) -> f64 {
    // -num*r/(tf*(1-invF)) : denom via (1-invF), single-fraction
    let tf = if ty == 0.0 { 1.0 } else { add(m, 1.0, mul(m, r, ty)) };
    let denom = sub(m, 1.0, iv);
    let num = add(m, pv, mul(m, fv, iv));
    -div(m, mul(m, num, r), mul(m, tf, denom))
}

fn score(rows: &[Row], f: fn(M, f64, f64, f64, f64, f64) -> f64, m: M, fv: f64, ty: f64) -> (usize, usize, BTreeMap<i64, usize>) {
    let mut ok = 0; let mut cnt = 0; let mut h: BTreeMap<i64, usize> = BTreeMap::new();
    for row in rows.iter().filter(|r| r.fv == fv && r.ty == ty) {
        let iv = invf(row.r, row.n);
        let got = f(m, iv, row.r, row.pv, row.fv, row.ty).to_bits();
        let d = got as i64 - row.want as i64;
        *h.entry(d).or_default() += 1;
        if d == 0 { ok += 1; } cnt += 1;
    }
    (ok, cnt, h)
}

fn main() {
    let res = load("../../work/w109/G6-solvers/answers-pmt-fv1sweep.json");
    let res360: Vec<Row> = res.iter().cloned().filter(|r| (r.n - 360.0).abs() < 0.5).collect();
    let groups = [(-1.0, 0.0), (-1.0, 1.0), (1.0, 0.0), (1.0, 1.0)];
    let fns: Vec<(&str, fn(M, f64, f64, f64, f64, f64) -> f64)> = vec![
        ("qf_tfr ((num/em)/tf)*r", v_qf_tfr),
        ("qf_rtf ((num/em)*r)/tf", v_qf_rtf),
        ("adv end/(1+r)         ", v_adv),
        ("negem -num*r/(tf*1mv) ", v_negem),
    ];
    for m in [M::X, M::S] {
        println!("=== ops {} ===", if m == M::X { "x87" } else { "sse" });
        for (nm, f) in &fns {
            let mut line = format!("  {:26}", nm); let mut tot = 0;
            for (fv, ty) in groups {
                let (ok, _c, _h) = score(&res360, *f, m, fv, ty);
                line.push_str(&format!(" ({:+},{}):{:3}", fv as i32, ty as i32, ok)); tot += ok;
            }
            println!("{}  tot={}", line, tot);
        }
    }
    // Histograms for the leading variant (qf_tfr x87) per group.
    println!("\n=== residual hist qf_tfr x87 ===");
    for (fv, ty) in groups {
        let (ok, c, h) = score(&res360, v_qf_tfr, M::X, fv, ty);
        println!("  fv={:+} ty={}: {}/{} hist={:?}", fv as i32, ty as i32, ok, c, h);
    }
    println!("\n=== residual hist qf_rtf x87 (ty=1 groups) ===");
    for (fv, ty) in [(-1.0,1.0),(1.0,1.0)] {
        let (ok, c, h) = score(&res360, v_qf_rtf, M::X, fv, ty);
        println!("  fv={:+} ty={}: {}/{} hist={:?}", fv as i32, ty as i32, ok, c, h);
    }
}
