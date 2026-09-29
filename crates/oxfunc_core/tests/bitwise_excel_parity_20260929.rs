//! Permanent reduced regressions from the 20430/CV2 Value2 broad campaign.
//! Source: smart-fuzzer/runs/w111-broad-20260929/answers/answers-bit*.json.
use oxfunc_core::functions::surface_dispatch::eval_surface_value_call;
use oxfunc_core::resolver::NULL_REFERENCE_SYSTEM_PROVIDER;
use oxfunc_core::value::{CalcValue, CoreValue, WorksheetErrorCode};

fn eval(name: &str, a: f64, b: f64) -> Result<CalcValue, WorksheetErrorCode> {
    eval_surface_value_call(
        name,
        &[CalcValue::number(a), CalcValue::number(b)],
        &NULL_REFERENCE_SYSTEM_PROVIDER,
        None,
        None,
        None,
        None,
    )
}

fn assert_num_error(name: &str, a: f64, b: f64) {
    match eval(name, a, b) {
        Err(WorksheetErrorCode::Num) => (),
        Ok(v) if matches!(v.core, CoreValue::Error(WorksheetErrorCode::Num)) => (),
        other => panic!("{name}({a},{b}) expected #NUM!, got {other:?}"),
    }
}

fn assert_bits(name: &str, a: f64, b: f64, expected: f64) {
    match eval(name, a, b).unwrap().core {
        CoreValue::Number(v) => assert_eq!(v.to_bits(), expected.to_bits(), "{name}({a},{b})"),
        other => panic!("{name}({a},{b}) expected numeric {expected}, got {other:?}"),
    }
}

#[test]
fn bitwise_dispatch_rejects_fractional_operands_on_both_sides() {
    for name in ["FUNC.BITAND", "FUNC.BITOR", "FUNC.BITXOR"] {
        assert_num_error(name, 0.9, 1.0);
        assert_num_error(name, 130_806_392_774_274.0, 33_072_894_177_336.25);
    }
    for name in ["FUNC.BITLSHIFT", "FUNC.BITRSHIFT"] {
        assert_num_error(name, 0.9, 0.0);
    }
}

#[test]
fn bitwise_dispatch_shift_direction_overflow_and_shortcuts() {
    assert_num_error("FUNC.BITLSHIFT", 140_737_488_355_328.0, 47.0);
    assert_num_error("FUNC.BITRSHIFT", 140_737_488_355_328.0, -48.0);
    assert_num_error("FUNC.BITRSHIFT", 1.0, -48.0);
    assert_bits("FUNC.BITLSHIFT", 1.0, 47.0, 140_737_488_355_328.0);
    assert_bits("FUNC.BITLSHIFT", 1.0, 53.0, 0.0);
    assert_bits("FUNC.BITRSHIFT", 1.0, -53.0, 0.0);
    assert_bits("FUNC.BITLSHIFT", 0.0, 54.0, 0.0);
    assert_bits("FUNC.BITRSHIFT", 0.0, -54.0, 0.0);
    assert_bits(
        "FUNC.BITLSHIFT",
        197_029_275_778_236.0,
        -13.5,
        24_051_425_265.0,
    );
}

#[test]
fn bitwise_dispatch_explicit_missing_arguments_coerce_to_zero() {
    for (name, expected) in [
        ("FUNC.BITAND", [0.0_f64, 0.0]),
        ("FUNC.BITOR", [2.0, 3.0]),
        ("FUNC.BITXOR", [2.0, 3.0]),
        ("FUNC.BITLSHIFT", [0.0, 3.0]),
        ("FUNC.BITRSHIFT", [0.0, 3.0]),
    ] {
        for (args, expected) in [
            ([CalcValue::missing(), CalcValue::number(2.0)], expected[0]),
            ([CalcValue::number(3.0), CalcValue::missing()], expected[1]),
        ] {
            let result = eval_surface_value_call(
                name,
                &args,
                &NULL_REFERENCE_SYSTEM_PROVIDER,
                None,
                None,
                None,
                None,
            )
            .unwrap();
            let CoreValue::Number(actual) = result.core() else {
                panic!("{name}: {result:?}");
            };
            assert_eq!(actual.to_bits(), expected.to_bits(), "{name}");
        }
    }
}
