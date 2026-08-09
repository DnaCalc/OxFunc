//! Deterministic, answer-blind W109 COMBINA discovery and held-out generator.
//!
//! The selected hypothesis is the already identified current-reference COMBIN
//! graph applied after COMBINA's *separate* argument truncations:
//!
//! `COMBIN(trunc(n) + trunc(k) - 1, trunc(k))`.
//!
//! This program uses no COMBINA oracle answers for row selection.  Historical
//! cell-reference rows are read only to keep the publication held-out disjoint
//! from evidence already used to motivate the hypothesis.  The output batches
//! contain exact binary64 argument bits and are intended for
//! `Run-W109BulkBatch.ps1 -NoCache`.

use oxfunc_core::excel_numeric::research as rx;
use oxfunc_core::functions::combin::combin_kernel;
use oxfunc_core::functions::combina::combina_kernel;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

const DISCOVERY_TOTAL_MAX: u32 = 256;
const DISCOVERY_FRACTIONAL_ROWS: usize = 2_048;
const HELDOUT_TOTAL_MIN: u32 = 257;
const HELDOUT_TOTAL_MAX: u32 = 1_024;
const HELDOUT_REDUCED_K_MAX: u32 = 128;

const HISTORICAL_COMPARISONS: &[&str] = &[
    "smart-fuzzer/runs/broad-scalar-cycle-010-cellref/comparisons/excel_sample_comparisons.jsonl",
    "smart-fuzzer/runs/broad-scalar-cycle-011-cellref/comparisons/excel_sample_comparisons.jsonl",
    "smart-fuzzer/runs/broad-scalar-cycle-012-cellref/comparisons/excel_sample_comparisons.jsonl",
    "smart-fuzzer/runs/broad-scalar-cycle-013-cellref/comparisons/excel_sample_comparisons.jsonl",
    "smart-fuzzer/runs/broad-scalar-cycle-014-cellref/comparisons/excel_sample_comparisons.jsonl",
    "smart-fuzzer/runs/broad-scalar-cycle-015-cellref/comparisons/excel_sample_comparisons.jsonl",
];

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct RawBits {
    n: u64,
    k: u64,
}

#[derive(Clone, Copy, Debug)]
struct IntegerPair {
    total: u32,
    raw_k: u32,
}

#[derive(Clone, Copy, Debug)]
struct PoolRow {
    pair: IntegerPair,
    n: f64,
    k: f64,
    selected: u64,
    product: u64,
    pc53: u64,
    continuous: u64,
    forward: u64,
}

#[derive(Clone, Debug)]
struct SelectedRow {
    family: &'static str,
    n: f64,
    k: f64,
    total: u32,
    raw_k: u32,
}

fn bits(value: f64) -> String {
    format!("0x{:016x}", value.to_bits())
}

fn parse_formula_args(value: &Value, context: &str) -> Result<(f64, f64), String> {
    let formula = value["formula_text"]
        .as_str()
        .ok_or_else(|| format!("{context}: missing formula_text"))?;
    let body = formula
        .strip_prefix("=COMBINA(")
        .and_then(|text| text.strip_suffix(')'))
        .ok_or_else(|| format!("{context}: unsupported formula {formula}"))?;
    let (n, k) = body
        .split_once(',')
        .ok_or_else(|| format!("{context}: formula does not have two args"))?;
    let n = n
        .trim()
        .parse::<f64>()
        .map_err(|error| format!("{context}: invalid n: {error}"))?;
    let k = k
        .trim()
        .parse::<f64>()
        .map_err(|error| format!("{context}: invalid k: {error}"))?;
    Ok((n, k))
}

