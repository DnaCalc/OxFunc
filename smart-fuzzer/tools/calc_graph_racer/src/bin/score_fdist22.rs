//! Score F(2,2) CDF/RT closed forms vs live 20326 capture.

use oxfunc_core::excel_numeric::research as rx;
use oxfunc_core::functions::chi_f_t_family::{f_dist_kernel, f_dist_rt_kernel};
use serde::Deserialize;
use std::fs;

#[derive(Deserialize)]
struct Row {
    x: f64,
    cdf_bits: String,
    rt_bits: String,
}

fn bits(s: &str) -> u64 {
    u64::from_str_radix(s.trim_start_matches("0x"), 16).unwrap()
}

fn main() {
    let path = std::env::args().nth(1).unwrap_or_else(|| {
        "../../work/w109/lastbit-12h-20260912/fdist-22/capture.jsonl".into()
    });
    let mut tot = 0usize;
    let mut e = [0usize; 8];
    let names = [
        "prod CDF",
        "prod RT",
        "x/(1+x)",
        "1-1/(1+x)",
        "1-x87_recip(1+x)",
        "x*x87_recip(1+x)",
        "x87_div(x,1+x)",
        "x87_recip(1+x) as RT",
    ];
    for line in fs::read_to_string(&path).unwrap().lines() {
        if line.is_empty() {
            continue;
        }
        let r: Row = serde_json::from_str(line).unwrap();
        let want_c = bits(&r.cdf_bits);
        let want_q = bits(&r.rt_bits);
        let x = r.x;
        let rec = rx::x87_recip(1.0 + x);
        let cands_c = [
            f_dist_kernel(x, 2.0, 2.0, true).unwrap(),
            x / (1.0 + x),
            1.0 - 1.0 / (1.0 + x),
            1.0 - rec,
            x * rec,
            rx::ext_to_f64(
                &rx::ext_div(
                    &rx::ext_from_f64(x),
                    &rx::ext_from_f64(1.0 + x),
                    rx::CW_PC64_RN,
                ),
                rx::CW_PC64_RN,
            ),
        ];
        tot += 1;
        if cands_c[0].to_bits() == want_c {
            e[0] += 1;
        }
        if f_dist_rt_kernel(x, 2.0, 2.0).unwrap().to_bits() == want_q {
            e[1] += 1;
        }
        if cands_c[1].to_bits() == want_c {
            e[2] += 1;
        }
        if cands_c[2].to_bits() == want_c {
            e[3] += 1;
        }
        if cands_c[3].to_bits() == want_c {
            e[4] += 1;
        }
        if cands_c[4].to_bits() == want_c {
            e[5] += 1;
        }
        if cands_c[5].to_bits() == want_c {
            e[6] += 1;
        }
        if rec.to_bits() == want_q {
            e[7] += 1;
        }
    }
    println!("F(2,2) vs Excel n={tot}");
    for (i, name) in names.iter().enumerate() {
        println!("  {:>5}/{:<5}  {name}", e[i], tot);
    }
}
