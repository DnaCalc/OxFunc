//! QUOTIENT exact-bit observations, with Value2 input exclusions explicit.
use oxfunc_core::functions::surface_dispatch::eval_surface_value_call;
use oxfunc_core::resolver::NULL_REFERENCE_SYSTEM_PROVIDER;
use oxfunc_core::value::{CalcValue, CoreValue};
use serde_json::Value;

fn replay(source: &str) {
    let evidence: Value = serde_json::from_str(source).unwrap();
    let mut misses = Vec::new();
    for row in evidence["witnesses"].as_array().unwrap() {
        if evidence["ingress_excluded_ids"]
            .as_array()
            .is_some_and(|ids| ids.contains(&row["id"]))
        {
            continue;
        }
        let args: Vec<_> = row["args"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| {
                CalcValue::number(f64::from_bits(
                    u64::from_str_radix(&v.as_str().unwrap()[2..], 16).unwrap(),
                ))
            })
            .collect();
        let got = eval_surface_value_call(
            "FUNC.QUOTIENT",
            &args,
            &NULL_REFERENCE_SYSTEM_PROVIDER,
            None,
            None,
            None,
            None,
        )
        .unwrap_or_else(CalcValue::error);
        let actual = match got.core() {
            CoreValue::Number(n) => format!("0x{:016x}", n.to_bits()),
            CoreValue::Error(e) => format!("error:{e:?}"),
            other => panic!("unexpected {other:?}"),
        };
        if actual != row["expected_bits"].as_str().unwrap() {
            misses.push(format!(
                "{} expected={} actual={actual}",
                row["id"], row["expected_bits"]
            ));
        }
    }
    assert!(
        misses.is_empty(),
        "{} differing rows:\n{}",
        misses.len(),
        misses.join("\n")
    );
}

#[test]
fn quotient_zero_and_overflow_match_discovery() {
    replay(include_str!(
        "../../../docs/function-lane/evidence/w111-broad-20260929/quotient/discovery.json"
    ));
}

#[test]
fn quotient_zero_overflow_and_integer_neighbors_match_fresh_heldout() {
    replay(include_str!(
        "../../../docs/function-lane/evidence/w111-broad-20260929/quotient/heldout.json"
    ));
}
