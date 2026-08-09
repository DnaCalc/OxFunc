//! Freeze answer-blind COMBINA admission-boundary discovery batches.
//!
//! The first batch probes COMBINA raw-input, transformed-total, signed-zero,
//! and separate-truncation guards.  The second batch probes the corresponding
//! `COMBIN(trunc(n) + trunc(k) - 1, trunc(k))` controls.  A separate manifest
//! maps every COMBINA row with a nonnegative finite transformed pair to its
//! deduplicated COMBIN control.  No Excel answers are read by this generator.

use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
struct Pair(u64, u64);

#[derive(Clone)]
struct RawRow {
    id: String,
    n: f64,
    k: f64,
}

fn bits(value: f64) -> String {
    format!("0x{:016x}", value.to_bits())
}

fn probe(id: &str, n: f64, k: f64) -> Value {
    json!({
        "probe": { "id": id, "args": [bits(n), bits(k)] },
        "distinct_outputs": 0,
        "outputs": []
    })
}

fn write_json(path: &Path, value: &Value) -> Result<(), String> {
    let mut text = serde_json::to_string_pretty(value).map_err(|error| error.to_string())?;
    text.push('\n');
    fs::write(path, text).map_err(|error| format!("{}: {error}", path.display()))
}

fn add_integer_band(values: &mut BTreeSet<u64>, start: u64, end: u64) {
    values.extend(start..=end);
}

fn integer_candidates() -> Vec<f64> {
    let mut integers = BTreeSet::new();
    add_integer_band(&mut integers, 0, 16);
    add_integer_band(&mut integers, 158, 182);
    add_integer_band(&mut integers, 248, 264);
    add_integer_band(&mut integers, 500, 524);
    add_integer_band(&mut integers, 1000, 1060);
    add_integer_band(&mut integers, 2040, 2056);
    for center in [
        32_u64,
        64,
        128,
        256,
        512,
        1024,
        2048,
        32_768,
        65_536,
        1_048_576,
        2_147_483_648,
        4_294_967_296,
    ] {
        for offset in [-2_i64, -1, 0, 1, 2] {
            if let Some(value) = center.checked_add_signed(offset) {
                integers.insert(value);
            }
        }
    }
    for value in [
        10_000_u64,
        100_000,
        1_000_000,
        10_000_000,
        100_000_000,
        1_000_000_000,
        10_000_000_000,
        1_000_000_000_000,
        1_000_000_000_000_000,
    ] {
        integers.insert(value);
    }

    let mut values: Vec<f64> = integers.into_iter().map(|value| value as f64).collect();
    let two53 = 9_007_199_254_740_992.0_f64;
    for raw in [
        f64::from_bits(two53.to_bits() - 3),
        f64::from_bits(two53.to_bits() - 2),
        f64::from_bits(two53.to_bits() - 1),
        two53,
        f64::from_bits(two53.to_bits() + 1),
        f64::from_bits(two53.to_bits() + 2),
        f64::from_bits(two53.to_bits() + 3),
    ] {
        values.push(raw);
    }
    values.sort_by_key(|value| value.to_bits());
    values.dedup_by_key(|value| value.to_bits());
    values
}

