//! Identify CONVERT's arithmetic graph after fitting only the units-per-base
//! constants exposed by discovery `x = 1` base-to-unit probes.
//!
//! This is deliberately a split scorer: constants are learned from one small,
//! explicitly identified subset, while operation schedules are scored on all
//! remaining rows (or on a separate held-out capture).  It is therefore useful
//! both for clean-room discovery and for a later frozen-model validation.
//!
//! Usage:
//!   cargo run --release --bin race_convert_fitted_graphs -- \
//!     --fit-meta <discovery-meta.json> \
//!     --fit-answers <discovery-answers.json> \
//!     --score-meta <discovery-or-heldout-meta.json> \
//!     --score-answers <discovery-or-heldout-answers.json> \
//!     [--out <report.json>]

#[path = "convert_research/common.rs"]
mod common;

use common::{MetaDocument, ordered_bits};
use oxfunc_core::excel_numeric::research as rx;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

const MODELS: [&str; 40] = [
    "f64_ratio_mul",
    "f64_div_mul",
    "f64_mul_div",
    "x87_f64_ratio_cont_pc64",
    "x87_f64_div_mul_cont_pc64",
    "x87_f64_mul_div_cont_pc64",
    "x87_f64_ratio_store_f64_mul",
    "x87_f64_div_store_f64_mul",
    "x87_f64_mul_store_f64_div",
    "x87_f64_ratio_cont_pc53",
    "table_f64_ratio_mul_pow10",
    "table_f64_mul_div_pow10",
    "table_x87_f64_ratio_cont_pow10_f64",
    "table_x87_f64_ratio_store_f64_mul_pow10",
    "table_x87_f64_ratio_cont_pow10_ext",
    "table_x87_f64_ratio_pow10_store_f64_mul",
    "table_x87_f64_ratio_pow10_cont",
    "physical_f64_ratio_mul_pow10",
    "physical_f64_mul_div_pow10",
    "physical_x87_decimal_cont_pow10_f64",
    "physical_x87_decimal_ratio_store_f64_mul_pow10",
    "physical_x87_decimal_cont_pow10_ext",
    "physical_x87_decimal_ratio_pow10_store_f64_mul",
    "physical_x87_decimal_ratio_pow10_cont",
    "public_direct_f64_ratio_mul_pow10",
    "public_direct_x87_decimal_ratio_store_f64_mul_pow10",
    "public_direct_x87_decimal_ratio_cont_pow10_f64",
    "public_direct_x87_decimal_ratio_pow10_store_f64_mul",
    "public_direct_x87_decimal_ratio_pow10_cont",
    "table_x87_f64_ratio_prefix_staged_final_rz",
    "table_x87_f64_divmul_prefix_staged_final_rz",
    "table_x87_f64_prefix_divmul_final_rz",
    "physical_x87_decimal_ratio_prefix_staged_final_rz",
    "physical_x87_decimal_muldiv_prefix_staged_final_rz",
    "physical_x87_decimal_prefix_muldiv_final_rz",
    "public_direct_x87_decimal_ratio_prefix_staged_final_rz",
    "public_direct_x87_decimal_divmul_prefix_staged_final_rz",
    "table_x87_f64_ratio_prefix_staged_all_rz",
    "physical_x87_decimal_ratio_prefix_staged_all_rz",
    "public_direct_x87_decimal_ratio_prefix_staged_all_rz",
];

#[derive(Clone, Debug)]
struct ResolvedUnit {
    direct_name: String,
    prefix_exponent: i32,
}

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

#[derive(Clone, Serialize)]
struct FittedConstant {
    category: String,
    unit: String,
    source_id: String,
    bits: String,
}

