//! Score the frozen v3 clean-room CONVERT graph.
//!
//! The scorer is immutable publication-gate machinery: model selection used
//! only discovery, explicitly retired v1/v2 sets, the refinement-only v3
//! discriminator, and the independent Value2/readback control.

#[path = "convert_research/common.rs"]
mod common;
#[path = "convert_research/model_v3.rs"]
mod model_v3;

use common::{MetaDocument, ordered_bits};
use model_v3::{FREEZE_ID, Prediction};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
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
    args: Value,
    expected_bits: String,
}

#[derive(Default, Serialize)]
struct Score {
    total: usize,
    exact: usize,
    mismatch: usize,
    numeric_compared: usize,
    nonnumeric_compared: usize,
    max_abs_ulp: String,
    sum_abs_ulp: String,
    category_exact: BTreeMap<String, usize>,
    category_total: BTreeMap<String, usize>,
    class_exact: BTreeMap<String, usize>,
    class_total: BTreeMap<String, usize>,
    first_misses: Vec<String>,
    #[serde(skip)]
    max_value: u128,
    #[serde(skip)]
    sum_value: u128,
}

impl Score {
    fn add(&mut self, row: &common::MetaRow, prediction: Prediction, actual: &str) {
        self.total += 1;
        *self.category_total.entry(row.category.clone()).or_default() += 1;
        *self.class_total.entry(row.class.clone()).or_default() += 1;
        let predicted = prediction.display();
        if predicted == actual {
            self.exact += 1;
            *self.category_exact.entry(row.category.clone()).or_default() += 1;
            *self.class_exact.entry(row.class.clone()).or_default() += 1;
            match prediction {
                Prediction::Numeric(_) => self.numeric_compared += 1,
                Prediction::ErrorNa => self.nonnumeric_compared += 1,
            }
            return;
        }
        self.mismatch += 1;
        match (prediction, parse_bits(actual)) {
            (Prediction::Numeric(predicted_bits), Some(actual_bits)) => {
                self.numeric_compared += 1;
                let residual = ordered_bits(actual_bits) - ordered_bits(predicted_bits);
                let abs = residual.unsigned_abs();
                self.max_value = self.max_value.max(abs);
                self.sum_value = self.sum_value.saturating_add(abs);
                if self.first_misses.len() < 64 {
                    self.first_misses.push(format!(
                        "{} {} {}({},{},{}) residual={:+} predicted=0x{:016x} oracle=0x{:016x}",
                        row.id,
                        row.category,
                        row.class,
                        row.number_bits,
                        row.from_unit,
                        row.to_unit,
                        residual,
                        predicted_bits,
                        actual_bits,
                    ));
                }
            }
            _ => {
                self.nonnumeric_compared += 1;
                if self.first_misses.len() < 64 {
                    self.first_misses.push(format!(
                        "{} {} {}({},{},{}) predicted={} oracle={}",
                        row.id,
                        row.category,
                        row.class,
                        row.number_bits,
                        row.from_unit,
                        row.to_unit,
                        predicted,
                        actual,
                    ));
                }
            }
        }
    }

    fn finish(&mut self) {
        self.max_abs_ulp = self.max_value.to_string();
        self.sum_abs_ulp = self.sum_value.to_string();
    }
}

#[derive(Default, Serialize)]
struct Partitions {
    all: Score,
    even_numeric_id_suffix: Score,
    odd_numeric_id_suffix: Score,
}

impl Partitions {
    fn add(&mut self, row: &common::MetaRow, prediction: Prediction, actual: &str) {
        self.all.add(row, prediction, actual);
        if row_id_even(&row.id) {
            self.even_numeric_id_suffix.add(row, prediction, actual);
        } else {
            self.odd_numeric_id_suffix.add(row, prediction, actual);
        }
    }

    fn finish(&mut self) {
        self.all.finish();
        self.even_numeric_id_suffix.finish();
        self.odd_numeric_id_suffix.finish();
    }
}

