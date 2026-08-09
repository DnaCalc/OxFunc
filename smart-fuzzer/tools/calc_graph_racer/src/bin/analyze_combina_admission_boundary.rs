//! Deterministic typed analysis for the paired COMBINA/COMBIN boundary probe.

use serde::Deserialize;
use serde_json::{Map, Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Deserialize)]
struct Batch {
    function: String,
    probes: Vec<BatchProbe>,
}

#[derive(Deserialize)]
struct BatchProbe {
    probe: Probe,
}

#[derive(Deserialize)]
struct Probe {
    id: String,
    args: [String; 2],
}

#[derive(Clone, Deserialize)]
struct Witness {
    id: String,
    args: [String; 2],
    expected_bits: String,
}

#[derive(Deserialize)]
struct Answers {
    function: String,
    witnesses: Vec<Witness>,
}

#[derive(Deserialize)]
struct PairingManifest {
    pairings: Vec<Pairing>,
}

#[derive(Deserialize)]
struct Pairing {
    combina_id: String,
    control_id: Option<String>,
    #[serde(default)]
    transformed_total: Option<String>,
}

fn read<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, String> {
    let text = fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))?;
    serde_json::from_str(&text).map_err(|error| format!("{}: {error}", path.display()))
}

fn decode(raw: &str) -> Result<f64, String> {
    let hex = raw
        .strip_prefix("0x")
        .ok_or_else(|| format!("not a binary64 bit string: {raw}"))?;
    let value = u64::from_str_radix(hex, 16).map_err(|error| format!("{raw}: {error}"))?;
    Ok(f64::from_bits(value))
}

fn family(id: &str) -> Result<&'static str, String> {
    for family in [
        "raw-n-small-k",
        "small-n-raw-k",
        "fixed-transformed-total",
        "fractional-guard-order",
        "signed-zero-domain-matrix",
    ] {
        if id.starts_with(&format!("combina-boundary-{family}-")) {
            return Ok(family);
        }
    }
    Err(format!("unknown COMBINA boundary family: {id}"))
}

fn is_numeric(result: &str) -> bool {
    result.starts_with("0x")
}

fn assert_alignment(
    batch: &Batch,
    answers: &Answers,
    expected_function: &str,
) -> Result<(), String> {
    if batch.function != expected_function || answers.function != expected_function {
        return Err(format!(
            "function mismatch: batch={} answers={} expected={expected_function}",
            batch.function, answers.function
        ));
    }
    if batch.probes.len() != answers.witnesses.len() {
        return Err(format!(
            "count mismatch for {expected_function}: batch={} answers={}",
            batch.probes.len(),
            answers.witnesses.len()
        ));
    }
    let mut ids = BTreeSet::new();
    for (index, (probe, witness)) in batch.probes.iter().zip(&answers.witnesses).enumerate() {
        if probe.probe.id != witness.id || probe.probe.args != witness.args {
            return Err(format!(
                "ordered id/args mismatch for {expected_function} at row {index}"
            ));
        }
        if !ids.insert(&witness.id) {
            return Err(format!("duplicate witness id: {}", witness.id));
        }
    }
    Ok(())
}

fn bump(map: &mut BTreeMap<String, usize>, key: impl Into<String>) {
    *map.entry(key.into()).or_default() += 1;
}

fn result_kind(value: &str) -> &'static str {
    if is_numeric(value) { "numeric" } else { "Num" }
}