fn historical_pairs() -> Result<(BTreeSet<RawBits>, BTreeMap<String, usize>), String> {
    let mut pairs = BTreeSet::new();
    let mut counts = BTreeMap::new();
    for raw_path in HISTORICAL_COMPARISONS {
        let path = Path::new(raw_path);
        let text = fs::read_to_string(path)
            .map_err(|error| format!("historical evidence {}: {error}", path.display()))?;
        let mut count = 0_usize;
        for (line_index, line) in text.lines().enumerate() {
            if line.trim().is_empty() {
                continue;
            }
            let value: Value = serde_json::from_str(line)
                .map_err(|error| format!("{}:{}: {error}", path.display(), line_index + 1))?;
            if value["function_id"].as_str() != Some("FUNC.COMBINA") {
                continue;
            }
            let context = format!("{}:{}", path.display(), line_index + 1);
            let (n, k) = parse_formula_args(&value, &context)?;
            pairs.insert(RawBits {
                n: n.to_bits(),
                k: k.to_bits(),
            });
            count += 1;
        }
        counts.insert((*raw_path).to_owned(), count);
    }
    Ok((pairs, counts))
}

fn mapped_args(pair: IntegerPair) -> (f64, f64) {
    if pair.raw_k == 0 {
        ((pair.total + 1) as f64, 0.0)
    } else {
        ((pair.total - pair.raw_k + 1) as f64, pair.raw_k as f64)
    }
}

