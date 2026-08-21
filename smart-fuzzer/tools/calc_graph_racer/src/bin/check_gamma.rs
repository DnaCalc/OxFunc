//! W109 G3-02 scratch: score the CURRENT production GAMMA kernel against the
//! answered oracle rows, split positive vs negative, and show worst misses.

use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{WitnessArg, WitnessSet, ulp_distance};
use oxfunc_core::functions::special_dist_family::gamma_kernel;

fn main() {
    let files: Vec<String> = std::env::args().skip(1).collect();
    let mut stats: std::collections::BTreeMap<&str, (u32, u32, u64)> = Default::default();
    let mut misses: Vec<(u64, String, f64, String, String)> = Vec::new();
    for file in &files {
        let ws: WitnessSet =
            serde_json::from_str(&std::fs::read_to_string(file).expect("read")).expect("parse");
        for w in &ws.witnesses {
            let x = match &w.args[0] {
                WitnessArg::Scalar(s) => parse_bits_hex(s).unwrap(),
                _ => continue,
            };
            let Some(expected) = parse_bits_hex(&w.expected_bits) else {
                continue;
            };
            let key = if x < 0.0 { "neg" } else { "pos" };
            let e = stats.entry(key).or_default();
            e.1 += 1;
            match gamma_kernel(x) {
                Ok(v) if v.to_bits() == expected.to_bits() => e.0 += 1,
                Ok(v) => {
                    let d = ulp_distance(v, expected).unwrap_or(u64::MAX);
                    e.2 = e.2.max(d);
                    misses.push((
                        d,
                        w.id.clone().unwrap_or_default(),
                        x,
                        format!("0x{:016x}", v.to_bits()),
                        w.expected_bits.clone(),
                    ));
                }
                Err(err) => {
                    misses.push((
                        u64::MAX,
                        w.id.clone().unwrap_or_default(),
                        x,
                        format!("{err:?}"),
                        w.expected_bits.clone(),
                    ));
                }
            }
        }
    }
    for (k, (ok, tot, maxulp)) in &stats {
        println!("{k}: {ok}/{tot} exact, max_ulp {maxulp}");
    }
    misses.sort_by(|a, b| b.0.cmp(&a.0));
    for (d, id, x, got, want) in misses.iter().take(15) {
        println!("  {d:>8} ulp {id:12} x={x:+.9e} got {got} want {want}");
    }
}
