//! Freeze a finite-input COMBINA transform/conversion edge discovery batch.
//!
//! This batch is independent of the central graph selection. It targets the
//! uncharacterized seam before the cyclic COMBIN body: separate truncation,
//! large-integer conversion, `n+k-1` construction, and early `k=0/1/2/3`
//! publication around binary64 and signed-64-bit boundaries. It contains no
//! Excel answers and deliberately excludes NaN/infinity, which cannot be
//! injected into worksheet cells through the required Range.Value2 path.

use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
struct Pair(u64, u64);

fn bits(value: f64) -> String {
    format!("0x{:016x}", value.to_bits())
}

fn probe(id: String, n: f64, k: f64) -> Value {
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

fn main() -> Result<(), String> {
    let out_dir = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("smart-fuzzer/work/w109/G4-04-combina"));
    fs::create_dir_all(&out_dir)
        .map_err(|error| format!("cannot create {}: {error}", out_dir.display()))?;

    let two52 = f64::from_bits(0x4330_0000_0000_0000);
    let two53 = f64::from_bits(0x4340_0000_0000_0000);
    let two63 = f64::from_bits(0x43e0_0000_0000_0000);
    let sqrt_max = f64::MAX.sqrt();
    let n_values = [
        ("zero", 0.0),
        ("one", 1.0),
        ("two", 2.0),
        ("two52-prev", f64::from_bits(two52.to_bits() - 1)),
        ("two52", two52),
        ("two52-next", f64::from_bits(two52.to_bits() + 1)),
        ("two53-prev2", f64::from_bits(two53.to_bits() - 2)),
        ("two53-prev", f64::from_bits(two53.to_bits() - 1)),
        ("two53", two53),
        ("two53-next", f64::from_bits(two53.to_bits() + 1)),
        ("two53-next2", f64::from_bits(two53.to_bits() + 2)),
        ("two63-prev2", f64::from_bits(two63.to_bits() - 2)),
        ("two63-prev", f64::from_bits(two63.to_bits() - 1)),
        ("two63", two63),
        ("two63-next", f64::from_bits(two63.to_bits() + 1)),
        ("two63-next2", f64::from_bits(two63.to_bits() + 2)),
        ("one-e100", 1.0e100),
        ("sqrt-max-prev", f64::from_bits(sqrt_max.to_bits() - 1)),
        ("sqrt-max", sqrt_max),
        ("sqrt-max-next", f64::from_bits(sqrt_max.to_bits() + 1)),
        ("one-e200", 1.0e200),
        ("one-e308", 1.0e308),
        ("max-prev", f64::from_bits(f64::MAX.to_bits() - 1)),
        ("max", f64::MAX),
    ];
    let small_k = [0.0, 1.0, 2.0, 3.0, 10.0];

    let huge_k_values = [
        ("two52-prev", f64::from_bits(two52.to_bits() - 1)),
        ("two52", two52),
        ("two52-next", f64::from_bits(two52.to_bits() + 1)),
        ("two53-prev", f64::from_bits(two53.to_bits() - 1)),
        ("two53", two53),
        ("two53-next", f64::from_bits(two53.to_bits() + 1)),
        ("two63-prev", f64::from_bits(two63.to_bits() - 1)),
        ("two63", two63),
        ("two63-next", f64::from_bits(two63.to_bits() + 1)),
        ("one-e100", 1.0e100),
        ("one-e200", 1.0e200),
        ("max", f64::MAX),
    ];

    let mut probes = Vec::new();
    let mut seen = BTreeSet::new();
    let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
    let mut add = |family: &'static str, label: String, n: f64, k: f64| {
        if !n.is_finite() || !k.is_finite() {
            return Err(format!("non-finite row attempted: {label}"));
        }
        if !seen.insert(Pair(n.to_bits(), k.to_bits())) {
            return Ok(());
        }
        let id = format!("combina-edge-{family}-{label}");
        probes.push(probe(id, n, k));
        *counts.entry(family).or_default() += 1;
        Ok(())
    };

    for (n_label, n) in n_values {
        for k in small_k {
            add(
                "large-n-small-k",
                format!("n-{n_label}-k-{}", k as u32),
                n,
                k,
            )?;
        }
    }
    for (k_label, k) in huge_k_values {
        for n in [0.0, 1.0, 2.0, 3.0, 10.0] {
            add(
                "small-n-large-k",
                format!("n-{}-k-{k_label}", n as u32),
                n,
                k,
            )?;
        }
    }

    // Exact representable fractional/truncation points at the last binary64
    // ranges where quarter/half fractions survive.
    for n in [
        f64::from_bits(two52.to_bits() - 2),
        f64::from_bits(two52.to_bits() - 1),
        two52,
    ] {
        for delta in [-0.5, 0.5] {
            for k in [0.75, 1.75, 2.75] {
                add(
                    "large-fractional-truncation",
                    format!(
                        "n-{:016x}-delta-{}-k-{}",
                        n.to_bits(),
                        if delta < 0.0 {
                            "minus-half"
                        } else {
                            "plus-half"
                        },
                        k
                    ),
                    n + delta,
                    k,
                )?;
            }
        }
    }

    // Negative finite and signed-zero guards around the source conversion.
    for n in [-0.0_f64, -0.25, -1.0] {
        for k in [0.0, 1.0, 2.0] {
            add(
                "sign-domain-guard",
                format!("n-{:016x}-k-{}", n.to_bits(), k as u32),
                n,
                k,
            )?;
        }
    }
    for k in [-0.0_f64, -0.25, -1.0] {
        for n in [0.0, 1.0, two53, two63] {
            add(
                "sign-domain-guard",
                format!("n-{:016x}-k-{:016x}", n.to_bits(), k.to_bits()),
                n,
                k,
            )?;
        }
    }

    let batch_path = out_dir.join("batch-combina-transform-edge-discovery-v1.json");
    let meta_path = out_dir.join("batch-combina-transform-edge-discovery-v1.meta.json");
    write_json(
        &batch_path,
        &json!({
            "function": "COMBINA",
            "row_id": "G4-04-combina-transform-edge-discovery-v1",
            "probes": probes
        }),
    )?;
    write_json(
        &meta_path,
        &json!({
            "schema_version": 1,
            "batch": batch_path.file_name().and_then(|name| name.to_str()),
            "generator": "smart-fuzzer/tools/calc_graph_racer/src/bin/generate_combina_transform_edges.rs",
            "selection": "answer-blind finite edge grid; contains no Excel answers",
            "purpose": [
                "pin separate truncation and large integer conversion around 2^52, 2^53, and 2^63",
                "pin n+k-1 transform overflow/rounding with k=0/1/2/3/10",
                "pin huge-k complement reductions with n=0/1/2/3/10",
                "pin finite maximum and output-overflow result kinds",
                "record that NaN/infinity are excluded because Range.Value2 cannot represent them as worksheet numeric cells"
            ],
            "counts": counts,
            "total_rows": probes.len(),
            "excel_capture_contract": {
                "runner": "smart-fuzzer/tools/Run-W109BulkBatch.ps1",
                "input_plumbing": "Range.Value2 bulk matrix with cell-reference Formula2R1C1",
                "result_capture": "Range.Value2 bulk result column",
                "cache_mode": "NoCache",
                "required_reference_profile": "Excel 16.0 build 20228 x64 / CV2"
            },
            "invariants": {
                "unique_ids": true,
                "unique_raw_argument_bit_pairs": true,
                "all_arguments_finite": true,
                "oracle_answers_used_for_selection": false
            }
        }),
    )?;
    println!(
        "{} finite edge rows -> {}, {}",
        probes.len(),
        batch_path.display(),
        meta_path.display()
    );
    Ok(())
}