#[derive(Serialize)]
struct Report {
    schema_version: &'static str,
    function: &'static str,
    freeze_id: &'static str,
    source_split: String,
    row_count: usize,
    model_manifest: Value,
    score: Partitions,
    capture_provenance: Value,
}

struct Args {
    meta: PathBuf,
    answers: PathBuf,
    out: Option<PathBuf>,
}

fn args() -> Args {
    let mut meta = None;
    let mut answers = None;
    let mut out = None;
    let mut values = std::env::args().skip(1);
    while let Some(arg) = values.next() {
        match arg.as_str() {
            "--meta" => meta = values.next().map(PathBuf::from),
            "--answers" => answers = values.next().map(PathBuf::from),
            "--out" => out = values.next().map(PathBuf::from),
            "-h" | "--help" => {
                println!(
                    "score_convert_unified_v3 --meta <meta.json> --answers <answers.json> [--out <report.json>]"
                );
                std::process::exit(0);
            }
            other => panic!("unknown argument {other}"),
        }
    }
    Args {
        meta: meta.expect("--meta"),
        answers: answers.expect("--answers"),
        out,
    }
}

fn parse_bits(raw: &str) -> Option<u64> {
    let digits = raw.strip_prefix("0x")?;
    (digits.len() == 16)
        .then(|| u64::from_str_radix(digits, 16).ok())
        .flatten()
}

fn row_id_even(id: &str) -> bool {
    id.rsplit('-')
        .next()
        .expect("row id suffix")
        .parse::<usize>()
        .expect("numeric row id suffix")
        % 2
        == 0
}

fn expected_args(row: &common::MetaRow) -> Value {
    json!([row.number_bits, row.from_unit, row.to_unit])
}

fn main() {
    let args = args();
    let metadata: MetaDocument =
        serde_json::from_slice(&std::fs::read(&args.meta).unwrap()).unwrap();
    let answers: WitnessSet =
        serde_json::from_slice(&std::fs::read(&args.answers).unwrap()).unwrap();
    assert_eq!(metadata.function, "CONVERT");
    assert_eq!(answers.function, "CONVERT");
    let mut by_id = BTreeMap::new();
    for witness in &answers.witnesses {
        assert!(
            by_id.insert(witness.id.as_str(), witness).is_none(),
            "duplicate witness {}",
            witness.id
        );
    }
    assert_eq!(metadata.rows.len(), by_id.len(), "row count drift");

    let mut score = Partitions::default();
    for row in &metadata.rows {
        let witness = by_id[&row.id.as_str()];
        assert_eq!(
            witness.args,
            expected_args(row),
            "argument drift at {}",
            row.id
        );
        let number = common::f64_from_hex(&row.number_bits).unwrap();
        score.add(
            row,
            model_v3::predict_frozen(number, &row.category, &row.from_unit, &row.to_unit),
            &witness.expected_bits,
        );
    }
    score.finish();

    let source_split = metadata
        .rows
        .first()
        .map(|row| row.split.clone())
        .unwrap_or_default();
    println!("freeze_id={FREEZE_ID}");
    println!("source_split={source_split} rows={}", metadata.rows.len());
    println!(
        "all={}/{} even={}/{} odd={}/{} max_ulp={} sum_ulp={}",
        score.all.exact,
        score.all.total,
        score.even_numeric_id_suffix.exact,
        score.even_numeric_id_suffix.total,
        score.odd_numeric_id_suffix.exact,
        score.odd_numeric_id_suffix.total,
        score.all.max_abs_ulp,
        score.all.sum_abs_ulp,
    );
    for (category, total) in &score.all.category_total {
        println!(
            "  {category}: {}/{}",
            score.all.category_exact.get(category).copied().unwrap_or(0),
            total
        );
    }

    let report = Report {
        schema_version: "w109.convert.unified_graph_score.v3",
        function: "CONVERT",
        freeze_id: FREEZE_ID,
        source_split,
        row_count: metadata.rows.len(),
        model_manifest: model_v3::model_manifest(),
        score,
        capture_provenance: answers.capture_provenance,
    };
    if let Some(path) = args.out {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(&path, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
        println!("wrote unified v3 score -> {}", path.display());
    }
}
