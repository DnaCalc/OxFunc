//! W109 G3-06: offline race for private F.TEST tail/publication graphs.
//!
//! This consumes only the already captured build-20228 F.TEST discovery and
//! its frozen ratio/df metadata. It evaluates the public TOMS-708 BRATIO
//! substrate through alternative direct/complement and publication routes;
//! it never launches Excel.

use oxfunc_core::excel_numeric::research as rx;
use oxfunc_core::functions::special_math_common::bratio;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

const CW: u16 = rx::CW_PC64_RN;
const FORWARD_NATIVE_MODEL: &str =
    "two-pass mean=Native body=Native rev=false corr=false ratio=Native";
const FORWARD_X87_MODEL: &str = "two-pass mean=Native body=Native rev=false corr=false ratio=X87";

#[derive(Clone, Copy, Debug)]
enum Op {
    Native,
    X87,
}

fn ext(value: f64) -> rx::Ext80 {
    rx::ext_from_f64(value)
}

fn to_f64(value: &rx::Ext80) -> f64 {
    rx::ext_to_f64(value, CW)
}

fn add(op: Op, left: f64, right: f64) -> f64 {
    match op {
        Op::Native => left + right,
        Op::X87 => to_f64(&rx::ext_add(&ext(left), &ext(right), CW)),
    }
}

fn sub(op: Op, left: f64, right: f64) -> f64 {
    match op {
        Op::Native => left - right,
        Op::X87 => to_f64(&rx::ext_sub(&ext(left), &ext(right), CW)),
    }
}

fn mul(op: Op, left: f64, right: f64) -> f64 {
    match op {
        Op::Native => left * right,
        Op::X87 => to_f64(&rx::ext_mul(&ext(left), &ext(right), CW)),
    }
}

fn div(op: Op, left: f64, right: f64) -> f64 {
    match op {
        Op::Native => left / right,
        Op::X87 => to_f64(&rx::ext_div(&ext(left), &ext(right), CW)),
    }
}

fn parse_bits(text: &str) -> Result<u64, String> {
    u64::from_str_radix(
        text.strip_prefix("0x")
            .ok_or_else(|| format!("expected hex bits, got {text}"))?,
        16,
    )
    .map_err(|error| error.to_string())
}

fn load_json(path: &Path) -> Result<Value, String> {
    serde_json::from_str(
        &fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))?,
    )
    .map_err(|error| format!("{}: {error}", path.display()))
}

fn load_ftest(directory: &Path) -> Result<BTreeMap<String, u64>, String> {
    let batch = load_json(&directory.join("batch-ftest-variance-discovery-v2.json"))?;
    let answers = load_json(&directory.join("answers-ftest-variance-discovery-v2.json"))?;
    if batch["function"].as_str() != Some("F.TEST")
        || answers["function"].as_str() != Some("F.TEST")
    {
        return Err("F.TEST function mismatch".to_string());
    }
    let environment = &answers["capture_provenance"]["environment"];
    let cache = &answers["capture_provenance"]["oracle_cache"];
    if environment["excel_version"].as_str() != Some("16.0")
        || environment["excel_build"].as_str() != Some("20228")
        || environment["excel_bitness"].as_str() != Some("64-bit")
        || environment["workbook_compatibility"].as_str() != Some("2")
        || cache["mode"].as_str() != Some("no_cache")
        || cache["hits"].as_u64() != Some(0)
        || cache["misses"].as_u64() != Some(0)
    {
        return Err("F.TEST provenance mismatch".to_string());
    }
    let probes = batch["probes"]
        .as_array()
        .ok_or_else(|| "F.TEST probes missing".to_string())?;
    let witnesses = answers["witnesses"]
        .as_array()
        .ok_or_else(|| "F.TEST witnesses missing".to_string())?;
    if probes.len() != 48 || witnesses.len() != probes.len() {
        return Err("F.TEST count mismatch".to_string());
    }
    let mut out = BTreeMap::new();
    for (index, (probe, witness)) in probes.iter().zip(witnesses).enumerate() {
        let probe = &probe["probe"];
        let id = probe["id"]
            .as_str()
            .filter(|value| !value.is_empty())
            .ok_or_else(|| format!("missing F.TEST id at {index}"))?;
        if witness["id"].as_str() != Some(id) || witness["args"] != probe["args"] {
            return Err(format!("F.TEST ID/argument drift at {index}"));
        }
        let bits = parse_bits(
            witness["expected_bits"]
                .as_str()
                .ok_or_else(|| format!("nonnumeric F.TEST result at {index}"))?,
        )?;
        if out.insert(id.to_string(), bits).is_some() {
            return Err(format!("duplicate F.TEST id {id}"));
        }
    }
    Ok(out)
}

