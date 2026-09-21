//! Score T.DIST df=2 CDF closed forms vs live 20326 capture.

use oxfunc_core::excel_numeric::research as rx;
use oxfunc_core::functions::chi_f_t_family::t_dist_kernel;
use serde::Deserialize;
use std::fs;

#[derive(Deserialize)]
struct Row {
    x: f64,
    cdf_bits: String,
}

fn bits(s: &str) -> u64 {
    u64::from_str_radix(s.trim_start_matches("0x"), 16).unwrap()
}

fn main() {
    let path = std::env::args().nth(1).unwrap_or_else(|| {
        "../../work/w109/lastbit-12h-20260912/tdist-df2-cdf/capture.jsonl".into()
    });
    let names = [
        "production",
        "0.5+x/(2*sqrt(2+x*x))",
        "0.5+x/(2*excel_sqrt(2+x*x))",
        "0.5+x/x87_mul(2,excel_sqrt)",
        "0.5+x87_div(x, 2*excel_sqrt)",
        "0.5+x87_div(x, x87_mul(2,excel_sqrt))",
        "0.5+x/(2*excel_sqrt(x87 2+x*x))",
    ];
    let mut exact = [0usize; 7];
    let mut tot = 0usize;
    for line in fs::read_to_string(&path).unwrap().lines() {
        if line.is_empty() {
            continue;
        }
        let r: Row = serde_json::from_str(line).unwrap();
        let want = bits(&r.cdf_bits);
        let x = r.x;
        let s_nat = (2.0 + x * x).sqrt();
        let s_ex = rx::excel_sqrt(2.0 + x * x);
        let s_ex2 = rx::excel_sqrt(rx::ext_to_f64(
            &rx::ext_add(
                &rx::ext_from_f64(2.0),
                &rx::ext_from_f64(x * x),
                rx::CW_PC64_RN,
            ),
            rx::CW_PC64_RN,
        ));
        let real = [
            t_dist_kernel(x, 2.0, true).unwrap(),
            0.5 + x / (2.0 * s_nat),
            0.5 + x / (2.0 * s_ex),
            0.5 + x / rx::x87_mul(2.0, s_ex),
            0.5 + rx::ext_to_f64(
                &rx::ext_div(
                    &rx::ext_from_f64(x),
                    &rx::ext_from_f64(2.0 * s_ex),
                    rx::CW_PC64_RN,
                ),
                rx::CW_PC64_RN,
            ),
            0.5 + rx::ext_to_f64(
                &rx::ext_div(
                    &rx::ext_from_f64(x),
                    &rx::ext_from_f64(rx::x87_mul(2.0, s_ex)),
                    rx::CW_PC64_RN,
                ),
                rx::CW_PC64_RN,
            ),
            0.5 + x / (2.0 * s_ex2),
        ];
        tot += 1;
        for (i, v) in real.iter().enumerate() {
            if v.to_bits() == want {
                exact[i] += 1;
            }
        }
    }
    println!("T.DIST df=2 CDF vs Excel n={tot}");
    for (i, name) in names.iter().enumerate() {
        println!("  {:>5}/{:<5}  {name}", exact[i], tot);
    }
}
