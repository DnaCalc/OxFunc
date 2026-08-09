//! Freeze disjoint answer-blind COMBIN/COMBINA admission publication heldouts.

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
];

const PRIOR_COMBINA: &[&str] = &[
    "smart-fuzzer/work/w109/G4-04-combina/batch-combina-identity-discovery-v1.json",
    "smart-fuzzer/work/w109/G4-04-combina/batch-combina-identity-heldout-v1.json",
    "smart-fuzzer/work/w109/G4-04-combina/batch-combina-transform-edge-discovery-v1.json",
    "smart-fuzzer/work/w109/G4-04-combina/batch-combina-admission-boundary-discovery-v1.json",
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

fn combina_model(
    n: f64,
    k: f64,
    daz: bool,
    early_negative: bool,
) -> Result<f64, WorksheetErrorCode> {
    if !n.is_finite() || !k.is_finite() {
        return Err(WorksheetErrorCode::Num);
    }
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

fn probe(row: &Row) -> Value {
    json!({
        "probe": { "id": row.id, "args": [bits(row.n), bits(row.k)] },
        "distinct_outputs": 0,
        "outputs": []
    })
}

fn prediction(function: &str, row: &Row) -> Value {
    let selected = if function == "COMBIN" {
        combin_kernel(row.n, row.k)
    } else {
        combina_kernel(row.n, row.k)
    };
    let controls = if function == "COMBIN" {
        json!({
            "raw_pretrunc_ceiling": encode(if row.n > COMBIN_MAX_N {
                Err(WorksheetErrorCode::Num)
            } else {
                combin_kernel(row.n, row.k)
            })
        })
    } else {
        let raw_pretrunc_transform = if row.n.is_finite() && row.k.is_finite() {
            combin_kernel(row.n + row.k - 1.0, row.k)
        } else {
            Err(WorksheetErrorCode::Num)
        };
        json!({
            "no_daz": encode(combina_model(row.n, row.k, false, false)),
            "negative_guard_before_zero_pool": encode(combina_model(row.n, row.k, true, true)),
            "raw_pretrunc_transform": encode(raw_pretrunc_transform)
        })
    };
    json!({
        "id": row.id,
        "family": row.family,
        "args": [bits(row.n), bits(row.k)],
        "selected": encode(selected),
        "controls": controls
    })
}

fn write_json(path: &Path, value: &Value) -> Result<(), String> {
    let mut text = serde_json::to_string_pretty(value).map_err(|error| error.to_string())?;
    text.push('\n');
    fs::write(path, text).map_err(|error| format!("{}: {error}", path.display()))
}

fn add_row(
    rows: &mut Vec<Row>,
    seen: &mut BTreeSet<Pair>,
    prior: &BTreeSet<Pair>,
    counts: &mut BTreeMap<&'static str, usize>,
    family: &'static str,
    label: String,
    n: f64,
    k: f64,
) -> Result<(), String> {
    let pair = Pair(n.to_bits(), k.to_bits());
    if prior.contains(&pair) {
        return Err(format!(
            "heldout overlaps prior row: n={} k={}",
            bits(n),
            bits(k)
        ));
    }
    if !seen.insert(pair) {
        return Ok(());
    }
    rows.push(Row {
        id: format!("{family}-{label}"),
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
    let prediction_path = out_dir.join(format!("predictions-{stem}.json"));
    write_json(
        &batch_path,
        &json!({
            "function": function,
            "row_id": format!("G4-04-{stem}"),
            "probes": rows.iter().map(probe).collect::<Vec<_>>()
        }),
    )?;
    write_json(
        &prediction_path,
        &json!({
            "function": function,
            "status": "frozen answer-blind production candidate and declared controls",
            "predictions": rows.iter().map(|row| prediction(function, row)).collect::<Vec<_>>()
        }),
    )?;
    write_json(
        &meta_path,
        &json!({
            "schema_version": 1,
            "batch": batch_path.file_name().and_then(|name| name.to_str()),
            "predictions": prediction_path.file_name().and_then(|name| name.to_str()),
            "generator": "smart-fuzzer/tools/calc_graph_racer/src/bin/generate_combina_admission_heldout.rs",
            "selection": "answer-blind after boundary model freeze; disjoint from all listed prior batches",
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
                "candidate_frozen_before_oracle_capture": true
            }
        }),
    )?;
    Ok(())
}

fn main() -> Result<(), String> {
    let out_dir = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("smart-fuzzer/work/w109/G4-04-combina"));
    fs::create_dir_all(&out_dir)
        .map_err(|error| format!("cannot create {}: {error}", out_dir.display()))?;
    let prior_combin = prior_pairs(PRIOR_COMBIN)?;
    let prior_combina = prior_pairs(PRIOR_COMBINA)?;

    let mut combin_rows = Vec::new();
    let mut combin_seen = BTreeSet::new();
    let mut combin_counts = BTreeMap::new();
    for base in [
        2_147_483_645.0_f64,
        2_147_483_646.0,
        2_147_483_647.0,
        2_147_483_648.0,
    ] {
        for delta in [0.25, 0.75] {
            let n = base + delta;
            for k in [0.0_f64, 0.25, 0.75, 1.0, 1.25, 1.75, 2.0, 2.25, 2.75] {
                add_row(
                    &mut combin_rows,
                    &mut combin_seen,
                    &prior_combin,
                    &mut combin_counts,
                    "combin-admission-heldout-fractional-ceiling",
                    format!("n-{:016x}-k-{:016x}", n.to_bits(), k.to_bits()),
                    n,
                    k,
                )?;
            }
        }
    }
    for n in [
        2_147_483_645.25_f64,
        2_147_483_645.75,
        2_147_483_646.25,
        2_147_483_646.75,
    ] {
        let truncated = n.trunc();
        for k in [
            truncated - 1.75,
            truncated - 1.25,
            truncated - 0.75,
            truncated - 0.25,
        ] {
            add_row(
                &mut combin_rows,
                &mut combin_seen,
                &prior_combin,
                &mut combin_counts,
                "combin-admission-heldout-complement",
                format!("n-{:016x}-k-{:016x}", n.to_bits(), k.to_bits()),
                n,
                k,
            )?;
        }
    }
    for k in [
        1_073_741_821.25_f64,
        1_073_741_822.75,
        1_073_741_823.25,
        1_073_741_823.75,
    ] {
        add_row(
            &mut combin_rows,
            &mut combin_seen,
            &prior_combin,
            &mut combin_counts,
            "combin-admission-heldout-large-central",
            format!("n-max-admitted-frac-k-{:016x}", k.to_bits()),
            2_147_483_646.75,
            k,
        )?;
    }
    for tiny_bits in [
        1_u64,
        2,
        0x0008_0000_0000_0000,
        0x000f_ffff_ffff_ffff,
        0x0010_0000_0000_0000,
        0x0010_0000_0000_0001,
    ] {
        let tiny = f64::from_bits(tiny_bits);
        for (n, k) in [(-tiny, 0.25), (tiny, 0.25), (1.25, -tiny), (1.25, tiny)] {
            add_row(
                &mut combin_rows,
                &mut combin_seen,
                &prior_combin,
                &mut combin_counts,
                "combin-admission-heldout-tiny-sign",
                format!("n-{:016x}-k-{:016x}", n.to_bits(), k.to_bits()),
                n,
                k,
            )?;
        }
    }

    let mut combina_rows = Vec::new();
    let mut combina_seen = BTreeSet::new();
    let mut combina_counts = BTreeMap::new();
    for base in [
        2_147_483_645.0_f64,
        2_147_483_646.0,
        2_147_483_647.0,
        2_147_483_648.0,
    ] {
        for delta in [0.25, 0.75] {
            let n = base + delta;
            for k in [0.0_f64, 0.25, 0.75, 1.0, 1.25, 1.75, 2.0, 2.25, 2.75] {
                add_row(
                    &mut combina_rows,
                    &mut combina_seen,
                    &prior_combina,
                    &mut combina_counts,
                    "combina-admission-heldout-fractional-n-ceiling",
                    format!("n-{:016x}-k-{:016x}", n.to_bits(), k.to_bits()),
                    n,
                    k,
                )?;
            }
        }
    }
    for base in [
        2_147_483_645.0_f64,
        2_147_483_646.0,
        2_147_483_647.0,
        2_147_483_648.0,
    ] {
        for delta in [0.25, 0.75] {
            let k = base + delta;
            for n in [0.25_f64, 0.75, 1.25, 1.75, 2.25, 2.75] {
                add_row(
                    &mut combina_rows,
                    &mut combina_seen,
                    &prior_combina,
                    &mut combina_counts,
                    "combina-admission-heldout-fractional-k-ceiling",
                    format!("n-{:016x}-k-{:016x}", n.to_bits(), k.to_bits()),
                    n,
                    k,
                )?;
            }
        }
    }
    for (n, k) in [
        (1_073_741_823.25_f64, 1_073_741_824.75_f64),
        (1_073_741_823.75, 1_073_741_824.25),
        (1_073_741_824.25, 1_073_741_823.75),
        (1_073_741_824.75, 1_073_741_823.25),
    ] {
        add_row(
            &mut combina_rows,
            &mut combina_seen,
            &prior_combina,
            &mut combina_counts,
            "combina-admission-heldout-large-central",
            format!("n-{:016x}-k-{:016x}", n.to_bits(), k.to_bits()),
            n,
            k,
        )?;
    }
    for tiny_bits in [
        1_u64,
        2,
        0x0008_0000_0000_0000,
        0x000f_ffff_ffff_ffff,
        0x0010_0000_0000_0000,
        0x0010_0000_0000_0001,
    ] {
        let tiny = f64::from_bits(tiny_bits);
        for (n, k) in [(-tiny, 1.25), (tiny, 1.25), (1.25, -tiny), (1.25, tiny)] {
            add_row(
                &mut combina_rows,
                &mut combina_seen,
                &prior_combina,
                &mut combina_counts,
                "combina-admission-heldout-daz",
                format!("n-{:016x}-k-{:016x}", n.to_bits(), k.to_bits()),
                n,
                k,
            )?;
        }
    }
    for n in [-0.875_f64, -0.5, -0.125, 0.125, 0.5, 0.875] {
        for k in [-0.875_f64, -0.5, -0.125, 0.125, 0.5, 0.875] {
            add_row(
                &mut combina_rows,
                &mut combina_seen,
                &prior_combina,
                &mut combina_counts,
                "combina-admission-heldout-zero-pool",
                format!("n-{:016x}-k-{:016x}", n.to_bits(), k.to_bits()),
                n,
                k,
            )?;
        }
    }
    for (n, k) in [
        (-0.875_f64, 1.125_f64),
        (-0.125, 1.875),
        (1.125, -0.875),
        (1.875, -0.125),
    ] {
        add_row(
            &mut combina_rows,
            &mut combina_seen,
            &prior_combina,
            &mut combina_counts,
            "combina-admission-heldout-guard-order",
            format!("n-{:016x}-k-{:016x}", n.to_bits(), k.to_bits()),
            n,
            k,
        )?;
    }

    freeze(
        &out_dir,
        "COMBIN",
        "combin-admission-heldout-v1",
        &combin_rows,
        &combin_counts,
    )?;
    freeze(
        &out_dir,
        "COMBINA",
        "combina-admission-heldout-v1",
        &combina_rows,
        &combina_counts,
    )?;
    println!(
        "froze {} COMBIN and {} COMBINA answer-blind heldout rows in {}",
        combin_rows.len(),
        combina_rows.len(),
        out_dir.display()
    );
    Ok(())
}
