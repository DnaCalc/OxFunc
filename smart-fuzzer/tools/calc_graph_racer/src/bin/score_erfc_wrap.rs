//! Score ERFC wrapper candidates against the signed complement capture.

use oxfunc_core::functions::special_dist_family::{erf_precise_kernel, erfc_precise_kernel};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs;

#[derive(Deserialize)]
struct Row {
    z: f64,
    erf_bits: String,
    erfc_bits: String,
}

fn parse_hex(s: &str) -> u64 {
    u64::from_str_radix(s.trim_start_matches("0x"), 16).unwrap()
}

fn sulp(got: u64, want: u64) -> i64 {
    fn ord(v: u64) -> u64 {
        if v >> 63 == 0 {
            v | (1u64 << 63)
        } else {
            !v
        }
    }
    (i128::from(ord(got)) - i128::from(ord(want))) as i64
}

fn hist(name: &str, ds: &[i64]) {
    let mut c: BTreeMap<i64, usize> = BTreeMap::new();
    let mut exact = 0;
    let mut max = 0u64;
    for &d in ds {
        if d == 0 {
            exact += 1;
        }
        max = max.max(d.unsigned_abs());
        *c.entry(d.clamp(-8, 8)).or_default() += 1;
    }
    println!("{name:<48} {exact}/{n} max={max} hist={c:?}", n = ds.len());
}

fn erf_body(x: f64) -> f64 {
    erf_precise_kernel(x).unwrap()
}
fn erfc_body(x: f64) -> f64 {
    erfc_precise_kernel(x).unwrap()
}

fn wrap_erfc(x: f64) -> f64 {
    let ax = x.abs();
    let q = if ax < 0.5 {
        1.0 - erf_body(ax)
    } else {
        erfc_body(ax)
    };
    if x.is_sign_positive() || x == 0.0 {
        q
    } else {
        2.0 - q
    }
}

fn wrap_erf_odd_only(x: f64) -> f64 {
    let ax = x.abs();
    let p = if ax < 0.5 {
        erf_body(ax)
    } else {
        erf_body(ax)
    };
    if x.is_sign_negative() && x != 0.0 {
        -p
    } else {
        p
    }
}

fn wrap_erf_full(x: f64) -> f64 {
    let ax = x.abs();
    let p = if ax < 0.5 {
        erf_body(ax)
    } else {
        1.0 - erfc_body(ax)
    };
    if x.is_sign_negative() && x != 0.0 {
        -p
    } else {
        p
    }
}

fn main() {
    let path = std::env::args().nth(1).unwrap_or_else(|| {
        "../../work/w109/lastbit-12h-20260912/erf-sign/capture.jsonl".into()
    });
    let rows: Vec<Row> = fs::read_to_string(&path)
        .unwrap()
        .lines()
        .filter(|l| !l.is_empty())
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    println!("rows {}", rows.len());

    let score = |name: &str, kind: &str, eval: fn(f64) -> f64| {
        let ds: Vec<i64> = rows
            .iter()
            .map(|r| {
                let want = if kind == "erf" {
                    parse_hex(&r.erf_bits)
                } else {
                    parse_hex(&r.erfc_bits)
                };
                sulp(eval(r.z).to_bits(), want)
            })
            .collect();
        hist(name, &ds);
        for &(lo, hi, lab) in &[
            (-2.0, -0.5, "neg-large"),
            (-0.5, 0.0, "neg-small"),
            (0.0, 0.5, "pos-small"),
            (0.5, 2.1, "pos-large"),
        ] {
            let ds: Vec<i64> = rows
                .iter()
                .filter(|r| r.z >= lo && r.z < hi)
                .map(|r| {
                    let want = if kind == "erf" {
                        parse_hex(&r.erf_bits)
                    } else {
                        parse_hex(&r.erfc_bits)
                    };
                    sulp(eval(r.z).to_bits(), want)
                })
                .collect();
            hist(&format!("  {lab}"), &ds);
        }
    };

    println!("\n# ERFC");
    score("production ERFC", "erfc", erfc_body);
    score("wrap 1-erf |x|<0.5, 2-q for neg", "erfc", wrap_erfc);
    println!("\n# ERF");
    score("production ERF", "erf", erf_body);
    score("wrap odd-only", "erf", wrap_erf_odd_only);
    score("wrap full 1-erfc |x|>=0.5", "erf", wrap_erf_full);
}
