//! Search two-cell CONVERT length-core schedules after the v3 live m->m
//! readback exposed a missing orientation/store axis.

#[path = "convert_research/common.rs"]
mod common;

use common::{MetaDocument, ordered_bits};
use oxfunc_core::excel_numeric::research as rx;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

const FACTOR_CONSTRUCTIONS: [&str; 2] = [
    "physical_mul_scale_f64",
    "decimal_physical_mul_scale_pc64_store",
];

const INVERSE_CONSTRUCTIONS: [&str; 5] = [
    "one_div_factor_f64",
    "hidden_div_physical_f64",
    "public_units_per_meter_div_scale_f64",
    "public_units_per_meter_mul_hidden_f64",
    "decimal_hidden_div_physical_pc64_store",
];

const FORMS: [&str; 4] = [
    "mul_factor_div_factor",
    "div_inverse_div_factor",
    "mul_factor_mul_inverse",
    "div_inverse_mul_inverse",
];

const SCHEDULES: [&str; 7] = [
    "f64_f64",
    "f64_then_pc64",
    "pc64_store_then_f64",
    "pc64_each_store",
    "pc64_continuous",
    "pc53_each_store",
    "pc53_continuous",
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

#[derive(Deserialize)]
struct ReadbackDoc {
    rows: Vec<ReadbackRow>,
}

#[derive(Deserialize)]
struct ReadbackRow {
    requested_bits: String,
    base_m_identity_bits: String,
}

#[derive(Clone)]
struct Resolved {
    direct: String,
    prefix_exponent: i32,
}

#[derive(Clone)]
struct Row {
    id: String,
    dataset: String,
    number: f64,
    from: Resolved,
    to: Resolved,
    actual: u64,
}

#[derive(Clone, Copy)]
enum Op {
    Mul,
    Div,
}

#[derive(Default, Clone, Copy, Serialize)]
struct Fitness {
    exact: usize,
    total: usize,
    sum_abs_ulp: u128,
    max_abs_ulp: u128,
}

#[derive(Serialize)]
struct Candidate {
    scale_exponent: i32,
    factor_construction: String,
    inverse_construction: String,
    form: String,
    schedule: String,
    readback_fitness: Fitness,
    evidence_fitness: Fitness,
    dataset_fitness: BTreeMap<String, Fitness>,
    factor_bits: BTreeMap<String, String>,
    inverse_bits: BTreeMap<String, String>,
    first_misses: Vec<String>,
}

#[derive(Serialize)]
struct Report {
    schema_version: &'static str,
    function: &'static str,
    readback_rows: usize,
    evidence_rows: usize,
    maximum_readback_exact: usize,
    readback_exact_candidates: usize,
    candidates: Vec<Candidate>,
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

fn resolve(unit: &str) -> Resolved {
    if common::direct_unit(unit).is_some() {
        return Resolved {
            direct: unit.to_string(),
            prefix_exponent: 0,
        };
    }
    for (prefix, _) in common::PREFIXES {
        if unit == format!("{prefix}m") {
            return Resolved {
                direct: "m".to_string(),
                prefix_exponent: prefix_exponent(prefix),
            };
        }
    }
    panic!("cannot resolve length unit {unit}");
}

fn physical(unit: &str) -> &'static str {
    match unit {
        "m" => "1",
        "in" => "0.0254",
        "ft" => "0.3048",
        "yd" => "0.9144",
        "mi" => "1609.344",
        "Nmi" => "1852",
        other => panic!("no physical factor for {other}"),
    }
}

fn public_units_per_meter(unit: &str) -> &'static str {
    match unit {
        "m" => "1",
        "mi" => "6.2137119223733397E-04",
        "Nmi" => "5.3995680345572354E-04",
        "in" => "3.9370078740157480E01",
        "ft" => "3.2808398950131234E00",
        "yd" => "1.0936132983377078E00",
        other => panic!("no public units-per-meter for {other}"),
    }
}