#[derive(Default, Serialize)]
struct ModelScore {
    numeric_total: usize,
    exact: usize,
    mismatch: usize,
    structural_exact: usize,
    structural_mismatch: usize,
    max_abs_ulp: String,
    sum_abs_ulp: String,
    class_exact: BTreeMap<String, usize>,
    class_total: BTreeMap<String, usize>,
    category_exact: BTreeMap<String, usize>,
    category_total: BTreeMap<String, usize>,
    pair_exact: BTreeMap<String, usize>,
    pair_total: BTreeMap<String, usize>,
    first_misses: Vec<String>,
}

#[derive(Serialize)]
struct ScoreReport {
    schema_version: &'static str,
    function: &'static str,
    fit_rows: usize,
    fitted_constants: Vec<FittedConstant>,
    score_rows: usize,
    excluded_fit_rows: usize,
    unmodeled_controls: usize,
    exact_survivors: Vec<String>,
    scores: BTreeMap<String, ModelScore>,
    fit_capture_provenance: Value,
    score_capture_provenance: Value,
}

struct Args {
    fit_meta: PathBuf,
    fit_answers: PathBuf,
    score_meta: PathBuf,
    score_answers: PathBuf,
    out: Option<PathBuf>,
}

fn parse_args() -> Args {
    let mut fit_meta = None;
    let mut fit_answers = None;
    let mut score_meta = None;
    let mut score_answers = None;
    let mut out = None;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--fit-meta" => fit_meta = args.next().map(PathBuf::from),
            "--fit-answers" => fit_answers = args.next().map(PathBuf::from),
            "--score-meta" => score_meta = args.next().map(PathBuf::from),
            "--score-answers" => score_answers = args.next().map(PathBuf::from),
            "--out" => out = args.next().map(PathBuf::from),
            "-h" | "--help" => {
                println!(
                    "race_convert_fitted_graphs --fit-meta <meta.json> --fit-answers <answers.json> --score-meta <meta.json> --score-answers <answers.json> [--out <report.json>]"
                );
                std::process::exit(0);
            }
            other => panic!("unknown argument {other}"),
        }
    }
    Args {
        fit_meta: fit_meta.expect("--fit-meta is required"),
        fit_answers: fit_answers.expect("--fit-answers is required"),
        score_meta: score_meta.expect("--score-meta is required"),
        score_answers: score_answers.expect("--score-answers is required"),
        out,
    }
}

fn parse_bits(raw: &str) -> Option<u64> {
    let digits = raw.strip_prefix("0x")?;
    (digits.len() == 16)
        .then(|| u64::from_str_radix(digits, 16).ok())
        .flatten()
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
        _ => panic!("unknown prefix {prefix}"),
    }
}

fn resolve_unit(name: &str, category: &str) -> Option<ResolvedUnit> {
    if let Some(direct) = common::direct_unit(name) {
        return (direct.category.name() == category).then(|| ResolvedUnit {
            direct_name: name.to_string(),
            prefix_exponent: 0,
        });
    }
    let base = base_unit(category)?;
    for (prefix, _) in common::PREFIXES {
        if name == format!("{prefix}{base}") {
            return Some(ResolvedUnit {
                direct_name: base.to_string(),
                prefix_exponent: prefix_exponent(prefix),
            });
        }
    }
    None
}

fn answer_map(answers: &WitnessSet) -> BTreeMap<&str, &Witness> {
    let mut by_id = BTreeMap::new();
    for witness in &answers.witnesses {
        assert!(
            by_id.insert(witness.id.as_str(), witness).is_none(),
            "duplicate witness id {}",
            witness.id
        );
    }
    by_id
}

fn expected_args(row: &common::MetaRow) -> Value {
    serde_json::json!([row.number_bits, row.from_unit, row.to_unit])
}

