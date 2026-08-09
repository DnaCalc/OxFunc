//! Mandatory compiled-production replay for the W109 G4-05 CONVERT landing.
//!
//! This validates IDs, typed argument bits, NoCache/current-build provenance,
//! and then calls the production `convert_kernel` over every discovery,
//! retired, refinement-only, readback-control, and frozen publication row.

#[path = "convert_research/common.rs"]
mod common;

use common::MetaDocument;
use oxfunc_core::functions::misc_conversion_family::convert_kernel;
use oxfunc_core::value::WorksheetErrorCode;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Deserialize)]
struct WitnessSet {
    function: String,
    witnesses: Vec<Witness>,
    capture_provenance: Value,
}

#[derive(Deserialize)]
struct Witness {
    id: String,
    args: Value,
    expected_bits: String,
}

#[derive(Deserialize)]
struct ReadbackDoc {
    environment: Value,
    rows: Vec<ReadbackRow>,
}

#[derive(Deserialize)]
struct ReadbackRow {
    requested_bits: String,
    from_unit: String,
    to_unit: String,
    requested_convert_bits: String,
}

struct Row {
    id: String,
    dataset: String,
    number: f64,
    from: String,
    to: String,
    expected: String,
}

#[derive(Default, Serialize)]
struct Fitness {
    exact: usize,
    total: usize,
    first_misses: Vec<String>,
}

#[derive(Serialize)]
struct Report {
    schema_version: &'static str,
    function: &'static str,
    freeze_id: &'static str,
    exact: usize,
    total: usize,
    datasets: BTreeMap<String, Fitness>,
}

fn validate_provenance(provenance: &Value, label: &str) {
    assert_eq!(provenance["oracle_cache"]["mode"], "no_cache", "{label}");
    assert_eq!(
        provenance["environment"]["excel_version"], "16.0",
        "{label}"
    );
    assert_eq!(provenance["environment"]["excel_build"], "20228", "{label}");
    assert_eq!(
        provenance["environment"]["excel_bitness"], "64-bit",
        "{label}"
    );
    assert_eq!(
        provenance["environment"]["workbook_compatibility"], "2",
        "{label}"
    );
    assert_eq!(
        provenance["environment"]["excel_input_plumbing"], "cell_value2_bulk",
        "{label}"
    );
}

fn load(root: &Path, label: &str, meta_name: &str, answer_name: &str) -> Vec<Row> {
    let metadata: MetaDocument =
        serde_json::from_slice(&std::fs::read(root.join(meta_name)).unwrap()).unwrap();
    let answers: WitnessSet =
        serde_json::from_slice(&std::fs::read(root.join(answer_name)).unwrap()).unwrap();
    assert_eq!(metadata.function, "CONVERT", "{label}");
    assert_eq!(answers.function, "CONVERT", "{label}");
    validate_provenance(&answers.capture_provenance, label);
    assert_eq!(
        metadata.rows.len(),
        answers.witnesses.len(),
        "{label} row drift"
    );
    metadata
        .rows
        .into_iter()
        .zip(answers.witnesses)
        .map(|(meta, witness)| {
            assert_eq!(meta.id, witness.id, "{label} id drift");
            assert_eq!(
                witness.args,
                serde_json::json!([meta.number_bits, meta.from_unit, meta.to_unit]),
                "{label} typed argument drift at {}",
                meta.id,
            );
            Row {
                id: meta.id,
                dataset: label.to_string(),
                number: common::f64_from_hex(&meta.number_bits).unwrap(),
                from: meta.from_unit,
                to: meta.to_unit,
                expected: witness.expected_bits,
            }
        })
        .collect()
}

fn load_readback(root: &Path) -> Vec<Row> {
    let document: ReadbackDoc = serde_json::from_slice(
        &std::fs::read(root.join("capture-convert-value2-readback-v2-20260809.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(document.environment["excel_version"], "16.0");
    assert_eq!(document.environment["excel_build"], "20228");
    assert_eq!(document.environment["workbook_compatibility"], "2");
    assert_eq!(
        document.environment["excel_input_plumbing"],
        "cell_value2_bulk_with_argument_readback"
    );
    document
        .rows
        .into_iter()
        .enumerate()
        .map(|(index, row)| Row {
            id: format!("readback-{index:02}"),
            dataset: "value2-readback-control".to_string(),
            number: common::f64_from_hex(&row.requested_bits).unwrap(),
            from: row.from_unit,
            to: row.to_unit,
            expected: row.requested_convert_bits,
        })
        .collect()
}

fn predicted(number: f64, from: &str, to: &str) -> String {
    match convert_kernel(number, from, to) {
        Ok(value) => format!("0x{:016x}", value.to_bits()),
        Err(WorksheetErrorCode::NA) => "error:NA".to_string(),
        Err(other) => format!("error:{other:?}"),
    }
}

fn main() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../work/w109/G4-convert");
    let sets = [
        (
            "discovery-bank",
            "batch-convert-discovery-20260809-meta.json",
            "answers-convert-discovery-20260809-clean.json",
        ),
        (
            "retired-v1-bank",
            "batch-convert-heldout-20260809-meta.json",
            "answers-convert-heldout-20260809.json",
        ),
        (
            "retired-v2-bank",
            "batch-convert-publication-heldout-v2-20260809-meta.json",
            "answers-convert-publication-heldout-v2-20260809.json",
        ),
        (
            "v3-refinement-only",
            "batch-convert-v3-length-discriminator-20260809-meta.json",
            "answers-convert-v3-length-discriminator-20260809.json",
        ),
        (
            "v3-publication",
            "batch-convert-publication-heldout-v3-20260809-meta.json",
            "answers-convert-publication-heldout-v3-20260809.json",
        ),
    ];
    let mut rows: Vec<_> = sets
        .into_iter()
        .flat_map(|(label, meta, answers)| load(&root, label, meta, answers))
        .collect();
    rows.extend(load_readback(&root));

    let mut datasets: BTreeMap<String, Fitness> = BTreeMap::new();
    let mut exact = 0;
    for row in &rows {
        let actual = predicted(row.number, &row.from, &row.to);
        let fitness = datasets.entry(row.dataset.clone()).or_default();
        fitness.total += 1;
        if actual == row.expected {
            fitness.exact += 1;
            exact += 1;
        } else if fitness.first_misses.len() < 32 {
            fitness.first_misses.push(format!(
                "{} (0x{:016x},{},{}) predicted={} oracle={}",
                row.id,
                row.number.to_bits(),
                row.from,
                row.to,
                actual,
                row.expected,
            ));
        }
    }
    for (dataset, fitness) in &datasets {
        println!("production {dataset}: {}/{}", fitness.exact, fitness.total);
    }
    println!("production total: {exact}/{}", rows.len());
    let report = Report {
        schema_version: "w109.convert.production_replay.v3",
        function: "CONVERT",
        freeze_id: "g4-05.convert.unified.20260809.v3",
        exact,
        total: rows.len(),
        datasets,
    };
    let out = root.join("score-convert-production-replay-v3.json");
    std::fs::write(&out, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
    println!("wrote -> {}", out.display());
    assert_eq!(exact, rows.len(), "compiled production replay mismatch");
}
