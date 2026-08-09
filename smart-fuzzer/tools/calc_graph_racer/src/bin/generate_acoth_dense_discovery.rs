//! Generate an answer-free, deterministic ACOTH discovery battery.
//!
//! Probe selection depends only on input bit patterns and public candidate
//! thresholds.  It spans the complete positive finite domain, mirrors every
//! magnitude across the sign bit, and concentrates adjacent-double ladders at
//! plausible reciprocal/logarithm/approximation seams.

use serde_json::json;
use std::collections::BTreeMap;
use std::path::PathBuf;

fn insert(rows: &mut BTreeMap<u64, &'static str>, bits: u64, class: &'static str) {
    let value = f64::from_bits(bits);
    if value.is_finite() && value > 1.0 {
        rows.entry(bits).or_insert(class);
    }
}

fn insert_ladder(
    rows: &mut BTreeMap<u64, &'static str>,
    center: f64,
    radius: i64,
    class: &'static str,
) {
    let bits = center.to_bits();
    for delta in -radius..=radius {
        insert(rows, bits.wrapping_add_signed(delta), class);
    }
}

fn output_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../work/w109/G4-03-acoth")
}

fn main() {
    let mut rows = BTreeMap::new();

    // Uniform bit-space coverage where most competing bodies disagree.
    for (lo, hi, class) in [
        (1.0_f64.to_bits() + 1, 4.0_f64.to_bits(), "near_domain_grid"),
        (3.0_f64.to_bits(), 16.0_f64.to_bits(), "midrange_grid"),
        (16.0_f64.to_bits(), 1.0e8_f64.to_bits(), "reciprocal_grid"),
    ] {
        let span = hi - lo;
        for index in 0_u64..=1024 {
            let offset = ((span as u128) * (index as u128) / 1024_u128) as u64;
            insert(&mut rows, lo + offset, class);
        }
    }

    // Every binary exponent, with two deterministic mantissas, catches hidden
    // passthrough/underflow/overflow branches without decimal serialization.
    let mut state = 0x243f_6a88_85a3_08d3_u64;
    for exponent in 0x3ff_u64..=0x7fe_u64 {
        for _ in 0..2 {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            let mantissa = state & 0x000f_ffff_ffff_ffff;
            insert(
                &mut rows,
                (exponent << 52) | mantissa,
                "exponent_stratified",
            );
        }
    }

    // Disjoint random finite doubles over the whole admitted positive domain.
    for _ in 0..2048 {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        let exponent = 0x3ff + ((state >> 52) % 0x400);
        let mantissa = state & 0x000f_ffff_ffff_ffff;
        insert(&mut rows, (exponent << 52) | mantissa, "random_finite_bits");
    }

    // Adjacent-double neighborhoods around public/observed candidate seams.
    for (center, class) in [
        (1.0_f64 + f64::EPSILON, "domain_edge_ladder"),
        (2.0_f64, "reciprocal_half_ladder"),
        (2.0_f64 + std::f64::consts::SQRT_2, "legacy_log1p_ladder"),
        (3.5_f64, "provisional_switch_ladder"),
        (5.0_f64, "legacy_residual_ladder"),
        (8.0_f64, "midrange_ladder"),
        (8.100_000_072_902_997_f64, "legacy_residual_ladder"),
        (10.0_f64, "fixed_ladder"),
        (1.0e7_f64, "cephes_passthrough_ladder"),
        (
            f64::from_bits(0x4340_0000_0000_0000),
            "reciprocal_ulp_ladder",
        ),
        (
            f64::from_bits(0x5fef_ffff_ffff_ffff),
            "square_overflow_ladder",
        ),
    ] {
        insert_ladder(&mut rows, center, 96, class);
    }

    let mut probes = Vec::with_capacity(rows.len() * 2);
    let mut metadata = String::from("id,class,input_bits\n");
    let mut ordinal = 0_usize;
    for (magnitude_bits, class) in rows {
        for input_bits in [magnitude_bits, magnitude_bits | (1_u64 << 63)] {
            let id = format!("acoth-dense-{ordinal:05}");
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
    let batch_path = directory.join("batch-acoth-dense-discovery-20260809.json");
    let metadata_path = directory.join("meta-acoth-dense-discovery-20260809.csv");
    let batch = json!({
        "function": "ACOTH",
        "row_id": "G4-03",
        "generated_utc": "2026-08-09",
        "selection": "oracle-blind bit grids + exponent strata + deterministic random bits + adjacent ladders",
        "probes": probes
    });
    std::fs::write(&batch_path, serde_json::to_vec_pretty(&batch).unwrap()).expect("write batch");
    std::fs::write(&metadata_path, metadata).expect("write metadata");
    println!("{ordinal} probes");
    println!("{}", batch_path.display());
    println!("{}", metadata_path.display());
}
