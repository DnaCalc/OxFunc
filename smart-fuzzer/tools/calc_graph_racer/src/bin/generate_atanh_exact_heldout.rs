//! Generate the fresh post-refinement held-out for the exact W109 ATANH graph.
//!
//! Frozen before this answer-free generator was run:
//!
//! * positive/negative subnormal inputs are DAZ-normalized and publish `+0`;
//! * below `0x3f1af82b729c1d83`, use binary64 `x + x*x*x/3`;
//! * otherwise form `(1+x)/(1-x)` with x87 PC64-to-binary64 double rounding
//!   at every add, subtract and divide, then use the established x87 LN body.

use oxfunc_core::excel_numeric::research as rx;
use serde_json::json;
use std::collections::BTreeMap;
use std::path::PathBuf;

const CW: u16 = 0x133f;
const THRESHOLD_BITS: u64 = 0x3f1a_f82b_729c_1d83;

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

fn frozen(x: f64) -> f64 {
    if x.abs() < f64::MIN_POSITIVE {
        0.0
    } else if x.abs().to_bits() < THRESHOLD_BITS {
        cubic(x)
    } else {
        ratio_mask(x, 0b111)
    }
}

fn prior(x: f64) -> f64 {
    if x.abs().to_bits() < 0x3f1a_f82b_729c_1d84 {
        cubic(x)
    } else {
        ratio_mask(x, 0)
    }
}

fn insert(rows: &mut BTreeMap<u64, &'static str>, bits: u64, class: &'static str) -> bool {
    let value = f64::from_bits(bits);
    if value.is_finite() && value > 0.0 && value < 1.0 {
        return rows.insert(bits, class).is_none();
    }
    false
}

fn output_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../work/w109/G4-02-atanh")
}

fn main() {
    let mut rows = BTreeMap::new();
    let min_normal = f64::MIN_POSITIVE.to_bits();
    let domain_high = 1.0_f64.to_bits() - 1;

    // Fresh normal-domain coverage, disjoint from both earlier random seeds.
    let mut broad_state = 0xa54f_f53a_5f1d_36f1_u64;
    let mut broad = 0_usize;
    while broad < 1200 {
        broad_state = broad_state
            .wrapping_mul(3_202_034_522_624_059_733)
            .wrapping_add(1);
        let bits = min_normal + broad_state % (domain_high - min_normal + 1);
        broad += usize::from(insert(&mut rows, bits, "fresh_normal_bits"));
    }

    // Explicit DAZ coverage across the full subnormal mantissa range.
    for bits in [1_u64, 2, 3, 1_u64 << 51, min_normal - 2, min_normal - 1] {
        insert(&mut rows, bits, "subnormal_edges");
    }
    let mut sub_state = 0x510e_527f_ade6_82d1_u64;
    let mut subnormal = 0_usize;
    while subnormal < 512 {
        sub_state = sub_state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        let bits = 1 + sub_state % (min_normal - 1);
        subnormal += usize::from(insert(&mut rows, bits, "subnormal_random"));
    }

    // Wrapper-staging discriminators: at least two of the eight add/sub/div
    // x87 double-rounding masks publish different results.
    let mut wrapper_state = 0x1f83_d9ab_fb41_bd6b_u64;
    let mut wrapper = 0_usize;
    while wrapper < 512 {
        wrapper_state = wrapper_state
            .wrapping_mul(2_862_933_555_777_941_757)
            .wrapping_add(3_037_000_493);
        let bits = THRESHOLD_BITS + wrapper_state % (domain_high - THRESHOLD_BITS + 1);
        let x = f64::from_bits(bits);
        let mut outputs = [0_u64; 8];
        for mask in 0_u8..8 {
            outputs[mask as usize] = ratio_mask(x, mask).to_bits();
        }
        outputs.sort_unstable();
        if outputs.windows(2).any(|window| window[0] != window[1]) {
            wrapper += usize::from(insert(&mut rows, bits, "wrapper_disagreement"));
        }
    }

    // Fresh cubic-vs-old-pair discriminators in the small normal band.
    let cubic_low = 5.0e-5_f64.to_bits();
    let mut cubic_state = 0x9b05_688c_2b3e_6c1f_u64;
    let mut cubic_rows = 0_usize;
    while cubic_rows < 1000 {
        cubic_state = cubic_state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        let bits = cubic_low + cubic_state % (THRESHOLD_BITS - cubic_low);
        let x = f64::from_bits(bits);
        if cubic(x).to_bits() != ratio_mask(x, 0).to_bits() {
            cubic_rows += usize::from(insert(&mut rows, bits, "small_body_disagreement"));
        }
    }

    // Exact seam guards around the sign-sensitive representative threshold.
    for delta in -512_i64..=512 {
        insert(
            &mut rows,
            THRESHOLD_BITS.wrapping_add_signed(delta),
            "threshold_adjacent",
        );
    }

    let mut probes = Vec::with_capacity(rows.len() * 2);
    let mut meta = String::from("id,class,input_bits,frozen_bits,prior_bits\n");
    let mut ordinal = 0_usize;
    for (bits, class) in rows {
        let magnitude = f64::from_bits(bits);
        for sign in [1.0_f64, -1.0_f64] {
            let x = sign * magnitude;
            let input_bits = x.to_bits();
            let frozen_bits = frozen(x).to_bits();
            let prior_bits = prior(x).to_bits();
            let id = format!("atanh-exact-ho-{ordinal:05}");
            ordinal += 1;
            probes.push(json!({
                "probe": {"id": id, "args": [format!("0x{input_bits:016x}")]},
                "distinct_outputs": usize::from(frozen_bits != prior_bits) + 1,
                "outputs": [
                    {"candidate_id": "frozen_exact_v2", "kind": "Number", "bits": format!("0x{frozen_bits:016x}")},
                    {"candidate_id": "retired_v1", "kind": "Number", "bits": format!("0x{prior_bits:016x}")}
                ]
            }));
            meta.push_str(&format!(
                "{id},{class},0x{input_bits:016x},0x{frozen_bits:016x},0x{prior_bits:016x}\n"
            ));
        }
    }

    let dir = output_dir();
    std::fs::create_dir_all(&dir).expect("create output directory");
    let batch_path = dir.join("batch-atanh-exact-heldout-20260809.json");
    let meta_path = dir.join("meta-atanh-exact-heldout-20260809.csv");
    let batch = json!({
        "function": "ATANH",
        "row_id": "G4-02",
        "generated_utc": "2026-08-09",
        "frozen_threshold_bits": format!("0x{THRESHOLD_BITS:016x}"),
        "selection": "oracle-blind fresh normals + DAZ subnormals + wrapper masks + small-body disagreements + threshold adjacent",
        "probes": probes
    });
    std::fs::write(&batch_path, serde_json::to_vec_pretty(&batch).unwrap()).expect("write batch");
    std::fs::write(&meta_path, meta).expect("write metadata");
    println!("{} probes", ordinal);
    println!("{}", batch_path.display());
    println!("{}", meta_path.display());
}
