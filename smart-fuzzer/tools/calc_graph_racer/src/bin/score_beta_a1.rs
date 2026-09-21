//! Score BETA a=1 CDF 1-(1-x)^b vs live 20326.

use oxfunc_core::excel_numeric::research as rx;
use oxfunc_core::functions::special_math_common::regularized_beta;
use serde::Deserialize;
use std::fs;

#[derive(Deserialize)]
struct Row {
    x: f64,
    b: f64,
    cdf_bits: String,
}

fn bits(s: &str) -> u64 {
    u64::from_str_radix(s.trim_start_matches("0x"), 16).unwrap()
}

fn main() {
    let path = std::env::args().nth(1).unwrap_or_else(|| {
        "../../work/w109/lastbit-12h-20260912/beta-a1/capture.jsonl".into()
    });
    let mut tot = 0usize;
    let mut e = [0usize; 4];
    let names = [
        "regularized_beta(x,1,b)",
        "1-pow_chain(1-x,b)",
        "1-pow_positive(1-x,b)",
        "x87 1-pow_chain",
    ];
    for line in fs::read_to_string(&path).unwrap().lines() {
        if line.is_empty() {
            continue;
        }
        let r: Row = serde_json::from_str(line).unwrap();
        let want = bits(&r.cdf_bits);
        let pc = 1.0 - rx::excel_pow_chain(1.0 - r.x, r.b);
        let cands = [
            regularized_beta(r.x, 1.0, r.b),
            pc,
            1.0 - rx::excel_pow_positive(1.0 - r.x, r.b),
            rx::ext_to_f64(
                &rx::ext_sub(
                    &rx::ext_from_f64(1.0),
                    &rx::ext_from_f64(rx::excel_pow_chain(1.0 - r.x, r.b)),
                    rx::CW_PC64_RN,
                ),
                rx::CW_PC64_RN,
            ),
        ];
        tot += 1;
        for (i, v) in cands.iter().enumerate() {
            if v.to_bits() == want {
                e[i] += 1;
            }
        }
    }
    println!("BETA a=1 CDF vs Excel n={tot}");
    for (i, name) in names.iter().enumerate() {
        println!("  {:>5}/{:<5}  {name}", e[i], tot);
    }
}