fn fit_constants(
    metadata: &MetaDocument,
    answers: &WitnessSet,
) -> (
    BTreeMap<(String, String), f64>,
    BTreeSet<String>,
    Vec<FittedConstant>,
) {
    let answer_by_id = answer_map(answers);
    let mut constants = BTreeMap::new();
    let mut fit_ids = BTreeSet::new();
    let mut report = Vec::new();

    for row in &metadata.rows {
        if row.number_bits != "0x3ff0000000000000"
            || base_unit(&row.category) != Some(row.from_unit.as_str())
        {
            continue;
        }
        let witness = answer_by_id
            .get(row.id.as_str())
            .unwrap_or_else(|| panic!("missing fit answer {}", row.id));
        assert_eq!(witness.args, expected_args(row), "fit argument drift at {}", row.id);
        let Some(bits) = parse_bits(&witness.expected_bits) else {
            // Unsupported units (notably `bar` on the current reference
            // baseline) intentionally have no fitted constant.
            continue;
        };
        let key = (row.category.clone(), row.to_unit.clone());
        let value = f64::from_bits(bits);
        if let Some(previous) = constants.insert(key.clone(), value) {
            assert_eq!(previous.to_bits(), bits, "conflicting constant for {key:?}");
        }
        fit_ids.insert(row.id.clone());
        report.push(FittedConstant {
            category: row.category.clone(),
            unit: row.to_unit.clone(),
            source_id: row.id.clone(),
            bits: witness.expected_bits.clone(),
        });
    }
    report.sort_by(|a, b| (&a.category, &a.unit).cmp(&(&b.category, &b.unit)));
    (constants, fit_ids, report)
}

fn x87_ratio_mul(number: f64, from: f64, to: f64, cw: u16) -> f64 {
    let ratio = rx::ext_div(&rx::ext_from_f64(to), &rx::ext_from_f64(from), cw);
    let result = rx::ext_mul(&rx::ext_from_f64(number), &ratio, cw);
    rx::ext_to_f64(&result, cw)
}

fn x87_div_mul(number: f64, from: f64, to: f64, cw: u16) -> f64 {
    let quotient = rx::ext_div(&rx::ext_from_f64(number), &rx::ext_from_f64(from), cw);
    let result = rx::ext_mul(&quotient, &rx::ext_from_f64(to), cw);
    rx::ext_to_f64(&result, cw)
}

fn x87_mul_div(number: f64, from: f64, to: f64, cw: u16) -> f64 {
    let product = rx::ext_mul(&rx::ext_from_f64(number), &rx::ext_from_f64(to), cw);
    let result = rx::ext_div(&product, &rx::ext_from_f64(from), cw);
    rx::ext_to_f64(&result, cw)
}

