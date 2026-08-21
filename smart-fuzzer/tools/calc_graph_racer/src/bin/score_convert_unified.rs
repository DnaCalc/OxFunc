//! Freeze and score the clean-room CONVERT graph family.
//!
//! The fixed portions of the model were selected only from the discovery
//! battery:
//! - length: binary64 `(x * from_angstroms) / to_angstroms`, followed by one
//!   binary64 multiply by `10^(from_prefix-to_prefix)`;
//! - pressure: public units-per-pascal binary64 ratio, followed by the same
//!   prefix multiply; `bar` is structurally unsupported on this baseline;
//! - temperature: direct-pair binary64 affine formulas using `1.8`, with an
//!   identity passthrough.
//!
//! Discovery does not distinguish a small, explicitly frozen family for
//! mass, time, and volume.  A held-out run selects each category's schedule
//! using even numeric row IDs only; odd IDs are the untouched post-selection
//! validation partition.  Constants and candidate graphs are never fitted
//! from the held-out answers.

#[path = "convert_research/common.rs"]
mod common;

use common::{MetaDocument, ordered_bits};
use oxfunc_core::excel_numeric::research as rx;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::path::PathBuf;

const FREEZE_ID: &str = "g4-05.convert.graph-family.20260809.v1";

const MASS_CANDIDATES: [&str; 5] = [
    "physical_f64_ratio_mul_pow10",
    "physical_f64_mul_div_pow10",
    "physical_x87_decimal_cont_pow10_f64",
    "physical_x87_decimal_ratio_store_f64_mul_pow10",
    "public_direct_f64_ratio_mul_pow10",
];

const TIME_CANDIDATES: [&str; 7] = [
    "physical_f64_ratio_mul_pow10",
    "physical_f64_mul_div_pow10",
    "physical_x87_decimal_cont_pow10_f64",
    "physical_x87_decimal_ratio_store_f64_mul_pow10",
    "public_direct_f64_ratio_mul_pow10",
    "public_direct_x87_decimal_cont_pow10_f64",
    "public_direct_x87_decimal_ratio_store_f64_mul_pow10",
];

const VOLUME_CANDIDATES: [&str; 2] = ["physical_f64_ratio_mul_pow10", "physical_f64_mul_div_pow10"];

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
    direct_name: String,
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
    first_misses: Vec<String>,
    #[serde(skip)]
    max_abs_ulp_value: u128,
    #[serde(skip)]
    sum_abs_ulp_value: u128,
}

impl Score {
    fn add(&mut self, row: &common::MetaRow, prediction: Prediction, actual: &str) {
        self.total += 1;
        let predicted = prediction.display();
        if predicted == actual {
            self.exact += 1;
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
                self.max_abs_ulp_value = self.max_abs_ulp_value.max(abs);
                self.sum_abs_ulp_value = self.sum_abs_ulp_value.saturating_add(abs);
                if self.first_misses.len() < 32 {
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
                if self.first_misses.len() < 32 {
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
        self.max_abs_ulp = self.max_abs_ulp_value.to_string();
        self.sum_abs_ulp = self.sum_abs_ulp_value.to_string();
    }
}

#[derive(Default, Serialize)]
struct Partitions {
    all: Score,
    calibration_even_id: Score,
    validation_odd_id: Score,
}

impl Partitions {
    fn add(&mut self, row: &common::MetaRow, prediction: Prediction, actual: &str) {
        self.all.add(row, prediction, actual);
        if calibration_row(&row.id) {
            self.calibration_even_id.add(row, prediction, actual);
        } else {
            self.validation_odd_id.add(row, prediction, actual);
        }
    }

    fn finish(&mut self) {
        self.all.finish();
        self.calibration_even_id.finish();
        self.validation_odd_id.finish();
    }
}

#[derive(Serialize)]
struct Report {
    schema_version: &'static str,
    function: &'static str,
    freeze_id: &'static str,
    metadata_schema_version: String,
    source_split: String,
    row_count: usize,
    model_manifest: Value,
    selected_by_calibration: BTreeMap<String, String>,
    candidate_scores: BTreeMap<String, BTreeMap<String, Partitions>>,
    unified_selected_score: Partitions,
    capture_provenance: Value,
}

struct Args {
    meta: PathBuf,
    answers: PathBuf,
    out: Option<PathBuf>,
}

fn parse_args() -> Args {
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
                    "score_convert_unified --meta <meta.json> --answers <answers.json> [--out <report.json>]"
                );
                std::process::exit(0);
            }
            other => panic!("unknown argument {other}"),
        }
    }
    Args {
        meta: meta.expect("--meta is required"),
        answers: answers.expect("--answers is required"),
        out,
    }
}

