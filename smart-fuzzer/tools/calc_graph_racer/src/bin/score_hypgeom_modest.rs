//! Score HYPGEOM PMF candidates vs live Excel modest grid.
use oxfunc_core::excel_numeric::research as rx;
use oxfunc_core::functions::combin::combin_kernel;
use oxfunc_core::functions::discrete_dist_family::hypergeom_dist_kernel;
use serde::Deserialize;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Deserialize)]
struct Row {
    #[serde(rename = "kDraw")]
    k_draw: i64,
    #[serde(rename = "nDraw")]
    n_draw: i64,
    #[serde(rename = "kPop")]
    k_pop: i64,
    #[serde(rename = "nPop")]
    n_pop: i64,
    hypgeom: String,
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

fn seq_mul(k: i64, n: i64, k_pop: i64, n_pop: i64, x87: bool) -> f64 {
    let mut p = 1.0;
    for i in 0..k {
        let num1 = (k_pop - i) as f64;
        let num2 = (n - i) as f64;
        let den1 = (i + 1) as f64;
        let den2 = (n_pop - i) as f64;
        if x87 {
            p = rx::x87_mul(p, num1);
            p = x87_div(p, den1);
            p = rx::x87_mul(p, num2);
            p = x87_div(p, den2);
        } else {
            p = p * num1 / den1 * num2 / den2;
        }
    }
    p
}

fn main() {
    let path = std::env::args().nth(1).unwrap_or_else(|| {
        "smart-fuzzer/work/w109/lastbit-12h-20260912/hypgeom/modest.jsonl".into()
    });
    let file = File::open(&path).unwrap();
    let mut rows = Vec::new();
    for line in BufReader::new(file).lines() {
        let line = line.unwrap();
        let line = line.trim_start_matches('\u{feff}');
        if line.is_empty() {
            continue;
        }
        rows.push(serde_json::from_str::<Row>(&line).unwrap());
    }
    println!("loaded {} rows", rows.len());

    let names = [
        "production",
        "combin (c*c)/c",
        "combin c*(c/c)",
        "combin x87 (c*c)/c",
        "combin x87 c*(c/c)",
        "seq native",
        "seq x87",
    ];
    let mut exact = [0usize; 7];
    let mut maxu = [0u64; 7];
    for r in &rows {
        let want = parse_bits(&r.hypgeom);
        let k = r.k_draw as f64;
        let n = r.n_draw as f64;
        let kp = r.k_pop as f64;
        let np = r.n_pop as f64;
        let c1 = combin_kernel(kp, k).unwrap();
        let c2 = combin_kernel(np - kp, n - k).unwrap();
        let c3 = combin_kernel(np, n).unwrap();
        let cands = [
            hypergeom_dist_kernel(k, n, kp, np, false).unwrap(),
            (c1 * c2) / c3,
            c1 * (c2 / c3),
            x87_div(rx::x87_mul(c1, c2), c3),
            rx::x87_mul(c1, x87_div(c2, c3)),
            seq_mul(r.k_draw, r.n_draw, r.k_pop, r.n_pop, false),
            seq_mul(r.k_draw, r.n_draw, r.k_pop, r.n_pop, true),
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
    for i in 0..7 {
        println!(
            "  {:>2}/{:<2} maxUlp={}  {}",
            exact[i],
            rows.len(),
            maxu[i],
            names[i]
        );
    }
}