fn predictions(
    number: f64,
    effective_from: f64,
    effective_to: f64,
    direct_from: f64,
    direct_to: f64,
    physical_from_decimal: &str,
    physical_to_decimal: &str,
    public_direct_from_decimal: &str,
    public_direct_to_decimal: &str,
    from_prefix_exponent: i32,
    to_prefix_exponent: i32,
) -> [f64; MODELS.len()] {
    let cw = rx::CW_PC64_RN;
    let cw_rz = cw | 0x0c00;
    let ext_ratio = rx::ext_div(
        &rx::ext_from_f64(effective_to),
        &rx::ext_from_f64(effective_from),
        cw,
    );
    let ext_div = rx::ext_div(
        &rx::ext_from_f64(number),
        &rx::ext_from_f64(effective_from),
        cw,
    );
    let ext_mul = rx::ext_mul(
        &rx::ext_from_f64(number),
        &rx::ext_from_f64(effective_to),
        cw,
    );
    let prefix_delta = from_prefix_exponent - to_prefix_exponent;
    let pow10 = format!("1e{prefix_delta}").parse::<f64>().unwrap();
    let from_prefix = format!("1e{from_prefix_exponent}").parse::<f64>().unwrap();
    let to_prefix = format!("1e{to_prefix_exponent}").parse::<f64>().unwrap();
    // The public ATP-compatible graph applies a binary64 pow10 scale after
    // the unit ratio.  Keep that table value in an x87 register for the
    // continuous variants below; it is intentionally not reinterpreted as an
    // arbitrary-precision decimal.
    let pow10_ext = rx::ext_from_f64(pow10);
    let core_ratio = rx::ext_div(
        &rx::ext_from_f64(direct_to),
        &rx::ext_from_f64(direct_from),
        cw,
    );
    let core_cont = rx::ext_mul(&rx::ext_from_f64(number), &core_ratio, cw);
    let combined_factor = rx::ext_mul(&core_ratio, &pow10_ext, cw);
    let physical_from = physical_from_decimal.parse::<f64>().unwrap();
    let physical_to = physical_to_decimal.parse::<f64>().unwrap();
    let physical_from_ext = common::ext_from_decimal(physical_from_decimal);
    let physical_to_ext = common::ext_from_decimal(physical_to_decimal);
    let physical_ratio_ext = rx::ext_div(&physical_from_ext, &physical_to_ext, cw);
    let physical_core_cont = rx::ext_mul(&rx::ext_from_f64(number), &physical_ratio_ext, cw);
    let physical_combined = rx::ext_mul(&physical_ratio_ext, &pow10_ext, cw);
    let public_direct_from = public_direct_from_decimal.parse::<f64>().unwrap();
    let public_direct_to = public_direct_to_decimal.parse::<f64>().unwrap();
    let public_direct_from_ext = common::ext_from_decimal(public_direct_from_decimal);
    let public_direct_to_ext = common::ext_from_decimal(public_direct_to_decimal);
    let public_direct_ratio_ext = rx::ext_div(
        &public_direct_to_ext,
        &public_direct_from_ext,
        cw,
    );
    let public_direct_core = rx::ext_mul(
        &rx::ext_from_f64(number),
        &public_direct_ratio_ext,
        cw,
    );
    let public_direct_combined = rx::ext_mul(&public_direct_ratio_ext, &pow10_ext, cw);

    let stage_prefixes = |value: rx::Ext80| {
        let with_from = rx::ext_mul(&value, &rx::ext_from_f64(from_prefix), cw);
        rx::ext_div(&with_from, &rx::ext_from_f64(to_prefix), cw)
    };
    let table_ratio_core = rx::ext_mul(&rx::ext_from_f64(number), &core_ratio, cw);
    let table_ratio_staged = stage_prefixes(table_ratio_core);
    let table_div = rx::ext_div(
        &rx::ext_from_f64(number),
        &rx::ext_from_f64(direct_from),
        cw,
    );
    let table_divmul = rx::ext_mul(&table_div, &rx::ext_from_f64(direct_to), cw);
    let table_divmul_staged = stage_prefixes(table_divmul);
    let table_prefix_first = rx::ext_mul(
        &rx::ext_from_f64(number),
        &rx::ext_from_f64(from_prefix),
        cw,
    );
    let table_prefix_div = rx::ext_div(
        &table_prefix_first,
        &rx::ext_from_f64(direct_from),
        cw,
    );
    let table_prefix_mul = rx::ext_mul(
        &table_prefix_div,
        &rx::ext_from_f64(direct_to),
        cw,
    );
    let table_prefix_divmul = rx::ext_div(
        &table_prefix_mul,
        &rx::ext_from_f64(to_prefix),
        cw,
    );

    let physical_ratio_staged = stage_prefixes(physical_core_cont);
    let physical_mul = rx::ext_mul(
        &rx::ext_from_f64(number),
        &physical_from_ext,
        cw,
    );
    let physical_muldiv = rx::ext_div(&physical_mul, &physical_to_ext, cw);
    let physical_muldiv_staged = stage_prefixes(physical_muldiv);
    let physical_prefix_first = rx::ext_mul(
        &rx::ext_from_f64(number),
        &rx::ext_from_f64(from_prefix),
        cw,
    );
    let physical_prefix_mul = rx::ext_mul(&physical_prefix_first, &physical_from_ext, cw);
    let physical_prefix_div = rx::ext_div(&physical_prefix_mul, &physical_to_ext, cw);
    let physical_prefix_muldiv = rx::ext_div(
        &physical_prefix_div,
        &rx::ext_from_f64(to_prefix),
        cw,
    );

    let public_ratio_staged = stage_prefixes(public_direct_core);
    let public_div = rx::ext_div(
        &rx::ext_from_f64(number),
        &public_direct_from_ext,
        cw,
    );
    let public_divmul = rx::ext_mul(&public_div, &public_direct_to_ext, cw);
    let public_divmul_staged = stage_prefixes(public_divmul);

    let stage_prefixes_rz = |value: rx::Ext80| {
        let with_from = rx::ext_mul(
            &value,
            &rx::ext_from_f64(from_prefix),
            cw_rz,
        );
        rx::ext_div(
            &with_from,
            &rx::ext_from_f64(to_prefix),
            cw_rz,
        )
    };
    let table_ratio_rz = rx::ext_div(
        &rx::ext_from_f64(direct_to),
        &rx::ext_from_f64(direct_from),
        cw_rz,
    );
    let table_core_rz = rx::ext_mul(&rx::ext_from_f64(number), &table_ratio_rz, cw_rz);
    let table_all_rz = stage_prefixes_rz(table_core_rz);
    let physical_ratio_rz = rx::ext_div(&physical_from_ext, &physical_to_ext, cw_rz);
    let physical_core_rz = rx::ext_mul(&rx::ext_from_f64(number), &physical_ratio_rz, cw_rz);
    let physical_all_rz = stage_prefixes_rz(physical_core_rz);
    let public_ratio_rz = rx::ext_div(&public_direct_to_ext, &public_direct_from_ext, cw_rz);
    let public_core_rz = rx::ext_mul(&rx::ext_from_f64(number), &public_ratio_rz, cw_rz);
    let public_all_rz = stage_prefixes_rz(public_core_rz);
    [
        number * (effective_to / effective_from),
        (number / effective_from) * effective_to,
        (number * effective_to) / effective_from,
        x87_ratio_mul(number, effective_from, effective_to, cw),
        x87_div_mul(number, effective_from, effective_to, cw),
        x87_mul_div(number, effective_from, effective_to, cw),
        number * rx::ext_to_f64(&ext_ratio, cw),
        rx::ext_to_f64(&ext_div, cw) * effective_to,
        rx::ext_to_f64(&ext_mul, cw) / effective_from,
        x87_ratio_mul(number, effective_from, effective_to, rx::CW_PC53_RN),
        (number * (direct_to / direct_from)) * pow10,
        ((number * direct_to) / direct_from) * pow10,
        rx::ext_to_f64(&core_cont, cw) * pow10,
        (number * rx::ext_to_f64(&core_ratio, cw)) * pow10,
        rx::ext_to_f64(&rx::ext_mul(&core_cont, &pow10_ext, cw), cw),
        number * rx::ext_to_f64(&combined_factor, cw),
        rx::ext_to_f64(
            &rx::ext_mul(&rx::ext_from_f64(number), &combined_factor, cw),
            cw,
        ),
        (number * (physical_from / physical_to)) * pow10,
        ((number * physical_from) / physical_to) * pow10,
        rx::ext_to_f64(&physical_core_cont, cw) * pow10,
        (number * rx::ext_to_f64(&physical_ratio_ext, cw)) * pow10,
        rx::ext_to_f64(
            &rx::ext_mul(&physical_core_cont, &pow10_ext, cw),
            cw,
        ),
        number * rx::ext_to_f64(&physical_combined, cw),
        rx::ext_to_f64(
            &rx::ext_mul(&rx::ext_from_f64(number), &physical_combined, cw),
            cw,
        ),
        (number * (public_direct_to / public_direct_from)) * pow10,
        (number * rx::ext_to_f64(&public_direct_ratio_ext, cw)) * pow10,
        rx::ext_to_f64(&public_direct_core, cw) * pow10,
        number * rx::ext_to_f64(&public_direct_combined, cw),
        rx::ext_to_f64(
            &rx::ext_mul(
                &rx::ext_from_f64(number),
                &public_direct_combined,
                cw,
            ),
            cw,
        ),
        rx::ext_to_f64(&table_ratio_staged, cw_rz),
        rx::ext_to_f64(&table_divmul_staged, cw_rz),
        rx::ext_to_f64(&table_prefix_divmul, cw_rz),
        rx::ext_to_f64(&physical_ratio_staged, cw_rz),
        rx::ext_to_f64(&physical_muldiv_staged, cw_rz),
        rx::ext_to_f64(&physical_prefix_muldiv, cw_rz),
        rx::ext_to_f64(&public_ratio_staged, cw_rz),
        rx::ext_to_f64(&public_divmul_staged, cw_rz),
        rx::ext_to_f64(&table_all_rz, cw_rz),
        rx::ext_to_f64(&physical_all_rz, cw_rz),
        rx::ext_to_f64(&public_all_rz, cw_rz),
    ]
}

