//! W105 oxf-y2uw.3 — routing-precedence regression guard for the unary-numeric pilot.
//!
//! The pilot deleted `eval_shared_unary_numeric_calc_dispatch`, moving every id it served
//! onto the single by-index-table -> `unary_numeric::execute(spec, …)` executor path. The
//! cross-surface `unary_numeric_equivalence_law` harness proves bit-exact behaviour for the
//! 33-member Q-eligible family; this test closes the remaining routing risk the deletion
//! introduced:
//!
//!   * **No silent `#VALUE!` fallthrough.** Every id formerly intercepted by the deleted
//!     calc-dispatch (38 of them — including the seven that are NOT in the equivalence
//!     family because they are not XLL-`Q`-eligible: ACOS, ACOSH, ACOTH, FISHER, FISHERINV,
//!     GAUSS, PHI) must still resolve through the real `eval_surface_value_call` surface to a
//!     genuine numeric / domain-error result, never the `#VALUE!` that an unrouted id would
//!     produce by falling off the end of dispatch.
//!
//! This is a behaviour-preservation guard, not a behaviour assertion: it pins that the
//! collapse kept every id live on exactly one path.

use oxfunc_core::functions::surface_dispatch::{self as sd, eval_surface_value_call};
use oxfunc_core::resolver::{
    ReferenceDereferenceRequest, ReferenceResolutionError, ReferenceSystemCapabilities,
    ReferenceSystemProvider,
};
use oxfunc_core::value::{CalcValue, CoreValue, WorksheetErrorCode};

struct NoResolver;

impl ReferenceSystemProvider for NoResolver {
    fn capabilities(&self) -> ReferenceSystemCapabilities {
        ReferenceSystemCapabilities::permissive_local()
    }

    fn dereference(
        &self,
        request: &ReferenceDereferenceRequest,
    ) -> Result<CalcValue, ReferenceResolutionError> {
        Err(ReferenceResolutionError::UnresolvedReference {
            target: request.reference.target().to_string(),
        })
    }
}

/// Every id formerly served by `eval_shared_unary_numeric_calc_dispatch`, paired with an
/// in-domain argument whose evaluation must succeed (a finite number). If an id had silently
/// fallen off dispatch onto the `#VALUE!` arm, this would catch it.
fn former_calc_dispatch_ids() -> Vec<(&'static str, f64)> {
    vec![
        (sd::FUNC_ID_ABS, -2.0),
        (sd::FUNC_ID_ACOS, 0.5),
        (sd::FUNC_ID_ACOT, 0.5),
        (sd::FUNC_ID_ACOSH, 2.0),
        (sd::FUNC_ID_ACOTH, 2.0),
        (sd::FUNC_ID_ASINH, 0.5),
        (sd::FUNC_ID_ATAN, 0.5),
        (sd::FUNC_ID_ATANH, 0.5),
        (sd::FUNC_ID_COS, 0.5),
        (sd::FUNC_ID_COSH, 0.5),
        (sd::FUNC_ID_COT, 0.5),
        (sd::FUNC_ID_COTH, 2.0),
        (sd::FUNC_ID_CSC, 0.5),
        (sd::FUNC_ID_CSCH, 0.5),
        (sd::FUNC_ID_DEGREES, 1.0),
        (sd::FUNC_ID_EVEN, 1.5),
        (sd::FUNC_ID_EXP, 1.0),
        (sd::FUNC_ID_FACT, 5.0),
        (sd::FUNC_ID_FACTDOUBLE, 5.0),
        (sd::FUNC_ID_FISHER, 0.5),
        (sd::FUNC_ID_FISHERINV, 0.5),
        (sd::FUNC_ID_GAUSS, 1.0),
        (sd::FUNC_ID_INT, 1.7),
        (sd::FUNC_ID_LN, 2.0),
        (sd::FUNC_ID_LOG10, 100.0),
        (sd::FUNC_ID_ODD, 1.5),
        (sd::FUNC_ID_OP_NEGATE, 3.0),
        (sd::FUNC_ID_OP_PERCENT, 50.0),
        (sd::FUNC_ID_PHI, 0.0),
        (sd::FUNC_ID_RADIANS, 180.0),
        (sd::FUNC_ID_SEC, 0.5),
        (sd::FUNC_ID_SECH, 0.5),
        (sd::FUNC_ID_SIGN, -3.0),
        (sd::FUNC_ID_SINH, 0.5),
        (sd::FUNC_ID_SQRT, 4.0),
        (sd::FUNC_ID_SQRTPI, 4.0),
        (sd::FUNC_ID_TAN, 0.5),
        (sd::FUNC_ID_TANH, 0.5),
    ]
}

#[test]
fn former_calc_dispatch_ids_all_route_to_a_real_result() {
    let resolver = NoResolver;
    let ids = former_calc_dispatch_ids();
    // Guard against the family list silently shrinking below the documented 38.
    assert_eq!(
        ids.len(),
        38,
        "expected the 38 former calc-dispatch unary-numeric ids"
    );

    let mut failures: Vec<String> = Vec::new();
    for (function_id, arg) in ids {
        let result = eval_surface_value_call(
            function_id,
            &[CalcValue::number(arg)],
            &resolver,
            None,
            None,
            None,
            None,
        );
        match result {
            Ok(value) => match value.core() {
                CoreValue::Number(n) if n.is_finite() => {}
                CoreValue::Number(n) => failures.push(format!(
                    "{function_id} @ {arg}: resolved to non-finite number {n:?}"
                )),
                CoreValue::Error(code) => failures.push(format!(
                    "{function_id} @ {arg}: resolved to error {code:?} (expected a finite number \
                     for this in-domain input)"
                )),
                other => failures.push(format!(
                    "{function_id} @ {arg}: resolved to non-numeric {other:?}"
                )),
            },
            // A top-level `#VALUE!` for an in-domain numeric argument is exactly the
            // fall-off-dispatch signature this guard exists to catch.
            Err(WorksheetErrorCode::Value) => failures.push(format!(
                "{function_id} @ {arg}: top-level #VALUE! — likely fell through dispatch (no \
                 route after the calc-dispatch deletion)"
            )),
            Err(code) => failures.push(format!(
                "{function_id} @ {arg}: unexpected top-level error {code:?}"
            )),
        }
    }

    assert!(
        failures.is_empty(),
        "{} former calc-dispatch id(s) did not route to a real result:\n{}",
        failures.len(),
        failures.join("\n")
    );
}
