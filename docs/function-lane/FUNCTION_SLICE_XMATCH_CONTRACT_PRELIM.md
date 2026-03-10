# Function Slice Contract (Prelim) - XMATCH()

## 1. Slice Identity
1. `function_id`: `FUNC.XMATCH`
2. `display_name`: `XMATCH`
3. `owner_lane`: `OxFunc`
4. `status`: `provisional`

## 2. Signature and Admission Contract
1. arity:
   - minimum: `2`
   - maximum: `4`
2. admission policy:
   - logical signature is `XMATCH(lookup_value, lookup_array, [match_mode], [search_mode])`.
   - `lookup_array` is flattened to a one-dimensional vector after surface preparation.

## 3. Semantic Class Axes
1. `determinism_class`: `deterministic`
2. `volatility_class`: `nonvolatile`
3. `host_interaction_class`: `none`
4. `thread_safety_class`: `safe_pure`
5. `arg_preparation_profile`: `values_only_pre_adapter`
6. `coercion_lift_profile`: `lookup_match_profile`
7. `kernel_signature_class`: `lookup_match`
8. `error_policy_class`: `mixed`
9. `compat_version_policy`: `version_scoped` (draft)
10. `function_adapter_fec_dependency_profile`: `none`
11. `surface_fec_dependency_profile`: `ref_only`
12. `fec_facility_tags`: `cap_reference_resolution`
13. `compile_eval_class`: `runtime_ref_dependent`

## 4. Pre-call Coercion Policy
1. `lookup_array` is flattened from one-dimensional arrays or references; two-dimensional lookup arrays are rejected.
2. omitted `match_mode` defaults to exact and omitted `search_mode` defaults to first-to-last, including fixed-arity surfaces where omission arrives as `MissingArg`.
3. omitted or true-blank `lookup_value` matches true blank cells in exact mode.
4. literal empty string lookup matches formula-empty text cells, not true blank cells.
5. typed comparison is number/text/logical only; mixed-type candidates are non-matches in the current baseline.

## 5. Core Outcome Model
1. `match_mode` supports exact (`0`), wildcard (`2`), exact-or-next-larger (`1`), and exact-or-next-smaller (`-1`).
2. `search_mode` supports forward (`1`), reverse (`-1`), binary ascending (`2`), and binary descending (`-2`).
3. wildcard matching is case-insensitive and honors `*`, `?`, and `~` escaping.
4. wildcard plus binary search is invalid and returns worksheet `#VALUE!`.
5. binary ascending duplicate selection returns the first duplicate; binary descending duplicate selection returns the last duplicate.
6. selected unsorted binary invalid-result lanes are pinned empirically and reproduced by the runtime.
7. lookup-array candidate errors are skipped as non-candidates; no-match returns worksheet `#N/A`.

## 6. Post-call Adaptation Policy
1. successful evaluation returns a 1-based numeric position as `EvalValue::Number`.
2. worksheet errors already carried by the lookup value propagate through the coercion path.
3. XLL parity claims for `XMATCH` are limited only by ordinary bridge mechanics, not by a known XMATCH-specific semantic gap in the current phase.

## 7. Version Scope (Required Axes)
1. Excel application version/channel scope:
   - bounded local empirical baseline: Excel `16.0 (build 19725)`, channel `http://officecdn.microsoft.com/pr/492350f6-3a01-4f97-b9c0-c7c6ddf67d60`, locale `en-US`.
2. Workbook Compatibility Version scope:
   - bounded dual-run workbook lanes: `default` and `compat_template`.
   - `compat_template` is the `.xls` compatibility template emitted by `tools/xmatch-probe/new-xmatch-compat-template.ps1`.

## 8. Evidence Posture
1. `spec_anchor`:
   - conformance row `FDEF-031` in `EXCEL_FUNCTION_DEFINITION_PRELIM_CONFORMANCE.csv`
   - public reference ids linked there: `XLS-CF-FN-011`, `XLS-CF-FN-007`, `XLS-CF-TV-007`
2. evidence anchors:
   - `W7-STR-BL-20260305`
   - `W6-XMATCH-SEED-20260308`
   - `W6-XMATCH-BL-20260308`
   - `W6-XMATCH-EXP-20260310`
3. policy decision anchors:
   - `docs/function-lane/XMATCH_EXECUTION_RECORD.md`
   - `docs/worksets/W006_XMATCH_DETERMINISTIC_QUIRKS.md`
4. current status rationale:
   - function-phase-complete and locally verified for the current reference Excel baseline,
   - blank-vs-empty lookup behavior, wildcard escaping, binary duplicate selection, binary invalid-result lanes, and reference-fed/spill-fed workbook rows are replayed and test-backed,
   - remaining locale/version expansion is orthogonal validation-phase work rather than a current-phase function-semantic gap.

## 9. W6 Coverage
1. exact, reverse, wildcard, approximate, binary, blank lookup, empty-string lookup, reference-fed, spill-fed, error-skipping, and persistence lanes are replayed in `XMATCH_SCENARIO_MANIFEST_SEED.csv`.
2. array-constant built-in vs `ox_XMATCH` parity is replayed in `LOOKUP_XLL_BRIDGE_SCENARIO_MANIFEST_SEED.csv`.

## 10. Artifact Bindings
1. Rust: `crates/oxfunc_core/src/functions/xmatch.rs`
2. Lean: `formal/lean/OxFunc/Functions/Xmatch.lean`
3. surface Rust: `crates/oxfunc_core/src/functions/xmatch_surface.rs`
4. surface Lean: `formal/lean/OxFunc/Functions/XmatchSurface.lean`