fn physical_factor_decimal(unit: &str) -> &'static str {
    if unit == "psi" {
        // Current NIST definition.  The production table's shorter spelling
        // is one of the constant candidates this research lane is testing.
        "6894.757293168361"
    } else {
        common::DIRECT_UNITS
            .iter()
            .find(|candidate| candidate.name == unit)
            .unwrap_or_else(|| panic!("no physical factor for {unit}"))
            .factor_decimal
    }
}

/// Units-per-category-base constants from the public Analysis add-in
/// compatible table, with the modern exact-definition updates exposed by the
/// current reference baseline for pound/ounce mass, atm, and psi.
fn public_direct_decimal(unit: &str) -> &'static str {
    match unit {
        "m" | "g" | "sec" | "Pa" | "l" => "1",
        "mi" => "6.2137119223733397E-04",
        "Nmi" => "5.3995680345572354E-04",
        "in" => "3.9370078740157480E01",
        "ft" => "3.2808398950131234E00",
        "yd" => "1.0936132983377078E00",
        "lbm" => "2.2046226218487758E-03",
        "ozm" => "3.5273961949580414E-02",
        "day" => "1.1574074074074074E-05",
        "hr" => "2.7777777777777778E-04",
        "mn" => "1.6666666666666667E-02",
        "atm" => "9.8692326671601280E-06",
        "psi" => "1.4503773773020920E-04",
        "tsp" => "2.0288413621105798E02",
        "tbs" => "6.7628045403685994E01",
        "oz" => "3.3814022701842997E01",
        "cup" => "4.2267528377303746E00",
        "pt" => "2.1133764188651873E00",
        "qt" => "1.0566882094325937E00",
        "gal" => "2.6417205235814842E-01",
        "bar" => "0.00001",
        _ => panic!("no public direct constant for {unit}"),
    }
}

