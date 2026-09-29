//! Surface-dispatch replay of exact live date observations. WEEKNUM's unsupported
//! selector lane is retained but withheld because duplicate oracle inputs disagree.
use oxfunc_core::functions::surface_dispatch::eval_surface_value_call;
use oxfunc_core::resolver::NULL_REFERENCE_SYSTEM_PROVIDER;
use oxfunc_core::value::{CalcValue, CoreValue};
use std::path::Path;

fn numeric(bits: &str) -> f64 {
    f64::from_bits(u64::from_str_radix(bits.trim_start_matches("0x"), 16).unwrap())
}

fn selector(number: f64) -> i64 {
    let ceiling = number.ceil();
    let threshold = if ceiling > 0.0 { 2.0_f64.powi(-22) + 2.0_f64.powi(-33) }
        else { 2.0_f64.powi(-23) + 2.0_f64.powi(-34) };
    if ceiling - number <= threshold { ceiling as i64 } else { number.floor() as i64 }
}

fn replay(directory: &str) {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/function-lane/evidence/w111-broad-20260929/dates").join(directory);
    let mut admitted = 0;
    let mut withheld = 0;
    let mut mismatches = Vec::new();
    for entry in std::fs::read_dir(root).unwrap() {
        let path = entry.unwrap().path();
        if !path.file_name().unwrap().to_string_lossy().starts_with("answers-") { continue; }
        let bank: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
        let name = bank["function"].as_str().unwrap();
        let function = format!("FUNC.{name}");
        for row in bank["witnesses"].as_array().unwrap() {
            let numbers: Vec<_> = row["args"].as_array().unwrap().iter().map(|v| numeric(v.as_str().unwrap())).collect();
            if name == "WEEKNUM" && !matches!(selector(numbers[1]), 1 | 2 | 11..=17 | 21) {
                withheld += 1;
                continue;
            }
            let args: Vec<_> = numbers.into_iter().map(CalcValue::number).collect();
            let result = eval_surface_value_call(&function, &args, &NULL_REFERENCE_SYSTEM_PROVIDER,
                None, None, None, None).unwrap_or_else(CalcValue::error);
            let actual = match result.core() {
                CoreValue::Number(n) => format!("0x{:016x}", n.to_bits()),
                CoreValue::Error(e) => format!("error:{e:?}"),
                other => panic!("unexpected date result {other:?}"),
            };
            if actual != row["expected_bits"].as_str().unwrap() {
                mismatches.push(format!("{function} {} args={} expected={} actual={actual}", row["id"], row["args"], row["expected_bits"]));
            }
            admitted += 1;
        }
    }
    assert!(admitted > 0);
    eprintln!("{directory}: {admitted} compared, {} mismatches; {withheld} unsupported WEEKNUM selectors withheld", mismatches.len());
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
}

#[test] fn date_boundary_live_replay() { replay("boundaries"); }
#[test] fn date_conversion_live_replay() { replay("conversion"); }
#[test] fn date_integer_live_replay() { replay("integer"); }
#[test] fn date_independent_live_replay() { replay("heldout"); }
#[test] fn date_refinement_live_replay() { replay("refinement"); }

#[test] fn date_second_independent_live_replay() { replay("heldout2"); }
