//! W111 complex coefficient rendering: replay independently captured Excel text byte-for-byte.
//! All numeric inputs are normal binary64 values transported through Excel Range.Value2.
//! No numeric tolerance is used for a function whose result is text.

use oxfunc_core::functions::surface_dispatch::eval_surface_value_call;
use oxfunc_core::resolver::NULL_REFERENCE_SYSTEM_PROVIDER;
use oxfunc_core::value::{CalcValue, CoreValue, WorksheetErrorCode};
use serde::Deserialize;

#[derive(Deserialize)]
struct Witness {
    id: String,
    args: Vec<String>,
    expected_bits: String,
}

#[derive(Deserialize)]
struct Capture {
    function: String,
    witnesses: Vec<Witness>,
}

fn replay(raw: &str, expected_rows: usize) {
    let capture: Capture = serde_json::from_str(raw).expect("retained Excel capture");
    assert_eq!(capture.witnesses.len(), expected_rows);
    // Source probe IDs in the initial format capture repeat for four normal-limit values.
    // Iterate every retained row directly; never index those rows by ID and lose a witness.
    for (index, row) in capture.witnesses.iter().enumerate() {
        let args: Vec<_> = row
            .args
            .iter()
            .map(|arg| {
                let bits = u64::from_str_radix(arg.strip_prefix("0x").expect("bit input"), 16)
                    .expect("binary64 bits");
                let number = f64::from_bits(bits);
                assert!(
                    number.is_finite() && (number == 0.0 || number.is_normal()),
                    "this capture excludes subnormal ingress: {}",
                    row.id
                );
                CalcValue::number(number)
            })
            .collect();
        let actual = eval_surface_value_call(
            &format!("FUNC.{}", capture.function),
            &args,
            &NULL_REFERENCE_SYSTEM_PROVIDER,
            None,
            None,
            None,
            None,
        )
        .unwrap_or_else(CalcValue::error);
        match actual.core() {
            CoreValue::Text(text) => assert_eq!(
                format!("text:{}", text.to_string_lossy()),
                row.expected_bits,
                "{} row {index}, args {:?}",
                row.id,
                row.args
            ),
            CoreValue::Error(WorksheetErrorCode::Num) => assert_eq!(
                "error:Num", row.expected_bits,
                "{} row {index}, args {:?}",
                row.id, row.args
            ),
            other => panic!("{} row {index}: unexpected outcome {other:?}", row.id),
        }
    }
}

#[test]
fn complex_coefficient_notation_precision_and_boundaries_match_retained_excel() {
    replay(
        include_str!(
            "../../../docs/function-lane/evidence/w111-broad-20260929/complex/format-complex.json"
        ),
        2382,
    );
    replay(
        include_str!(
            "../../../docs/function-lane/evidence/w111-broad-20260929/complex/extremes-complex.json"
        ),
        528,
    );
    replay(
        include_str!(
            "../../../docs/function-lane/evidence/w111-broad-20260929/complex/ties-complex.json"
        ),
        252,
    );
}

#[test]
fn adjacent_complex_identity_functions_share_the_observed_coefficient_rendering() {
    replay(
        include_str!(
            "../../../docs/function-lane/evidence/w111-broad-20260929/complex/format-imconjugate.json"
        ),
        794,
    );
    replay(
        include_str!(
            "../../../docs/function-lane/evidence/w111-broad-20260929/complex/format-imsum.json"
        ),
        794,
    );
    replay(
        include_str!(
            "../../../docs/function-lane/evidence/w111-broad-20260929/complex/format-improduct.json"
        ),
        794,
    );
}

#[test]
fn first_independent_holdout_and_midpoint_refinement_match_retained_excel() {
    // The first candidate failed four identity variants in this holdout. Keep the entire
    // capture and its rejected-candidate report; it is now regression evidence, not a fresh
    // independent validation of the revised arithmetic model.
    replay(
        include_str!(
            "../../../docs/function-lane/evidence/w111-broad-20260929/complex/heldout-complex.json"
        ),
        2500,
    );
    replay(
        include_str!(
            "../../../docs/function-lane/evidence/w111-broad-20260929/complex/heldout-imconjugate.json"
        ),
        625,
    );
    replay(
        include_str!(
            "../../../docs/function-lane/evidence/w111-broad-20260929/complex/heldout-imsum.json"
        ),
        625,
    );
    replay(
        include_str!(
            "../../../docs/function-lane/evidence/w111-broad-20260929/complex/heldout-improduct.json"
        ),
        625,
    );
    replay(
        include_str!(
            "../../../docs/function-lane/evidence/w111-broad-20260929/complex/midpoints-complex.json"
        ),
        280,
    );
}

#[test]
fn second_independent_ordinary_holdout_matches_retained_excel() {
    replay(
        include_str!(
            "../../../docs/function-lane/evidence/w111-broad-20260929/complex/heldout2-complex.json"
        ),
        2500,
    );
    replay(
        include_str!(
            "../../../docs/function-lane/evidence/w111-broad-20260929/complex/heldout2-imconjugate.json"
        ),
        625,
    );
    replay(
        include_str!(
            "../../../docs/function-lane/evidence/w111-broad-20260929/complex/heldout2-imsum.json"
        ),
        625,
    );
    replay(
        include_str!(
            "../../../docs/function-lane/evidence/w111-broad-20260929/complex/heldout2-improduct.json"
        ),
        625,
    );
    // A separate fresh arithmetic-discriminator capture has 40 known midpoint failures.
    // All 944 rows and its complete failure report are retained as scaling-heldout-*.json;
    // those results are explicitly unresolved, not represented as a passing parity test.
}