fn main() -> Result<(), String> {
    let root = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("smart-fuzzer/work/w109/G4-04-combina"));
    let combina_batch_path = root.join("batch-combina-admission-boundary-discovery-v1.json");
    let combina_answers_path =
        root.join("answers-combina-admission-boundary-discovery-v1-excel.json");
    let combin_batch_path =
        root.join("batch-combin-transformed-boundary-control-discovery-v1.json");
    let combin_answers_path =
        root.join("answers-combin-transformed-boundary-control-discovery-v1-excel.json");
    let pairing_path = root.join("pairing-combina-combin-boundary-discovery-v1.json");
    let output_path = root.join("analysis-combina-admission-boundary-v1.json");

    let combina_batch: Batch = read(&combina_batch_path)?;
    let combina_answers: Answers = read(&combina_answers_path)?;
    let combin_batch: Batch = read(&combin_batch_path)?;
    let combin_answers: Answers = read(&combin_answers_path)?;
    let pairing: PairingManifest = read(&pairing_path)?;
    assert_alignment(&combina_batch, &combina_answers, "COMBINA")?;
    assert_alignment(&combin_batch, &combin_answers, "COMBIN")?;
    if pairing.pairings.len() != combina_answers.witnesses.len() {
        return Err("pairing count does not match COMBINA answers".to_owned());
    }

    let combina: BTreeMap<&str, &Witness> = combina_answers
        .witnesses
        .iter()
        .map(|row| (row.id.as_str(), row))
        .collect();
    let combin: BTreeMap<&str, &Witness> = combin_answers
        .witnesses
        .iter()
        .map(|row| (row.id.as_str(), row))
        .collect();

    let mut typed_counts = BTreeMap::new();
    for row in &combina_answers.witnesses {
        bump(&mut typed_counts, result_kind(&row.expected_bits));
    }
    let mut control_typed_counts = BTreeMap::new();
    for row in &combin_answers.witnesses {
        bump(&mut control_typed_counts, result_kind(&row.expected_bits));
    }

    let mut paired_counts = BTreeMap::new();
    let mut family_counts: BTreeMap<String, BTreeMap<String, usize>> = BTreeMap::new();
    let mut differences = Vec::new();
    let mut fixed_totals: BTreeMap<String, BTreeMap<String, usize>> = BTreeMap::new();
    for link in &pairing.pairings {
        let row = combina
            .get(link.combina_id.as_str())
            .ok_or_else(|| format!("missing COMBINA answer: {}", link.combina_id))?;
        let family = family(&row.id)?;
        let class = match link.control_id.as_deref() {
            None => "unpaired-invalid-transform".to_owned(),
            Some(control_id) => {
                let control = combin
                    .get(control_id)
                    .ok_or_else(|| format!("missing COMBIN control: {control_id}"))?;
                if row.expected_bits == control.expected_bits {
                    "exact-same".to_owned()
                } else if !is_numeric(&row.expected_bits) && is_numeric(&control.expected_bits) {
                    "combina-num-control-numeric".to_owned()
                } else if is_numeric(&row.expected_bits) && !is_numeric(&control.expected_bits) {
                    "combina-numeric-control-num".to_owned()
                } else if is_numeric(&row.expected_bits) {
                    "numeric-bits-differ".to_owned()
                } else {
                    "error-kind-differ".to_owned()
                }
            }
        };
        bump(&mut paired_counts, &class);
        bump(family_counts.entry(family.to_owned()).or_default(), &class);
        if class != "exact-same" && class != "unpaired-invalid-transform" {
            let control = link.control_id.as_deref().and_then(|id| combin.get(id));
            differences.push(json!({
                "class": class,
                "family": family,
                "combina_id": row.id,
                "n": row.args[0],
                "k": row.args[1],
                "combina_result": row.expected_bits,
                "control_id": link.control_id,
                "control_result": control.map(|item| &item.expected_bits),
                "transformed_total": link.transformed_total
            }));
        }
        if family == "fixed-transformed-total" {
            let total = link
                .transformed_total
                .as_ref()
                .ok_or_else(|| format!("fixed-total row lacks transform: {}", row.id))?;
            let entry = fixed_totals.entry(total.clone()).or_default();
            bump(
                entry,
                format!("combina-{}", result_kind(&row.expected_bits)),
            );
            bump(entry, format!("pair-{class}"));
        }
    }

    let mut transitions = Vec::new();
    for (axis_family, fixed_index, varied_index) in [
        ("raw-n-small-k", 1_usize, 0_usize),
        ("small-n-raw-k", 0_usize, 1_usize),
    ] {
        let mut groups: BTreeMap<String, Vec<&Witness>> = BTreeMap::new();
        for row in &combina_answers.witnesses {
            if family(&row.id)? == axis_family {
                groups
                    .entry(row.args[fixed_index].clone())
                    .or_default()
                    .push(row);
            }
        }
        for (fixed, mut group) in groups {
            group.sort_by(|left, right| {
                let left_value = decode(&left.args[varied_index]).unwrap();
                let right_value = decode(&right.args[varied_index]).unwrap();
                left_value.total_cmp(&right_value)
            });
            for pair in group.windows(2) {
                let before = pair[0];
                let after = pair[1];
                if result_kind(&before.expected_bits) != result_kind(&after.expected_bits) {
                    transitions.push(json!({
                        "family": axis_family,
                        "fixed": fixed,
                        "before_arg": before.args[varied_index],
                        "before_value": decode(&before.args[varied_index])?,
                        "before_result": before.expected_bits,
                        "after_arg": after.args[varied_index],
                        "after_value": decode(&after.args[varied_index])?,
                        "after_result": after.expected_bits
                    }));
                }
            }
        }
    }

    let mut signed_and_fractional = Vec::new();
    for row in &combina_answers.witnesses {
        let n = decode(&row.args[0])?;
        let k = decode(&row.args[1])?;
        if n.is_sign_negative() || k.is_sign_negative() || n.fract() != 0.0 || k.fract() != 0.0 {
            signed_and_fractional.push(json!({
                "id": row.id,
                "family": family(&row.id)?,
                "n": row.args[0],
                "n_value": n,
                "k": row.args[1],
                "k_value": k,
                "result": row.expected_bits
            }));
        }
    }

    let fixed_total_json: Map<String, Value> = fixed_totals
        .into_iter()
        .map(|(total, counts)| (total, json!(counts)))
        .collect();
    let report = json!({
        "schema_version": 1,
        "combina_rows": combina_answers.witnesses.len(),
        "combin_control_rows": combin_answers.witnesses.len(),
        "typed_counts": typed_counts,
        "control_typed_counts": control_typed_counts,
        "paired_counts": paired_counts,
        "paired_counts_by_family": family_counts,
        "typed_axis_transitions": transitions,
        "fixed_transformed_total_summary": fixed_total_json,
        "paired_differences": differences,
        "signed_and_fractional_rows": signed_and_fractional
    });
    let mut text = serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?;
    text.push('\n');
    fs::write(&output_path, text)
        .map_err(|error| format!("cannot write {}: {error}", output_path.display()))?;

    println!("COMBINA typed: {:?}", report["typed_counts"]);
    println!(
        "COMBIN controls typed: {:?}",
        report["control_typed_counts"]
    );
    println!("paired: {:?}", report["paired_counts"]);
    println!(
        "{} typed transitions, {} paired differences -> {}",
        report["typed_axis_transitions"]
            .as_array()
            .map_or(0, Vec::len),
        report["paired_differences"].as_array().map_or(0, Vec::len),
        output_path.display()
    );
    Ok(())
}
