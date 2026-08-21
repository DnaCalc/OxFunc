//! Score the frozen v2 clean-room CONVERT graph.
//!
//! Identification evidence is limited to the original discovery battery and
//! the explicitly retired v1 held-out.  The graph is fixed before generation
//! or capture of the disjoint publication held-out.

#[path = "convert_research/common.rs"]
mod common;

use common::{MetaDocument, ordered_bits};
use oxfunc_core::excel_numeric::research as rx;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::path::PathBuf;

const FREEZE_ID: &str = "g4-05.convert.unified.20260809.v2";

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

#[derive(Clone)]
struct ResolvedUnit {
    direct: String,
    prefix_exponent: i32,
}

#[derive(Clone, Copy)]
enum Prediction {
    Numeric(u64),
    ErrorNa,
}

impl Prediction {
    fn display(self) -> String {
        match self {
            Self::Numeric(bits) => format!("0x{bits:016x}"),
            Self::ErrorNa => "error:NA".to_string(),
        }
    }
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
        let predicted = prediction.display();
        if predicted == actual {
            self.exact += 1;
            *self.category_exact.entry(row.category.clone()).or_default() += 1;
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
                    "score_convert_unified_v2 --meta <meta.json> --answers <answers.json> [--out <report.json>]"
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

fn prefix_exponent(prefix: &str) -> i32 {
    match prefix {
        "Y" => 24,
        "Z" => 21,
        "E" => 18,
        "P" => 15,
        "T" => 12,
        "G" => 9,
        "M" => 6,
        "k" => 3,
        "h" => 2,
        "da" => 1,
        "d" => -1,
        "c" => -2,
        "m" => -3,
        "u" => -6,
        "n" => -9,
        "p" => -12,
        "f" => -15,
        other => panic!("unknown prefix {other}"),
    }
}

fn prefix_base(category: &str) -> Option<&'static str> {
    match category {
        "length" => Some("m"),
        "mass" => Some("g"),
        "time" => Some("sec"),
        "pressure" => Some("Pa"),
        "volume" => Some("l"),
        _ => None,
    }
}

fn resolve(unit: &str, category: &str) -> ResolvedUnit {
    if let Some(direct) = common::direct_unit(unit) {
        assert_eq!(
            direct.category.name(),
            category,
            "category drift for {unit}"
        );
        return ResolvedUnit {
            direct: unit.to_string(),
            prefix_exponent: 0,
        };
    }
    let base = prefix_base(category).unwrap_or_else(|| panic!("no prefix base for {category}"));
    for (prefix, _) in common::PREFIXES {
        if unit == format!("{prefix}{base}") {
            return ResolvedUnit {
                direct: base.to_string(),
                prefix_exponent: prefix_exponent(prefix),
            };
        }
    }
    panic!("cannot resolve {category} unit {unit}");
}

fn angstroms(unit: &str) -> f64 {
    match unit {
        "m" => 10_000_000_000.0,
        "in" => 254_000_000.0,
        "ft" => 3_048_000_000.0,
        "yd" => 9_144_000_000.0,
        "mi" => 16_093_440_000_000.0,
        "Nmi" => 18_520_000_000_000.0,
        other => panic!("no angstrom factor for {other}"),
    }
}

fn physical_factor(unit: &str) -> f64 {
    common::DIRECT_UNITS
        .iter()
        .find(|candidate| candidate.name == unit)
        .unwrap_or_else(|| panic!("no physical factor for {unit}"))
        .factor_decimal
        .parse()
        .unwrap()
}

fn pressure_factor(unit: &str) -> f64 {
    // The public table spells units per pascal.  Excel's two-cell graph is
    // reproduced by first rounding each reciprocal independently to f64.
    match unit {
        "Pa" => 1.0,
        "atm" => 1.0 / "9.8692326671601280E-06".parse::<f64>().unwrap(),
        "psi" => 1.0 / "1.4503773773020920E-04".parse::<f64>().unwrap(),
        other => panic!("no supported pressure factor for {other}"),
    }
}

fn factor_to_base(category: &str, unit: &str) -> f64 {
    match category {
        "length" => angstroms(unit),
        "mass" | "time" | "volume" => physical_factor(unit),
        "pressure" => pressure_factor(unit),
        other => panic!("no linear factor for {other}"),
    }
}

fn pow10(exponent: i32) -> f64 {
    format!("1e{exponent}").parse().unwrap()
}

fn linear_prediction(number: f64, category: &str, from: &ResolvedUnit, to: &ResolvedUnit) -> f64 {
    let core =
        (number * factor_to_base(category, &from.direct)) / factor_to_base(category, &to.direct);
    let delta = pow10(from.prefix_exponent - to.prefix_exponent);
    let cw = rx::CW_PC64_RN;
    let extended = rx::ext_mul(&rx::ext_from_f64(core), &rx::ext_from_f64(delta), cw);
    rx::ext_to_f64(&extended, cw)
}

fn temperature_prediction(number: f64, from: &str, to: &str) -> f64 {
    match (from, to) {
        (a, b) if a == b => number,
        ("K", "C") => number - 273.15,
        ("C", "K") => number + 273.15,
        ("K", "F") => (number - 273.15) * 1.8 + 32.0,
        ("F", "K") => (number - 32.0) / 1.8 + 273.15,
        ("C", "F") => number * 1.8 + 32.0,
        ("F", "C") => (number - 32.0) / 1.8,
        _ => panic!("unknown temperature pair {from}->{to}"),
    }
}

fn prediction(row: &common::MetaRow) -> Prediction {
    let number = common::f64_from_hex(&row.number_bits).unwrap();
    if row.category == "temperature" {
        return Prediction::Numeric(
            temperature_prediction(number, &row.from_unit, &row.to_unit).to_bits(),
        );
    }
    let from = resolve(&row.from_unit, &row.category);
    let to = resolve(&row.to_unit, &row.category);
    if row.category == "pressure" && (from.direct == "bar" || to.direct == "bar") {
        return Prediction::ErrorNa;
    }
    Prediction::Numeric(linear_prediction(number, &row.category, &from, &to).to_bits())
}

fn expected_args(row: &common::MetaRow) -> Value {
    json!([row.number_bits, row.from_unit, row.to_unit])
}

fn model_manifest() -> Value {
    json!({
        "freeze_id": FREEZE_ID,
        "selection_status": "fixed before disjoint publication heldout generation",
        "linear_graph": [
            "from_factor = f64(table[from_direct])",
            "to_factor = f64(table[to_direct])",
            "core = f64(f64(number * from_factor) / to_factor)",
            "delta = f64(decimal 10^(from_prefix_exponent-to_prefix_exponent))",
            "result = f64(x87_pc64(core * delta))"
        ],
        "tables": {
            "length_integer_angstroms_per_unit": {
                "m":"10000000000", "in":"254000000", "ft":"3048000000",
                "yd":"9144000000", "mi":"16093440000000", "Nmi":"18520000000000"
            },
            "mass_grams_per_unit": {"g":"1", "lbm":"453.59237", "ozm":"28.349523125"},
            "time_seconds_per_unit": {"sec":"1", "mn":"60", "hr":"3600", "day":"86400"},
            "pressure_factor_construction": {
                "Pa":"1", "atm":"f64(1 / f64(9.8692326671601280E-06))",
                "psi":"f64(1 / f64(1.4503773773020920E-04))", "bar":"unsupported=>error:NA"
            },
            "volume_liters_per_unit": {
                "l":"1", "tsp":"0.00492892159375", "tbs":"0.01478676478125",
                "oz":"0.0295735295625", "cup":"0.2365882365", "pt":"0.473176473",
                "qt":"0.946352946", "gal":"3.785411784"
            }
        },
        "temperature_graph": "direct pair binary64 affine formulas with 273.15, 1.8, and 32; identity passthrough"
    })
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
        score.add(row, prediction(row), &witness.expected_bits);
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
            total,
        );
    }

    let report = Report {
        schema_version: "w109.convert.unified_graph_score.v2",
        function: "CONVERT",
        freeze_id: FREEZE_ID,
        source_split,
        row_count: metadata.rows.len(),
        model_manifest: model_manifest(),
        score,
        capture_provenance: answers.capture_provenance,
    };
    if let Some(path) = args.out {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(&path, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
        println!("wrote unified v2 score -> {}", path.display());
    }
}