fn factor(unit: &str, scale_exponent: i32, construction: &str) -> f64 {
    let p = physical(unit).parse::<f64>().unwrap();
    let scale_decimal = format!("1e{scale_exponent}");
    let scale = scale_decimal.parse::<f64>().unwrap();
    match construction {
        "physical_mul_scale_f64" => p * scale,
        "decimal_physical_mul_scale_pc64_store" => {
            let cw = rx::CW_PC64_RN;
            rx::ext_to_f64(
                &rx::ext_mul(
                    &common::ext_from_decimal(physical(unit)),
                    &common::ext_from_decimal(&scale_decimal),
                    cw,
                ),
                cw,
            )
        }
        other => panic!("unknown factor construction {other}"),
    }
}

fn inverse(unit: &str, factor: f64, scale_exponent: i32, construction: &str) -> f64 {
    let hidden_decimal = format!("1e{}", -scale_exponent);
    let scale_decimal = format!("1e{scale_exponent}");
    let hidden = hidden_decimal.parse::<f64>().unwrap();
    let scale = scale_decimal.parse::<f64>().unwrap();
    let p = physical(unit).parse::<f64>().unwrap();
    let public = public_units_per_meter(unit).parse::<f64>().unwrap();
    match construction {
        "one_div_factor_f64" => 1.0 / factor,
        "hidden_div_physical_f64" => hidden / p,
        "public_units_per_meter_div_scale_f64" => public / scale,
        "public_units_per_meter_mul_hidden_f64" => public * hidden,
        "decimal_hidden_div_physical_pc64_store" => {
            let cw = rx::CW_PC64_RN;
            rx::ext_to_f64(
                &rx::ext_div(
                    &common::ext_from_decimal(&hidden_decimal),
                    &common::ext_from_decimal(physical(unit)),
                    cw,
                ),
                cw,
            )
        }
        other => panic!("unknown inverse construction {other}"),
    }
}

fn form_ops(form: &str, ff: f64, fi: f64, tf: f64, ti: f64) -> (Op, f64, Op, f64) {
    match form {
        "mul_factor_div_factor" => (Op::Mul, ff, Op::Div, tf),
        "div_inverse_div_factor" => (Op::Div, fi, Op::Div, tf),
        "mul_factor_mul_inverse" => (Op::Mul, ff, Op::Mul, ti),
        "div_inverse_mul_inverse" => (Op::Div, fi, Op::Mul, ti),
        other => panic!("unknown form {other}"),
    }
}

fn f64_op(left: f64, op: Op, right: f64) -> f64 {
    match op {
        Op::Mul => left * right,
        Op::Div => left / right,
    }
}

fn ext_op(left: &rx::Ext80, op: Op, right: f64, cw: u16) -> rx::Ext80 {
    match op {
        Op::Mul => rx::ext_mul(left, &rx::ext_from_f64(right), cw),
        Op::Div => rx::ext_div(left, &rx::ext_from_f64(right), cw),
    }
}

fn core(number: f64, ops: (Op, f64, Op, f64), schedule: &str) -> f64 {
    let (op1, c1, op2, c2) = ops;
    match schedule {
        "f64_f64" => f64_op(f64_op(number, op1, c1), op2, c2),
        "f64_then_pc64" => {
            let first = f64_op(number, op1, c1);
            rx::ext_to_f64(
                &ext_op(&rx::ext_from_f64(first), op2, c2, rx::CW_PC64_RN),
                rx::CW_PC64_RN,
            )
        }
        "pc64_store_then_f64" => {
            let first = rx::ext_to_f64(
                &ext_op(&rx::ext_from_f64(number), op1, c1, rx::CW_PC64_RN),
                rx::CW_PC64_RN,
            );
            f64_op(first, op2, c2)
        }
        "pc64_each_store" => {
            let first = rx::ext_to_f64(
                &ext_op(&rx::ext_from_f64(number), op1, c1, rx::CW_PC64_RN),
                rx::CW_PC64_RN,
            );
            rx::ext_to_f64(
                &ext_op(&rx::ext_from_f64(first), op2, c2, rx::CW_PC64_RN),
                rx::CW_PC64_RN,
            )
        }
        "pc64_continuous" => {
            let first = ext_op(&rx::ext_from_f64(number), op1, c1, rx::CW_PC64_RN);
            rx::ext_to_f64(&ext_op(&first, op2, c2, rx::CW_PC64_RN), rx::CW_PC64_RN)
        }
        "pc53_each_store" => {
            let first = rx::ext_to_f64(
                &ext_op(&rx::ext_from_f64(number), op1, c1, rx::CW_PC53_RN),
                rx::CW_PC53_RN,
            );
            rx::ext_to_f64(
                &ext_op(&rx::ext_from_f64(first), op2, c2, rx::CW_PC53_RN),
                rx::CW_PC53_RN,
            )
        }
        "pc53_continuous" => {
            let first = ext_op(&rx::ext_from_f64(number), op1, c1, rx::CW_PC53_RN);
            rx::ext_to_f64(&ext_op(&first, op2, c2, rx::CW_PC53_RN), rx::CW_PC53_RN)
        }
        other => panic!("unknown schedule {other}"),
    }
}

