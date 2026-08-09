//! Generate one answer-free ACOTH route-switch refinement round.
//!
//! Usage:
//!   generate_acoth_switch_round <round> <low-bits> <high-bits>
//!
//! Bounds are prior observed route discriminators.  The generator subdivides
//! their bit interval deterministically and retains only inputs where the
//! frozen ratio and inverse-power-series bodies publish different bits.

use oxfunc_core::excel_numeric::research as rx;
use serde_json::json;
use std::collections::BTreeMap;
use std::path::PathBuf;

const CW: u16 = rx::CW_PC64_RN;
const SUBDIVISIONS: u64 = 65_536;

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

fn parse_bits(text: &str) -> u64 {
    u64::from_str_radix(text.trim_start_matches("0x"), 16).expect("hex input bits")
}

fn output_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../work/w109/G4-03-acoth")
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    assert_eq!(args.len(), 4, "round, low bits, high bits required");
    let round = &args[1];
    assert!(
        round
            .chars()
            .all(|character| character.is_ascii_alphanumeric()),
        "round must be alphanumeric"
    );
    let low = parse_bits(&args[2]);
    let high = parse_bits(&args[3]);
    assert!(low < high, "ordered positive bounds required");
    assert!(f64::from_bits(low) > 1.0 && f64::from_bits(high).is_finite());

    let mut magnitudes = BTreeMap::new();
    for index in 0..=SUBDIVISIONS {
        let bits = low + (((high - low) as u128 * index as u128) / SUBDIVISIONS as u128) as u64;
        let a = f64::from_bits(bits);
        if ratio(a).to_bits() != series(a).to_bits() {
            magnitudes.insert(bits, "route_disagreement");
        }
    }
    // Preserve the prior route-classified endpoints even if their body values
    // happen to agree after a future candidate edit.
    magnitudes.insert(low, "prior_ratio_endpoint");
    magnitudes.insert(high, "prior_series_endpoint");

    let mut probes = Vec::with_capacity(magnitudes.len() * 2);
    let mut metadata = String::from("id,class,input_bits\n");
    let mut ordinal = 0_usize;
    for (magnitude_bits, class) in magnitudes {
        for input_bits in [magnitude_bits, magnitude_bits | (1_u64 << 63)] {
            let id = format!("acoth-switch-{round}-{ordinal:06}");
            ordinal += 1;
            probes.push(json!({
                "probe": {"id": id, "args": [format!("0x{input_bits:016x}")]},
                "distinct_outputs": 0,
                "outputs": []
            }));
            metadata.push_str(&format!("{id},{class},0x{input_bits:016x}\n"));
        }
    }

    let directory = output_dir();
    std::fs::create_dir_all(&directory).expect("create output directory");
    let batch_path = directory.join(format!("batch-acoth-switch-{round}-20260809.json"));
    let metadata_path = directory.join(format!("meta-acoth-switch-{round}-20260809.csv"));
    let batch = json!({
        "function": "ACOTH",
        "row_id": "G4-03",
        "generated_utc": "2026-08-09",
        "selection": "oracle-blind 65536-way subdivision retaining only frozen-body disagreements",
        "prior_ratio_endpoint_bits": format!("0x{low:016x}"),
        "prior_series_endpoint_bits": format!("0x{high:016x}"),
        "probes": probes
    });
    std::fs::write(&batch_path, serde_json::to_vec_pretty(&batch).unwrap()).expect("write batch");
    std::fs::write(&metadata_path, metadata).expect("write metadata");
    println!("{ordinal} signed probes");
    println!("{}", batch_path.display());
    println!("{}", metadata_path.display());
}
