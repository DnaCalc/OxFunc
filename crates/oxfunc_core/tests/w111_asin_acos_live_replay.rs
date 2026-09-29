//! Staged inverse-angle graphs bound to retained exact Excel observations.
use oxfunc_core::functions::surface_dispatch::eval_surface_value_call;
use oxfunc_core::resolver::NULL_REFERENCE_SYSTEM_PROVIDER;
use oxfunc_core::value::{CalcValue, CoreValue};
use serde_json::Value;

fn replay(raw: &str, function: &str, rows: usize) {
    let evidence: Value = serde_json::from_str(raw).unwrap();
    assert_eq!(evidence["function"], function);
    let witnesses = evidence["witnesses"].as_array().unwrap();
    assert_eq!(witnesses.len(), rows);
    let mut misses = Vec::new();
    for row in witnesses {
        let bits = u64::from_str_radix(
            row["args"][0].as_str().unwrap().trim_start_matches("0x"),
            16,
        )
        .unwrap();
        let x = f64::from_bits(bits);
        assert!(x.is_finite() && !x.is_subnormal());
        let result = eval_surface_value_call(
            &format!("FUNC.{function}"),
            &[CalcValue::number(x)],
            &NULL_REFERENCE_SYSTEM_PROVIDER,
            None,
            None,
            None,
            None,
        )
        .unwrap_or_else(CalcValue::error);
        let actual = match result.core() {
            CoreValue::Number(n) => format!("0x{:016x}", n.to_bits()),
            CoreValue::Error(code) => format!("error:{code:?}"),
            other => panic!("unexpected {other:?}"),
        };
        if actual != row["expected_bits"].as_str().unwrap() {
            misses.push(format!(
                "{} {} actual={actual} expected={}",
                row["id"], row["args"], row["expected_bits"]
            ));
        }
    }
    assert!(
        misses.is_empty(),
        "{} differences:\n{}",
        misses.len(),
        misses.join("\n")
    );
}

#[test]
fn asin_reused_factor_matches_numeric_discovery() {
    replay(
        include_str!(
            "../../../docs/function-lane/evidence/w111-broad-20260929/asin/discovery.json"
        ),
        "ASIN",
        1224,
    );
}
#[test]
fn asin_signed_factor_reuse_matches_exact_readback_controls() {
    replay(
        include_str!(
            "../../../docs/function-lane/evidence/w111-broad-20260929/asin/isolation-asin.json"
        ),
        "ASIN",
        14,
    );
    replay(
        include_str!(
            "../../../docs/function-lane/evidence/w111-broad-20260929/asin/signed-discovery.json"
        ),
        "ASIN",
        6180,
    );
}
#[test]
fn asin_reduced_angle_matches_independent_discriminators() {
    replay(
        include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/asin/heldout.json"),
        "ASIN",
        6722,
    );
}
#[test]
fn acos_staged_subtraction_matches_signed_controls() {
    replay(
        include_str!(
            "../../../docs/function-lane/evidence/w111-broad-20260929/asin/acos-caller.json"
        ),
        "ACOS",
        6180,
    );
}

#[test]
fn asin_acos_fresh_combined_holdout_matches_frozen_kernels() {
    replay(include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/asin/heldout2-asin.json"),"ASIN",8052);
    replay(include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/asin/heldout2-acos.json"),"ACOS",8052);
}
