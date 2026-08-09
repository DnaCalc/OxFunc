//! Generate the frozen, answer-free W109 G4-03 ACOTH exact-parity held-out.
//!
//! The candidate was frozen before this file reads any held-out answers:
//! * `|x| < 3.69662725` (`0x400d92b14ec204f3`): direct ratio, x87-PC64
//!   stored division, FYL2X logarithm, extended half-scale;
//! * otherwise: direct inverse odd-power series with every reciprocal,
//!   multiply, divide, and accumulator add stored through x87 PC64;
//! * stop when the stored accumulator no longer changes (32-term safety cap);
//! * flush a subnormal reciprocal to positive zero and restore sign otherwise.
//!
//! Input selection is oracle-blind and excludes every prior ACOTH discovery or
//! legacy witness bit pattern.  It combines disjoint random seeds, frozen-body
//! disagreement filters, complete exponent strata, and IEEE boundary ladders.

use oxfunc_core::excel_numeric::research as rx;
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

const CW: u16 = rx::CW_PC64_RN;
const THRESHOLD: f64 = f64::from_bits(0x400d_92b1_4ec2_04f3);

fn xadd(a: f64, b: f64) -> f64 {
    rx::ext_to_f64(
        &rx::ext_add(&rx::ext_from_f64(a), &rx::ext_from_f64(b), CW),
        CW,
    )
}

