//! Exact ACOT numeric observations, with the arithmetic substrate explicitly bounded.
use oxfunc_core::functions::surface_dispatch::eval_surface_value_call;
use oxfunc_core::resolver::NULL_REFERENCE_SYSTEM_PROVIDER;
use oxfunc_core::value::{CalcValue,CoreValue};
use serde_json::Value;

fn replay(raw:&str,rows:usize) {
    let evidence:Value=serde_json::from_str(raw).unwrap();
    assert_eq!(evidence["function"],"ACOT");
    let witnesses=evidence["witnesses"].as_array().unwrap();
    assert_eq!(witnesses.len(),rows);
    let mut misses=Vec::new();
    for row in witnesses {
        let bits=u64::from_str_radix(row["args"][0].as_str().unwrap().trim_start_matches("0x"),16).unwrap();
        let x=f64::from_bits(bits);assert!(x.is_finite()&&!x.is_subnormal());
        let result=eval_surface_value_call("FUNC.ACOT",&[CalcValue::number(x)],
            &NULL_REFERENCE_SYSTEM_PROVIDER,None,None,None,None).unwrap_or_else(CalcValue::error);
        let actual=match result.core() {
            CoreValue::Number(n)=>format!("0x{:016x}",n.to_bits()),
            CoreValue::Error(code)=>format!("error:{code:?}"),other=>panic!("unexpected {other:?}"),
        };
        if actual!=row["expected_bits"].as_str().unwrap() {
            misses.push(format!("{} {} actual={actual} expected={}",row["id"],row["args"],row["expected_bits"]));
        }
    }
    assert!(misses.is_empty(),"{} differences:\n{}",misses.len(),misses.join("\n"));
}

#[test]
fn acot_reciprocal_graph_matches_retained_discovery() {
    replay(include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/acot/discovery.json"),1224);
}

#[test]
fn acot_reduced_angle_resolves_all_retained_holdout_rows() {
    replay(include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/acot/heldout.json"),7946);
}

#[test]
fn acot_fresh_reduced_angle_holdout_matches_retained_excel() {
    replay(include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/acot/heldout2.json"),7976);
}