fn parse_bits(raw: &str) -> Option<u64> {
    let digits = raw.strip_prefix("0x")?;
    (digits.len() == 16)
        .then(|| u64::from_str_radix(digits, 16).ok())
        .flatten()
}

fn calibration_row(id: &str) -> bool {
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

fn base_unit(category: &str) -> Option<&'static str> {
    match category {
        "length" => Some("m"),
        "mass" => Some("g"),
        "time" => Some("sec"),
        "pressure" => Some("Pa"),
        "volume" => Some("l"),
        _ => None,
    }
}

fn resolve_unit(name: &str, category: &str) -> ResolvedUnit {
    if let Some(direct) = common::direct_unit(name) {
        assert_eq!(
            direct.category.name(),
            category,
            "category drift for {name}"
        );
        return ResolvedUnit {
            direct_name: name.to_string(),
            prefix_exponent: 0,
        };
    }
    let base = base_unit(category).unwrap_or_else(|| panic!("no prefix base for {category}"));
    for (prefix, _) in common::PREFIXES {
        if name == format!("{prefix}{base}") {
            return ResolvedUnit {
                direct_name: base.to_string(),
                prefix_exponent: prefix_exponent(prefix),
            };
        }
    }
    panic!("cannot resolve {category} unit {name}");
}

fn pow10(exponent: i32) -> f64 {
    format!("1e{exponent}").parse::<f64>().unwrap()
}

fn physical_decimal(unit: &str) -> &'static str {
    common::DIRECT_UNITS
        .iter()
        .find(|candidate| candidate.name == unit)
        .unwrap_or_else(|| panic!("no physical constant for {unit}"))
        .factor_decimal
}

fn public_direct_decimal(unit: &str) -> &'static str {
    match unit {
        "g" | "sec" | "Pa" => "1",
        "lbm" => "2.2046226218487758E-03",
        "ozm" => "3.5273961949580414E-02",
        "day" => "1.1574074074074074E-05",
        "hr" => "2.7777777777777778E-04",
        "mn" => "1.6666666666666667E-02",
        "atm" => "9.8692326671601280E-06",
        "psi" => "1.4503773773020920E-04",
        other => panic!("no frozen public direct constant for {other}"),
    }
}

fn physical_prediction(model: &str, number: f64, from: &ResolvedUnit, to: &ResolvedUnit) -> f64 {
    let from_decimal = physical_decimal(&from.direct_name);
    let to_decimal = physical_decimal(&to.direct_name);
    let prefix = pow10(from.prefix_exponent - to.prefix_exponent);
    let from_f64 = from_decimal.parse::<f64>().unwrap();
    let to_f64 = to_decimal.parse::<f64>().unwrap();
    let cw = rx::CW_PC64_RN;
    match model {
        "physical_f64_ratio_mul_pow10" => (number * (from_f64 / to_f64)) * prefix,
        "physical_f64_mul_div_pow10" => ((number * from_f64) / to_f64) * prefix,
        "physical_x87_decimal_cont_pow10_f64" => {
            let ratio = rx::ext_div(
                &common::ext_from_decimal(from_decimal),
                &common::ext_from_decimal(to_decimal),
                cw,
            );
            let core = rx::ext_mul(&rx::ext_from_f64(number), &ratio, cw);
            rx::ext_to_f64(&core, cw) * prefix
        }
        "physical_x87_decimal_ratio_store_f64_mul_pow10" => {
            let ratio = rx::ext_div(
                &common::ext_from_decimal(from_decimal),
                &common::ext_from_decimal(to_decimal),
                cw,
            );
            (number * rx::ext_to_f64(&ratio, cw)) * prefix
        }
        other => panic!("unknown physical model {other}"),
    }
}

