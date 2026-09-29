//! Exact public-dispatch checks against independently captured Excel complex-kernel results.
use std::collections::HashMap;

use oxfunc_core::functions::surface_dispatch::eval_surface_value_call;
use oxfunc_core::resolver::NULL_REFERENCE_SYSTEM_PROVIDER;
use oxfunc_core::value::{CalcValue, CoreValue, ExcelText};
use serde_json::Value;

fn result(function: &str, args: &[CalcValue]) -> String {
    let value = eval_surface_value_call(
        &format!("FUNC.{function}"),
        args,
        &NULL_REFERENCE_SYSTEM_PROVIDER,
        None,
        None,
        None,
        None,
    )
    .unwrap_or_else(CalcValue::error);
    match value.core() {
        CoreValue::Number(n) => format!("number:0x{:016x}",n.to_bits()),
        CoreValue::Text(text) => format!("text:{}", text.to_string_lossy()),
        CoreValue::Error(code) => format!("error:{code:?}"),
        other => panic!("unexpected {other:?}"),
    }
}

fn replay_numeric(raw: &str, rows: usize, ingress_limited: usize) {
    let capture: Value = serde_json::from_str(raw).unwrap();
    let witnesses = capture["witnesses"].as_array().unwrap();
    assert_eq!(witnesses.len(), rows);
    let function = capture["function"].as_str().unwrap();
    let mut skipped = 0;
    for row in witnesses {
        let numbers: Vec<_> = row["args"]
            .as_array()
            .unwrap()
            .iter()
            .map(|x| {
                f64::from_bits(
                    u64::from_str_radix(x.as_str().unwrap().strip_prefix("0x").unwrap(), 16)
                        .unwrap(),
                )
            })
            .collect();
        assert!(numbers.iter().all(|n| n.is_finite()));
        if numbers.iter().any(|n| n.is_subnormal()) {
            // Value2 changed these source inputs to zero: ../numeric-ingress.json.
            // Keep the complete capture, but do not assert parity for a different input.
            skipped += 1;
            continue;
        }
        let args: Vec<_> = numbers.into_iter().map(CalcValue::number).collect();
        assert_eq!(
            result(function, &args),
            row["expected_bits"].as_str().unwrap(),
            "{} {:?}",
            row["id"],
            row["args"]
        );
    }
    assert_eq!(
        skipped, ingress_limited,
        "input-transport qualification changed"
    );
}

