use oxfunc_core::functions::special_dist_family::gammaln_kernel;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs;

#[derive(Deserialize)]
struct Row {
    x: f64,
    gammaln_bits: String,
}

fn bits(s: &str) -> u64 {
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

fn main() {
    let path = std::env::args().nth(1).unwrap_or_else(|| {
        "../../work/w109/lastbit-12h-20260912/gammaln-b1/capture.jsonl".into()
    });
    let mut hist: BTreeMap<i64, usize> = BTreeMap::new();
    let mut exact = 0;
    let mut n = 0;
    let mut max = 0u64;
    for line in fs::read_to_string(&path).unwrap().lines() {
        if line.is_empty() {
            continue;
        }
        let r: Row = serde_json::from_str(line).unwrap();
        let got = gammaln_kernel(r.x).unwrap().to_bits();
        let want = bits(&r.gammaln_bits);
        let d = sulp(got, want);
        n += 1;
        if d == 0 {
            exact += 1;
        }
        max = max.max(d.unsigned_abs());
        *hist.entry(d.clamp(-8, 8)).or_default() += 1;
    }
    println!("B1 production vs Excel {exact}/{n} max={max} hist={hist:?}");
}
