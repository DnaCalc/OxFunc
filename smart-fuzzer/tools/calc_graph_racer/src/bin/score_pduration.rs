//! Score PDURATION graphs vs live Excel bits.
use oxfunc_core::excel_numeric::research as rx;
use oxfunc_core::functions::financial_time_value_family::pduration;
use serde::Deserialize;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Deserialize)]
struct Row {
    rate: f64,
    pv: f64,
    fv: f64,
    pduration: String,
}

fn parse_bits(s: &str) -> u64 {
    u64::from_str_radix(s.trim().trim_start_matches("0x"), 16).unwrap()
}

fn x87_div(a: f64, b: f64) -> f64 {
    rx::ext_to_f64(
        &rx::ext_div(
            &rx::ext_from_f64(a),
            &rx::ext_from_f64(b),
            rx::CW_PC64_RN,
        ),
        rx::CW_PC64_RN,
    )
}
fn x87_add(a: f64, b: f64) -> f64 {
    rx::ext_to_f64(
        &rx::ext_add(
            &rx::ext_from_f64(a),
            &rx::ext_from_f64(b),
            rx::CW_PC64_RN,
        ),
        rx::CW_PC64_RN,
    )
}
fn x87_sub(a: f64, b: f64) -> f64 {
    rx::ext_to_f64(
        &rx::ext_sub(
            &rx::ext_from_f64(a),
            &rx::ext_from_f64(b),
            rx::CW_PC64_RN,
        ),
        rx::CW_PC64_RN,
    )
}

fn main() {
    let path = std::env::args().nth(1).unwrap_or_else(|| {
        "smart-fuzzer/work/w109/lastbit-12h-20260912/pduration/wide.jsonl".into()
    });
    let file = File::open(&path).unwrap_or_else(|e| panic!("open {path}: {e}"));
    let mut rows = Vec::new();
    for line in BufReader::new(file).lines() {
        let line = line.unwrap();
        let line = line.trim_start_matches('\u{feff}');
        if line.is_empty() {
            continue;
        }
        rows.push(serde_json::from_str::<Row>(&line).unwrap());
    }
    println!("loaded {} rows from {path}", rows.len());

    let names = [
        "production",
        "(ln fv-ln pv)/ln(1+r) native",
        "x87_div(ln fv-ln pv, ln(1+r))",
        "x87_div(x87_sub(ln fv,ln pv), ln(1+r))",
        "x87_div(x87_sub(ln fv,ln pv), ln(x87_add(1,r)))",
        "x87_div(ln fv-ln pv, ln(x87_add(1,r)))",
        "ln(fv/pv)/ln(1+r) excel_ln",
    ];
    let mut exact = vec![0usize; names.len()];
    let mut maxu = vec![0u64; names.len()];
    for r in &rows {
        let want = parse_bits(&r.pduration);
        let rate = r.rate;
        let pv = r.pv;
        let fv = r.fv;
        let ln_fv = rx::excel_ln(fv);
        let ln_pv = rx::excel_ln(pv);
        let ln_1r = rx::excel_ln(1.0 + rate);
        let ln_1r_x = rx::excel_ln(x87_add(1.0, rate));
        let cands = [
            pduration(rate, pv, fv).unwrap(),
            (ln_fv - ln_pv) / ln_1r,
            x87_div(ln_fv - ln_pv, ln_1r),
            x87_div(x87_sub(ln_fv, ln_pv), ln_1r),
            x87_div(x87_sub(ln_fv, ln_pv), ln_1r_x),
            x87_div(ln_fv - ln_pv, ln_1r_x),
            rx::excel_ln(fv / pv) / ln_1r,
        ];
        for (i, v) in cands.iter().enumerate() {
            let u = v.to_bits().abs_diff(want);
            if u == 0 {
                exact[i] += 1;
            }
            if u > maxu[i] {
                maxu[i] = u;
            }
        }
    }
    let n = rows.len();
    for i in 0..names.len() {
        println!("  {:>3}/{n} maxUlp={}  {}", exact[i], maxu[i], names[i]);
    }
}
