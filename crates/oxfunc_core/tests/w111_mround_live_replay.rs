//! Retained public Excel numeric MROUND controls, checked through dispatch.
use oxfunc_core::functions::surface_dispatch::eval_surface_value_call;
use oxfunc_core::resolver::NULL_REFERENCE_SYSTEM_PROVIDER;
use oxfunc_core::value::{CalcValue, CoreValue};

fn replay(raw: &str, count: usize) {
    let data: serde_json::Value = serde_json::from_str(raw).unwrap();
    let rows = data["witnesses"].as_array().unwrap();
    assert_eq!(rows.len(), count);
    for row in rows {
        let args: Vec<_> = row["args"].as_array().unwrap().iter().map(|value| {
            CalcValue::number(f64::from_bits(u64::from_str_radix(
                value.as_str().unwrap().trim_start_matches("0x"), 16).unwrap()))
        }).collect();
        let actual = match eval_surface_value_call("FUNC.MROUND", &args,
            &NULL_REFERENCE_SYSTEM_PROVIDER, None, None, None, None) {
            Err(error) => format!("error:{error:?}"),
            Ok(value) => match value.core {
                CoreValue::Number(number) => format!("0x{:016x}", number.to_bits()),
                CoreValue::Error(error) => format!("error:{error:?}"),
                other => panic!("unexpected MROUND result: {other:?}"),
            },
        };
        assert_eq!(actual, row["expected_bits"].as_str().unwrap(), "{}", row["id"]);
    }
}

#[test]
fn mround_broad_discovery_matches_public_bits() {
    replay(include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/mround/broad-answers.json"), 16018);
}

#[test]
fn mround_fractional_cutoff_matches_public_neighbors() {
    replay(include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/mround/half-boundary-answers.json"), 7734);
    replay(include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/mround/fraction-cutoff-answers.json"), 5460);
}

#[test]
fn mround_frozen_graph_matches_fresh_independent_inputs() {
    replay(include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/mround/heldout-answers.json"), 15552);
}

#[test]
fn mround_frozen_staging_matches_independent_graph_disagreements() {
    replay(include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/mround/staged-heldout-answers.json"), 3200);
}
