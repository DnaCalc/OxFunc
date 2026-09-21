use oxfunc_core::excel_numeric::research as rx;
use oxfunc_core::functions::discrete_dist_family::hypergeom_dist_kernel;
use serde::Deserialize;
use std::fs;

#[derive(Deserialize)]
struct Row {
    sample_s: f64,
    number_sample: f64,
    population_s: f64,
    number_pop: f64,
    pmf_bits: String,
}

fn bits(s: &str) -> u64 {
    u64::from_str_radix(s.trim_start_matches("0x"), 16).unwrap()
}

fn choose_div_last(n: u64, k: u64) -> f64 {
    let k = k.min(n.saturating_sub(k));
    let mut acc = 1.0;
    for i in 1..=k {
        acc = acc * (n - k + i) as f64 / i as f64;
    }
    acc
}

fn choose_div_first(n: u64, k: u64) -> f64 {
    let k = k.min(n.saturating_sub(k));
    let mut acc = 1.0;
    for i in 1..=k {
        acc = acc / i as f64 * (n - k + i) as f64;
    }
    acc
}

fn main() {
    let path = std::env::args().nth(1).unwrap_or_else(|| {
        "../../work/w109/lastbit-12h-20260912/hypgeom/capture.jsonl".into()
    });
    let names = [
        "production",
        "(c1*c2)/c3",
        "c1*(c2/c3)",
        "x87_div(c1*c2, c3)",
        "c1*x87_div(c2,c3)",
        "(d1*d2)/d3 div-first choose",
    ];
    let mut exact = [0usize; 6];
    let mut tot = 0usize;
    for line in fs::read_to_string(&path).unwrap().lines() {
        if line.is_empty() {
            continue;
        }
        let r: Row = serde_json::from_str(line).unwrap();
        let want = bits(&r.pmf_bits);
        let k = r.sample_s as u64;
        let n = r.number_sample as u64;
        let kk = r.population_s as u64;
        let nn = r.number_pop as u64;
        let c1 = choose_div_last(kk, k);
        let c2 = choose_div_last(nn - kk, n - k);
        let c3 = choose_div_last(nn, n);
        let d1 = choose_div_first(kk, k);
        let d2 = choose_div_first(nn - kk, n - k);
        let d3 = choose_div_first(nn, n);
        let cands = [
            hypergeom_dist_kernel(
                r.sample_s,
                r.number_sample,
                r.population_s,
                r.number_pop,
                false,
            )
            .unwrap(),
            (c1 * c2) / c3,
            c1 * (c2 / c3),
            rx::ext_to_f64(
                &rx::ext_div(
                    &rx::ext_from_f64(c1 * c2),
                    &rx::ext_from_f64(c3),
                    rx::CW_PC64_RN,
                ),
                rx::CW_PC64_RN,
            ),
            c1 * rx::ext_to_f64(
                &rx::ext_div(
                    &rx::ext_from_f64(c2),
                    &rx::ext_from_f64(c3),
                    rx::CW_PC64_RN,
                ),
                rx::CW_PC64_RN,
            ),
            (d1 * d2) / d3,
        ];
        tot += 1;
        for (i, v) in cands.iter().enumerate() {
            if v.to_bits() == want {
                exact[i] += 1;
            }
        }
    }
    println!("HYPGEOM vs Excel n={tot}");
    for (i, name) in names.iter().enumerate() {
        println!("  {:>5}/{:<5}  {name}", exact[i], tot);
    }
}
