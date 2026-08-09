//! Freeze the fresh v2 COMBIN/COMBINA admission publication gate.
//!
//! V1 was retired after exposing shared COMBIN DAZ behavior. This generator
//! uses new fractional and subnormal bit patterns, rejects overlap with every
//! prior batch including v1, and reads no Excel answers.

use oxfunc_core::functions::combin::combin_kernel;
use oxfunc_core::functions::combina::combina_kernel;
use oxfunc_core::value::WorksheetErrorCode;
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

const COMBIN_MAX_N: f64 = 2_147_483_646.0;

const PRIOR_COMBIN: &[&str] = &[
    "smart-fuzzer/work/w109/G4-04-combin/batch-r1-excel.json",
    "smart-fuzzer/work/w109/G4-04-combin/batch-current-discovery-v1.json",
    "smart-fuzzer/work/w109/G4-04-combin/batch-cyclic-heldout-v1.json",
    "smart-fuzzer/work/w109/G4-04-combina/batch-combin-transformed-boundary-control-discovery-v1.json",
    "smart-fuzzer/work/w109/G4-04-combina/batch-combin-admission-heldout-v1.json",
];

const PRIOR_COMBINA: &[&str] = &[
    "smart-fuzzer/work/w109/G4-04-combina/batch-combina-identity-discovery-v1.json",
    "smart-fuzzer/work/w109/G4-04-combina/batch-combina-identity-heldout-v1.json",
    "smart-fuzzer/work/w109/G4-04-combina/batch-combina-transform-edge-discovery-v1.json",
    "smart-fuzzer/work/w109/G4-04-combina/batch-combina-admission-boundary-discovery-v1.json",
    "smart-fuzzer/work/w109/G4-04-combina/batch-combina-admission-heldout-v1.json",
];

#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
struct Pair(u64, u64);

#[derive(Deserialize)]
struct PriorBatch {
    probes: Vec<PriorProbe>,
}

#[derive(Deserialize)]
struct PriorProbe {
    probe: PriorArgs,
}

#[derive(Deserialize)]
struct PriorArgs {
    args: [String; 2],
}

#[derive(Clone)]
struct Row {
    id: String,
    family: &'static str,
    n: f64,
    k: f64,
}

fn bits(value: f64) -> String {
    format!("0x{:016x}", value.to_bits())
}

fn parse_bits(raw: &str) -> Result<u64, String> {
    u64::from_str_radix(
        raw.strip_prefix("0x")
            .ok_or_else(|| format!("invalid bit string: {raw}"))?,
        16,
    )
    .map_err(|error| error.to_string())
}

fn prior_pairs(paths: &[&str]) -> Result<BTreeSet<Pair>, String> {
    let mut pairs = BTreeSet::new();
    for raw_path in paths {
        let path = Path::new(raw_path);
        let text = fs::read_to_string(path)
            .map_err(|error| format!("cannot read prior {}: {error}", path.display()))?;
        let batch: PriorBatch = serde_json::from_str(&text)
            .map_err(|error| format!("cannot parse prior {}: {error}", path.display()))?;
        for row in batch.probes {
            pairs.insert(Pair(
                parse_bits(&row.probe.args[0])?,
                parse_bits(&row.probe.args[1])?,
            ));
        }
    }
    Ok(pairs)
}

fn encode(result: Result<f64, WorksheetErrorCode>) -> String {
    match result {
        Ok(value) => bits(value),
        Err(error) => format!("error:{error:?}"),
    }
}

fn combina_control(
    n: f64,
    k: f64,
    daz: bool,
    early_negative: bool,
) -> Result<f64, WorksheetErrorCode> {
    let n = if daz && n.is_subnormal() {
        0.0_f64.copysign(n)
    } else {
        n
    };
    let k = if daz && k.is_subnormal() {
        0.0_f64.copysign(k)
    } else {
        k
    };
    let tn = n.trunc();
    let tk = k.trunc();
    if early_negative && (tn < 0.0 || k < 0.0) {
        return Err(WorksheetErrorCode::Num);
    }
    if tn == 0.0 && tk == 0.0 {
        return Ok(1.0);
    }
    if tn < 0.0 || k < 0.0 {
        return Err(WorksheetErrorCode::Num);
    }
    combin_kernel(tn + tk - 1.0, tk)
}

