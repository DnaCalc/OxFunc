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
   - current runtime models `lookup_array` as a flattened one-dimensional lookup vector after surface preparation.
3. scope note:
   - this slice remains an exploration scaffold, not full Excel-complete closure.
   - non-standard coercion/error handling is intentionally function-local in XMATCH for now (no generalized cross-function abstraction yet).

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

## 4. Current Behavior Lanes
1. defaults lane:
   - `match_mode` default = `0` (exact),
   - `search_mode` default = `1` (first-to-last).
2. match-mode lane:
   - exact (`0`),
   - wildcard (`2`),
   - exact-or-next-larger (`1`),
   - exact-or-next-smaller (`-1`).
3. search-order lane:
   - forward (`1`),
   - reverse (`-1`),
   - binary ascending (`2`),
   - binary descending (`-2`).
4. mode-validation lane:
   - invalid modes error.
5. comparison lane:
   - typed comparisons for number/text/logical.
   - text comparison is currently case-insensitive in the runtime.
   - wildcard text matching honors `*`, `?`, and `~` escaping.
   - mixed-type lookup is non-match in the current runtime.
6. boundary lane:
   - surface module resolves references pre-adapter (`values_only_pre_adapter`).
   - flattened lookup vectors admit one-dimensional arrays and reject two-dimensional lookup arrays.
7. error lane:
   - lookup value and mode coercion errors propagate.
   - lookup-array candidate errors are skipped as non-candidates in the current runtime.
   - no-match returns explicit not-available lane.

## 5. Remaining Open Lanes
1. full Excel coercion parity for cross-type, blank, and richer collation behavior remains open.
2. full spill/range-shape semantics remain open.
3. binary-mode behavior is aligned for current observed lanes but still needs broader empirical edge-case saturation.

## 6. Version Scope (Required Axes)
1. Excel application version/channel scope:
   - `unbounded_provisional` pending empirical matrix.
2. Workbook Compatibility Version scope:
   - `unbounded_provisional` pending compatibility replay.

## 7. Proof/Implementation Obligations
1. Lean obligations:
   - adapter/kernel module: `formal/lean/OxFunc/Functions/Xmatch.lean`
   - surface module: `formal/lean/OxFunc/Functions/XmatchSurface.lean`
   - profile theorem (`xmatchMeta_seed_profiles`)
   - defaults/order/not-found theorems
   - surface-prepared-to-adapter correspondence theorem
2. Rust obligations:
   - adapter/kernel module: `crates/oxfunc_core/src/functions/xmatch.rs`
   - surface module: `crates/oxfunc_core/src/functions/xmatch_surface.rs`
   - mode/default/search behavior tests
   - surface reference-preparation and adapter-correspondence tests

## 8. Evidence Posture
1. `spec_anchor`:
   - `TBD-SPEC-XMATCH-001`
2. evidence anchors:
   - `W7-STR-BL-20260305` (string behavior feed for comparison lane)
   - `W6-XMATCH-SEED-20260308` (local scaffold closure: docs + Lean + Rust + tests)
   - `W6-XMATCH-BL-20260308` (empirical dual-run replay baseline)
   - empirical-run scaffold:
     - `docs/function-lane/XMATCH_SCENARIO_MANIFEST_SEED.csv`
     - `docs/function-lane/XMATCH_PROBE_RUNTIME_REQUIREMENTS.md`
     - `tools/xmatch-probe/*`
3. status rationale:
   - runtime coverage now includes wildcard, binary, and approximate lanes observed in replay,
   - empirical replay baseline is captured for declared lanes,
   - broader comparison/collation/shape parity remains explicit follow-on.
