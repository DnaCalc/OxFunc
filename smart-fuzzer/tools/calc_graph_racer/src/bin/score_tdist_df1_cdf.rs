//! Score T.DIST df=1 CDF Cauchy forms vs live 20326 capture.

use oxfunc_core::excel_numeric::research as rx;
use oxfunc_core::functions::atan::atan_kernel;
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
        "../../work/w109/lastbit-12h-20260912/tdist-df1-cdf/capture.jsonl".into()
    });
    let pi = std::f64::consts::PI;
    let mut tot = 0usize;
    let mut e = [0usize; 5];
    let names = [
        "production",
        "0.5+atan_kernel(x)/PI",
        "0.5+atan_kernel(x)/excel_pi?",
        "0.5+x87_div(atan, PI)",
        "0.5+atan(x)/PI native",
    ];
    for line in fs::read_to_string(&path).unwrap().lines() {
        if line.is_empty() {
            continue;
        }
        let r: Row = serde_json::from_str(line).unwrap();
        let want = bits(&r.cdf_bits);
        let a = atan_kernel(r.x);
        let cands = [
            t_dist_kernel(r.x, 1.0, true).unwrap(),
            0.5 + a / pi,
            0.5 + a / f64::from_bits(0x400921fb54442d18),
            0.5 + rx::ext_to_f64(
                &rx::ext_div(
                    &rx::ext_from_f64(a),
                    &rx::ext_from_f64(pi),
                    rx::CW_PC64_RN,
                ),
                rx::CW_PC64_RN,
            ),
            0.5 + r.x.atan() / pi,
        ];
        tot += 1;
        for (i, v) in cands.iter().enumerate() {
            if v.to_bits() == want {
                e[i] += 1;
            }
        }
    }
    println!("T.DIST df=1 CDF vs Excel n={tot}");
    for (i, name) in names.iter().enumerate() {
        println!("  {:>5}/{:<5}  {name}", e[i], tot);
    }
}
