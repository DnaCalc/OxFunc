//! Classify captured ACOTH route-discriminator rows under the frozen bodies.

use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use oxfunc_core::functions::acoth::acoth_kernel;
use std::collections::BTreeMap;
use std::path::PathBuf;

const CW: u16 = rx::CW_PC64_RN;
const FROZEN_THRESHOLD: f64 = f64::from_bits(0x400d_92b1_4ec2_04f3);

fn xadd(a: f64, b: f64) -> f64 {
    rx::ext_to_f64(
        &rx::ext_add(&rx::ext_from_f64(a), &rx::ext_from_f64(b), CW),
        CW,
    )
}

fn xmul(a: f64, b: f64) -> f64 {
    rx::ext_to_f64(
        &rx::ext_mul(&rx::ext_from_f64(a), &rx::ext_from_f64(b), CW),
        CW,
    )
}

fn xdiv(a: f64, b: f64) -> f64 {
    rx::ext_to_f64(
        &rx::ext_div(&rx::ext_from_f64(a), &rx::ext_from_f64(b), CW),
        CW,
    )
}

fn ratio(a: f64) -> f64 {
    let quotient = xdiv(a + 1.0, a - 1.0);
    let logarithm = rx::ext_fyl2x(&rx::ext_ln2(), &rx::ext_from_f64(quotient), CW);
    rx::ext_to_f64(&rx::ext_mul(&logarithm, &rx::ext_from_f64(0.5), CW), CW)
}

fn series(a: f64) -> f64 {
    let reciprocal = xdiv(1.0, a);
    if reciprocal < f64::MIN_POSITIVE {
        return 0.0;
    }
    let square = xmul(a, a);
    let mut power = a;
    let mut sum = reciprocal;
    for k in 1..32 {
        power = xmul(power, square);
        let denominator = xmul((2 * k + 1) as f64, power);
        sum = xadd(sum, xdiv(1.0, denominator));
    }
    sum
}

fn main() {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../work/w109/G4-03-acoth");
    let filenames: Vec<String> = std::env::args().skip(1).collect();
    assert!(
        !filenames.is_empty(),
        "one or more answer filenames required"
    );
    let mut last_ratio = None;
    let mut first_series = None;
    let mut classified = 0_usize;
    let mut overlap = 0_usize;
    let mut anomalies = 0_usize;
    let mut signed_rows = BTreeMap::new();
    for filename in filenames {
        let path = base.join(filename);
        let document: WitnessSet =
            serde_json::from_str(&std::fs::read_to_string(&path).expect("read answers"))
                .expect("parse answers");
        for witness in document.witnesses {
            let input = match &witness.args[0] {
                WitnessArg::Scalar(text) => parse_bits_hex(text).expect("input bits"),
                _ => continue,
            };
            let expected = parse_bits_hex(&witness.expected_bits)
                .expect("numeric expected")
                .to_bits();
            if let Some((_, prior)) = signed_rows.insert(input.to_bits(), (input, expected)) {
                assert_eq!(prior, expected, "inconsistent duplicate oracle answer");
            }
            if input.is_sign_negative() {
                continue;
            }
            let r = ratio(input).to_bits();
            let s = series(input).to_bits();
            if r == s {
                overlap += 1;
            } else if expected == r {
                classified += 1;
                if last_ratio.is_none_or(|prior: f64| input > prior) {
                    last_ratio = Some(input);
                }
            } else if expected == s {
                classified += 1;
                if first_series.is_none_or(|prior: f64| input < prior) {
                    first_series = Some(input);
                }
            } else {
                anomalies += 1;
                println!(
                    "anomaly x=0x{:016x} expected=0x{expected:016x} ratio=0x{r:016x} series=0x{s:016x}",
                    input.to_bits()
                );
            }
        }
    }
    let low = last_ratio.expect("ratio endpoint");
    let high = first_series.expect("series endpoint");
    println!("classified={classified} overlap={overlap} anomalies={anomalies}");
    println!("last_ratio  bits=0x{:016x} value={low:.17e}", low.to_bits());
    println!(
        "first_series bits=0x{:016x} value={high:.17e}",
        high.to_bits()
    );
    println!("bit_gap={}", high.to_bits() - low.to_bits());

    let exact = signed_rows
        .values()
        .filter(|(input, expected)| {
            let magnitude = input.abs();
            let value = if magnitude < FROZEN_THRESHOLD {
                ratio(magnitude)
            } else {
                series(magnitude)
            };
            let published = if value == 0.0 {
                0.0
            } else {
                value.copysign(*input)
            };
            published.to_bits() == *expected
        })
        .count();
    println!(
        "frozen_threshold=0x{:016x} value={FROZEN_THRESHOLD:.17e}",
        FROZEN_THRESHOLD.to_bits()
    );
    println!("frozen_score={exact}/{}", signed_rows.len());

    let production_exact = signed_rows
        .values()
        .filter(|(input, expected)| {
            acoth_kernel(*input).is_ok_and(|value| value.to_bits() == *expected)
        })
        .count();
    println!("production_score={production_exact}/{}", signed_rows.len());
    assert_eq!(
        production_exact,
        signed_rows.len(),
        "production ACOTH diverges from the frozen graph corpus"
    );
}