fn public_prediction(model: &str, number: f64, from: &ResolvedUnit, to: &ResolvedUnit) -> f64 {
    let from_decimal = public_direct_decimal(&from.direct_name);
    let to_decimal = public_direct_decimal(&to.direct_name);
    let prefix = pow10(from.prefix_exponent - to.prefix_exponent);
    let from_f64 = from_decimal.parse::<f64>().unwrap();
    let to_f64 = to_decimal.parse::<f64>().unwrap();
    let cw = rx::CW_PC64_RN;
    match model {
        "public_direct_f64_ratio_mul_pow10" => (number * (to_f64 / from_f64)) * prefix,
        "public_direct_x87_decimal_cont_pow10_f64" => {
            let ratio = rx::ext_div(
                &common::ext_from_decimal(to_decimal),
                &common::ext_from_decimal(from_decimal),
                cw,
            );
            let core = rx::ext_mul(&rx::ext_from_f64(number), &ratio, cw);
            rx::ext_to_f64(&core, cw) * prefix
        }
        "public_direct_x87_decimal_ratio_store_f64_mul_pow10" => {
            let ratio = rx::ext_div(
                &common::ext_from_decimal(to_decimal),
                &common::ext_from_decimal(from_decimal),
                cw,
            );
            (number * rx::ext_to_f64(&ratio, cw)) * prefix
        }
        other => panic!("unknown public-direct model {other}"),
    }
}

fn candidate_prediction(
    category: &str,
    model: &str,
    number: f64,
    from: &ResolvedUnit,
    to: &ResolvedUnit,
) -> Prediction {
    let value = if model.starts_with("physical_") {
        physical_prediction(model, number, from, to)
    } else {
        public_prediction(model, number, from, to)
    };
    assert!(matches!(category, "mass" | "time" | "volume"));
    Prediction::Numeric(value.to_bits())
}

fn angstroms(unit: &str) -> f64 {
    match unit {
        "m" => 10_000_000_000.0,
        "in" => 254_000_000.0,
        "ft" => 3_048_000_000.0,
        "yd" => 9_144_000_000.0,
        "mi" => 16_093_440_000_000.0,
        "Nmi" => 18_520_000_000_000.0,
        other => panic!("no angstrom constant for {other}"),
    }
}

fn length_prediction(number: f64, from: &ResolvedUnit, to: &ResolvedUnit) -> f64 {
    let core = (number * angstroms(&from.direct_name)) / angstroms(&to.direct_name);
    core * pow10(from.prefix_exponent - to.prefix_exponent)
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

fn fixed_prediction(row: &common::MetaRow, selected: &BTreeMap<String, String>) -> Prediction {
    let number = common::f64_from_hex(&row.number_bits).unwrap();
    if row.category == "temperature" {
        return Prediction::Numeric(
            temperature_prediction(number, &row.from_unit, &row.to_unit).to_bits(),
        );
    }
    let from = resolve_unit(&row.from_unit, &row.category);
    let to = resolve_unit(&row.to_unit, &row.category);
    match row.category.as_str() {
        "length" => Prediction::Numeric(length_prediction(number, &from, &to).to_bits()),
        "pressure" => {
            if from.direct_name == "bar" || to.direct_name == "bar" {
                Prediction::ErrorNa
            } else {
                Prediction::Numeric(
                    public_prediction("public_direct_f64_ratio_mul_pow10", number, &from, &to)
                        .to_bits(),
                )
            }
        }
        "mass" | "time" | "volume" => {
            candidate_prediction(&row.category, &selected[&row.category], number, &from, &to)
        }
        other => panic!("unknown category {other}"),
    }
}

fn candidate_names(category: &str) -> &'static [&'static str] {
    match category {
        "mass" => &MASS_CANDIDATES,
        "time" => &TIME_CANDIDATES,
        "volume" => &VOLUME_CANDIDATES,
        other => panic!("no candidate family for {other}"),
    }
}

fn expected_args(row: &common::MetaRow) -> Value {
    json!([row.number_bits, row.from_unit, row.to_unit])
}

