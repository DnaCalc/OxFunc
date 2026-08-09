//! Generate the fresh oracle-blind held-out for the frozen W109 ATANH graph.
//!
//! The candidate was frozen before this generator was run:
//!
//! * `|x| < 0x3f1af82b729c1d84`: binary64 `x + x*x*x/3`;
//! * otherwise: stored binary64 ratio followed by the x87 LN publication.
//!
//! The adjacent input one bit below the threshold is observationally
//! equivalent between the two bodies, as recorded by the separate live
//! bisection artifact.  Probe selection below uses inputs and candidate
//! disagreement only; no Excel answer is read.

use oxfunc_core::excel_numeric::research as rx;
use serde_json::json;
use std::collections::BTreeMap;
use std::path::PathBuf;

const CW: u16 = 0x133f;
const THRESHOLD_BITS: u64 = 0x3f1a_f82b_729c_1d84;

fn ext(x: f64) -> rx::Ext80 {
    rx::ext_from_f64(x)
}

fn to_f64(value: &rx::Ext80) -> f64 {
    rx::ext_to_f64(value, CW)
}

fn pair_x87(x: f64) -> f64 {
    let ln2 = rx::ext_ln2();
    let positive = rx::ext_fyl2xp1(&ln2, &ext(x), CW);
    let negative = rx::ext_fyl2xp1(&ln2, &ext(-x), CW);
    let difference = rx::ext_sub(&positive, &negative, CW);
    to_f64(&rx::ext_mul(&difference, &ext(0.5), CW))
}

fn cubic(x: f64) -> f64 {
    x + (x * x * x) / 3.0
}

fn ratio_x87(x: f64) -> f64 {
    let ratio = (1.0 + x) / (1.0 - x);
    let logarithm = rx::ext_fyl2x(&rx::ext_ln2(), &ext(ratio), CW);
    to_f64(&rx::ext_mul(&logarithm, &ext(0.5), CW))
}

fn frozen(x: f64) -> f64 {
    if x.abs().to_bits() < THRESHOLD_BITS {
        cubic(x)
    } else {
        ratio_x87(x)
    }
}

fn old_production(x: f64) -> f64 {
    if x.abs() < 1.05e-4 {
        pair_x87(x)
    } else {
        ratio_x87(x)
    }
}

fn insert(rows: &mut BTreeMap<u64, &'static str>, bits: u64, class: &'static str) {
    let value = f64::from_bits(bits);
    if value.is_finite() && value > 0.0 && value < 1.0 {
        rows.entry(bits).or_insert(class);
    }
}

fn output_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../work/w109/G4-02-atanh")
}

fn main() {
    let mut rows = BTreeMap::new();

    // Broad exponent/mantissa coverage over the entire admitted positive
    // domain.  Uniform bit selection deliberately spreads probes across
    // binades instead of clustering in value space.
    let domain_high = 1.0_f64.to_bits() - 1;
    let mut broad_state = 0xbb67_ae85_84ca_a73b_u64;
    while rows
        .values()
        .filter(|class| **class == "broad_bits")
        .count()
        < 1500
    {
        broad_state = broad_state
            .wrapping_mul(2_862_933_555_777_941_757)
            .wrapping_add(3_037_000_493);
        insert(&mut rows, 1 + broad_state % domain_high, "broad_bits");
    }

    // A disjoint focused sample selected where at least one frozen/old/body
    // candidate disagrees.  This is the high-information held-out stratum.
    let focus_low = 5.0e-5_f64.to_bits();
    let focus_high = 2.0e-4_f64.to_bits();
    let focus_span = focus_high - focus_low;
    let mut focus_state = 0x3c6e_f372_fe94_f82b_u64;
    let mut focused = 0_usize;
    while focused < 1000 {
        focus_state = focus_state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        let bits = focus_low + focus_state % (focus_span + 1);
        let x = f64::from_bits(bits);
        if frozen(x).to_bits() != old_production(x).to_bits()
            || cubic(x).to_bits() != ratio_x87(x).to_bits()
        {
            let before = rows.len();
            insert(&mut rows, bits, "focused_disagreement");
            focused += usize::from(rows.len() != before);
        }
    }

    // Direct boundary guards, including the two discriminating neighbors and
    // the single intervening input where cubic and ratio publish the same bit.
    for delta in -256_i64..=256 {
        insert(
            &mut rows,
            THRESHOLD_BITS.wrapping_add_signed(delta),
            "boundary_adjacent",
        );
    }

    // Structured binade controls: leading mantissa, next-up, midpoint and
    // trailing mantissa over the admitted exponent range.
    for exponent in (1_u64..=1022).step_by(8) {
        let base = exponent << 52;
        for mantissa in [0_u64, 1, 1_u64 << 51, (1_u64 << 52) - 1] {
            insert(&mut rows, base | mantissa, "binade_controls");
        }
    }

    let mut probes = Vec::with_capacity(rows.len() * 2);
    let mut meta =
        String::from("id,class,input_bits,frozen_bits,old_bits,cubic_bits,ratio_bits,pair_bits\n");
    let mut ordinal = 0_usize;
    for (bits, class) in rows {
        let magnitude = f64::from_bits(bits);
        for sign in [1.0_f64, -1.0_f64] {
            let x = sign * magnitude;
            let input_bits = x.to_bits();
            let frozen_bits = frozen(x).to_bits();
            let old_bits = old_production(x).to_bits();
            let cubic_bits = cubic(x).to_bits();
            let ratio_bits = ratio_x87(x).to_bits();
            let pair_bits = pair_x87(x).to_bits();
            let id = format!("atanh-ho-{ordinal:05}");
            ordinal += 1;
            probes.push(json!({
                "probe": {"id": id, "args": [format!("0x{input_bits:016x}")]},
                "distinct_outputs": 2,
                "outputs": [
                    {"candidate_id": "frozen_three_regime", "kind": "Number", "bits": format!("0x{frozen_bits:016x}")},
                    {"candidate_id": "old_pair_ratio", "kind": "Number", "bits": format!("0x{old_bits:016x}")}
                ]
            }));
            meta.push_str(&format!(
                "{id},{class},0x{input_bits:016x},0x{frozen_bits:016x},0x{old_bits:016x},0x{cubic_bits:016x},0x{ratio_bits:016x},0x{pair_bits:016x}\n"
            ));
        }
    }

    let dir = output_dir();
    std::fs::create_dir_all(&dir).expect("create output directory");
    let batch_path = dir.join("batch-atanh-three-regime-heldout-20260809.json");
    let meta_path = dir.join("meta-atanh-three-regime-heldout-20260809.csv");
    let batch = json!({
        "function": "ATANH",
        "row_id": "G4-02",
        "generated_utc": "2026-08-09",
        "frozen_threshold_bits": format!("0x{THRESHOLD_BITS:016x}"),
        "selection": "oracle-blind broad bits + disjoint model disagreements + boundary adjacent + binade controls",
        "probes": probes
    });
    std::fs::write(&batch_path, serde_json::to_vec_pretty(&batch).unwrap()).expect("write batch");
    std::fs::write(&meta_path, meta).expect("write metadata");
    println!("{} probes", ordinal);
    println!("{}", batch_path.display());
    println!("{}", meta_path.display());
}
