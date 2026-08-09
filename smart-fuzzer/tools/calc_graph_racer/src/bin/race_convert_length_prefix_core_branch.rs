//! Race the length-core branch exposed by the retired v2 `nm -> Pm` kill.
//!
//! The row is explained exactly by applying the prefix multiplier to the
//! original input, while the frozen v2 graph first perturbs the input through
//! `(x * angstroms_per_meter) / angstroms_per_meter`.  Direct `m -> m`
//! diagnostics prove that an unconditional identity shortcut is wrong, so
//! this scorer races only prefix-path branch predicates.

#[path = "convert_research/common.rs"]
mod common;

use common::{MetaDocument, ordered_bits};
use oxfunc_core::excel_numeric::research as rx;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::PathBuf;

const MODELS: [&str; 11] = [
    "always_angstrom_core",
    "always_x87_angstrom_core_store",
    "always_x87_angstrom_core_continuous",
    "always_angstrom_ratio_core",
    "always_scale_1e13_core",
    "always_effective_factor_mul_div",
    "always_effective_factor_ratio_mul",
    "shortcut_same_direct_when_raw_differs",
    "shortcut_same_direct_when_any_prefix",
    "shortcut_same_direct_when_prefix_exponents_differ",
    "shortcut_same_direct_unconditional",
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

fn scale_1e13(unit: &str) -> f64 {
    match unit {
        "m" => 10_000_000_000_000.0,
        "in" => 254_000_000_000.0,
        "ft" => 3_048_000_000_000.0,
        "yd" => 9_144_000_000_000.0,
        "mi" => 16_093_440_000_000_000.0,
        "Nmi" => 18_520_000_000_000_000.0,
        other => panic!("no scale-1e13 factor for {other}"),
    }
}

fn pow10(exponent: i32) -> f64 {
    format!("1e{exponent}").parse().unwrap()
}

fn finish_prefix(core: f64, delta_exponent: i32) -> f64 {
    let cw = rx::CW_PC64_RN;
    let product = rx::ext_mul(
        &rx::ext_from_f64(core),
        &rx::ext_from_f64(pow10(delta_exponent)),
        cw,
    );
    rx::ext_to_f64(&product, cw)
}

fn predictions(number: f64, from_raw: &str, to_raw: &str) -> [f64; MODELS.len()] {
    let from = resolve(from_raw);
    let to = resolve(to_raw);
    let angstrom_core = (number * angstroms(&from.direct)) / angstroms(&to.direct);
    let cw = rx::CW_PC64_RN;
    let x87_product = rx::ext_mul(
        &rx::ext_from_f64(number),
        &rx::ext_from_f64(angstroms(&from.direct)),
        cw,
    );
    let x87_core = rx::ext_div(
        &x87_product,
        &rx::ext_from_f64(angstroms(&to.direct)),
        cw,
    );
    let x87_core_stored = rx::ext_to_f64(&x87_core, cw);
    let delta = pow10(from.prefix_exponent - to.prefix_exponent);
    let x87_continuous = rx::ext_to_f64(
        &rx::ext_mul(&x87_core, &rx::ext_from_f64(delta), cw),
        cw,
    );
    let angstrom_ratio_core = number * (angstroms(&from.direct) / angstroms(&to.direct));
    let scale_1e13_core = (number * scale_1e13(&from.direct)) / scale_1e13(&to.direct);
    let effective_from = angstroms(&from.direct) * pow10(from.prefix_exponent);
    let effective_to = angstroms(&to.direct) * pow10(to.prefix_exponent);
    let effective_mul_div = (number * effective_from) / effective_to;
    let effective_ratio_mul = number * (effective_from / effective_to);
    let same_direct = from.direct == to.direct;
    let any_prefix = from.prefix_exponent != 0 || to.prefix_exponent != 0;
    let prefix_differs = from.prefix_exponent != to.prefix_exponent;
    let raw_differs = from_raw != to_raw;
    let cores = [
        angstrom_core,
        x87_core_stored,
        // Placeholder core; this slot is overwritten after the common final
        // prefix map because the x87 value remains continuous here.
        x87_core_stored,
        angstrom_ratio_core,
        scale_1e13_core,
        // Prefixes are already folded into these two predictions, so their
        // common final-prefix map is overwritten below.
        number,
        number,
        if same_direct && raw_differs { number } else { angstrom_core },
        if same_direct && any_prefix { number } else { angstrom_core },
        if same_direct && prefix_differs { number } else { angstrom_core },
        if same_direct { number } else { angstrom_core },
    ];
    let mut values = cores.map(|core| finish_prefix(core, from.prefix_exponent - to.prefix_exponent));
    values[2] = x87_continuous;
    values[5] = effective_mul_div;
    values[6] = effective_ratio_mul;
    values
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
        if row.category != "length" {
            continue;
        }
        let actual = bits(&by_id[row.id.as_str()].expected_bits).expect("length must be numeric");
        let number = common::f64_from_hex(&row.number_bits).unwrap();
        for (name, predicted) in MODELS
            .iter()
            .zip(predictions(number, &row.from_unit, &row.to_unit))
        {
            let score = scores.get_mut(*name).unwrap();
            score.total += 1;
            let residual = ordered_bits(actual) - ordered_bits(predicted.to_bits());
            if residual == 0 {
                score.exact += 1;
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
                        "{} {}->{} x={} residual={:+} predicted=0x{:016x} oracle=0x{:016x}",
                        row.id,
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
    }
    let report = Report {
        schema_version: "w109.convert.length_prefix_core_branch_race.v1",
        function: "CONVERT",
        scores,
        capture_provenance: answers.capture_provenance,
    };
    if let Some(path) = args.out {
        std::fs::write(&path, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
        println!("wrote length prefix-core race -> {}", path.display());
    }
}