fn insert_output(outputs: &mut BTreeMap<String, u64>, name: String, value: f64) {
    if value.is_finite() {
        outputs.insert(name, value.to_bits());
    }
}

fn publish_pair_variants(
    outputs: &mut BTreeMap<String, u64>,
    prefix: &str,
    first: f64,
    second: f64,
) {
    let (small, large) = if first <= second {
        (first, second)
    } else {
        (second, first)
    };
    for scale_op in [Op::Native, Op::X87] {
        insert_output(
            outputs,
            format!("{prefix}/two-small/scale={scale_op:?}"),
            mul(scale_op, 2.0, small),
        );
        for subtract_op in [Op::Native, Op::X87] {
            insert_output(
                outputs,
                format!("{prefix}/two-minus-two-large/scale={scale_op:?}/sub={subtract_op:?}"),
                sub(subtract_op, 2.0, mul(scale_op, 2.0, large)),
            );
        }
    }
    for add_op in [Op::Native, Op::X87] {
        insert_output(
            outputs,
            format!("{prefix}/small-plus-small/add={add_op:?}"),
            add(add_op, small, small),
        );
        for subtract_op in [Op::Native, Op::X87] {
            insert_output(
                outputs,
                format!("{prefix}/two-minus-large-plus-large/add={add_op:?}/sub={subtract_op:?}"),
                sub(subtract_op, 2.0, add(add_op, large, large)),
            );
        }
    }
    for complement_op in [Op::Native, Op::X87] {
        let complement = sub(complement_op, 1.0, large);
        for scale_op in [Op::Native, Op::X87] {
            insert_output(
                outputs,
                format!("{prefix}/complement-first/comp={complement_op:?}/scale={scale_op:?}"),
                mul(scale_op, 2.0, complement),
            );
        }
        for add_op in [Op::Native, Op::X87] {
            insert_output(
                outputs,
                format!("{prefix}/complement-first/comp={complement_op:?}/add={add_op:?}"),
                add(add_op, complement, complement),
            );
        }
    }
}

fn graph_outputs(ratio: f64, d1: f64, d2: f64) -> BTreeMap<String, u64> {
    let mut outputs = BTreeMap::new();
    for num_op in [Op::Native, Op::X87] {
        let num = mul(num_op, d1, ratio);
        for den_op in [Op::Native, Op::X87] {
            let den = add(den_op, d2, num);
            for x_op in [Op::Native, Op::X87] {
                let x = div(x_op, num, den);
                for y_op in [Op::Native, Op::X87] {
                    let y = div(y_op, d2, den);
                    let cdf = bratio(d1 / 2.0, d2 / 2.0, x, y);
                    let rt = bratio(d2 / 2.0, d1 / 2.0, y, x);
                    let stem = format!("num={num_op:?}/den={den_op:?}/x={x_op:?}/y={y_op:?}");
                    publish_pair_variants(&mut outputs, &format!("{stem}/cdf-pair"), cdf.0, cdf.1);
                    publish_pair_variants(&mut outputs, &format!("{stem}/rt-pair"), rt.0, rt.1);
                    for complement_op in [Op::Native, Op::X87] {
                        let sources = [
                            ("cdf-p", cdf.0, sub(complement_op, 1.0, cdf.0)),
                            ("cdf-q", cdf.1, sub(complement_op, 1.0, cdf.1)),
                            ("rt-p", rt.0, sub(complement_op, 1.0, rt.0)),
                            ("rt-q", rt.1, sub(complement_op, 1.0, rt.1)),
                        ];
                        for (source, first, second) in sources {
                            publish_pair_variants(
                                &mut outputs,
                                &format!("{stem}/{source}/source-comp={complement_op:?}"),
                                first,
                                second,
                            );
                        }
                    }
                }
            }
        }
    }
    outputs
}