fn model_manifest() -> Value {
    json!({
        "freeze_id": FREEZE_ID,
        "selection_policy": {
            "calibration": "even numeric row-id suffix",
            "validation": "odd numeric row-id suffix",
            "ranking": "fewest calibration mismatches, then lowest calibration sum_abs_ulp, then frozen candidate order",
            "heldout_constant_fitting": false
        },
        "fixed_graphs": {
            "length": "f64(((number * integer_angstroms[from]) / integer_angstroms[to]) * f64_pow10(prefix_from-prefix_to))",
            "pressure": "f64((number * (units_per_pa[to] / units_per_pa[from])) * f64_pow10(prefix_from-prefix_to)); bar=>error:NA",
            "temperature": "direct binary64 affine pair graph using 1.8; from==to returns number"
        },
        "candidates": {
            "mass": MASS_CANDIDATES,
            "time": TIME_CANDIDATES,
            "volume": VOLUME_CANDIDATES
        },
        "constants": {
            "length_integer_angstroms_per_unit": {
                "m": "10000000000", "in": "254000000", "ft": "3048000000",
                "yd": "9144000000", "mi": "16093440000000", "Nmi": "18520000000000"
            },
            "pressure_units_per_pa_decimal": {
                "Pa": "1", "atm": "9.8692326671601280E-06", "psi": "1.4503773773020920E-04"
            },
            "temperature": {"offset_kelvin_celsius": "273.15", "fahrenheit_scale": "1.8", "fahrenheit_offset": "32"},
            "prefix_exponents": {"Y":24,"Z":21,"E":18,"P":15,"T":12,"G":9,"M":6,"k":3,"h":2,"da":1,"d":-1,"c":-2,"m":-3,"u":-6,"n":-9,"p":-12,"f":-15}
        }
    })
}

fn main() {
    let args = parse_args();
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
    assert_eq!(metadata.rows.len(), by_id.len(), "row-count drift");

    let mut candidate_scores: BTreeMap<String, BTreeMap<String, Partitions>> = BTreeMap::new();
    for category in ["mass", "time", "volume"] {
        candidate_scores.insert(
            category.to_string(),
            candidate_names(category)
                .iter()
                .map(|name| ((*name).to_string(), Partitions::default()))
                .collect(),
        );
    }

    for row in &metadata.rows {
        let witness = by_id[&row.id.as_str()];
        assert_eq!(
            witness.args,
            expected_args(row),
            "argument drift at {}",
            row.id
        );
        if !matches!(row.category.as_str(), "mass" | "time" | "volume") {
            continue;
        }
        let number = common::f64_from_hex(&row.number_bits).unwrap();
        let from = resolve_unit(&row.from_unit, &row.category);
        let to = resolve_unit(&row.to_unit, &row.category);
        for model in candidate_names(&row.category) {
            let prediction = candidate_prediction(&row.category, model, number, &from, &to);
            candidate_scores
                .get_mut(&row.category)
                .unwrap()
                .get_mut(*model)
                .unwrap()
                .add(row, prediction, &witness.expected_bits);
        }
    }

    let mut selected = BTreeMap::new();
    for category in ["mass", "time", "volume"] {
        let scores = &candidate_scores[category];
        let best = candidate_names(category)
            .iter()
            .min_by_key(|name| {
                let score = &scores[**name].calibration_even_id;
                (score.mismatch, score.sum_abs_ulp_value)
            })
            .unwrap();
        selected.insert(category.to_string(), (**best).to_string());
    }

    let mut unified = Partitions::default();
    for row in &metadata.rows {
        let witness = by_id[&row.id.as_str()];
        unified.add(
            row,
            fixed_prediction(row, &selected),
            &witness.expected_bits,
        );
    }

    for category in candidate_scores.values_mut() {
        for score in category.values_mut() {
            score.finish();
        }
    }
    unified.finish();

    let source_split = metadata
        .rows
        .first()
        .map(|row| row.split.clone())
        .unwrap_or_default();
    println!("freeze_id={FREEZE_ID}");
    println!("source_split={source_split} rows={}", metadata.rows.len());
    for (category, model) in &selected {
        let score = &candidate_scores[category][model];
        println!(
            "selected {category}: {model}; calibration={}/{} validation={}/{} all={}/{}",
            score.calibration_even_id.exact,
            score.calibration_even_id.total,
            score.validation_odd_id.exact,
            score.validation_odd_id.total,
            score.all.exact,
            score.all.total,
        );
    }
    println!(
        "unified: calibration={}/{} validation={}/{} all={}/{}",
        unified.calibration_even_id.exact,
        unified.calibration_even_id.total,
        unified.validation_odd_id.exact,
        unified.validation_odd_id.total,
        unified.all.exact,
        unified.all.total,
    );

    let report = Report {
        schema_version: "w109.convert.unified_graph_score.v1",
        function: "CONVERT",
        freeze_id: FREEZE_ID,
        metadata_schema_version: metadata.schema_version,
        source_split,
        row_count: metadata.rows.len(),
        model_manifest: model_manifest(),
        selected_by_calibration: selected,
        candidate_scores,
        unified_selected_score: unified,
        capture_provenance: answers.capture_provenance,
    };
    if let Some(path) = args.out {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(&path, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
        println!("wrote unified score -> {}", path.display());
    }
}
