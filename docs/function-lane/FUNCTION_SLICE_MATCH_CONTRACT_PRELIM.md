# Function Slice Contract (Prelim) - MATCH()

## 1. Slice Identity
1. `function_id`: `FUNC.MATCH`
2. `display_name`: `MATCH`
3. `owner_lane`: `OxFunc`
4. `status`: `provisional`

## 2. Signature and Admission Contract
1. arity:
   - minimum: `2`
   - maximum: `3`
2. admission policy:
   - admitted for `MATCH(lookup_value, lookup_array[, match_type])`.
   - `lookup_array` is flattened to a one-dimensional vector after surface preparation.

## 3. Semantic Class Axes
1. `determinism_class`: `deterministic`
2. `volatility_class`: `nonvolatile`
3. `host_interaction_class`: `none`
4. `thread_safety_class`: `safe_pure`
5. `arg_preparation_profile`: `refs_visible_in_adapter`
6. `coercion_lift_profile`: `lookup_match_profile`
7. `kernel_signature_class`: `lookup_match`
8. `function_adapter_fec_dependency_profile`: `ref_only`
9. `surface_fec_dependency_profile`: `ref_only`

## 4. Pre-call Coercion Policy
1. references remain visible to the adapter so `lookup_value`, `lookup_array`, and `match_type` can be prepared with lookup-family policy rather than eager scalar coercion.
2. omitted `match_type` defaults to `1`, including fixed-arity surfaces where omission arrives as `MissingArg`.
3. exact `match_type = 0` uses wildcard semantics when the text lookup value carries Excel wildcard syntax, including escaped literal patterns such as `a~*`.
4. blank lookup values are not coerced to empty string for `MATCH`; true blank or omitted lookup values in exact mode produce the no-match lane.

## 5. Core Outcome Model
1. `match_type = 0` returns the first exact or wildcard match from left to right.
2. `match_type = 1` returns the empirically observed approximate-next-smaller result, including Excel's invalid-result behavior on unsorted arrays.
3. `match_type = -1` returns the empirically observed descending exact-or-next-larger result, including Excel's invalid-result behavior on unsorted arrays.
4. ascending duplicate selection for `match_type = 1` returns the last duplicate.
5. descending duplicate selection for `match_type = -1` returns the first duplicate.
6. two-dimensional lookup arrays are rejected with worksheet-visible `#VALUE!`.

## 6. Post-call Adaptation Policy
1. successful evaluation returns a 1-based numeric position as `EvalValue::Number`.
2. no-match returns worksheet `#N/A`.
3. worksheet errors already carried by the lookup value propagate through the coercion path.
4. XLL reference-resolution limits are external seam limits and do not weaken the core function claim.

## 7. Version Scope (Required Axes)
1. Excel application version/channel scope:
   - bounded local empirical baseline: Excel `16.0 (build 19725)`, channel `http://officecdn.microsoft.com/pr/492350f6-3a01-4f97-b9c0-c7c6ddf67d60`, locale `en-US`.
2. Workbook Compatibility Version scope:
   - bounded dual-run workbook lanes: `default` and `compat_template`.
   - `compat_template` is the `.xls` compatibility template emitted by `tools/w10-probe/new-w10-compat-template.ps1`.

## 8. Evidence Posture
1. `spec_anchor`:
   - packet conformance row `FDEF-035` in `EXCEL_FUNCTION_DEFINITION_PRELIM_CONFORMANCE.csv`
   - public reference ids linked there: `XLS-CF-FN-009`, `XLS-CF-FN-007`, `XLS-CF-TV-007`
2. `empirical_anchor`:
   - `W10-TENMIX-SEED-20260308`
   - `W10-LOOKUP-XLL-20260310`
3. policy decision anchors:
   - `docs/function-lane/W10_PROFILE_SYSTEM_SIDE_NOTES.md` (note 6)
   - `docs/function-lane/W10_EXECUTION_RECORD.md`
   - `docs/function-lane/XLL_VERIFICATION_SEAM_LIMITATIONS.md`
4. current status rationale:
   - function-phase-complete and locally verified for the current reference Excel baseline,
   - approximate duplicate-selection and unsorted invalid-result lanes are now pinned in both Rust tests and dual-run workbook replay,
   - remaining locale/version expansion is orthogonal validation-phase work rather than a current-phase function-semantic gap.

## 9. W10 Coverage
1. exact matching, wildcard escaping, approximate ascending, approximate descending, duplicate selection, blank lookup, and selected unsorted invalid-result lanes are replayed in `W10_S2_SCENARIO_MANIFEST_SEED.csv`.
2. array-constant built-in vs `ox_MATCH` parity is replayed in `LOOKUP_XLL_BRIDGE_SCENARIO_MANIFEST_SEED.csv`.

## 10. Artifact Bindings
1. Rust: `crates/oxfunc_core/src/functions/match_fn.rs`
2. Lean: `formal/lean/OxFunc/Functions/MatchFn.lean`
3. side-note linkage: `docs/function-lane/W10_PROFILE_SYSTEM_SIDE_NOTES.md` (note 6)
