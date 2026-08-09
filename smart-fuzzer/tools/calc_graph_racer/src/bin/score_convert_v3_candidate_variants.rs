//! Retrospective score of the frozen v3 graph and its remaining schedule
//! controls over discovery, explicitly retired, and refinement-only evidence.

#[path = "convert_research/common.rs"]
mod common;
#[path = "convert_research/model_v3.rs"]
mod model_v3;

use common::MetaDocument;
use model_v3::CoreVariant;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Deserialize)]
struct WitnessSet {
    witnesses: Vec<Witness>,
}

#[derive(Deserialize)]
struct Witness {
    id: String,
    args: Value,
    expected_bits: String,
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
    evidence_rows: usize,
    variants: BTreeMap<String, Fitness>,
}

fn load(
    root: &Path,
    label: &str,
    meta: &str,
    answers: &str,
) -> Vec<(String, common::MetaRow, String)> {
    let metadata: MetaDocument =
        serde_json::from_slice(&std::fs::read(root.join(meta)).unwrap()).unwrap();
    let witnesses: WitnessSet =
        serde_json::from_slice(&std::fs::read(root.join(answers)).unwrap()).unwrap();
    let by_id: BTreeMap<_, _> = witnesses
        .witnesses
        .into_iter()
        .map(|w| (w.id.clone(), w))
        .collect();
    metadata
        .rows
        .into_iter()
        .map(|row| {
            let witness = &by_id[&row.id];
            assert_eq!(
                witness.args,
                serde_json::json!([row.number_bits, row.from_unit, row.to_unit])
            );
            (label.to_string(), row, witness.expected_bits.clone())
        })
        .collect()
}

fn main() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../work/w109/G4-convert");
    let sets = [
        (
            "discovery",
            "batch-convert-discovery-20260809-meta.json",
            "answers-convert-discovery-20260809-clean.json",
        ),
        (
            "retired-v1",
            "batch-convert-heldout-20260809-meta.json",
            "answers-convert-heldout-20260809.json",
        ),
        (
            "retired-v2",
            "batch-convert-publication-heldout-v2-20260809-meta.json",
            "answers-convert-publication-heldout-v2-20260809.json",
        ),
        (
            "v3-refinement",
            "batch-convert-v3-length-discriminator-20260809-meta.json",
            "answers-convert-v3-length-discriminator-20260809.json",
        ),
    ];
    let rows: Vec<_> = sets
        .into_iter()
        .flat_map(|(l, m, a)| load(&root, l, m, a))
        .collect();
    let variants = [
        ("frozen_v3", CoreVariant::FrozenV3),
        ("retired_v2", CoreVariant::RetiredV2),
        (
            "length_first_product_pc64",
            CoreVariant::LengthFirstProductPc64,
        ),
        (
            "first_product_pc64_all_linear",
            CoreVariant::FirstProductPc64AllLinear,
        ),
        (
            "length_second_operation_pc64",
            CoreVariant::LengthSecondOperationPc64,
        ),
    ];
    let mut scores = BTreeMap::new();
    for (name, variant) in variants {
        let mut fitness = Fitness::default();
        for (dataset, row, actual) in &rows {
            fitness.total += 1;
            let number = common::f64_from_hex(&row.number_bits).unwrap();
            let predicted =
                model_v3::predict(number, &row.category, &row.from_unit, &row.to_unit, variant)
                    .display();
            if predicted == *actual {
                fitness.exact += 1;
            } else if fitness.first_misses.len() < 32 {
                fitness.first_misses.push(format!(
                    "{dataset} {} predicted={predicted} oracle={actual}",
                    row.id
                ));
            }
        }
        println!("{name}: {}/{}", fitness.exact, fitness.total);
        scores.insert(name.to_string(), fitness);
    }
    let report = Report {
        schema_version: "w109.convert.v3_candidate_variant_score.v1",
        function: "CONVERT",
        evidence_rows: rows.len(),
        variants: scores,
    };
    let out = root.join("score-convert-v3-candidate-variants.json");
    std::fs::write(&out, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
    println!("wrote -> {}", out.display());
}