fn pow10(exponent: i32) -> f64 {
    format!("1e{exponent}").parse().unwrap()
}

fn finish(core: f64, delta: i32) -> u64 {
    let cw = rx::CW_PC64_RN;
    rx::ext_to_f64(
        &rx::ext_mul(&rx::ext_from_f64(core), &rx::ext_from_f64(pow10(delta)), cw),
        cw,
    )
    .to_bits()
}

fn add_fit(fit: &mut Fitness, predicted: u64, actual: u64) {
    fit.total += 1;
    let residual = ordered_bits(actual) - ordered_bits(predicted);
    if residual == 0 {
        fit.exact += 1;
    } else {
        let abs = residual.unsigned_abs();
        fit.sum_abs_ulp = fit.sum_abs_ulp.saturating_add(abs);
        fit.max_abs_ulp = fit.max_abs_ulp.max(abs);
    }
}

fn load(root: &Path, dataset: &str, meta_name: &str, answer_name: &str) -> Vec<Row> {
    let metadata: MetaDocument =
        serde_json::from_slice(&std::fs::read(root.join(meta_name)).unwrap()).unwrap();
    let answers: WitnessSet =
        serde_json::from_slice(&std::fs::read(root.join(answer_name)).unwrap()).unwrap();
    assert_eq!(answers.function, "CONVERT");
    let by_id: BTreeMap<_, _> = answers
        .witnesses
        .iter()
        .map(|w| (w.id.as_str(), w))
        .collect();
    metadata
        .rows
        .into_iter()
        .filter(|row| row.category == "length")
        .map(|row| Row {
            id: row.id.clone(),
            dataset: dataset.to_string(),
            number: common::f64_from_hex(&row.number_bits).unwrap(),
            from: resolve(&row.from_unit),
            to: resolve(&row.to_unit),
            actual: bits(&by_id[row.id.as_str()].expected_bits).unwrap(),
        })
        .collect()
}