fn replay_typed(raw_cases: &str, raw_outcomes: &str, function: &str, rows: usize) {
    let cases: Value = serde_json::from_str(raw_cases).unwrap();
    let mut outcomes = HashMap::new();
    for line in raw_outcomes.lines() {
        let row: Value = serde_json::from_str(line).unwrap();
        assert!(
            outcomes
                .insert(row["case_id"].as_str().unwrap().to_owned(), row)
                .is_none()
        );
    }
    let mut tested = 0;
    for row in cases["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c["canonical_surface_name"] == function)
    {
        let oracle = &outcomes[row["case_id"].as_str().unwrap()];
        assert_eq!(oracle["function_id"], row["function_id"]);
        assert_eq!(oracle["formula_text"], row["formula_text"]);
        assert_eq!(oracle["execution_status"], "ok");
        let args: Vec<_> = row["args"]
            .as_array()
            .unwrap()
            .iter()
            .map(|a| {
                assert_eq!(a["kind"], "text");
                CalcValue::text(ExcelText::from_utf16_code_units(
                    a["value"].as_str().unwrap().encode_utf16().collect(),
                ))
            })
            .collect();
        let expected = match oracle["outcome"]["kind"].as_str().unwrap() {
            "number" => format!("number:{}",oracle["outcome"]["bits_hex"].as_str().unwrap()),
            "text" => format!("text:{}", oracle["outcome"]["value"].as_str().unwrap()),
            "error" => format!("error:{}", oracle["outcome"]["code"].as_str().unwrap()),
            other => panic!("unexpected {other}"),
        };
        assert_eq!(
            result(function, &args),
            expected,
            "{} {:?}",
            row["case_id"],
            row["args"]
        );
        tested += 1;
    }
    assert_eq!(tested, rows);
}

#[test]
fn imdiv_real_numeric_inputs_match_retained_excel() {
    replay_numeric(
        include_str!(
            "../../../docs/function-lane/evidence/w111-broad-20260929/complex-kernels/extended-imdiv.json"
        ),
        1216,
        0,
    );
    replay_numeric(
        include_str!(
            "../../../docs/function-lane/evidence/w111-broad-20260929/complex-kernels/extremes-imdiv.json"
        ),
        852,
        6,
    );
}

#[test]
fn imdiv_complex_text_quadrants_and_scales_match_retained_excel() {
    replay_typed(
        include_str!(
            "../../../docs/function-lane/evidence/w111-broad-20260929/complex-kernels/typed-cases.json"
        ),
        include_str!(
            "../../../docs/function-lane/evidence/w111-broad-20260929/complex-kernels/typed-excel.jsonl"
        ),
        "IMDIV",
        400,
    );
}

#[test]
fn imdiv_independent_complex_text_holdout_matches_retained_excel() {
    replay_typed(
        include_str!(
            "../../../docs/function-lane/evidence/w111-broad-20260929/complex-kernels/imdiv-heldout-cases.json"
        ),
        include_str!(
            "../../../docs/function-lane/evidence/w111-broad-20260929/complex-kernels/imdiv-heldout-excel.jsonl"
        ),
        "IMDIV",
        512,
    );
}

#[test]
fn imsqrt_numeric_and_complex_boundary_graph_matches_retained_excel() {
    replay_numeric(
        include_str!(
            "../../../docs/function-lane/evidence/w111-broad-20260929/complex-kernels/extended-imsqrt.json"
        ),
        1209,
        0,
    );
    replay_numeric(
        include_str!(
            "../../../docs/function-lane/evidence/w111-broad-20260929/complex-kernels/extremes-imsqrt.json"
        ),
        827,
        6,
    );
    replay_typed(
        include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/complex-kernels/imsqrt-heldout-cases.json"),
        include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/complex-kernels/imsqrt-heldout-excel.jsonl"),
        "IMSQRT",960,
    );
    replay_typed(
        include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/complex-kernels/imsqrt-heldout2-cases.json"),
        include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/complex-kernels/imsqrt-heldout2-excel.jsonl"),
        "IMSQRT",960,
    );
    replay_typed(
        include_str!(
            "../../../docs/function-lane/evidence/w111-broad-20260929/complex-kernels/trig-cases.json"
        ),
        include_str!(
            "../../../docs/function-lane/evidence/w111-broad-20260929/complex-kernels/trig-excel.jsonl"
        ),
        "IMSQRT",
        64,
    );
    replay_numeric(
        include_str!(
            "../../../docs/function-lane/evidence/w111-broad-20260929/complex-kernels/imsqrt-output-format-complex.json"
        ),
        50,
        0,
    );
    replay_typed(
        include_str!(
            "../../../docs/function-lane/evidence/w111-broad-20260929/complex-kernels/typed-cases.json"
        ),
        include_str!(
            "../../../docs/function-lane/evidence/w111-broad-20260929/complex-kernels/typed-excel.jsonl"
        ),
        "IMSQRT",
        93,
    );
    replay_typed(
        include_str!(
            "../../../docs/function-lane/evidence/w111-broad-20260929/complex-kernels/imsqrt-boundary-cases.json"
        ),
        include_str!(
            "../../../docs/function-lane/evidence/w111-broad-20260929/complex-kernels/imsqrt-boundary-excel.jsonl"
        ),
        "IMSQRT",
        523,
    );
}

#[test]
fn complex_sine_and_cosine_general_graph_matches_discovery() {
    for function in ["IMCOS", "IMSIN"] {
        replay_typed(
            include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/complex-kernels/trig-cases.json"),
            include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/complex-kernels/trig-excel.jsonl"),
            function,
            200,
        );
        replay_typed(
            include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/complex-kernels/trig-heldout-cases.json"),
            include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/complex-kernels/trig-heldout-excel.jsonl"),
            function,
            384,
        );
        replay_typed(
            include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/complex-kernels/trig-heldout2-cases.json"),
            include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/complex-kernels/trig-heldout2-excel.jsonl"),
            function,
            384,
        );
    }
}

#[test]
fn complex_argument_ratio_boundary_matches_retained_excel() {
    replay_typed(
        include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/complex-kernels/trig-cases.json"),
        include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/complex-kernels/trig-excel.jsonl"),
        "IMARGUMENT",64,
    );
    replay_typed(
        include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/complex-kernels/imargument-heldout-cases.json"),
        include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/complex-kernels/imargument-heldout-excel.jsonl"),
        "IMARGUMENT",476,
    );
    replay_typed(
        include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/complex-kernels/imargument-heldout2-cases.json"),
        include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/complex-kernels/imargument-heldout2-excel.jsonl"),
        "IMARGUMENT",476,
    );
}

#[test]
fn complex_coefficient_lexical_precision_and_admission_matches_excel() {
    for (function,rows) in [("IMREAL",117),("IMAGINARY",51)] {
        replay_typed(
            include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/complex-kernels/parser-cases.json"),
            include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/complex-kernels/parser-excel.jsonl"),
            function,rows,
        );
    }
    replay_typed(
        include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/complex-kernels/parser-edge-cases.json"),
        include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/complex-kernels/parser-edge-excel.jsonl"),
        "IMREAL",275,
    );
}

#[test]
fn reduced_angle_fresh_complex_holdouts_match_retained_excel() {
    replay_typed(
        include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/complex-kernels/imargument-heldout3-cases.json"),
        include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/complex-kernels/imargument-heldout3-excel.jsonl"),
        "IMARGUMENT",476,
    );
    replay_typed(
        include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/complex-kernels/imsqrt-heldout3-cases.json"),
        include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/complex-kernels/imsqrt-heldout3-excel.jsonl"),
        "IMSQRT",960,
    );
}
