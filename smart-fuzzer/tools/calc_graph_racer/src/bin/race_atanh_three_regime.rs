//! Replay and refine the W109 ATANH three-regime publication graph.

use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

const CW: u16 = 0x133f;
const THRESHOLD_BITS_V1: u64 = 0x3f1a_f82b_729c_1d84;
const THRESHOLD_BITS_V2: u64 = 0x3f1a_f82b_729c_1d83;

fn ext(x: f64) -> rx::Ext80 {
    rx::ext_from_f64(x)
}

fn to_f64(value: &rx::Ext80) -> f64 {
    rx::ext_to_f64(value, CW)
}

fn x87_add(a: f64, b: f64) -> f64 {
    to_f64(&rx::ext_add(&ext(a), &ext(b), CW))
}

fn x87_sub(a: f64, b: f64) -> f64 {
    to_f64(&rx::ext_sub(&ext(a), &ext(b), CW))
}

fn x87_div(a: f64, b: f64) -> f64 {
    to_f64(&rx::ext_div(&ext(a), &ext(b), CW))
}

fn cubic(x: f64) -> f64 {
    x + (x * x * x) / 3.0
}

fn ratio_log(ratio: f64) -> f64 {
    let logarithm = rx::ext_fyl2x(&rx::ext_ln2(), &ext(ratio), CW);
    to_f64(&rx::ext_mul(&logarithm, &ext(0.5), CW))
}

fn ratio_mask(x: f64, mask: u8) -> f64 {
    let numerator = if mask & 1 != 0 {
        x87_add(1.0, x)
    } else {
        1.0 + x
    };
    let denominator = if mask & 2 != 0 {
        x87_sub(1.0, x)
    } else {
        1.0 - x
    };
    let ratio = if mask & 4 != 0 {
        x87_div(numerator, denominator)
    } else {
        numerator / denominator
    };
    ratio_log(ratio)
}

fn ratio_extended(x: f64) -> f64 {
    let ex = ext(x);
    let numerator = rx::ext_add(&rx::ext_one(), &ex, CW);
    let denominator = rx::ext_sub(&rx::ext_one(), &ex, CW);
    let ratio = rx::ext_div(&numerator, &denominator, CW);
    let logarithm = rx::ext_fyl2x(&rx::ext_ln2(), &ratio, CW);
    to_f64(&rx::ext_mul(&logarithm, &ext(0.5), CW))
}

fn frozen(x: f64, threshold: u64, mask: u8, daz: bool) -> f64 {
    if daz && x.abs() < f64::MIN_POSITIVE {
        return 0.0;
    }
    if x.abs().to_bits() < threshold {
        cubic(x)
    } else {
        ratio_mask(x, mask)
    }
}

fn load(path: &Path) -> Vec<(String, f64, u64)> {
    let document: WitnessSet =
        serde_json::from_str(&std::fs::read_to_string(path).expect("read answers"))
            .expect("parse answers");
    document
        .witnesses
        .iter()
        .filter_map(|witness| {
            let input = match &witness.args[0] {
                WitnessArg::Scalar(text) => parse_bits_hex(text)?,
                _ => return None,
            };
            let expected = parse_bits_hex(&witness.expected_bits)?;
            Some((
                witness.id.clone().unwrap_or_else(|| "<missing-id>".into()),
                input,
                expected.to_bits(),
            ))
        })
        .collect()
}

fn score(rows: &[(String, f64, u64)], candidate: impl Fn(f64) -> f64) -> usize {
    rows.iter()
        .filter(|(_, input, expected)| candidate(*input).to_bits() == *expected)
        .count()
}

fn main() {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../work/w109");
    let cohorts = [
        ("legacy-hyp", base.join("G4-hyp-answers-atanh.json")),
        ("legacy-band", base.join("G4-02-answers-atanh-band.json")),
        ("legacy-gap", base.join("G4-02-answers-atanh-gap.json")),
        (
            "legacy-switch",
            base.join("G4-02-answers-atanh-switch.json"),
        ),
        (
            "dense-discovery",
            base.join("G4-02-atanh/answers-atanh-switch-dense-20260809.json"),
        ),
        (
            "retired-heldout",
            base.join("G4-02-atanh/answers-atanh-three-regime-heldout-20260809.json"),
        ),
        (
            "fresh-heldout",
            base.join("G4-02-atanh/answers-atanh-exact-heldout-20260809.json"),
        ),
    ];

    let mut all = BTreeMap::new();
    for (name, path) in cohorts {
        let rows = load(&path);
        println!("{name:18} {} rows", rows.len());
        for row in rows {
            all.insert(row.1.to_bits(), row);
        }
    }
    let rows: Vec<_> = all.into_values().collect();
    println!("{} distinct rows", rows.len());

    println!(
        "v1 plain ratio       {}/{}",
        score(&rows, |x| frozen(x, THRESHOLD_BITS_V1, 0, false)),
        rows.len()
    );
    for mask in 0_u8..8 {
        let count = score(&rows, |x| frozen(x, THRESHOLD_BITS_V2, mask, true));
        println!("v2 DAZ ratio mask {mask:03b} {count}/{}", rows.len());
    }
    println!(
        "all-extended ratio   {}/{}",
        score(&rows, |x| {
            if x.abs() < f64::MIN_POSITIVE {
                0.0
            } else if x.abs().to_bits() < THRESHOLD_BITS_V2 {
                cubic(x)
            } else {
                ratio_extended(x)
            }
        }),
        rows.len()
    );

    let best_mask = (0_u8..8)
        .max_by_key(|mask| score(&rows, |x| frozen(x, THRESHOLD_BITS_V2, *mask, true)))
        .unwrap();
    println!("best mask {best_mask:03b} misses:");
    for (id, input, expected) in &rows {
        let got = frozen(*input, THRESHOLD_BITS_V2, best_mask, true).to_bits();
        if got != *expected {
            println!(
                "  {id} input=0x{:016x} got=0x{got:016x} expected=0x{expected:016x}",
                input.to_bits()
            );
        }
    }
}
