//! Rounding count/width/assembly observations; exceptional initial15 lanes remain open.
use oxfunc_core::functions::surface_dispatch::eval_surface_value_call;
use oxfunc_core::resolver::NULL_REFERENCE_SYSTEM_PROVIDER;
use oxfunc_core::value::{CalcValue, CoreValue};
use serde_json::Value;

fn replay(source: &str) {
    replay_with_publication_scope(source, false);
}

fn replay_with_publication_scope(source: &str, finite_only: bool) {
    let evidence: Value = serde_json::from_str(source).unwrap();
    let mut misses = Vec::new();
    for row in evidence["witnesses"].as_array().unwrap() {
        let expected = row["expected_bits"].as_str().unwrap();
        // The retained 572 raw nonfinite Value2 payloads are an explicit open
        // publication lane. This test exercises every finite/error endpoint;
        // it does not rewrite those payload rows as worksheet errors.
        if finite_only && expected.starts_with("0x") {
            let bits = u64::from_str_radix(&expected[2..], 16).unwrap();
            if !f64::from_bits(bits).is_finite() {
                continue;
            }
        }
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
            &format!("FUNC.{}", evidence["function"].as_str().unwrap()),
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
fn trunc_count_and_scale_refinement() {
    replay(include_str!(
        "../../../docs/function-lane/evidence/w111-broad-20260929/rounding-boundaries/numeric-trunc.json"
    ));
}

#[test]
fn rounddown_count_and_scale_refinement() {
    replay(include_str!(
        "../../../docs/function-lane/evidence/w111-broad-20260929/rounding-boundaries/numeric-rounddown.json"
    ));
}

#[test]
fn roundup_count_and_scale_refinement() {
    replay(include_str!(
        "../../../docs/function-lane/evidence/w111-broad-20260929/rounding-boundaries/numeric-roundup.json"
    ));
}

#[test]
fn round_count_and_scale_refinement() {
    replay(include_str!(
        "../../../docs/function-lane/evidence/w111-broad-20260929/rounding-boundaries/numeric-round.json"
    ));
}

#[test]
fn finite_trunc_endpoints() {
    replay_with_publication_scope(include_str!(
        "../../../docs/function-lane/evidence/w111-broad-20260929/rounding-boundaries/endpoint-trunc.json"
    ), true);
}

#[test]
fn finite_rounddown_endpoints() {
    replay_with_publication_scope(include_str!(
        "../../../docs/function-lane/evidence/w111-broad-20260929/rounding-boundaries/endpoint-rounddown.json"
    ), true);
}

#[test]
fn finite_roundup_endpoints() {
    replay_with_publication_scope(include_str!(
        "../../../docs/function-lane/evidence/w111-broad-20260929/rounding-boundaries/endpoint-roundup.json"
    ), true);
}

#[test]
fn finite_round_endpoints() {
    replay_with_publication_scope(include_str!(
        "../../../docs/function-lane/evidence/w111-broad-20260929/rounding-boundaries/endpoint-round.json"
    ), true);
}

#[test]
fn roundup_tiny_signed_residual_discovery() {
    replay(include_str!(
        "../../../docs/function-lane/evidence/w111-broad-20260929/rounding-boundaries/tiny-difference-roundup.json"
    ));
    replay(include_str!(
        "../../../docs/function-lane/evidence/w111-broad-20260929/rounding-boundaries/tiny-threshold-roundup.json"
    ));
    replay(include_str!(
        "../../../docs/function-lane/evidence/w111-broad-20260929/rounding-boundaries/refined-heldout-roundup.json"
    ));
}

#[test]
fn roundup_signed_residual_frozen_independent_heldout() {
    replay(include_str!(
        "../../../docs/function-lane/evidence/w111-broad-20260929/rounding-boundaries/residual-heldout-roundup.json"
    ));
    replay(include_str!(
        "../../../docs/function-lane/evidence/w111-broad-20260929/rounding-boundaries/residual-heldout-rounddown.json"
    ));
}
