//! Exact typed replay of retained Excel 20430/CV2 radix observations.
use oxfunc_core::functions::surface_dispatch::eval_surface_value_call;
use oxfunc_core::resolver::NULL_REFERENCE_SYSTEM_PROVIDER;
use oxfunc_core::value::{CalcValue, CoreValue};

fn replay(source: &str) {
    let data: serde_json::Value = serde_json::from_str(source).unwrap();
    let function = format!("FUNC.{}", data["function"].as_str().unwrap());
    for row in data["witnesses"].as_array().unwrap() {
        let args: Vec<_> = row["args"].as_array().unwrap().iter().map(|a| {
            let bits = u64::from_str_radix(a.as_str().unwrap().trim_start_matches("0x"), 16).unwrap();
            CalcValue::number(f64::from_bits(bits))
        }).collect();
        let actual = match eval_surface_value_call(&function, &args,
            &NULL_REFERENCE_SYSTEM_PROVIDER, None, None, None, None) {
            Err(e) => format!("error:{e:?}"),
            Ok(v) => match v.core {
                CoreValue::Number(n) => format!("0x{:016x}", n.to_bits()),
                CoreValue::Error(e) => format!("error:{e:?}"),
                CoreValue::Text(t) => format!("text:{}", t.to_string_lossy()),
                other => panic!("unexpected radix result: {other:?}"),
            },
        };
        assert_eq!(actual, row["expected_bits"].as_str().unwrap(), "{function} {} args={}", row["id"], row["args"]);
    }
}

macro_rules! live_replay {
    ($name:ident, $file:literal) => {
        #[test]
        fn $name() {
            replay(include_str!(concat!("../../../docs/function-lane/evidence/w111-broad-20260929/radix/", $file)));
        }
    };
}

live_replay!(base_bounds, "answers-batch-base.json");
live_replay!(dec2bin_bounds, "answers-batch-dec2bin.json");
live_replay!(dec2hex_bounds, "answers-batch-dec2hex.json");
live_replay!(dec2oct_bounds, "answers-batch-dec2oct.json");
live_replay!(bin2hex_bounds, "answers-batch-bin2hex.json");
live_replay!(bin2oct_bounds, "answers-batch-bin2oct.json");
live_replay!(hex2bin_bounds, "answers-batch-hex2bin.json");
live_replay!(hex2oct_bounds, "answers-batch-hex2oct.json");
live_replay!(oct2bin_bounds, "answers-batch-oct2bin.json");
live_replay!(oct2hex_bounds, "answers-batch-oct2hex.json");
live_replay!(base_independent_heldout, "heldout/answers-base.json");
live_replay!(dec2bin_independent_heldout, "heldout/answers-dec2bin.json");
live_replay!(dec2hex_independent_heldout, "heldout/answers-dec2hex.json");
live_replay!(dec2oct_independent_heldout, "heldout/answers-dec2oct.json");