fn main() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../work/w109/G4-convert");
    let readback: ReadbackDoc = serde_json::from_slice(
        &std::fs::read(root.join("capture-convert-value2-readback-v2-20260809.json")).unwrap(),
    )
    .unwrap();
    let evidence_sets = [
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
    let rows: Vec<_> = evidence_sets
        .into_iter()
        .flat_map(|(d, m, a)| load(&root, d, m, a))
        .collect();
    let units = ["m", "in", "ft", "yd", "mi", "Nmi"];
    let mut candidates = Vec::new();
    let mut maximum_readback_exact = 0;
    for scale_exponent in -24..=24 {
        for factor_construction in FACTOR_CONSTRUCTIONS {
            let factors: BTreeMap<_, _> = units
                .iter()
                .map(|unit| {
                    (
                        (*unit).to_string(),
                        factor(unit, scale_exponent, factor_construction),
                    )
                })
                .collect();
            for inverse_construction in INVERSE_CONSTRUCTIONS {
                let inverses: BTreeMap<_, _> = units
                    .iter()
                    .map(|unit| {
                        (
                            (*unit).to_string(),
                            inverse(unit, factors[*unit], scale_exponent, inverse_construction),
                        )
                    })
                    .collect();
                for form in FORMS {
                    for schedule in SCHEDULES {
                        let m_ops = form_ops(
                            form,
                            factors["m"],
                            inverses["m"],
                            factors["m"],
                            inverses["m"],
                        );
                        let mut readback_fitness = Fitness::default();
                        for row in &readback.rows {
                            add_fit(
                                &mut readback_fitness,
                                core(
                                    common::f64_from_hex(&row.requested_bits).unwrap(),
                                    m_ops,
                                    schedule,
                                )
                                .to_bits(),
                                bits(&row.base_m_identity_bits).unwrap(),
                            );
                        }
                        maximum_readback_exact = maximum_readback_exact.max(readback_fitness.exact);
                        candidates.push((
                            scale_exponent,
                            factor_construction,
                            inverse_construction,
                            form,
                            schedule,
                            factors.clone(),
                            inverses.clone(),
                            readback_fitness,
                        ));
                    }
                }
            }
        }
    }
    // Only the best readback layer is eligible for expensive full-evidence scoring.
    let mut scored = Vec::new();
    for (
        scale_exponent,
        factor_construction,
        inverse_construction,
        form,
        schedule,
        factors,
        inverses,
        readback_fitness,
    ) in candidates
    {
        if readback_fitness.exact != maximum_readback_exact {
            continue;
        }
        let mut evidence_fitness = Fitness::default();
        let mut dataset_fitness = BTreeMap::new();
        let mut first_misses = Vec::new();
        for row in &rows {
            let ops = form_ops(
                form,
                factors[&row.from.direct],
                inverses[&row.from.direct],
                factors[&row.to.direct],
                inverses[&row.to.direct],
            );
            let predicted = finish(
                core(row.number, ops, schedule),
                row.from.prefix_exponent - row.to.prefix_exponent,
            );
            add_fit(&mut evidence_fitness, predicted, row.actual);
            add_fit(
                dataset_fitness.entry(row.dataset.clone()).or_default(),
                predicted,
                row.actual,
            );
            if predicted != row.actual && first_misses.len() < 16 {
                first_misses.push(format!(
                    "{} {} predicted=0x{predicted:016x} oracle=0x{:016x}",
                    row.dataset, row.id, row.actual
                ));
            }
        }
        scored.push(Candidate {
            scale_exponent,
            factor_construction: factor_construction.to_string(),
            inverse_construction: inverse_construction.to_string(),
            form: form.to_string(),
            schedule: schedule.to_string(),
            readback_fitness,
            evidence_fitness,
            dataset_fitness,
            factor_bits: factors
                .iter()
                .map(|(u, v)| (u.clone(), format!("0x{:016x}", v.to_bits())))
                .collect(),
            inverse_bits: inverses
                .iter()
                .map(|(u, v)| (u.clone(), format!("0x{:016x}", v.to_bits())))
                .collect(),
            first_misses,
        });
    }
    scored.sort_by_key(|candidate| {
        (
            std::cmp::Reverse(candidate.evidence_fitness.exact),
            candidate.evidence_fitness.sum_abs_ulp,
            candidate.evidence_fitness.max_abs_ulp,
        )
    });
    for candidate in scored.iter().take(30) {
        println!(
            "rb={}/{} scale={} {} {} {} {} ev={}/{} sum={} max={}",
            candidate.readback_fitness.exact,
            candidate.readback_fitness.total,
            candidate.scale_exponent,
            candidate.factor_construction,
            candidate.inverse_construction,
            candidate.form,
            candidate.schedule,
            candidate.evidence_fitness.exact,
            candidate.evidence_fitness.total,
            candidate.evidence_fitness.sum_abs_ulp,
            candidate.evidence_fitness.max_abs_ulp,
        );
    }
    let report = Report {
        schema_version: "w109.convert.core_schedule_search.v3",
        function: "CONVERT",
        readback_rows: readback.rows.len(),
        evidence_rows: rows.len(),
        maximum_readback_exact,
        readback_exact_candidates: scored.len(),
        candidates: scored,
    };
    let out = root.join("score-convert-core-schedule-search-v3.json");
    std::fs::write(&out, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
    println!("wrote -> {}", out.display());
}