fn xsub(a: f64, b: f64) -> f64 {
    rx::ext_to_f64(
        &rx::ext_sub(&rx::ext_from_f64(a), &rx::ext_from_f64(b), CW),
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

fn ratio(a: f64, add_store: bool, sub_store: bool, div_store: bool) -> f64 {
    let numerator = if add_store { xadd(a, 1.0) } else { a + 1.0 };
    let denominator = if sub_store { xsub(a, 1.0) } else { a - 1.0 };
    let quotient = if div_store {
        xdiv(numerator, denominator)
    } else {
        numerator / denominator
    };
    let logarithm = rx::ext_fyl2x(&rx::ext_ln2(), &rx::ext_from_f64(quotient), CW);
    rx::ext_to_f64(&rx::ext_mul(&logarithm, &rx::ext_from_f64(0.5), CW), CW)
}

fn series(
    a: f64,
    recip_store: bool,
    mul_store: bool,
    div_store: bool,
    add_store: bool,
    terms: usize,
) -> f64 {
    let mul = |left, right| {
        if mul_store {
            xmul(left, right)
        } else {
            left * right
        }
    };
    let div = |left, right| {
        if div_store {
            xdiv(left, right)
        } else {
            left / right
        }
    };
    let add = |left, right| {
        if add_store {
            xadd(left, right)
        } else {
            left + right
        }
    };
    let reciprocal = if recip_store { xdiv(1.0, a) } else { 1.0 / a };
    if reciprocal < f64::MIN_POSITIVE {
        return 0.0;
    }
    let square = mul(a, a);
    let mut power = a;
    let mut sum = reciprocal;
    for k in 1..terms {
        power = mul(power, square);
        let denominator = mul((2 * k + 1) as f64, power);
        let next = add(sum, div(1.0, denominator));
        if next == sum {
            break;
        }
        sum = next;
    }
    sum
}

fn frozen_magnitude(a: f64) -> f64 {
    if a < THRESHOLD {
        ratio(a, false, false, true)
    } else {
        series(a, true, true, true, true, 32)
    }
}

fn parse_bits(text: &str) -> Option<u64> {
    u64::from_str_radix(text.trim_start_matches("0x"), 16).ok()
}

fn collect_prior_bits(path: &Path, prior: &mut BTreeSet<u64>) {
    if !path.exists() {
        return;
    }
    let document: Value =
        serde_json::from_str(&std::fs::read_to_string(path).expect("read prior artifact"))
            .expect("parse prior artifact");
    if let Some(probes) = document.get("probes").and_then(Value::as_array) {
        for probe in probes {
            if let Some(text) = probe
                .get("probe")
                .and_then(|value| value.get("args"))
                .and_then(|value| value.get(0))
                .and_then(Value::as_str)
            {
                if let Some(bits) = parse_bits(text) {
                    prior.insert(bits);
                }
            }
        }
    }
    if let Some(witnesses) = document.get("witnesses").and_then(Value::as_array) {
        for witness in witnesses {
            if let Some(text) = witness
                .get("args")
                .and_then(|value| value.get(0))
                .and_then(Value::as_str)
            {
                if let Some(bits) = parse_bits(text) {
                    prior.insert(bits);
                }
            }
        }
    }
}

fn insert(
    rows: &mut BTreeMap<u64, &'static str>,
    prior: &BTreeSet<u64>,
    bits: u64,
    class: &'static str,
) {
    let value = f64::from_bits(bits);
    if value.is_finite()
        && value > 1.0
        && !prior.contains(&bits)
        && !prior.contains(&(bits | (1_u64 << 63)))
    {
        rows.entry(bits).or_insert(class);
    }
}

fn insert_ladder(
    rows: &mut BTreeMap<u64, &'static str>,
    prior: &BTreeSet<u64>,
    center: f64,
    radius: i64,
    class: &'static str,
) {
    let bits = center.to_bits();
    for delta in -radius..=radius {
        insert(rows, prior, bits.wrapping_add_signed(delta), class);
    }
}

fn output_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../work/w109/G4-03-acoth")
}

fn main() {
    let w109 = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../work/w109");
    let directory = output_dir();
    let mut prior = BTreeSet::new();
    for path in [
        w109.join("G4-hyp-answers-acoth.json"),
        directory.join("batch-acoth-dense-discovery-20260809.json"),
        directory.join("batch-acoth-graph-discovery-20260809.json"),
        directory.join("batch-acoth-switch-r1-20260809.json"),
        directory.join("batch-acoth-switch-r2-20260809.json"),
    ] {
        collect_prior_bits(&path, &mut prior);
    }

    let mut rows = BTreeMap::new();
    let mut state = 0x1319_8a2e_0370_7344_u64;

    // Fresh random coverage over every admitted exponent, with independent
    // mantissas and no overlap with any discovery/legacy input.
    for exponent in 0x3ff_u64..=0x7fe_u64 {
        for _ in 0..2 {
            state = state
                .wrapping_mul(2_862_933_555_777_941_757)
                .wrapping_add(3_037_000_493);
            insert(
                &mut rows,
                &prior,
                (exponent << 52) | (state & 0x000f_ffff_ffff_ffff),
                "fresh_exponent_stratified",
            );
        }
    }
    for _ in 0..8192 {
        state = state
            .wrapping_mul(2_862_933_555_777_941_757)
            .wrapping_add(3_037_000_493);
        let exponent = 0x3ff + ((state >> 52) % 0x400);
        let bits = (exponent << 52) | (state & 0x000f_ffff_ffff_ffff);
        insert(&mut rows, &prior, bits, "fresh_random_finite_bits");
    }

    // Fresh seam points both inside the observational overlap and outside the
    // exhaustive discovery bracket.  Candidate disagreements are retained as
    // a separate class, but agreement rows remain genuine held-out coverage.
    for _ in 0..8192 {
        state = state
            .wrapping_mul(2_862_933_555_777_941_757)
            .wrapping_add(3_037_000_493);
        let signed_offset = ((state >> 1) % 4_000_000_001) as i64 - 2_000_000_000;
        let bits = THRESHOLD.to_bits().wrapping_add_signed(signed_offset);
        let a = f64::from_bits(bits);
        let class = if ratio(a, false, false, true).to_bits()
            != series(a, true, true, true, true, 32).to_bits()
        {
            "fresh_route_disagreement"
        } else {
            "fresh_route_overlap"
        };
        insert(&mut rows, &prior, bits, class);
    }

    // Discriminate the frozen staging from each simpler alternative using a
    // seed not used by discovery.  The selection cap keeps the gate compact.
    let mut counts = [0_usize; 10];
    for _ in 0..500_000 {
        state = state
            .wrapping_mul(2_862_933_555_777_941_757)
            .wrapping_add(3_037_000_493);
        let lo = 1.0_f64.to_bits() + 1;
        let hi = 128.0_f64.to_bits();
        let bits = lo + (((hi - lo) as u128 * state as u128) >> 64) as u64;
        if prior.contains(&bits) || prior.contains(&(bits | (1_u64 << 63))) {
            continue;
        }
        let a = f64::from_bits(bits);
        let frozen = frozen_magnitude(a).to_bits();
        let alternatives = if a < THRESHOLD {
            [
                ratio(a, true, false, true).to_bits(),
                ratio(a, false, true, true).to_bits(),
                ratio(a, true, true, true).to_bits(),
                ratio(a, false, false, false).to_bits(),
                frozen,
                frozen,
                frozen,
                frozen,
                frozen,
                frozen,
            ]
        } else {
            [
                frozen,
                frozen,
                frozen,
                frozen,
                series(a, false, true, true, true, 32).to_bits(),
                series(a, true, false, true, true, 32).to_bits(),
                series(a, true, true, false, true, 32).to_bits(),
                series(a, true, true, true, false, 32).to_bits(),
                series(a, true, true, true, true, 12).to_bits(),
                series(a, true, true, true, true, 13).to_bits(),
            ]
        };
        let classes = [
            "ratio_add_store_disagreement",
            "ratio_sub_store_disagreement",
            "ratio_both_store_disagreement",
            "ratio_div_store_disagreement",
            "series_recip_store_disagreement",
            "series_mul_store_disagreement",
            "series_div_store_disagreement",
            "series_add_store_disagreement",
            "series_term12_disagreement",
            "series_term13_disagreement",
        ];
        for index in 0..alternatives.len() {
            if alternatives[index] != frozen && counts[index] < 256 {
                insert(&mut rows, &prior, bits, classes[index]);
                counts[index] += 1;
            }
        }
        if counts[..9].iter().all(|count| *count >= 256) {
            break;
        }
    }

    // Extend rather than repeat the discovery ladders.
    for (center, radius, class) in [
        (THRESHOLD, 8192_i64, "frozen_switch_extended_ladder"),
        (
            1.0 / f64::MIN_POSITIVE,
            2048,
            "reciprocal_daz_extended_ladder",
        ),
        (f64::MAX.sqrt(), 1024, "square_overflow_extended_ladder"),
        (
            (f64::MAX / 3.0).cbrt(),
            1024,
            "cube_overflow_extended_ladder",
        ),
        (f64::MAX, 2048, "maximum_finite_extended_ladder"),
        (1.0 + f64::EPSILON, 2048, "domain_edge_extended_ladder"),
    ] {
        insert_ladder(&mut rows, &prior, center, radius, class);
    }

    let mut probes = Vec::with_capacity(rows.len() * 2);
    let mut metadata = String::from("id,class,input_bits\n");
    let mut ordinal = 0_usize;
    for (magnitude_bits, class) in rows {
        for input_bits in [magnitude_bits, magnitude_bits | (1_u64 << 63)] {
            let id = format!("acoth-heldout-{ordinal:06}");
            ordinal += 1;
            probes.push(json!({
                "probe": {"id": id, "args": [format!("0x{input_bits:016x}")]},
                "distinct_outputs": 0,
                "outputs": []
            }));
            metadata.push_str(&format!("{id},{class},0x{input_bits:016x}\n"));
        }
    }

    std::fs::create_dir_all(&directory).expect("create output directory");
    let batch_path = directory.join("batch-acoth-exact-heldout-20260809.json");
    let metadata_path = directory.join("meta-acoth-exact-heldout-20260809.csv");
    let batch = json!({
        "function": "ACOTH",
        "row_id": "G4-03",
        "generated_utc": "2026-08-09",
        "candidate_frozen_before_answers": true,
        "candidate": {
            "threshold_bits": "0x400d92b14ec204f3",
            "threshold_decimal": "3.69662725",
            "near_body": "native add/sub; x87-PC64 stored division; FYL2X; extended half",
            "far_body": "direct inverse odd-power series; x87-PC64 store after reciprocal/multiply/divide/add; stable-sum termination; 32-term cap",
            "publication": "subnormal reciprocal -> +0; otherwise copysign from input"
        },
        "selection": "oracle-blind disjoint-seed broad coverage + frozen-candidate discriminators + extended IEEE/seam ladders; all prior input bits excluded",
        "probes": probes
    });
    std::fs::write(&batch_path, serde_json::to_vec_pretty(&batch).unwrap()).expect("write batch");
    std::fs::write(&metadata_path, metadata).expect("write metadata");
    println!("candidate threshold=0x{:016x}", THRESHOLD.to_bits());
    println!("staging discriminator counts={counts:?}");
    println!("excluded prior signed bits={}", prior.len());
    println!("{ordinal} held-out probes");
    println!("{}", batch_path.display());
    println!("{}", metadata_path.display());
}
