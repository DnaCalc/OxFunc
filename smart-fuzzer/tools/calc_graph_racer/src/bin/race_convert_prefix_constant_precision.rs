//! Race CONVERT's final decimal-prefix constant construction.
//!
//! The v2 publication kill is a single large `nm -> Pm` row where f64 and a
//! PC64 multiply by the promoted f64 `1e-24` agree but Excel is one ULP lower.
//! This scorer keeps the already-validated f64 two-cell core frozen and varies
//! only how the prefix multiplier enters the final PC64 operation.

#[path = "convert_research/common.rs"]
mod common;

use common::{MetaDocument, ordered_bits};
use oxfunc_core::excel_numeric::research as rx;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::PathBuf;

const MODELS: [&str; 7] = [
    "f64_delta_final",
    "pc64_promoted_f64_delta_final",
    "pc64_decimal_delta_final",
    "pc64_decimal_prefix_ratio_final",
    "pc64_promoted_f64_prefix_ratio_final",
    "pc64_decimal_prefix_mul_div_final",
    "pc64_repeated_ten_delta_final",
];

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
struct Resolved {
    direct: String,
    prefix_exponent: i32,
}

#[derive(Default, Serialize)]
struct Score {
    exact: usize,
    total: usize,
    sum_abs_ulp: String,
    max_abs_ulp: String,
    category_exact: BTreeMap<String, usize>,
    category_total: BTreeMap<String, usize>,
    first_misses: Vec<String>,
}

#[derive(Serialize)]
struct Report {
    schema_version: &'static str,
    function: &'static str,
    scores: BTreeMap<String, Score>,
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
            other => panic!("unknown argument {other}"),
        }
    }
    Args {
        meta: meta.expect("--meta"),
        answers: answers.expect("--answers"),
        out,
    }
}

fn bits(raw: &str) -> Option<u64> {
    let digits = raw.strip_prefix("0x")?;
    (digits.len() == 16)
        .then(|| u64::from_str_radix(digits, 16).ok())
        .flatten()
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

fn base(category: &str) -> Option<&'static str> {
    match category {
        "length" => Some("m"),
        "mass" => Some("g"),
        "time" => Some("sec"),
        "pressure" => Some("Pa"),
        "volume" => Some("l"),
        _ => None,
    }
}