fn score(directory: &Path) -> Result<(), String> {
    let targets = load_ftest(directory)?;
    let meta = load_json(&directory.join("meta-ftest-variance-discovery-v2.json"))?;
    let rows = meta["rows"]
        .as_array()
        .ok_or_else(|| "metadata rows missing".to_string())?;
    if rows.len() != 48 {
        return Err(format!("metadata row count {}, expected 48", rows.len()));
    }

    let mut any_scores: BTreeMap<String, usize> = BTreeMap::new();
    let mut forward_native_scores: BTreeMap<String, usize> = BTreeMap::new();
    let mut forward_x87_scores: BTreeMap<String, usize> = BTreeMap::new();
    let mut per_row_reachable = Vec::new();
    let mut graph_count = None;
    for (row_index, row) in rows.iter().enumerate() {
        let id = row["ftest_id"]
            .as_str()
            .ok_or_else(|| format!("missing F.TEST id at row {row_index}"))?;
        let target = *targets
            .get(id)
            .ok_or_else(|| format!("missing F.TEST target {id}"))?;
        let groups = row["candidate_groups"]
            .as_array()
            .ok_or_else(|| format!("missing groups at row {row_index}"))?;
        let mut any_hit = BTreeSet::new();
        let mut forward_native_hit = BTreeSet::new();
        let mut forward_x87_hit = BTreeSet::new();
        for group in groups {
            let ratio = f64::from_bits(parse_bits(group["ratio_bits"].as_str().unwrap())?);
            let d1 = f64::from_bits(parse_bits(group["df_hi_bits"].as_str().unwrap())?);
            let d2 = f64::from_bits(parse_bits(group["df_lo_bits"].as_str().unwrap())?);
            let outputs = graph_outputs(ratio, d1, d2);
            if let Some(expected) = graph_count {
                if outputs.len() != expected {
                    return Err("graph count drift".to_string());
                }
            } else {
                graph_count = Some(outputs.len());
            }
            let is_forward_native = group["models"].as_array().is_some_and(|models| {
                models
                    .iter()
                    .any(|model| model.as_str() == Some(FORWARD_NATIVE_MODEL))
            });
            let is_forward_x87 = group["models"].as_array().is_some_and(|models| {
                models
                    .iter()
                    .any(|model| model.as_str() == Some(FORWARD_X87_MODEL))
            });
            for (name, bits) in outputs {
                if bits == target {
                    any_hit.insert(name.clone());
                    if is_forward_native {
                        forward_native_hit.insert(name.clone());
                    }
                    if is_forward_x87 {
                        forward_x87_hit.insert(name);
                    }
                }
            }
        }
        for name in &any_hit {
            *any_scores.entry(name.clone()).or_default() += 1;
        }
        for name in &forward_native_hit {
            *forward_native_scores.entry(name.clone()).or_default() += 1;
        }
        for name in &forward_x87_hit {
            *forward_x87_scores.entry(name.clone()).or_default() += 1;
        }
        per_row_reachable.push((
            id.to_string(),
            any_hit.len(),
            forward_native_hit.len(),
            forward_x87_hit.len(),
        ));
    }

    let mut any_ranked: Vec<_> = any_scores.into_iter().map(|(name, n)| (n, name)).collect();
    any_ranked.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
    let mut forward_native_ranked: Vec<_> = forward_native_scores
        .into_iter()
        .map(|(name, n)| (n, name))
        .collect();
    forward_native_ranked.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
    let mut forward_x87_ranked: Vec<_> = forward_x87_scores
        .into_iter()
        .map(|(name, n)| (n, name))
        .collect();
    forward_x87_ranked.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));

    println!("graphs_per_group={}", graph_count.unwrap_or(0));
    println!("top any-frozen-group graphs:");
    for (exact, name) in any_ranked.iter().take(40) {
        println!("  {exact}/48 {name}");
    }
    println!("top forward-native-ratio graphs:");
    for (exact, name) in forward_native_ranked.iter().take(40) {
        println!("  {exact}/48 {name}");
    }
    println!("top forward-x87-ratio graphs:");
    for (exact, name) in forward_x87_ranked.iter().take(40) {
        println!("  {exact}/48 {name}");
    }
    println!("publication-family best exact scores:");
    for (label, needle) in [
        ("two-small", "/two-small/"),
        ("small-plus-small", "/small-plus-small/"),
        ("two-minus-two-large", "/two-minus-two-large/"),
        ("two-minus-large-plus-large", "/two-minus-large-plus-large/"),
        ("complement-first", "/complement-first/"),
    ] {
        let best = |ranked: &[(usize, String)]| {
            ranked
                .iter()
                .filter(|(_, name)| name.contains(needle))
                .map(|(exact, _)| *exact)
                .max()
                .unwrap_or(0)
        };
        println!(
            "  {label} any={}/48 forward_native={}/48 forward_x87={}/48",
            best(&any_ranked),
            best(&forward_native_ranked),
            best(&forward_x87_ranked)
        );
    }
    let unreachable: Vec<_> = per_row_reachable
        .iter()
        .filter(|(_, any, _, _)| *any == 0)
        .collect();
    println!(
        "row_reachability={}/48 unreachable={}",
        48 - unreachable.len(),
        unreachable.len()
    );
    for (id, _, forward_native, forward_x87) in unreachable {
        println!(
            "  UNREACHABLE {id} forward_native_hits={forward_native} forward_x87_hits={forward_x87}"
        );
    }
    Ok(())
}

fn main() -> Result<(), String> {
    let directory = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .ok_or_else(|| "usage: race_ftest_private_tail <G3-06 directory>".to_string())?;
    score(&directory)
}
