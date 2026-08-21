//! Race direct-multiplier and sign-directed reciprocal schedules for CONVERT's
//! final decimal-prefix operation, keeping the frozen v2 two-cell core fixed.

#[path = "convert_research/common.rs"]
mod common;

use common::{MetaDocument, ordered_bits};
use oxfunc_core::excel_numeric::research as rx;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

const MODELS: [&str; 8] = [
    "v2_pc64_mul_signed_pow10",
    "pc64_sign_directed_mul_or_div",
    "pc64_mul_reciprocal_positive_pow10",
    "f64_sign_directed_mul_or_div",
    "f64_mul_reciprocal_positive_pow10",
    "pc64_div_exact_decimal_positive_pow10",
    "pc64_mul_exact_decimal_signed_pow10",
    "pc64_div_promoted_prefix_to_then_mul_prefix_from",
];

#[derive(Deserialize)]
struct WitnessSet {
    function: String,
    witnesses: Vec<Witness>,
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
    delta_misses: BTreeMap<i32, usize>,
    first_misses: Vec<String>,
}

#[derive(Serialize)]
struct Report {
    schema_version: &'static str,
    function: &'static str,
    rows: usize,
    scores: BTreeMap<String, Score>,
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

fn resolve(unit: &str, category: &str) -> Resolved {
    if common::direct_unit(unit).is_some() {
        return Resolved {
            direct: unit.to_string(),
            prefix_exponent: 0,
        };
    }
    let base = prefix_base(category).unwrap();
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

fn factor(category: &str, unit: &str) -> f64 {
    if category == "length" {
        return match unit {
            "m" => 10_000_000_000.0,
            "in" => 254_000_000.0,
            "ft" => 3_048_000_000.0,
            "yd" => 9_144_000_000.0,
            "mi" => 16_093_440_000_000.0,
            "Nmi" => 18_520_000_000_000.0,
            other => panic!("no length factor for {other}"),
        };
    }
    if category == "pressure" {
        return match unit {
            "Pa" => 1.0,
            "atm" => 1.0 / "9.8692326671601280E-06".parse::<f64>().unwrap(),
            "psi" => 1.0 / "1.4503773773020920E-04".parse::<f64>().unwrap(),
            "bar" => f64::NAN,
            other => panic!("no pressure factor for {other}"),
        };
    }
    common::DIRECT_UNITS
        .iter()
        .find(|candidate| candidate.name == unit)
        .unwrap()
        .factor_decimal
        .parse()
        .unwrap()
}

fn pow10(exponent: i32) -> f64 {
    format!("1e{exponent}").parse().unwrap()
}

fn exact_pow10(exponent: i32) -> rx::Ext80 {
    if exponent.unsigned_abs() <= 38 {
        common::ext_from_decimal(&format!("1e{exponent}"))
    } else {
        assert_eq!(exponent.unsigned_abs(), 39);
        let cw = rx::CW_PC64_RN;
        let a = common::ext_from_decimal(if exponent > 0 { "1e24" } else { "1e-24" });
        let b = common::ext_from_decimal(if exponent > 0 { "1e15" } else { "1e-15" });
        rx::ext_mul(&a, &b, cw)
    }
}

fn predictions(core: f64, from_exp: i32, to_exp: i32) -> [f64; MODELS.len()] {
    let cw = rx::CW_PC64_RN;
    let delta = from_exp - to_exp;
    let core_ext = rx::ext_from_f64(core);
    let signed = rx::ext_from_f64(pow10(delta));
    let positive = rx::ext_from_f64(pow10(delta.unsigned_abs() as i32));
    let v2 = rx::ext_to_f64(&rx::ext_mul(&core_ext, &signed, cw), cw);
    let sign_directed = if delta >= 0 {
        rx::ext_to_f64(&rx::ext_mul(&core_ext, &positive, cw), cw)
    } else {
        rx::ext_to_f64(&rx::ext_div(&core_ext, &positive, cw), cw)
    };
    let reciprocal = 1.0 / pow10((-delta).max(0));
    let pc64_recip = if delta >= 0 {
        v2
    } else {
        rx::ext_to_f64(
            &rx::ext_mul(&core_ext, &rx::ext_from_f64(reciprocal), cw),
            cw,
        )
    };
    let f64_sign_directed = if delta >= 0 {
        core * pow10(delta)
    } else {
        core / pow10(-delta)
    };
    let f64_recip = if delta >= 0 {
        core * pow10(delta)
    } else {
        core * reciprocal
    };
    let exact_sign_directed = if delta >= 0 {
        rx::ext_to_f64(&rx::ext_mul(&core_ext, &exact_pow10(delta), cw), cw)
    } else {
        rx::ext_to_f64(&rx::ext_div(&core_ext, &exact_pow10(-delta), cw), cw)
    };
    let exact_signed = rx::ext_to_f64(&rx::ext_mul(&core_ext, &exact_pow10(delta), cw), cw);
    let divided = rx::ext_div(&core_ext, &rx::ext_from_f64(pow10(to_exp)), cw);
    let separate = rx::ext_to_f64(
        &rx::ext_mul(&divided, &rx::ext_from_f64(pow10(from_exp)), cw),
        cw,
    );
    [
        v2,
        sign_directed,
        pc64_recip,
        f64_sign_directed,
        f64_recip,
        exact_sign_directed,
        exact_signed,
        separate,
    ]
}

fn score_row(score: &mut Score, row: &common::MetaRow, delta: i32, predicted: u64, actual: u64) {
    score.total += 1;
    *score
        .category_total
        .entry(row.category.clone())
        .or_default() += 1;
    let residual = ordered_bits(actual) - ordered_bits(predicted);
    if residual == 0 {
        score.exact += 1;
        *score
            .category_exact
            .entry(row.category.clone())
            .or_default() += 1;
        return;
    }
    *score.delta_misses.entry(delta).or_default() += 1;
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
    if score.first_misses.len() < 32 {
        score.first_misses.push(format!(
            "{} {} {}->{} x={} delta={delta} residual={residual:+} predicted=0x{predicted:016x} oracle=0x{actual:016x}",
            row.id, row.category, row.from_unit, row.to_unit, row.number_bits
        ));
    }
}

fn process(
    root: &Path,
    meta_name: &str,
    answer_name: &str,
    scores: &mut BTreeMap<String, Score>,
) -> usize {
    let metadata: MetaDocument =
        serde_json::from_slice(&std::fs::read(root.join(meta_name)).unwrap()).unwrap();
    let answers: WitnessSet =
        serde_json::from_slice(&std::fs::read(root.join(answer_name)).unwrap()).unwrap();
    assert_eq!(answers.function, "CONVERT");
    let by_id: BTreeMap<_, _> = answers
        .witnesses
        .iter()
        .map(|witness| (witness.id.as_str(), witness))
        .collect();
    let mut rows = 0;
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
        if from.direct == "bar" || to.direct == "bar" {
            continue;
        }
        let number = common::f64_from_hex(&row.number_bits).unwrap();
        let core =
            (number * factor(&row.category, &from.direct)) / factor(&row.category, &to.direct);
        let delta = from.prefix_exponent - to.prefix_exponent;
        for (name, prediction) in
            MODELS
                .iter()
                .zip(predictions(core, from.prefix_exponent, to.prefix_exponent))
        {
            score_row(
                scores.get_mut(*name).unwrap(),
                row,
                delta,
                prediction.to_bits(),
                actual,
            );
        }
        rows += 1;
    }
    rows
}

fn main() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../work/w109/G4-convert");
    let datasets = [
        (
            "batch-convert-discovery-20260809-meta.json",
            "answers-convert-discovery-20260809-clean.json",
        ),
        (
            "batch-convert-heldout-20260809-meta.json",
            "answers-convert-heldout-20260809.json",
        ),
        (
            "batch-convert-publication-heldout-v2-20260809-meta.json",
            "answers-convert-publication-heldout-v2-20260809.json",
        ),
    ];
    let mut scores: BTreeMap<_, _> = MODELS
        .iter()
        .map(|name| ((*name).to_string(), Score::default()))
        .collect();
    let rows = datasets
        .into_iter()
        .map(|(meta, answers)| process(&root, meta, answers, &mut scores))
        .sum();
    for score in scores.values_mut() {
        if score.sum_abs_ulp.is_empty() {
            score.sum_abs_ulp = "0".to_string();
        }
        if score.max_abs_ulp.is_empty() {
            score.max_abs_ulp = "0".to_string();
        }
    }
    for (name, score) in &scores {
        println!(
            "{name}: {}/{} sum={} max={}",
            score.exact, score.total, score.sum_abs_ulp, score.max_abs_ulp
        );
    }
    let report = Report {
        schema_version: "w109.convert.prefix_reciprocal_graph_race.v1",
        function: "CONVERT",
        rows,
        scores,
    };
    let out = root.join("score-convert-prefix-reciprocal-graph.json");
    std::fs::write(&out, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
    println!("wrote -> {}", out.display());
}