fn main() -> Result<(), String> {
    let out_dir = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("smart-fuzzer/work/w109/G4-04-combina"));
    fs::create_dir_all(&out_dir)
        .map_err(|error| format!("cannot create {}: {error}", out_dir.display()))?;

    let mut rows = Vec::<RawRow>::new();
    let mut seen = BTreeSet::<Pair>::new();
    let mut family_counts = BTreeMap::<&'static str, usize>::new();
    let mut add = |family: &'static str, label: String, n: f64, k: f64| {
        if !n.is_finite() || !k.is_finite() {
            return Err(format!("non-finite row attempted: {family}/{label}"));
        }
        if !seen.insert(Pair(n.to_bits(), k.to_bits())) {
            return Ok(());
        }
        rows.push(RawRow {
            id: format!("combina-boundary-{family}-{label}"),
            n,
            k,
        });
        *family_counts.entry(family).or_default() += 1;
        Ok(())
    };

    let integer_values = integer_candidates();
    for &n in &integer_values {
        for k in [0.0, 1.0, 2.0] {
            add(
                "raw-n-small-k",
                format!("n-{:016x}-k-{}", n.to_bits(), k as u32),
                n,
                k,
            )?;
        }
    }
    for &k in &integer_values {
        for n in [0.0, 1.0, 2.0] {
            add(
                "small-n-raw-k",
                format!("n-{}-k-{:016x}", n as u32, k.to_bits()),
                n,
                k,
            )?;
        }
    }

    // Hold the transformed total fixed while changing the raw decomposition.
    // This is the direct discriminator between a raw-input guard and inherited
    // COMBIN(total,k) admission.
    let mut totals = BTreeSet::new();
    add_integer_band(&mut totals, 158, 182);
    add_integer_band(&mut totals, 1000, 1060);
    for total in [
        255_u64, 256, 257, 511, 512, 513, 1023, 1024, 1025, 2047, 2048, 2049,
    ] {
        totals.insert(total);
    }
    for total in totals {
        let mut choices = BTreeSet::from([0_u64, 1, 2, 3, 10, 64, 128, 256, 512]);
        choices.insert(total / 2);
        choices.insert(total.saturating_sub(2));
        choices.insert(total.saturating_sub(1));
        choices.insert(total);
        for k in choices.into_iter().filter(|&k| k <= total) {
            let n = total - k + 1;
            add(
                "fixed-transformed-total",
                format!("t-{total}-n-{n}-k-{k}"),
                n as f64,
                k as f64,
            )?;
        }
    }

    // Separate-truncation and sign/order checks around the output-overflow
    // vicinity and the suspected input/total boundary.
    for base in [
        0_i64, 1, 2, 168, 169, 170, 171, 172, 1027, 1028, 1029, 1030, 1031, 1032,
    ] {
        for n_delta in [-0.75, -0.25, 0.25, 0.75] {
            let n = base as f64 + n_delta;
            for k in [
                -1.25_f64, -1.0, -0.75, -0.25, -0.0, 0.0, 0.25, 0.75, 1.0, 1.25, 1.75, 2.0, 2.25,
                2.75,
            ] {
                add(
                    "fractional-guard-order",
                    format!("n-{:016x}-k-{:016x}", n.to_bits(), k.to_bits()),
                    n,
                    k,
                )?;
            }
        }
    }

    let signed_values = [
        -2.0,
        -1.75,
        -1.25,
        -1.0,
        -0.999_999_999_999_999_9,
        -0.75,
        -0.25,
        -f64::MIN_POSITIVE,
        -f64::from_bits(1),
        -0.0,
        0.0,
        f64::from_bits(1),
        f64::MIN_POSITIVE,
        0.25,
        0.75,
        1.0,
    ];
    for n in signed_values {
        for k in signed_values {
            add(
                "signed-zero-domain-matrix",
                format!("n-{:016x}-k-{:016x}", n.to_bits(), k.to_bits()),
                n,
                k,
            )?;
        }
    }

    let combina_probes: Vec<Value> = rows
        .iter()
        .map(|row| probe(&row.id, row.n, row.k))
        .collect();

    let mut control_seen = BTreeMap::<Pair, String>::new();
    let mut control_rows = Vec::<(String, f64, f64)>::new();
    let mut pairings = Vec::<Value>::new();
    for row in &rows {
        let n = row.n.trunc();
        let k = row.k.trunc();
        let total = n + k - 1.0;
        if n < 0.0 || k < 0.0 || !total.is_finite() || total < 0.0 {
            pairings.push(json!({
                "combina_id": row.id,
                "control_id": null,
                "reason": "no nonnegative finite transformed COMBIN pair"
            }));
            continue;
        }
        let key = Pair(total.to_bits(), k.to_bits());
        let control_id = if let Some(id) = control_seen.get(&key) {
            id.clone()
        } else {
            let id = format!(
                "combin-boundary-control-total-{:016x}-k-{:016x}",
                total.to_bits(),
                k.to_bits()
            );
            control_seen.insert(key, id.clone());
            control_rows.push((id.clone(), total, k));
            id
        };
        pairings.push(json!({
            "combina_id": row.id,
            "control_id": control_id,
            "truncated_n": bits(n),
            "truncated_k": bits(k),
            "transformed_total": bits(total)
        }));
    }
    let combin_probes: Vec<Value> = control_rows
        .iter()
        .map(|(id, total, k)| probe(id, *total, *k))
        .collect();

    let combina_batch = out_dir.join("batch-combina-admission-boundary-discovery-v1.json");
    let combina_meta = out_dir.join("batch-combina-admission-boundary-discovery-v1.meta.json");
    let combin_batch = out_dir.join("batch-combin-transformed-boundary-control-discovery-v1.json");
    let combin_meta =
        out_dir.join("batch-combin-transformed-boundary-control-discovery-v1.meta.json");
    let pairing_path = out_dir.join("pairing-combina-combin-boundary-discovery-v1.json");

    write_json(
        &combina_batch,
        &json!({
            "function": "COMBINA",
            "row_id": "G4-04-combina-admission-boundary-discovery-v1",
            "probes": combina_probes
        }),
    )?;
    write_json(
        &combin_batch,
        &json!({
            "function": "COMBIN",
            "row_id": "G4-04-combin-transformed-boundary-control-discovery-v1",
            "probes": combin_probes
        }),
    )?;
    write_json(
        &pairing_path,
        &json!({
            "schema_version": 1,
            "combina_batch": combina_batch.file_name().and_then(|name| name.to_str()),
            "combin_control_batch": combin_batch.file_name().and_then(|name| name.to_str()),
            "selection": "deterministic answer-blind transformed-pair mapping",
            "pairings": pairings
        }),
    )?;

    let capture_contract = json!({
        "runner": "smart-fuzzer/tools/Run-W109BulkBatch.ps1",
        "input_plumbing": "Range.Value2 bulk matrix with cell-reference Formula2R1C1",
        "result_capture": "Range.Value2 bulk result column",
        "cache_mode": "NoCache",
        "required_reference_profile": "Excel 16.0 build 20228 x64 / CV2"
    });
    write_json(
        &combina_meta,
        &json!({
            "schema_version": 1,
            "batch": combina_batch.file_name().and_then(|name| name.to_str()),
            "generator": "smart-fuzzer/tools/calc_graph_racer/src/bin/generate_combina_admission_boundary.rs",
            "selection": "answer-blind finite boundary grid; contains no Excel answers",
            "purpose": [
                "locate COMBINA raw n and raw k admission boundaries",
                "locate transformed-total boundary near 170 and 1029/1030",
                "distinguish raw-input guards from transformed COMBIN admission",
                "pin signed-zero, negative fractional, and separate-truncation guard order",
                "bracket 2^31 and 2^53 conversion boundaries"
            ],
            "counts": family_counts,
            "total_rows": rows.len(),
            "pairing_manifest": pairing_path.file_name().and_then(|name| name.to_str()),
            "excel_capture_contract": capture_contract,
            "invariants": {
                "unique_ids": true,
                "unique_raw_argument_bit_pairs": true,
                "all_arguments_finite": true,
                "oracle_answers_used_for_selection": false
            }
        }),
    )?;
    write_json(
        &combin_meta,
        &json!({
            "schema_version": 1,
            "batch": combin_batch.file_name().and_then(|name| name.to_str()),
            "generator": "smart-fuzzer/tools/calc_graph_racer/src/bin/generate_combina_admission_boundary.rs",
            "selection": "deduplicated transformed controls derived without Excel answers",
            "purpose": [
                "distinguish inherited COMBIN(total,k) admission from a COMBINA-specific raw-input guard",
                "keep the signed-off central COMBIN publication graph separate from COMBINA wrapper discovery"
            ],
            "total_rows": control_rows.len(),
            "pairing_manifest": pairing_path.file_name().and_then(|name| name.to_str()),
            "excel_capture_contract": capture_contract,
            "invariants": {
                "unique_ids": true,
                "unique_raw_argument_bit_pairs": true,
                "all_arguments_finite": true,
                "oracle_answers_used_for_selection": false
            }
        }),
    )?;

    println!(
        "{} COMBINA rows, {} COMBIN controls -> {}, {}, {}",
        rows.len(),
        control_rows.len(),
        combina_batch.display(),
        combin_batch.display(),
        pairing_path.display()
    );
    Ok(())
}
