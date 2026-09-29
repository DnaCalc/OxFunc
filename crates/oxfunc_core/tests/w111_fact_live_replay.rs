//! FACT discovery and independent held-out Value2 observations, Excel 20430/CV2.
use oxfunc_core::functions::surface_dispatch::eval_surface_value_call;
use oxfunc_core::resolver::NULL_REFERENCE_SYSTEM_PROVIDER;
use oxfunc_core::value::{CalcValue, CoreValue};

fn replay(source: &str) {
    let data: serde_json::Value = serde_json::from_str(source).unwrap();
    for row in data["witnesses"].as_array().unwrap() {
        let args: Vec<_> = row["args"].as_array().unwrap().iter().map(|a| {
            let bits = u64::from_str_radix(a.as_str().unwrap().trim_start_matches("0x"), 16).unwrap();
            CalcValue::number(f64::from_bits(bits))
        }).collect();
        let actual = match eval_surface_value_call("FUNC.FACT", &args,
            &NULL_REFERENCE_SYSTEM_PROVIDER, None, None, None, None) {
            Err(e) => format!("error:{e:?}"),
            Ok(v) => match v.core {
                CoreValue::Number(n) => format!("0x{:016x}", n.to_bits()),
                CoreValue::Error(e) => format!("error:{e:?}"),
                other => panic!("unexpected FACT result: {other:?}"),
            },
        };
        assert_eq!(actual, row["expected_bits"].as_str().unwrap(), "{} args={}", row["id"],row["args"]);
    }
}

#[test]
fn fact_live_discovery_matches_every_retained_bit() {
    replay(include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/fact/discovery.json"));
}

#[test]
fn fact_live_heldout_matches_every_retained_bit() {
    replay(include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/fact/heldout.json"));
}