fn add(
    rows: &mut Vec<Row>,
    seen: &mut BTreeSet<Pair>,
    prior: &BTreeSet<Pair>,
    counts: &mut BTreeMap<&'static str, usize>,
    family: &'static str,
    n: f64,
    k: f64,
) -> Result<(), String> {
    let pair = Pair(n.to_bits(), k.to_bits());
    if prior.contains(&pair) {
        return Err(format!("v2 overlaps prior: n={} k={}", bits(n), bits(k)));
    }
    if !seen.insert(pair) {
        return Ok(());
    }
    rows.push(Row {
        id: format!("{family}-n-{:016x}-k-{:016x}", n.to_bits(), k.to_bits()),
        family,
        n,
        k,
    });
    *counts.entry(family).or_default() += 1;
    Ok(())
}

fn freeze(
    out_dir: &Path,
    function: &str,
    stem: &str,
    rows: &[Row],
    counts: &BTreeMap<&'static str, usize>,
) -> Result<(), String> {
    let batch_path = out_dir.join(format!("batch-{stem}.json"));
    let meta_path = out_dir.join(format!("batch-{stem}.meta.json"));
    let predictions_path = out_dir.join(format!("predictions-{stem}.json"));
    let probes: Vec<Value> = rows
        .iter()
        .map(|row| {
            json!({
                "probe": { "id": row.id, "args": [bits(row.n), bits(row.k)] },
                "distinct_outputs": 0,
                "outputs": []
            })
        })
        .collect();
    let predictions: Vec<Value> = rows
        .iter()
        .map(|row| {
            let selected = if function == "COMBIN" {
                combin_kernel(row.n, row.k)
            } else {
                combina_kernel(row.n, row.k)
            };
            let controls = if function == "COMBIN" {
                let no_daz = if (row.n.is_subnormal() && row.n.is_sign_negative())
                    || (row.k.is_subnormal() && row.k.is_sign_negative())
                {
                    Err(WorksheetErrorCode::Num)
                } else {
                    selected
                };
                let raw_ceiling = if row.n > COMBIN_MAX_N {
                    Err(WorksheetErrorCode::Num)
                } else {
                    selected
                };
                json!({
                    "no_daz": encode(no_daz),
                    "raw_pretrunc_ceiling": encode(raw_ceiling)
                })
            } else {
                json!({
                    "no_daz": encode(combina_control(row.n, row.k, false, false)),
                    "negative_guard_before_zero_pool": encode(combina_control(row.n, row.k, true, true)),
                    "raw_pretrunc_transform": encode(combin_kernel(row.n + row.k - 1.0, row.k))
                })
            };
            json!({
                "id": row.id,
                "family": row.family,
                "args": [bits(row.n), bits(row.k)],
                "selected": encode(selected),
                "controls": controls
            })
        })
        .collect();
    write_json(
        &batch_path,
        &json!({ "function": function, "row_id": format!("G4-04-{stem}"), "probes": probes }),
    )?;
    write_json(
        &predictions_path,
        &json!({
            "function": function,
            "status": "frozen fresh v2 answer-blind candidate and controls",
            "predictions": predictions
        }),
    )?;
    write_json(
        &meta_path,
        &json!({
            "schema_version": 2,
            "batch": batch_path.file_name().and_then(|name| name.to_str()),
            "predictions": predictions_path.file_name().and_then(|name| name.to_str()),
            "generator": "smart-fuzzer/tools/calc_graph_racer/src/bin/generate_combina_admission_heldout_v2.rs",
            "selection": "fresh answer-blind v2 after retiring v1; disjoint from every listed prior batch",
            "counts": counts,
            "total_rows": rows.len(),
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
                "oracle_answers_used_for_selection": false,
                "candidate_frozen_before_oracle_capture": true,
                "v1_rows_excluded": true
            }
        }),
    )?;
    Ok(())
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
    fs::create_dir_all(&out_dir).map_err(|error| error.to_string())?;
    let prior_combin = prior_pairs(PRIOR_COMBIN)?;
    let prior_combina = prior_pairs(PRIOR_COMBINA)?;

    let mut combin = Vec::new();
    let mut combin_seen = BTreeSet::new();
    let mut combin_counts = BTreeMap::new();
    for base in [
        2_147_483_645.0_f64,
        2_147_483_646.0,
        2_147_483_647.0,
        2_147_483_648.0,
    ] {
        for delta in [0.125_f64, 0.375, 0.625, 0.875] {
            for k in [0.0_f64, 1.0, 2.0] {
                add(
                    &mut combin,
                    &mut combin_seen,
                    &prior_combin,
                    &mut combin_counts,
                    "combin-admission-heldout-v2-fractional-ceiling",
                    base + delta,
                    k,
                )?;
            }
        }
    }
    for (n, k) in [
        (2_147_483_646.125_f64, 2_147_483_644.875_f64),
        (2_147_483_646.625, 2_147_483_645.375),
        (2_147_483_646.125, 1_073_741_822.375),
        (2_147_483_646.625, 1_073_741_823.625),
    ] {
        add(
            &mut combin,
            &mut combin_seen,
            &prior_combin,
            &mut combin_counts,
            "combin-admission-heldout-v2-complement-central",
            n,
            k,
        )?;
    }
    for tiny_bits in [
        3_u64,
        4,
        0x0004_0000_0000_0000,
        0x000c_0000_0000_0000,
        0x000f_ffff_ffff_fffe,
        0x0010_0000_0000_0002,
    ] {
        let tiny = f64::from_bits(tiny_bits);
        for (n, k) in [(-tiny, 0.375), (tiny, 0.375), (1.375, -tiny), (1.375, tiny)] {
            add(
                &mut combin,
                &mut combin_seen,
                &prior_combin,
                &mut combin_counts,
                "combin-admission-heldout-v2-daz",
                n,
                k,
            )?;
        }
    }

    let mut combina = Vec::new();
    let mut combina_seen = BTreeSet::new();
    let mut combina_counts = BTreeMap::new();
    for base in [
        2_147_483_645.0_f64,
        2_147_483_646.0,
        2_147_483_647.0,
        2_147_483_648.0,
    ] {
        for delta in [0.125_f64, 0.375, 0.625, 0.875] {
            for k in [0.0_f64, 1.0, 2.0] {
                add(
                    &mut combina,
                    &mut combina_seen,
                    &prior_combina,
                    &mut combina_counts,
                    "combina-admission-heldout-v2-fractional-n-ceiling",
                    base + delta,
                    k,
                )?;
            }
            for n in [0.375_f64, 1.375, 2.375] {
                add(
                    &mut combina,
                    &mut combina_seen,
                    &prior_combina,
                    &mut combina_counts,
                    "combina-admission-heldout-v2-fractional-k-ceiling",
                    n,
                    base + delta,
                )?;
            }
        }
    }
    for (n, k) in [
        (1_073_741_822.375_f64, 1_073_741_824.625_f64),
        (1_073_741_823.625, 1_073_741_823.375),
        (1_073_741_824.375, 1_073_741_822.625),
        (2.375, 2_147_483_644.625),
    ] {
        add(
            &mut combina,
            &mut combina_seen,
            &prior_combina,
            &mut combina_counts,
            "combina-admission-heldout-v2-complement-central",
            n,
            k,
        )?;
    }
    for tiny_bits in [
        3_u64,
        4,
        0x0004_0000_0000_0000,
        0x000c_0000_0000_0000,
        0x000f_ffff_ffff_fffe,
        0x0010_0000_0000_0002,
    ] {
        let tiny = f64::from_bits(tiny_bits);
        for (n, k) in [(-tiny, 1.375), (tiny, 1.375), (1.375, -tiny), (1.375, tiny)] {
            add(
                &mut combina,
                &mut combina_seen,
                &prior_combina,
                &mut combina_counts,
                "combina-admission-heldout-v2-daz",
                n,
                k,
            )?;
        }
    }
    for n in [-0.9375_f64, -0.3125, 0.0625, 0.6875] {
        for k in [-0.6875_f64, -0.0625, 0.3125, 0.9375] {
            add(
                &mut combina,
                &mut combina_seen,
                &prior_combina,
                &mut combina_counts,
                "combina-admission-heldout-v2-zero-pool",
                n,
                k,
            )?;
        }
    }
    for (n, k) in [
        (-0.9375_f64, 1.0625_f64),
        (1.0625, -0.9375),
        (-0.3125, 2.0625),
        (2.0625, -0.3125),
    ] {
        add(
            &mut combina,
            &mut combina_seen,
            &prior_combina,
            &mut combina_counts,
            "combina-admission-heldout-v2-guard-order",
            n,
            k,
        )?;
    }

    freeze(
        &out_dir,
        "COMBIN",
        "combin-admission-heldout-v2",
        &combin,
        &combin_counts,
    )?;
    freeze(
        &out_dir,
        "COMBINA",
        "combina-admission-heldout-v2",
        &combina,
        &combina_counts,
    )?;
    println!(
        "froze fresh v2: COMBIN={} COMBINA={}",
        combin.len(),
        combina.len()
    );
    Ok(())
}
