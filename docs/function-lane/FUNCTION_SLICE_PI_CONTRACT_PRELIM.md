# Function Slice Contract (Prelim) - PI()

## 1. Slice Identity
1. `function_id`: `FUNC.PI`
2. `display_name`: `PI`
3. `owner_lane`: `OxFunc`
4. `status`: `provisional`

## 2. Signature and Admission Contract
1. arity:
   - minimum: `0`
   - maximum: `0`
2. admission policy:
   - formula/evaluator contract admits only zero-argument form (`PI()`).
   - nonzero argument counts are admission failures in this slice model.

## 3. Semantic Class Axes
1. `determinism_class`: `deterministic`
2. `volatility_class`: `nonvolatile`
3. `host_interaction_class`: `none`
4. `thread_safety_class`: `safe_pure` (no host mutation/shared-state dependency in slice scope)
5. `arg_preparation_profile`: `values_only_pre_adapter`
6. `coercion_lift_profile`: `none`
7. `kernel_signature_class`: `nullary_const`
8. `coercion_policy_class`: `strict`
9. `error_policy_class`: `strict_propagate`
10. `compat_version_policy`: `stable_across_versions` (provisional)
11. `function_adapter_fec_dependency_profile`: `none`
12. `surface_fec_dependency_profile`: `none`
13. `fec_facility_tags`: none
14. `compile_eval_class`: `const_foldable_when_closed`

## 4. Trigger Classes
1. `T-DEP`: no direct dependency inputs.
2. `T-VOL`: none.
3. `T-HOST`: none.
4. `T-EXT`: none.
5. `T-VERSION`: possible only as version-scope divergence marker.

## 5. Core Outcome Model
1. admitted call (`PI()`):
   - returns numeric constant payload representing pi.
2. non-admitted call (`PI(...)` with args):
   - returns admission failure in current runtime adapter model.
3. no runtime coercion path:
   - because no arguments exist in admitted shape.

## 6. Version Scope (Required Axes)
1. Excel application version/channel scope:
   - `unbounded_provisional` until empirical matrix is attached.
2. Workbook Compatibility Version scope:
   - `unbounded_provisional` until empirical matrix is attached.

## 7. Proof/Implementation Obligations
1. Lean obligations:
   - admitted result theorem for zero-arg case.
   - nonzero-arity rejection theorem.
   - determinism theorem.
   - layered profile declaration theorem (`piMeta_layered_profiles`).
2. Rust obligations:
   - deterministic constant output.
   - explicit arity rejection behavior.
   - unit tests mapped in correlation ledger.

## 8. Evidence Posture
1. `spec_anchor`: to be attached from public references.
2. `empirical_anchor`: required for validated status.
3. current status rationale:
   - function-phase-complete and locally verified for the current reference Excel baseline,
   - Excel admission-boundary baseline captured (`2026-03-05`, Excel `16.0 (build 19725)`) in `FORMULA_ADMISSION_BEHAVIOR_NOTES.md`,
   - cross-build/channel, compatibility-variant, locale, and direct C API equivalence replay remain orthogonal validation-phase follow-up rather than current-phase function-semantic gaps.
