//! W105 oxf-y2uw.12.1 — routing-precedence regression guard for the binary-arithmetic family.
//!
//! This bead deleted `eval_binary_arithmetic_calc_dispatch`, moving every id it served
//! (MOD, OP_ADD, OP_SUBTRACT, OP_MULTIPLY, OP_DIVIDE, OP_POWER / POWER) onto the single
//! by-index-table -> `binary_numeric::execute(spec, …)` 2-arg executor path. Before the
//! deletion these ids were DOUBLE-SERVED: the calc-dispatch interception ran first and won,
//! while a now-removed shadowed by-index hand arm also existed. The spec-driven
//! interpreter-equals + drift-check tests (in the generator module) pin that the generated arm
//! routes each id to the right kernel/policy; this test closes the remaining routing risk the
//! deletion introduced, mirroring the `.3` `unary_numeric_executor_routing` guard:
//!
//!   * **No silent `#VALUE!` fallthrough / no lost route.** Every id formerly intercepted by the
//!     deleted calc-dispatch must still resolve through the real `eval_surface_value_call`
//!     surface to a genuine numeric / domain-error result, never the `#VALUE!` that an unrouted
//!     id would produce by falling off the end of dispatch onto the `_ => Err(#VALUE!)` arm.
//!
//! This is a behaviour-preservation guard, not a behaviour assertion: it pins that the collapse
//! kept every id live on exactly one path.

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

/// Every id formerly served by `eval_binary_arithmetic_calc_dispatch`, paired with an in-domain
/// `(lhs, rhs)` whose evaluation must succeed to a finite number. If an id had silently fallen
/// off dispatch onto the `#VALUE!` arm, this would catch it.
fn former_calc_dispatch_ids() -> Vec<(&'static str, f64, f64)> {
    vec![
        (sd::FUNC_ID_MOD, 7.0, 3.0),
        (sd::FUNC_ID_OP_ADD, 2.0, 3.0),
        (sd::FUNC_ID_OP_SUBTRACT, 5.0, 2.0),
        (sd::FUNC_ID_OP_MULTIPLY, 5.0, 2.0),
        (sd::FUNC_ID_OP_DIVIDE, 5.0, 2.0),
        (sd::FUNC_ID_OP_POWER, 2.0, 3.0),
        (sd::FUNC_ID_POWER, 2.0, 3.0),
    ]
}

#[test]
fn former_calc_dispatch_binary_ids_all_route_to_a_real_result() {
    let resolver = NoResolver;
    let ids = former_calc_dispatch_ids();
    // Guard against the family list silently shrinking below the documented 7.
    assert_eq!(
        ids.len(),
        7,
        "expected the 7 former binary-arithmetic calc-dispatch ids"
    );

    let mut failures: Vec<String> = Vec::new();
    for (function_id, lhs, rhs) in ids {
        let result = eval_surface_value_call(
            function_id,
            &[CalcValue::number(lhs), CalcValue::number(rhs)],
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
                    "{function_id} @ ({lhs},{rhs}): resolved to non-finite number {n:?}"
                )),
                CoreValue::Error(code) => failures.push(format!(
                    "{function_id} @ ({lhs},{rhs}): resolved to error {code:?} (expected a finite \
                     number for this in-domain input)"
                )),
                other => failures.push(format!(
                    "{function_id} @ ({lhs},{rhs}): resolved to non-numeric {other:?}"
                )),
            },
            // A top-level `#VALUE!` for an in-domain numeric pair is exactly the
            // fall-off-dispatch signature this guard exists to catch.
            Err(WorksheetErrorCode::Value) => failures.push(format!(
                "{function_id} @ ({lhs},{rhs}): top-level #VALUE! — likely fell through dispatch \
                 (no route after the calc-dispatch deletion)"
            )),
            Err(code) => failures.push(format!(
                "{function_id} @ ({lhs},{rhs}): unexpected top-level error {code:?}"
            )),
        }
    }

    assert!(
        failures.is_empty(),
        "{} former calc-dispatch binary id(s) did not route to a real result:\n{}",
        failures.len(),
        failures.join("\n")
    );
}
