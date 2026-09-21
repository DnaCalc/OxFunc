//! A&S 7.1.6 erf series on ERF.PRECISE z<0.5. Not an identity.
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{ulp_distance, WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC64_RN};
use std::collections::BTreeMap;
use std::env;
use std::fs;

const CW: u16 = CW_PC64_RN;
const ULP_CAP: u64 = 1 << 20;
const TWO_OVER_SQRTPI: f64 = 1.1283791670955125739;

fn ef(x: f64) -> Ext80 {
    ext_from_f64(x)
}
/// erf(z) = (2/√π) Σ (-1)^n z^{2n+1} / (n!(2n+1))
fn as716(z: f64, nterm: u32) -> f64 {
    let ze = ef(z);
    let z2 = ext_mul(&ze, &ze, CW);
    let mut term = ze;
    let mut acc = ze;
    for n in 1..nterm {
        let nf = n as f64;
        term = ext_mul(&term, &z2, CW);
        term = ext_div(&term, &ef(nf), CW);
        term = ext_from_f64(-ext_to_f64(&term, CW));
        let den = 2.0 * nf + 1.0;
        acc = ext_add(&acc, &ext_div(&term, &ef(den), CW), CW);
    }
    ext_to_f64(&ext_mul(&ef(TWO_OVER_SQRTPI), &acc, CW), CW)
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    const BANKS: [&str; 7] = [
        "answers-b9train.json",
        "answers-erfp.json",
        "answers-erfm.json",
        "answers-b8erf.json",
        "answers-b7erf.json",
        "answers-b11.json",
        "answers-b10.json",
    ];
    let mut map = BTreeMap::new();
    for name in BANKS {
        let path = format!("{dir}/{name}");
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        let bank: WitnessSet = serde_json::from_str(&text).unwrap();
        for w in &bank.witnesses {
            let x = match &w.args[0] {
                WitnessArg::Scalar(s) => parse_bits_hex(s).unwrap(),
                _ => continue,
            };
            let Some(e) = parse_bits_hex(&w.expected_bits) else {
                continue;
            };
            if x > 0.0 && x < 0.5 {
                map.entry(x.to_bits()).or_insert(e.to_bits());
            }
        }
    }
    let rows: Vec<_> = map
        .into_iter()
        .map(|(b, e)| (f64::from_bits(b), e))
        .collect();
    println!("P-side rows={}", rows.len());
    for nterm in [8u32, 12, 16, 20, 24, 32] {
        let mut ex = 0usize;
        let mut n = 0usize;
        let mut max = 0u64;
        for &(z, bits) in &rows {
            let g = as716(z, nterm);
            if !g.is_finite() {
                continue;
            }
            let d = ulp_distance(g, f64::from_bits(bits)).unwrap_or(u64::MAX);
            if d > ULP_CAP {
                continue;
            }
            n += 1;
            if d == 0 {
                ex += 1;
            } else {
                max = max.max(d);
            }
        }
        println!("n={nterm:2} {ex}/{n} max={max}");
    }
}
