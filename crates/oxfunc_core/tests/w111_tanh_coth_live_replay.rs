//! Composition observations, with unresolved SINH inputs explicitly preserved.
use oxfunc_core::functions::{coth, tanh};
use serde_json::Value;
use std::collections::BTreeMap;

fn replay(raw: &str, report_index: usize, exact: usize, open: usize, independent: bool) {
    let data: Value = serde_json::from_str(raw).unwrap();
    let reports: Value = serde_json::from_str(if independent { include_str!(
        "../../../docs/function-lane/evidence/w111-broad-20260929/tanh-coth/independent-replay.json"
    ) } else { include_str!(
        "../../../docs/function-lane/evidence/w111-broad-20260929/tanh-coth/composition-graph-research.json"
    ) }).unwrap();
    let report = &reports[report_index];
    assert_eq!(data["function"], report["function"]);
    let key = if independent { "production_misses" } else { "best_misses" };
    let residuals: BTreeMap<_, _> = report[key].as_array().into_iter().flatten()
        .map(|r| (r["row"]["id"].as_str().unwrap(), r)).collect();
    let mut counts = (0, 0);
    for row in data["witnesses"].as_array().unwrap() {
        let bits = u64::from_str_radix(row["args"][0].as_str().unwrap().trim_start_matches("0x"), 16).unwrap();
        let x = f64::from_bits(bits);
        assert!(bits != 1 << 63 && (x == 0. || x.abs() >= f64::MIN_POSITIVE), "unqualified ingress");
        let result = if data["function"] == "TANH" {
            tanh::TANH_META.real_result_policy.publish(x, tanh::tanh_kernel(x))
        } else {
            coth::coth_kernel(x)
        };
        let actual = match result { Ok(v) => format!("0x{:016x}", v.to_bits()), Err(e) => format!("error:{e:?}") };
        let id = row["id"].as_str().unwrap();
        let expected = row["expected_bits"].as_str().unwrap();
        if let Some(residual) = residuals.get(id) {
            assert_eq!(row, &residual["row"]);
            assert_eq!(actual, residual["actual"], "changed open row {id}");
            assert_ne!(actual, expected);
            counts.1 += 1;
        } else {
            assert_eq!(actual, expected, "{id}");
            counts.0 += 1;
        }
    }
    assert_eq!(counts, (exact, open));
}

#[test]
fn tanh_composition_preserves_six_shared_sinh_residuals() {
    replay(include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/hyperbolic/discovery/answers/answers-tanh.json"), 0, 8990, 0, false);
    replay(include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/hyperbolic/heldout/answers/answers-tanh.json"), 1, 23096, 6, false);
}

#[test]
fn coth_composition_preserves_five_shared_sinh_residuals() {
    replay(include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/hyperbolic/discovery/answers/answers-coth.json"), 2, 8990, 0, false);
    replay(include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/hyperbolic/heldout/answers/answers-coth.json"), 3, 23095, 5, false);
}

#[test]
fn independent_composition_holdout_preserves_all_new_failures() {
    replay(include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/tanh-coth/independent-tanh.json"), 0, 7090, 52, true);
    replay(include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/tanh-coth/independent-coth.json"), 1, 7092, 50, true);
}
