//! Score F.DIST.RT d1=2 closed forms vs live 20326 capture.

use oxfunc_core::excel_numeric::research as rx;
use oxfunc_core::functions::chi_f_t_family::f_dist_rt_kernel;
use serde::Deserialize;
use std::fs;

#[derive(Deserialize)]
struct Row {
    x: f64,
    d2: f64,
    rt_bits: String,
}

fn bits(s: &str) -> u64 {
    u64::from_str_radix(s.trim_start_matches("0x"), 16).unwrap()
}

fn main() {
    let path = std::env::args().nth(1).unwrap_or_else(|| {
        "../../work/w109/lastbit-12h-20260912/fdist-d1eq2/capture.jsonl".into()
    });
    let mut tot = 0usize;
    let mut e = [0usize; 4];
    let names = [
        "production",
        "pow_chain(d2/(d2+2x), d2/2)",
        "pow_chain((d2/(d2+2x)), d2*0.5)",
        "pow_positive(d2/(d2+2x), d2/2)",
    ];
    for line in fs::read_to_string(&path).unwrap().lines() {
        if line.is_empty() {
            continue;
        }
        let r: Row = serde_json::from_str(line).unwrap();
        let want = bits(&r.rt_bits);
        let base = r.d2 / (r.d2 + 2.0 * r.x);
        let exp = r.d2 / 2.0;
        let cands = [
            f_dist_rt_kernel(r.x, 2.0, r.d2).unwrap(),
            rx::excel_pow_chain(base, exp),
            rx::excel_pow_chain(base, r.d2 * 0.5),
            rx::excel_pow_positive(base, exp),
        ];
        tot += 1;
        for (i, v) in cands.iter().enumerate() {
            if v.to_bits() == want {
                e[i] += 1;
            }
        }
    }
    println!("F.DIST.RT d1=2 vs Excel n={tot}");
    for (i, name) in names.iter().enumerate() {
        println!("  {:>5}/{:<5}  {name}", e[i], tot);
    }
}
