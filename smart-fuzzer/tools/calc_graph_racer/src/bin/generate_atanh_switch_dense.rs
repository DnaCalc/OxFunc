//! Generate a deterministic, answer-free ATANH switch-band discovery battery.
//!
//! The legacy W109 corpus exposed a third body between the x87 ln1p-pair and
//! ratio-log regimes.  On every known residual it is exactly the binary64
//! cubic `x + x^3/3`.  This battery maps that body and both publication seams
//! without using any answer during probe selection.

use oxfunc_core::excel_numeric::research as rx;
use serde_json::json;
use std::collections::BTreeMap;
use std::path::PathBuf;

const CW: u16 = 0x133f;

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
    let lo = 5.0e-5_f64.to_bits();
    let hi = 2.0e-4_f64.to_bits();
    let span = hi - lo;
    let mut rows = BTreeMap::new();

    // A regular bit-space grid maps the body transitions without decimal
    // parser artifacts and covers both sides of the old narrow band.
    for index in 0_u64..=1024 {
        let offset = ((span as u128) * (index as u128) / 1024_u128) as u64;
        insert(&mut rows, lo + offset, "bit_grid");
    }

    // A disjoint deterministic pseudo-random sample catches non-contiguous
    // dispatch and mantissa-dependent publication behavior.
    let mut state = 0x6a09_e667_f3bc_c909_u64;
    for _ in 0..1024 {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        insert(&mut rows, lo + state % (span + 1), "random_bits");
    }

    // Adjacent-double ladders around the old safe brackets and the three
    // previously unexplained witnesses.  These rows are selected from inputs
    // alone; the expected outputs remain oracle-blind.
    for center in [
        9.0e-5_f64,
        9.563_088_447_185_053e-5_f64,
        9.999_639_989_730_8e-5_f64,
        1.0e-4_f64,
        1.013_687_375_401_615_6e-4_f64,
        1.05e-4_f64,
        1.1e-4_f64,
    ] {
        let center_bits = center.to_bits();
        for delta in -64_i64..=64 {
            insert(
                &mut rows,
                center_bits.wrapping_add_signed(delta),
                "adjacent_ladder",
            );
        }
    }

    let mut probes = Vec::with_capacity(rows.len() * 2);
    let mut meta =
        String::from("id,class,input_bits,pair_bits,cubic_bits,ratio_bits,distinct_candidates\n");
    let mut ordinal = 0_usize;
    for (bits, class) in rows {
        let magnitude = f64::from_bits(bits);
        for sign in [1.0_f64, -1.0_f64] {
            let x = sign * magnitude;
            let input_bits = x.to_bits();
            let id = format!("atanh-dense-{ordinal:05}");
            ordinal += 1;
            let pair = pair_x87(x).to_bits();
            let cubic = cubic(x).to_bits();
            let ratio = ratio_x87(x).to_bits();
            let mut outputs = [pair, cubic, ratio];
            outputs.sort_unstable();
            let distinct =
                1 + usize::from(outputs[1] != outputs[0]) + usize::from(outputs[2] != outputs[1]);
            probes.push(json!({
                "probe": {"id": id, "args": [format!("0x{input_bits:016x}")]},
                "distinct_outputs": distinct,
                "outputs": [
                    {"candidate_id": "x87_pair", "kind": "Number", "bits": format!("0x{pair:016x}")},
                    {"candidate_id": "binary64_cubic", "kind": "Number", "bits": format!("0x{cubic:016x}")},
                    {"candidate_id": "x87_ratio", "kind": "Number", "bits": format!("0x{ratio:016x}")}
                ]
            }));
            meta.push_str(&format!(
                "{id},{class},0x{input_bits:016x},0x{pair:016x},0x{cubic:016x},0x{ratio:016x},{distinct}\n"
            ));
        }
    }

    let dir = output_dir();
    std::fs::create_dir_all(&dir).expect("create output directory");
    let batch_path = dir.join("batch-atanh-switch-dense-20260809.json");
    let meta_path = dir.join("meta-atanh-switch-dense-20260809.csv");
    let batch = json!({
        "function": "ATANH",
        "row_id": "G4-02",
        "generated_utc": "2026-08-09",
        "selection": "oracle-blind bit grid + deterministic random bits + adjacent ladders",
        "probes": probes
    });
    std::fs::write(&batch_path, serde_json::to_vec_pretty(&batch).unwrap()).expect("write batch");
    std::fs::write(&meta_path, meta).expect("write metadata");
    println!("{} probes", ordinal);
    println!("{}", batch_path.display());
    println!("{}", meta_path.display());
}