fn resolve(unit: &str, category: &str) -> Resolved {
    if common::direct_unit(unit).is_some() {
        return Resolved {
            direct: unit.to_string(),
            prefix_exponent: 0,
        };
    }
    let base = base(category).unwrap();
    for (prefix, _) in common::PREFIXES {
        if unit == format!("{prefix}{base}") {
            return Resolved {
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

fn physical(unit: &str) -> f64 {
    common::DIRECT_UNITS
        .iter()
        .find(|candidate| candidate.name == unit)
        .unwrap()
        .factor_decimal
        .parse()
        .unwrap()
}

fn pressure(unit: &str) -> f64 {
    match unit {
        "Pa" => 1.0,
        "atm" => 1.0 / "9.8692326671601280E-06".parse::<f64>().unwrap(),
        "psi" => 1.0 / "1.4503773773020920E-04".parse::<f64>().unwrap(),
        other => panic!("no pressure factor for {other}"),
    }
}

fn factor(category: &str, unit: &str) -> f64 {
    match category {
        "length" => angstroms(unit),
        "mass" | "time" | "volume" => physical(unit),
        "pressure" => pressure(unit),
        other => panic!("not linear: {other}"),
    }
}

fn pow10_f64(exponent: i32) -> f64 {
    format!("1e{exponent}").parse().unwrap()
}

fn decimal_pow10_ext(exponent: i32) -> rx::Ext80 {
    if exponent.unsigned_abs() <= 38 {
        return common::ext_from_decimal(&format!("1e{exponent}"));
    }
    assert_eq!(exponent.unsigned_abs(), 39);
    let cw = rx::CW_PC64_RN;
    let left = common::ext_from_decimal(if exponent > 0 { "1e24" } else { "1e-24" });
    let right = common::ext_from_decimal(if exponent > 0 { "1e15" } else { "1e-15" });
    rx::ext_mul(&left, &right, cw)
}

fn repeated_ten_ext(exponent: i32) -> rx::Ext80 {
    let cw = rx::CW_PC64_RN;
    let ten = rx::ext_from_f64(10.0);
    let mut value = rx::ext_from_f64(1.0);
    for _ in 0..exponent.unsigned_abs() {
        value = if exponent >= 0 {
            rx::ext_mul(&value, &ten, cw)
        } else {
            rx::ext_div(&value, &ten, cw)
        };
    }
    value
}

fn predictions(core: f64, from_exponent: i32, to_exponent: i32) -> [f64; MODELS.len()] {
    let cw = rx::CW_PC64_RN;
    let delta_exponent = from_exponent - to_exponent;
    let delta_f64 = pow10_f64(delta_exponent);
    let core_ext = rx::ext_from_f64(core);
    let promoted_delta = rx::ext_from_f64(delta_f64);
    let decimal_delta = decimal_pow10_ext(delta_exponent);
    let decimal_from = decimal_pow10_ext(from_exponent);
    let decimal_to = decimal_pow10_ext(to_exponent);
    let promoted_from = rx::ext_from_f64(pow10_f64(from_exponent));
    let promoted_to = rx::ext_from_f64(pow10_f64(to_exponent));
    let decimal_ratio = rx::ext_div(&decimal_from, &decimal_to, cw);
    let promoted_ratio = rx::ext_div(&promoted_from, &promoted_to, cw);
    let decimal_product = rx::ext_mul(&core_ext, &decimal_from, cw);
    [
        core * delta_f64,
        rx::ext_to_f64(&rx::ext_mul(&core_ext, &promoted_delta, cw), cw),
        rx::ext_to_f64(&rx::ext_mul(&core_ext, &decimal_delta, cw), cw),
        rx::ext_to_f64(&rx::ext_mul(&core_ext, &decimal_ratio, cw), cw),
        rx::ext_to_f64(&rx::ext_mul(&core_ext, &promoted_ratio, cw), cw),
        rx::ext_to_f64(&rx::ext_div(&decimal_product, &decimal_to, cw), cw),
        rx::ext_to_f64(
            &rx::ext_mul(&core_ext, &repeated_ten_ext(delta_exponent), cw),
            cw,
        ),
    ]
}

fn main() {
    let args = args();
    let metadata: MetaDocument =
        serde_json::from_slice(&std::fs::read(&args.meta).unwrap()).unwrap();
    let answers: WitnessSet =
        serde_json::from_slice(&std::fs::read(&args.answers).unwrap()).unwrap();
    assert_eq!(answers.function, "CONVERT");
    let by_id: BTreeMap<_, _> = answers
        .witnesses
        .iter()
        .map(|witness| (witness.id.as_str(), witness))
        .collect();
    let mut scores: BTreeMap<String, Score> = MODELS
        .iter()
        .map(|name| ((*name).to_string(), Score::default()))
        .collect();
    for row in &metadata.rows {
        if !matches!(
            row.category.as_str(),
            "length" | "mass" | "time" | "pressure" | "volume"
        ) {
            continue;
        }
        let Some(actual) = bits(&by_id[row.id.as_str()].expected_bits) else {
            continue;
        };
        let from = resolve(&row.from_unit, &row.category);
        let to = resolve(&row.to_unit, &row.category);
        let number = common::f64_from_hex(&row.number_bits).unwrap();
        let core =
            (number * factor(&row.category, &from.direct)) / factor(&row.category, &to.direct);
        for (name, predicted) in
            MODELS
                .iter()
                .zip(predictions(core, from.prefix_exponent, to.prefix_exponent))
        {
            let score = scores.get_mut(*name).unwrap();
            score.total += 1;
            *score
                .category_total
                .entry(row.category.clone())
                .or_default() += 1;
            let residual = ordered_bits(actual) - ordered_bits(predicted.to_bits());
            if residual == 0 {
                score.exact += 1;
                *score
                    .category_exact
                    .entry(row.category.clone())
                    .or_default() += 1;
            } else {
                let abs = residual.unsigned_abs();
                score.sum_abs_ulp = score
                    .sum_abs_ulp
                    .parse::<u128>()
                    .unwrap_or(0)
                    .saturating_add(abs)
                    .to_string();
                score.max_abs_ulp = score
                    .max_abs_ulp
                    .parse::<u128>()
                    .unwrap_or(0)
                    .max(abs)
                    .to_string();
                if score.first_misses.len() < 64 {
                    score.first_misses.push(format!(
                        "{} {} {}->{} x={} residual={:+} predicted=0x{:016x} oracle=0x{:016x}",
                        row.id,
                        row.category,
                        row.from_unit,
                        row.to_unit,
                        row.number_bits,
                        residual,
                        predicted.to_bits(),
                        actual,
                    ));
                }
            }
        }
    }
    for (name, score) in &mut scores {
        if score.sum_abs_ulp.is_empty() {
            score.sum_abs_ulp = "0".to_string();
        }
        if score.max_abs_ulp.is_empty() {
            score.max_abs_ulp = "0".to_string();
        }
        println!(
            "{name}: {}/{} exact sum={} max={}",
            score.exact, score.total, score.sum_abs_ulp, score.max_abs_ulp
        );
        for (category, total) in &score.category_total {
            println!(
                "  {category}: {}/{}",
                score.category_exact.get(category).copied().unwrap_or(0),
                total,
            );
        }
    }
    let report = Report {
        schema_version: "w109.convert.prefix_constant_precision_race.v1",
        function: "CONVERT",
        scores,
        capture_provenance: answers.capture_provenance,
    };
    if let Some(path) = args.out {
        std::fs::write(&path, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
        println!("wrote prefix-constant race -> {}", path.display());
    }
}
