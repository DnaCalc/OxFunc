//! Generate an answer-free ACOTH graph-discriminator batch.
//!
//! The prior dense capture leaves a narrow ratio/series seam and a handful of
//! observationally equivalent series stagings.  Selection here depends only on
//! candidate disagreement and public IEEE-754 boundaries; no oracle answers are
//! read.  Every positive magnitude is mirrored to test sign publication.

use oxfunc_core::excel_numeric::research as rx;
use serde_json::json;
use std::collections::BTreeMap;
use std::path::PathBuf;

const CW: u16 = rx::CW_PC64_RN;

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

fn series(
    a: f64,
    x87_recip: bool,
    x87_mul: bool,
    x87_divide: bool,
    x87_add: bool,
    terms: usize,
) -> f64 {
    let div = |left, right| {
        if x87_divide {
            xdiv(left, right)
        } else {
            left / right
        }
    };
    let mul = |left, right| {
        if x87_mul {
            xmul(left, right)
        } else {
            left * right
        }
    };
    let add = |left, right| {
        if x87_add {
            xadd(left, right)
        } else {
            left + right
        }
    };
    let reciprocal = if x87_recip { xdiv(1.0, a) } else { 1.0 / a };
    if reciprocal < f64::MIN_POSITIVE {
        return 0.0;
    }
    let square = mul(a, a);
    let mut power = a;
    let mut sum = reciprocal;
    for k in 1..terms {
        power = mul(power, square);
        let denominator = mul((2 * k + 1) as f64, power);
        let term = div(1.0, denominator);
        sum = add(sum, term);
    }
    sum
}

fn forward_series(a: f64) -> f64 {
    let x = xdiv(1.0, a);
    if x < f64::MIN_POSITIVE {
        return 0.0;
    }
    let z = x * x;
    let mut power = x;
    let mut sum = x;
    for k in 1..16 {
        power *= z;
        sum += power / ((2 * k + 1) as f64);
    }
    sum
}

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

    // The discovery corpus brackets the first hard route discriminator between
    // these two exact grid points.  Sample the whole interval uniformly in bit
    // space, retaining only places where the two frozen bodies disagree.
    let lo = 3.693_359_375_f64.to_bits();
    let hi = 3.699_218_75_f64.to_bits();
    for index in 0_u64..=8192 {
        let bits = lo + (((hi - lo) as u128 * index as u128) / 8192_u128) as u64;
        let a = f64::from_bits(bits);
        if ratio(a).to_bits() != series(a, true, true, false, true, 16).to_bits() {
            insert(&mut rows, bits, "route_uniform_disagreement");
        }
    }

    // If the route constant is a short decimal, these adjacent-double ladders
    // identify the exact represented threshold rather than just bracketing it.
    for center in [
        3.693_f64,
        3.694,
        3.695,
        3.696,
        3.697,
        3.698,
        3.699,
        3.700,
        3.695_312_5,
    ] {
        insert_ladder(&mut rows, center, 256, "route_decimal_ladder");
    }

    // Candidate-staging disagreement search.  The seeds and stopping limits
    // are fixed, and the oracle is not involved in selection.
    let mut state = 0xa409_3822_299f_31d0_u64;
    let mut counts = [0_usize; 7];
    for _ in 0..500_000 {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        let lo_bits = 3.70_f64.to_bits();
        let hi_bits = 64.0_f64.to_bits();
        let bits = lo_bits + (((hi_bits - lo_bits) as u128 * state as u128) >> 64) as u64;
        let a = f64::from_bits(bits);
        let baseline = series(a, true, true, false, true, 16).to_bits();
        let alternatives = [
            series(a, false, true, false, true, 16).to_bits(),
            series(a, true, false, false, true, 16).to_bits(),
            series(a, true, true, true, true, 16).to_bits(),
            series(a, true, true, false, false, 16).to_bits(),
            series(a, true, true, false, true, 12).to_bits(),
            series(a, true, true, false, true, 13).to_bits(),
            forward_series(a).to_bits(),
        ];
        let classes = [
            "reciprocal_stage_disagreement",
            "multiply_stage_disagreement",
            "divide_stage_disagreement",
            "add_stage_disagreement",
            "term12_disagreement",
            "term13_disagreement",
            "series_shape_disagreement",
        ];
        for index in 0..alternatives.len() {
            if alternatives[index] != baseline && counts[index] < 384 {
                insert(&mut rows, bits, classes[index]);
                counts[index] += 1;
            }
        }
        if counts.iter().all(|count| *count >= 384) {
            break;
        }
    }

    // Exponent-stratified division and overflow discriminators.  Direct
    // inverse powers intentionally overflow their denominators; the leading
    // reciprocal must remain the published result after that point.
    for exponent in 0x401_u64..=0x7fe_u64 {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        insert(
            &mut rows,
            (exponent << 52) | (state & 0x000f_ffff_ffff_ffff),
            "series_exponent_stratified",
        );
    }

    let reciprocal_normal_boundary = 1.0 / f64::MIN_POSITIVE;
    let square_overflow_boundary = f64::MAX.sqrt();
    let cube_overflow_boundary = (f64::MAX / 3.0).cbrt();
    for (center, class, radius) in [
        (
            reciprocal_normal_boundary,
            "reciprocal_daz_boundary",
            512_i64,
        ),
        (
            square_overflow_boundary,
            "square_overflow_boundary",
            256_i64,
        ),
        (
            cube_overflow_boundary,
            "cube_denominator_overflow_boundary",
            256_i64,
        ),
        (f64::MAX, "maximum_finite_ladder", 512_i64),
    ] {
        insert_ladder(&mut rows, center, radius, class);
    }

    let mut probes = Vec::with_capacity(rows.len() * 2);
    let mut metadata = String::from("id,class,input_bits\n");
    let mut ordinal = 0_usize;
    for (magnitude_bits, class) in rows {
        for input_bits in [magnitude_bits, magnitude_bits | (1_u64 << 63)] {
            let id = format!("acoth-graph-{ordinal:05}");
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
    let batch_path = directory.join("batch-acoth-graph-discovery-20260809.json");
    let metadata_path = directory.join("meta-acoth-graph-discovery-20260809.csv");
    let batch = json!({
        "function": "ACOTH",
        "row_id": "G4-03",
        "generated_utc": "2026-08-09",
        "selection": "oracle-blind route-grid + candidate-staging disagreements + IEEE-754 boundary ladders",
        "probes": probes
    });
    std::fs::write(&batch_path, serde_json::to_vec_pretty(&batch).unwrap()).expect("write batch");
    std::fs::write(&metadata_path, metadata).expect("write metadata");
    println!("staging discriminator counts: {counts:?}");
    println!("{ordinal} probes");
    println!("{}", batch_path.display());
    println!("{}", metadata_path.display());
}
