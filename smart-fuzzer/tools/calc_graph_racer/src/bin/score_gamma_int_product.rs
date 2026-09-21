//! Race x87/native factorial-product graphs vs live Excel GAMMA(n) for n=89..170.
use oxfunc_core::excel_numeric::research as rx;
use serde::Deserialize;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Deserialize)]
struct Row {
    n: u32,
    gamma: String,
}

fn parse_bits(s: &str) -> u64 {
    u64::from_str_radix(s.trim().trim_start_matches("0x"), 16).expect("hex bits")
}

fn x87_rev(n: u32, store: bool, cw: u16) -> f64 {
    let mut a = rx::ext_from_f64(1.0);
    for k in (2..n).rev() {
        a = rx::ext_mul(&a, &rx::ext_from_f64(k as f64), cw);
        if store {
            a = rx::ext_from_f64(rx::ext_to_f64(&a, cw));
        }
    }
    rx::ext_to_f64(&a, cw)
}

fn x87_fwd(n: u32, store: bool, cw: u16) -> f64 {
    let mut a = rx::ext_from_f64(1.0);
    for k in 2..n {
        a = rx::ext_mul(&a, &rx::ext_from_f64(k as f64), cw);
        if store {
            a = rx::ext_from_f64(rx::ext_to_f64(&a, cw));
        }
    }
    rx::ext_to_f64(&a, cw)
}

fn native_rev(n: u32) -> f64 {
    let mut a = 1.0;
    for k in (2..n).rev() {
        a *= k as f64;
    }
    a
}

fn blk_rev_native(n: u32, b: usize) -> f64 {
    let xs: Vec<f64> = (2..n).rev().map(|k| k as f64).collect();
    let mut a = 1.0;
    let mut i = 0;
    while i < xs.len() {
        let mut blk = 1.0;
        for x in xs.iter().skip(i).take(b) {
            blk *= *x;
        }
        a *= blk;
        i += b;
    }
    a
}

fn blk_rev_x87(n: u32, b: usize, store_between: bool, store_inside: bool, cw: u16) -> f64 {
    let xs: Vec<f64> = (2..n).rev().map(|k| k as f64).collect();
    let mut acc = rx::ext_from_f64(1.0);
    let mut i = 0;
    while i < xs.len() {
        let mut blk = rx::ext_from_f64(1.0);
        for x in xs.iter().skip(i).take(b) {
            blk = rx::ext_mul(&blk, &rx::ext_from_f64(*x), cw);
            if store_inside {
                blk = rx::ext_from_f64(rx::ext_to_f64(&blk, cw));
            }
        }
        acc = rx::ext_mul(&acc, &blk, cw);
        if store_between {
            acc = rx::ext_from_f64(rx::ext_to_f64(&acc, cw));
        }
        i += b;
    }
    rx::ext_to_f64(&acc, cw)
}

fn x87_dr_rev(n: u32) -> f64 {
    let mut a = 1.0;
    for k in (2..n).rev() {
        a = rx::x87_mul(a, k as f64);
    }
    a
}

fn main() {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| {
            "smart-fuzzer/work/w109/lastbit-12h-20260912/identities/gamma-ints-89-170.jsonl"
                .into()
        });
    let file = File::open(&path).unwrap_or_else(|e| panic!("open {path}: {e}"));
    let mut rows = Vec::new();
    for line in BufReader::new(file).lines() {
        let line = line.unwrap();
        let line = line.trim_start_matches('\u{feff}');
        if line.is_empty() {
            continue;
        }
        rows.push(serde_json::from_str::<Row>(line).unwrap());
    }
    let want: Vec<(u32, u64)> = rows
        .iter()
        .map(|r| (r.n, parse_bits(&r.gamma)))
        .collect();
    println!("loaded {} GAMMA integer rows from {path}", want.len());

    let cw64 = rx::CW_PC64_RN;
    let cw53 = rx::CW_PC53_RN;

    let mut graphs: Vec<(&str, Box<dyn Fn(u32) -> f64>)> = vec![
        ("native_rev", Box::new(native_rev)),
        ("x87dr_rev", Box::new(x87_dr_rev)),
        ("x87pc64_rev_cont", Box::new(move |n| x87_rev(n, false, cw64))),
        ("x87pc64_rev_store", Box::new(move |n| x87_rev(n, true, cw64))),
        ("x87pc53_rev_cont", Box::new(move |n| x87_rev(n, false, cw53))),
        ("x87pc53_rev_store", Box::new(move |n| x87_rev(n, true, cw53))),
        ("x87pc64_fwd_cont", Box::new(move |n| x87_fwd(n, false, cw64))),
        ("x87pc64_fwd_store", Box::new(move |n| x87_fwd(n, true, cw64))),
        ("blk4_native", Box::new(|n| blk_rev_native(n, 4))),
        ("blk2_native", Box::new(|n| blk_rev_native(n, 2))),
        ("blk8_native", Box::new(|n| blk_rev_native(n, 8))),
        (
            "blk4_x87pc64_cont",
            Box::new(move |n| blk_rev_x87(n, 4, false, false, cw64)),
        ),
        (
            "blk4_x87pc64_store_between",
            Box::new(move |n| blk_rev_x87(n, 4, true, false, cw64)),
        ),
        (
            "blk4_x87pc64_store_inside",
            Box::new(move |n| blk_rev_x87(n, 4, true, true, cw64)),
        ),
        (
            "blk2_x87pc64_store_between",
            Box::new(move |n| blk_rev_x87(n, 2, true, false, cw64)),
        ),
        (
            "blk8_x87pc64_store_between",
            Box::new(move |n| blk_rev_x87(n, 8, true, false, cw64)),
        ),
        (
            "blk4_x87pc53_store_between",
            Box::new(move |n| blk_rev_x87(n, 4, true, false, cw53)),
        ),
    ];

    let mut scores: Vec<(usize, u64, &str)> = Vec::new();
    for (name, f) in graphs.iter_mut() {
        let mut exact = 0usize;
        let mut max_u = 0u64;
        for &(n, w) in &want {
            let g = f(n).to_bits();
            let u = g.abs_diff(w);
            if u == 0 {
                exact += 1;
            }
            if u > max_u {
                max_u = u;
            }
        }
        scores.push((exact, max_u, name));
    }
    scores.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
    println!("graph exact/{} maxUlp", want.len());
    for (exact, max_u, name) in scores {
        println!("  {exact:3}/{n} maxUlp={max_u} {name}", n = want.len());
    }
}