fn record_miss(score: &mut ModelScore, detail: String) {
    // Research scorer output is intentionally exhaustive: pair-level residual
    // analysis is needed to distinguish staging from constant-low-bit effects.
    if score.first_misses.len() < 10_000 {
        score.first_misses.push(detail);
    }
}

fn main() {
    let args = parse_args();
    let fit_meta: MetaDocument = serde_json::from_slice(&std::fs::read(&args.fit_meta).unwrap()).unwrap();
    let fit_answers: WitnessSet = serde_json::from_slice(&std::fs::read(&args.fit_answers).unwrap()).unwrap();
    let score_meta: MetaDocument = serde_json::from_slice(&std::fs::read(&args.score_meta).unwrap()).unwrap();
    let score_answers: WitnessSet = serde_json::from_slice(&std::fs::read(&args.score_answers).unwrap()).unwrap();
    assert_eq!(fit_meta.function, "CONVERT");
    assert_eq!(fit_answers.function, "CONVERT");
    assert_eq!(score_meta.function, "CONVERT");
    assert_eq!(score_answers.function, "CONVERT");

    let (constants, fit_ids, fitted_constants) = fit_constants(&fit_meta, &fit_answers);
    let score_by_id = answer_map(&score_answers);
    assert_eq!(score_meta.rows.len(), score_by_id.len(), "score row-count drift");
    let same_dataset = args.fit_meta == args.score_meta && args.fit_answers == args.score_answers;
    let mut scores: BTreeMap<String, ModelScore> = MODELS
        .iter()
        .map(|model| ((*model).to_string(), ModelScore::default()))
        .collect();
    let mut excluded_fit_rows = 0;
    let mut controls = 0;
    let mut score_rows = 0;

    for row in &score_meta.rows {
        let witness = score_by_id
            .get(row.id.as_str())
            .unwrap_or_else(|| panic!("missing score answer {}", row.id));
        assert_eq!(witness.args, expected_args(row), "score argument drift at {}", row.id);
        if row.predictions.is_empty() {
            controls += 1;
            continue;
        }
        if same_dataset && fit_ids.contains(&row.id) {
            excluded_fit_rows += 1;
            continue;
        }
        score_rows += 1;
        let resolved_from = resolve_unit(&row.from_unit, &row.category)
            .unwrap_or_else(|| panic!("cannot resolve {} in {}", row.from_unit, row.category));
        let resolved_to = resolve_unit(&row.to_unit, &row.category)
            .unwrap_or_else(|| panic!("cannot resolve {} in {}", row.to_unit, row.category));
        let effective_from = constants.get(&(row.category.clone(), row.from_unit.clone()));
        let effective_to = constants.get(&(row.category.clone(), row.to_unit.clone()));
        let direct_from = constants.get(&(row.category.clone(), resolved_from.direct_name.clone()));
        let direct_to = constants.get(&(row.category.clone(), resolved_to.direct_name.clone()));
        let actual = parse_bits(&witness.expected_bits);

        if effective_from.is_none()
            || effective_to.is_none()
            || direct_from.is_none()
            || direct_to.is_none()
        {
            for model in MODELS {
                let score = scores.get_mut(model).unwrap();
                *score.class_total.entry(row.class.clone()).or_default() += 1;
                *score.category_total.entry(row.category.clone()).or_default() += 1;
                let pair = format!("{}->{}", row.from_unit, row.to_unit);
                *score.pair_total.entry(pair.clone()).or_default() += 1;
                if actual.is_none() {
                    score.structural_exact += 1;
                    *score.class_exact.entry(row.class.clone()).or_default() += 1;
                    *score.category_exact.entry(row.category.clone()).or_default() += 1;
                    *score.pair_exact.entry(pair).or_default() += 1;
                } else {
                    score.structural_mismatch += 1;
                    record_miss(
                        score,
                        format!(
                            "{} unexpectedly numeric for absent constant {} -> {}: {}",
                            row.id, row.from_unit, row.to_unit, witness.expected_bits
                        ),
                    );
                }
            }
            continue;
        }
        let effective_from = *effective_from.unwrap();
        let effective_to = *effective_to.unwrap();
        let direct_from = *direct_from.unwrap();
        let direct_to = *direct_to.unwrap();
        let Some(actual_bits) = actual else {
            for model in MODELS {
                let score = scores.get_mut(model).unwrap();
                *score.class_total.entry(row.class.clone()).or_default() += 1;
                *score.category_total.entry(row.category.clone()).or_default() += 1;
                *score
                    .pair_total
                    .entry(format!("{}->{}", row.from_unit, row.to_unit))
                    .or_default() += 1;
                score.structural_mismatch += 1;
                record_miss(
                    score,
                    format!(
                        "{} unexpectedly nonnumeric with fitted constants {} -> {}: {}",
                        row.id, row.from_unit, row.to_unit, witness.expected_bits
                    ),
                );
            }
            continue;
        };
        let number = common::f64_from_hex(&row.number_bits).unwrap();
        let values = predictions(
            number,
            effective_from,
            effective_to,
            direct_from,
            direct_to,
            physical_factor_decimal(&resolved_from.direct_name),
            physical_factor_decimal(&resolved_to.direct_name),
            public_direct_decimal(&resolved_from.direct_name),
            public_direct_decimal(&resolved_to.direct_name),
            resolved_from.prefix_exponent,
            resolved_to.prefix_exponent,
        );
        for (model, predicted) in MODELS.iter().zip(values) {
            let score = scores.get_mut(*model).unwrap();
            score.numeric_total += 1;
            *score.class_total.entry(row.class.clone()).or_default() += 1;
            *score.category_total.entry(row.category.clone()).or_default() += 1;
            let pair = format!("{}->{}", row.from_unit, row.to_unit);
            *score.pair_total.entry(pair.clone()).or_default() += 1;
            let residual = ordered_bits(actual_bits) - ordered_bits(predicted.to_bits());
            if residual == 0 {
                score.exact += 1;
                *score.class_exact.entry(row.class.clone()).or_default() += 1;
                *score.category_exact.entry(row.category.clone()).or_default() += 1;
                *score.pair_exact.entry(pair).or_default() += 1;
            } else {
                score.mismatch += 1;
                let abs = residual.unsigned_abs();
                let old_max = score.max_abs_ulp.parse::<u128>().unwrap_or(0);
                if abs > old_max {
                    score.max_abs_ulp = abs.to_string();
                }
                let old_sum = score.sum_abs_ulp.parse::<u128>().unwrap_or(0);
                score.sum_abs_ulp = old_sum.saturating_add(abs).to_string();
                record_miss(
                    score,
                    format!(
                        "{} {}({},{},{}) residual={:+} predicted=0x{:016x} oracle=0x{:016x}",
                        row.id,
                        row.class,
                        row.number_bits,
                        row.from_unit,
                        row.to_unit,
                        residual,
                        predicted.to_bits(),
                        actual_bits
                    ),
                );
            }
        }
    }

    for score in scores.values_mut() {
        if score.max_abs_ulp.is_empty() {
            score.max_abs_ulp = "0".to_string();
        }
        if score.sum_abs_ulp.is_empty() {
            score.sum_abs_ulp = "0".to_string();
        }
    }
    let survivors = scores
        .iter()
        .filter(|(_, score)| score.mismatch == 0 && score.structural_mismatch == 0)
        .map(|(name, _)| name.clone())
        .collect::<Vec<_>>();

    println!(
        "fitted {} supported units from {} rows; scoring {} non-fit rows ({} typed controls)",
        fitted_constants.len(), fit_ids.len(), score_rows, controls
    );
    println!("{:<40} {:>8} {:>8} {:>8} {:>10}", "model", "exact", "numeric", "struct", "max_ulp");
    for model in MODELS {
        let score = &scores[model];
        println!(
            "{:<40} {:>8} {:>8} {:>8} {:>10}",
            model,
            score.exact,
            score.numeric_total,
            score.structural_exact,
            score.max_abs_ulp
        );
    }
    if survivors.is_empty() {
        println!("exact survivors: none");
    } else {
        println!("exact survivors: {}", survivors.join(", "));
    }

    let report = ScoreReport {
        schema_version: "w109.convert.fitted_graph_score.v1",
        function: "CONVERT",
        fit_rows: fit_ids.len(),
        fitted_constants,
        score_rows,
        excluded_fit_rows,
        unmodeled_controls: controls,
        exact_survivors: survivors,
        scores,
        fit_capture_provenance: fit_answers.capture_provenance,
        score_capture_provenance: score_answers.capture_provenance,
    };
    if let Some(path) = args.out {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(&path, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
        println!("wrote fitted-graph report -> {}", path.display());
    }
}