fn stable_rank(label: &str, total: u32, raw_k: u32, salt: u64) -> u64 {
    // Fixed FNV-1a followed by SplitMix64 avalanche.  Unlike DefaultHasher,
    // this is stable across Rust releases and process seeds.
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in label
        .bytes()
        .chain(total.to_le_bytes())
        .chain(raw_k.to_le_bytes())
        .chain(salt.to_le_bytes())
    {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    let mut value = hash.wrapping_add(0x9e3779b97f4a7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d049bb133111eb);
    value ^ (value >> 31)
}

fn cyclic_ratio(total: u32, raw_k: u32, mode: u32) -> f64 {
    if raw_k > total {
        return f64::NAN;
    }
    let k = raw_k.min(total - raw_k);
    if k == 0 {
        return 1.0;
    }
    let reverse_loop = mode & 1 != 0;
    let cw = if mode & 2 != 0 {
        rx::CW_PC53_RN
    } else {
        rx::CW_PC64_RN
    };
    let spill_term = mode & 4 != 0;
    let spill_acc = mode & 8 != 0;
    let reciprocal_division = mode & 16 != 0;
    let spill_reciprocal = mode & 32 != 0;
    let n_first = mode & 64 != 0;

    let mut acc = if n_first {
        rx::ext_from_f64(total as f64)
    } else {
        rx::ext_one()
    };
    let mut apply_factor = |j: u32| {
        let numerator = rx::ext_from_f64((total - j) as f64);
        let denominator = rx::ext_from_f64((k - j + 1) as f64);
        let mut term = if reciprocal_division {
            let mut reciprocal = rx::ext_div(&rx::ext_one(), &denominator, cw);
            if spill_reciprocal {
                reciprocal = rx::ext_from_f64(rx::ext_to_f64(&reciprocal, rx::CW_PC64_RN));
            }
            rx::ext_mul(&numerator, &reciprocal, cw)
        } else {
            rx::ext_div(&numerator, &denominator, cw)
        };
        if spill_term {
            term = rx::ext_from_f64(rx::ext_to_f64(&term, rx::CW_PC64_RN));
        }
        acc = rx::ext_mul(&acc, &term, cw);
        if spill_acc {
            acc = rx::ext_from_f64(rx::ext_to_f64(&acc, rx::CW_PC64_RN));
        }
    };
    if reverse_loop {
        for j in (1..k).rev() {
            apply_factor(j);
        }
    } else {
        for j in 1..k {
            apply_factor(j);
        }
    }
    if !n_first {
        acc = rx::ext_mul(&acc, &rx::ext_from_f64(total as f64), cw);
    }
    rx::ext_to_f64(&acc, rx::CW_PC64_RN)
}

fn selected_bits(total: u32, raw_k: u32) -> Result<u64, String> {
    let production = combin_kernel(total as f64, raw_k as f64)
        .map_err(|error| format!("COMBIN({total},{raw_k}) returned {error:?}"))?;
    let independently_spelled = cyclic_ratio(total, raw_k, 13);
    if production.to_bits() != independently_spelled.to_bits() {
        return Err(format!(
            "selected graph spelling disagrees at total={total} k={raw_k}: production={} independent={}",
            bits(production),
            bits(independently_spelled)
        ));
    }
    Ok(production.to_bits())
}

fn product_bits(n: f64, k: f64) -> Result<u64, String> {
    combina_kernel(n, k)
        .map(f64::to_bits)
        .map_err(|error| format!("COMBINA product control ({n},{k}) returned {error:?}"))
}

fn probe(id: String, n: f64, k: f64) -> Value {
    json!({
        "probe": {
            "id": id,
            "args": [bits(n), bits(k)]
        },
        "distinct_outputs": 0,
        "outputs": []
    })
}

fn prediction(id: &str, row: &SelectedRow) -> Result<Value, String> {
    let selected = selected_bits(row.total, row.raw_k)?;
    let product = if row.n.trunc() == 0.0 && row.k.trunc() > 0.0 {
        None
    } else {
        Some(product_bits(row.n, row.k)?)
    };
    let pretrunc = if row.k == 0.0 {
        Some(1.0_f64.to_bits())
    } else {
        combin_kernel(row.n + row.k - 1.0, row.k)
            .ok()
            .map(f64::to_bits)
    };
    Ok(json!({
        "id": id,
        "family": row.family,
        "n_bits": bits(row.n),
        "k_bits": bits(row.k),
        "mapped_total": row.total,
        "mapped_raw_k": row.raw_k,
        "selected_stored_x87_combin_bits": format!("0x{selected:016x}"),
        "production_product_bits": product.map(|value| format!("0x{value:016x}")),
        "worksheet_pretrunc_composition_bits": pretrunc.map(|value| format!("0x{value:016x}")),
        "pc53_control_bits": format!("0x{:016x}", cyclic_ratio(row.total, row.raw_k, 3).to_bits()),
        "continuous_x87_control_bits": format!("0x{:016x}", cyclic_ratio(row.total, row.raw_k, 1).to_bits()),
        "forward_order_control_bits": format!("0x{:016x}", cyclic_ratio(row.total, row.raw_k, 12).to_bits())
    }))
}

fn add_unique(rows: &mut Vec<SelectedRow>, seen: &mut BTreeSet<RawBits>, row: SelectedRow) -> bool {
    let raw = RawBits {
        n: row.n.to_bits(),
        k: row.k.to_bits(),
    };
    if seen.insert(raw) {
        rows.push(row);
        true
    } else {
        false
    }
}

fn discovery_rows() -> Result<Vec<SelectedRow>, String> {
    let mut rows = Vec::new();
    let mut seen = BTreeSet::new();

    for total in 0..=DISCOVERY_TOTAL_MAX {
        for raw_k in 0..=total {
            let (n, k) = mapped_args(IntegerPair { total, raw_k });
            add_unique(
                &mut rows,
                &mut seen,
                SelectedRow {
                    family: "dense_transformed_triangle",
                    n,
                    k,
                    total,
                    raw_k,
                },
            );
        }
    }

    let mut fractional_pool = Vec::new();
    for total in 8..=DISCOVERY_TOTAL_MAX {
        for raw_k in 1..total {
            fractional_pool.push(IntegerPair { total, raw_k });
        }
    }
    fractional_pool.sort_by_key(|pair| {
        stable_rank("combina-discovery-fractional-v1", pair.total, pair.raw_k, 0)
    });
    let mut fractional_count = 0_usize;
    for pair in fractional_pool {
        if fractional_count == DISCOVERY_FRACTIONAL_ROWS {
            break;
        }
        let (n, k) = mapped_args(pair);
        if add_unique(
            &mut rows,
            &mut seen,
            SelectedRow {
                family: "separate_truncation_discriminator",
                n: n + 0.75,
                k: k + 0.75,
                total: pair.total,
                raw_k: pair.raw_k,
            },
        ) {
            fractional_count += 1;
        }
    }
    if fractional_count != DISCOVERY_FRACTIONAL_ROWS {
        return Err(format!(
            "selected only {fractional_count}/{DISCOVERY_FRACTIONAL_ROWS} discovery fractional rows"
        ));
    }

    // Error/guard controls are part of discovery but deliberately have no
    // numeric candidate prediction.  The replay scorer skips typed errors.
    for k in 1..=32_u32 {
        add_unique(
            &mut rows,
            &mut seen,
            SelectedRow {
                family: "zero_pool_error_control",
                n: 0.0,
                k: k as f64,
                total: k - 1,
                raw_k: k,
            },
        );
    }

    Ok(rows)
}

fn heldout_pool() -> Result<Vec<PoolRow>, String> {
    let mut pool = Vec::new();
    for total in HELDOUT_TOTAL_MIN..=HELDOUT_TOTAL_MAX {
        let reduced_max = HELDOUT_REDUCED_K_MAX.min(total / 2);
        for reduced_k in 0..=reduced_max {
            let pair = IntegerPair {
                total,
                raw_k: reduced_k,
            };
            let (n, k) = mapped_args(pair);
            pool.push(PoolRow {
                pair,
                n,
                k,
                selected: selected_bits(total, reduced_k)?,
                product: product_bits(n, k)?,
                pc53: cyclic_ratio(total, reduced_k, 3).to_bits(),
                continuous: cyclic_ratio(total, reduced_k, 1).to_bits(),
                forward: cyclic_ratio(total, reduced_k, 12).to_bits(),
            });
            let mirror = total - reduced_k;
            if reduced_k != mirror {
                let pair = IntegerPair {
                    total,
                    raw_k: mirror,
                };
                let (n, k) = mapped_args(pair);
                pool.push(PoolRow {
                    pair,
                    n,
                    k,
                    selected: selected_bits(total, mirror)?,
                    product: product_bits(n, k)?,
                    pc53: cyclic_ratio(total, mirror, 3).to_bits(),
                    continuous: cyclic_ratio(total, mirror, 1).to_bits(),
                    forward: cyclic_ratio(total, mirror, 12).to_bits(),
                });
            }
        }
    }
    for row in &pool {
        if !f64::from_bits(row.selected).is_finite() {
            return Err(format!(
                "heldout pool contains non-finite candidate total={} k={}",
                row.pair.total, row.pair.raw_k
            ));
        }
    }
    Ok(pool)
}

fn choose(
    label: &'static str,
    count: usize,
    pool: &[PoolRow],
    used: &mut BTreeSet<RawBits>,
    excluded: &BTreeSet<RawBits>,
    predicate: impl Fn(&PoolRow) -> bool,
) -> Result<Vec<SelectedRow>, String> {
    let mut eligible = Vec::new();
    for row in pool {
        let raw = RawBits {
            n: row.n.to_bits(),
            k: row.k.to_bits(),
        };
        if used.contains(&raw) || excluded.contains(&raw) || !predicate(row) {
            continue;
        }
        eligible.push(*row);
    }
    eligible.sort_by_key(|row| stable_rank(label, row.pair.total, row.pair.raw_k, 0));
    if eligible.len() < count {
        return Err(format!(
            "{label}: only {} eligible rows for target {count}",
            eligible.len()
        ));
    }
    let mut selected = Vec::with_capacity(count);
    for row in eligible.into_iter().take(count) {
        let raw = RawBits {
            n: row.n.to_bits(),
            k: row.k.to_bits(),
        };
        if !used.insert(raw) {
            return Err(format!(
                "{label}: internal duplicate total={} k={}",
                row.pair.total, row.pair.raw_k
            ));
        }
        selected.push(SelectedRow {
            family: label,
            n: row.n,
            k: row.k,
            total: row.pair.total,
            raw_k: row.pair.raw_k,
        });
    }
    Ok(selected)
}

fn heldout_rows(historical: &BTreeSet<RawBits>) -> Result<Vec<SelectedRow>, String> {
    let pool = heldout_pool()?;
    let mut used = BTreeSet::new();
    let mut rows = Vec::new();

    rows.extend(choose(
        "product_discriminator",
        512,
        &pool,
        &mut used,
        historical,
        |row| row.selected != row.product,
    )?);
    rows.extend(choose(
        "pc53_discriminator",
        384,
        &pool,
        &mut used,
        historical,
        |row| row.selected != row.pc53,
    )?);
    rows.extend(choose(
        "continuous_x87_discriminator",
        256,
        &pool,
        &mut used,
        historical,
        |row| row.selected != row.continuous,
    )?);
    rows.extend(choose(
        "forward_order_discriminator",
        256,
        &pool,
        &mut used,
        historical,
        |row| row.selected != row.forward,
    )?);
    rows.extend(choose(
        "raw_high_k_routing",
        128,
        &pool,
        &mut used,
        historical,
        |row| row.pair.raw_k > row.pair.total / 2,
    )?);
    rows.extend(choose(
        "broad_hash_sample",
        256,
        &pool,
        &mut used,
        historical,
        |_| true,
    )?);

    // Fractional rows force the distinction between separate argument
    // truncation and worksheet-visible COMBIN(n+k-1,k), where addition occurs
    // before COMBIN truncates.  Use a disjoint integer base pool and exact .75
    // offsets, which cross one integer in the pre-trunc sum.
    let mut fractional_pool = pool.clone();
    fractional_pool.sort_by_key(|row| {
        stable_rank(
            "heldout_separate_truncation_discriminator",
            row.pair.total,
            row.pair.raw_k,
            0,
        )
    });
    let mut fractional_count = 0_usize;
    for pool_row in fractional_pool {
        if fractional_count == 256 {
            break;
        }
        let pair = pool_row.pair;
        if pair.raw_k == 0 || pair.raw_k > pair.total {
            continue;
        }
        let (base_n, base_k) = mapped_args(pair);
        let n = base_n + 0.75;
        let k = base_k + 0.75;
        let raw = RawBits {
            n: n.to_bits(),
            k: k.to_bits(),
        };
        if used.contains(&raw) || historical.contains(&raw) {
            continue;
        }
        let selected = selected_bits(pair.total, pair.raw_k)?;
        let pretrunc = combin_kernel(n + k - 1.0, k)
            .map_err(|error| format!("fractional pretrunc control returned {error:?}"))?
            .to_bits();
        if selected == pretrunc {
            continue;
        }
        used.insert(raw);
        rows.push(SelectedRow {
            family: "heldout_separate_truncation_discriminator",
            n,
            k,
            total: pair.total,
            raw_k: pair.raw_k,
        });
        fractional_count += 1;
    }
    if fractional_count != 256 {
        return Err(format!(
            "selected only {fractional_count}/256 heldout fractional rows"
        ));
    }

    if rows.len() != 2_048 {
        return Err(format!(
            "heldout row count is {}, expected 2048",
            rows.len()
        ));
    }
    Ok(rows)
}

fn write_json(path: &Path, value: &Value) -> Result<(), String> {
    let mut text = serde_json::to_string_pretty(value).map_err(|error| error.to_string())?;
    text.push('\n');
    fs::write(path, text).map_err(|error| format!("{}: {error}", path.display()))
}

fn emit(
    out_dir: &Path,
    stem: &str,
    row_id: &str,
    role: &str,
    rows: &[SelectedRow],
    historical_counts: &BTreeMap<String, usize>,
) -> Result<(), String> {
    let mut probes = Vec::with_capacity(rows.len());
    let mut predictions = Vec::new();
    let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
    let mut ids = BTreeSet::new();
    let mut raw_pairs = BTreeSet::new();
    for (index, row) in rows.iter().enumerate() {
        let id = format!(
            "combina-{role}-{:05}-{}-t{:04}-k{:04}",
            index, row.family, row.total, row.raw_k
        );
        if !ids.insert(id.clone()) {
            return Err(format!("duplicate generated id {id}"));
        }
        let raw = RawBits {
            n: row.n.to_bits(),
            k: row.k.to_bits(),
        };
        if !raw_pairs.insert(raw) {
            return Err(format!("duplicate generated raw pair for {id}"));
        }
        probes.push(probe(id.clone(), row.n, row.k));
        if row.family != "zero_pool_error_control" {
            predictions.push(prediction(&id, row)?);
        }
        *counts.entry(row.family).or_default() += 1;
    }

    let batch_path = out_dir.join(format!("batch-{stem}.json"));
    let predictions_path = out_dir.join(format!("predictions-{stem}.json"));
    let meta_path = out_dir.join(format!("batch-{stem}.meta.json"));
    write_json(
        &batch_path,
        &json!({
            "function": "COMBINA",
            "row_id": row_id,
            "probes": probes
        }),
    )?;
    write_json(
        &predictions_path,
        &json!({
            "schema": "w109-combina-combin-identity-predictions-v1",
            "status": "frozen answer-blind candidate predictions; contains no COMBINA oracle answers",
            "selected_model": "COMBIN(trunc(n)+trunc(k)-1,trunc(k)) through the signed-off cyclic stored-x87 COMBIN graph",
            "rows": predictions
        }),
    )?;
    write_json(
        &meta_path,
        &json!({
            "schema_version": 1,
            "batch": batch_path.file_name().and_then(|name| name.to_str()),
            "predictions": predictions_path.file_name().and_then(|name| name.to_str()),
            "generator": "smart-fuzzer/tools/calc_graph_racer/src/bin/generate_combina_identity_batches.rs",
            "selection": "answer-blind; no COMBINA oracle output is read for selection",
            "role": role,
            "selected_model_frozen_before_capture": true,
            "excel_capture_contract": {
                "runner": "smart-fuzzer/tools/Run-W109BulkBatch.ps1",
                "input_plumbing": "Range.Value2 bulk matrix with cell-reference Formula2R1C1",
                "result_capture": "Range.Value2 bulk result column",
                "cache_mode": "NoCache",
                "workbook_compatibility": "2",
                "required_reference_profile": "Excel 16.0 build 20228 x64"
            },
            "counts": counts,
            "total_rows": rows.len(),
            "historical_exclusion_inputs": historical_counts,
            "invariants": {
                "unique_ids": true,
                "unique_raw_argument_bit_pairs": true,
                "predictions_contain_no_excel_answers": true,
                "heldout_excludes_all_historical_cellref_combina_pairs": role == "heldout",
                "discovery_and_heldout_transformed_integer_total_ranges_are_disjoint": true
            }
        }),
    )?;
    println!(
        "{} rows -> {}, {}, {}",
        rows.len(),
        batch_path.display(),
        predictions_path.display(),
        meta_path.display()
    );
    Ok(())
}

fn main() -> Result<(), String> {
    let out_dir = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("smart-fuzzer/work/w109/G4-04-combina"));
    fs::create_dir_all(&out_dir)
        .map_err(|error| format!("cannot create {}: {error}", out_dir.display()))?;

    let (historical, historical_counts) = historical_pairs()?;
    let discovery = discovery_rows()?;
    let heldout = heldout_rows(&historical)?;

    let discovery_raw: BTreeSet<_> = discovery
        .iter()
        .map(|row| RawBits {
            n: row.n.to_bits(),
            k: row.k.to_bits(),
        })
        .collect();
    let heldout_raw: BTreeSet<_> = heldout
        .iter()
        .map(|row| RawBits {
            n: row.n.to_bits(),
            k: row.k.to_bits(),
        })
        .collect();
    if !discovery_raw.is_disjoint(&heldout_raw) {
        return Err("discovery and heldout raw pairs overlap".to_owned());
    }
    if !historical.is_disjoint(&heldout_raw) {
        return Err("heldout overlaps historical COMBINA pairs".to_owned());
    }

    emit(
        &out_dir,
        "combina-identity-discovery-v1",
        "G4-04-combina-identity-current-discovery-v1",
        "discovery",
        &discovery,
        &historical_counts,
    )?;
    emit(
        &out_dir,
        "combina-identity-heldout-v1",
        "G4-04-combina-identity-heldout-v1",
        "heldout",
        &heldout,
        &historical_counts,
    )?;
    println!(
        "historical raw pairs excluded from heldout={} discovery={} heldout={}",
        historical.len(),
        discovery.len(),
        heldout.len()
    );
    Ok(())
}
