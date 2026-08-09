//! Fit directed CONVERT multipliers as x87 64-bit-significand constants.
//!
//! An `x=1` oracle result fixes the containing binary64 bin but leaves roughly
//! 2^11 possible extended values.  Adjacent-input discovery rows determine the
//! low extended bits without consulting held-out inputs.  This scorer searches
//! that complete local bin and emits a frozen directed-ratio table suitable for
//! later held-out replay.

#[path = "convert_research/common.rs"]
mod common;

use common::{MetaDocument, ordered_bits};
use oxfunc_core::excel_numeric::research as rx;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Deserialize)]
struct WitnessSet {
    function: String,
    witnesses: Vec<Witness>,
    #[serde(default)]
    capture_provenance: Value,
}

#[derive(Deserialize)]
struct Witness {
    id: String,
    expected_bits: String,
}

#[derive(Clone)]
struct Row {
    id: String,
    class: String,
    number: f64,
    actual: u64,
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
struct Fitness {
    exact: usize,
    sum_abs_ulp: u128,
    max_abs_ulp: u128,
}

#[derive(Serialize)]
struct PairFit {
    category: String,
    from_unit: String,
    to_unit: String,
    discovery_rows: usize,
    anchor_bits: String,
    chosen_ext80: String,
    chosen_significand_offset_from_f64_anchor: i64,
    exact: usize,
    sum_abs_ulp: String,
    max_abs_ulp: String,
    equally_fit_offset_min: i64,
    equally_fit_offset_max: i64,
    equally_fit_offset_count: usize,
    first_misses: Vec<String>,
}

#[derive(Serialize)]
struct Report {
    schema_version: &'static str,
    function: &'static str,
    search_radius: i64,
    modeled_pairs: usize,
    modeled_rows: usize,
    exact_rows: usize,
    pair_fits: Vec<PairFit>,
    capture_provenance: Value,
}

struct Args {
    meta: PathBuf,
    answers: PathBuf,
    out: Option<PathBuf>,
    radius: i64,
}

fn args() -> Args {
    let mut meta = None;
    let mut answers = None;
    let mut out = None;
    let mut radius = 4096_i64;
    let mut values = std::env::args().skip(1);
    while let Some(arg) = values.next() {
        match arg.as_str() {
            "--meta" => meta = values.next().map(PathBuf::from),
            "--answers" => answers = values.next().map(PathBuf::from),
            "--out" => out = values.next().map(PathBuf::from),
            "--radius" => radius = values.next().unwrap().parse().unwrap(),
            other => panic!("unknown argument {other}"),
        }
    }
    Args {
        meta: meta.expect("--meta"),
        answers: answers.expect("--answers"),
        out,
        radius,
    }
}

fn bits(raw: &str) -> Option<u64> {
    let digits = raw.strip_prefix("0x")?;
    (digits.len() == 16)
        .then(|| u64::from_str_radix(digits, 16).ok())
        .flatten()
}

fn ext_with_significand_offset(anchor: rx::Ext80, offset: i64) -> rx::Ext80 {
    let mut bytes = anchor.0;
    let significand = u64::from_le_bytes(bytes[..8].try_into().unwrap());
    let adjusted = if offset >= 0 {
        significand.checked_add(offset as u64).unwrap()
    } else {
        significand.checked_sub(offset.unsigned_abs()).unwrap()
    };
    bytes[..8].copy_from_slice(&adjusted.to_le_bytes());
    rx::Ext80(bytes)
}

fn ext_hex(value: rx::Ext80) -> String {
    let mut result = String::from("0x");
    for byte in value.0.iter().rev() {
        result.push_str(&format!("{byte:02x}"));
    }
    result
}

fn predict(number: f64, ratio: rx::Ext80) -> f64 {
    let cw = rx::CW_PC64_RN;
    rx::ext_to_f64(
        &rx::ext_mul(&rx::ext_from_f64(number), &ratio, cw),
        cw,
    )
}

fn fitness(rows: &[Row], ratio: rx::Ext80) -> Fitness {
    let mut result = Fitness::default();
    for row in rows {
        let predicted = predict(row.number, ratio);
        let residual = ordered_bits(row.actual) - ordered_bits(predicted.to_bits());
        if residual == 0 {
            result.exact += 1;
        } else {
            let abs = residual.unsigned_abs();
            result.sum_abs_ulp = result.sum_abs_ulp.saturating_add(abs);
            result.max_abs_ulp = result.max_abs_ulp.max(abs);
        }
    }
    result
}

fn better(left: Fitness, right: Fitness) -> bool {
    left.exact > right.exact
        || (left.exact == right.exact && left.sum_abs_ulp < right.sum_abs_ulp)
        || (left.exact == right.exact
            && left.sum_abs_ulp == right.sum_abs_ulp
            && left.max_abs_ulp < right.max_abs_ulp)
}

fn main() {
    let args = args();
    assert!(args.radius >= 0);
    let metadata: MetaDocument =
        serde_json::from_slice(&std::fs::read(&args.meta).unwrap()).unwrap();
    let answer_set: WitnessSet =
        serde_json::from_slice(&std::fs::read(&args.answers).unwrap()).unwrap();
    assert_eq!(answer_set.function, "CONVERT");
    let by_id: BTreeMap<_, _> = answer_set
        .witnesses
        .iter()
        .map(|witness| (witness.id.as_str(), witness))
        .collect();
    type Key = (String, String, String);
    let mut groups: BTreeMap<Key, Vec<Row>> = BTreeMap::new();
    let mut anchors: BTreeMap<Key, u64> = BTreeMap::new();
    for row in &metadata.rows {
        if row.predictions.is_empty() {
            continue;
        }
        let witness = by_id[row.id.as_str()];
        let Some(actual) = bits(&witness.expected_bits) else {
            continue;
        };
        let key = (
            row.category.clone(),
            row.from_unit.clone(),
            row.to_unit.clone(),
        );
        if row.number_bits == "0x3ff0000000000000" {
            anchors.insert(key.clone(), actual);
        }
        groups.entry(key).or_default().push(Row {
            id: row.id.clone(),
            class: row.class.clone(),
            number: common::f64_from_hex(&row.number_bits).unwrap(),
            actual,
        });
    }

    let mut fits = Vec::new();
    let mut total_rows = 0;
    let mut exact_rows = 0;
    for ((category, from_unit, to_unit), rows) in groups {
        let key = (category.clone(), from_unit.clone(), to_unit.clone());
        let anchor_bits = anchors[&key];
        let anchor = rx::ext_from_f64(f64::from_bits(anchor_bits));
        let mut best = Fitness::default();
        let mut best_offset = 0_i64;
        let mut equal_offsets = Vec::new();
        for offset in -args.radius..=args.radius {
            let candidate = ext_with_significand_offset(anchor, offset);
            // The fitted value must remain in the x=1 binary64 bin.  This
            // guard also prevents an arbitrary low-bit search from changing
            // the observed directed multiplier.
            if rx::ext_to_f64(&candidate, rx::CW_PC64_RN).to_bits() != anchor_bits {
                continue;
            }
            let result = fitness(&rows, candidate);
            if equal_offsets.is_empty() || better(result, best) {
                best = result;
                best_offset = offset;
                equal_offsets.clear();
                equal_offsets.push(offset);
            } else if result == best {
                equal_offsets.push(offset);
                // Prefer the smallest magnitude when discovery leaves a
                // range underdetermined.  This deterministic tie-break is
                // frozen before held-out replay.
                if offset.unsigned_abs() < best_offset.unsigned_abs() {
                    best_offset = offset;
                }
            }
        }
        let chosen = ext_with_significand_offset(anchor, best_offset);
        let mut misses = Vec::new();
        for row in &rows {
            let predicted = predict(row.number, chosen);
            let residual = ordered_bits(row.actual) - ordered_bits(predicted.to_bits());
            if residual != 0 && misses.len() < 100 {
                misses.push(format!(
                    "{} {} x=0x{:016x} residual={:+} predicted=0x{:016x} oracle=0x{:016x}",
                    row.id,
                    row.class,
                    row.number.to_bits(),
                    residual,
                    predicted.to_bits(),
                    row.actual
                ));
            }
        }
        total_rows += rows.len();
        exact_rows += best.exact;
        fits.push(PairFit {
            category,
            from_unit,
            to_unit,
            discovery_rows: rows.len(),
            anchor_bits: format!("0x{anchor_bits:016x}"),
            chosen_ext80: ext_hex(chosen),
            chosen_significand_offset_from_f64_anchor: best_offset,
            exact: best.exact,
            sum_abs_ulp: best.sum_abs_ulp.to_string(),
            max_abs_ulp: best.max_abs_ulp.to_string(),
            equally_fit_offset_min: *equal_offsets.iter().min().unwrap(),
            equally_fit_offset_max: *equal_offsets.iter().max().unwrap(),
            equally_fit_offset_count: equal_offsets.len(),
            first_misses: misses,
        });
    }
    fits.sort_by(|left, right| {
        (&left.category, &left.from_unit, &left.to_unit)
            .cmp(&(&right.category, &right.from_unit, &right.to_unit))
    });
    let imperfect: Vec<_> = fits
        .iter()
        .filter(|fit| fit.exact != fit.discovery_rows)
        .collect();
    println!(
        "fitted {} directed ratios: {exact_rows}/{total_rows} discovery rows exact; imperfect pairs={}",
        fits.len(),
        imperfect.len()
    );
    for fit in imperfect.iter().take(40) {
        println!(
            "  {} {}->{} {}/{} offset={} tied=[{},{}] count={}",
            fit.category,
            fit.from_unit,
            fit.to_unit,
            fit.exact,
            fit.discovery_rows,
            fit.chosen_significand_offset_from_f64_anchor,
            fit.equally_fit_offset_min,
            fit.equally_fit_offset_max,
            fit.equally_fit_offset_count
        );
    }

    let report = Report {
        schema_version: "w109.convert.ext_pair_ratio_fit.v1",
        function: "CONVERT",
        search_radius: args.radius,
        modeled_pairs: fits.len(),
        modeled_rows: total_rows,
        exact_rows,
        pair_fits: fits,
        capture_provenance: answer_set.capture_provenance,
    };
    if let Some(path) = args.out {
        std::fs::write(&path, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
        println!("wrote extended-ratio fit -> {}", path.display());
    }
}
