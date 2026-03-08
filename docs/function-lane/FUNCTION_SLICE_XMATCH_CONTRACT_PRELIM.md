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
   - seed runtime currently models `lookup_array` as an explicit prepared list input lane.
3. seed-scope note:
   - this slice is an exploration scaffold, not full Excel-complete closure.
   - non-standard coercion/error handling is intentionally function-local in XMATCH for now (no generalized cross-function abstraction yet).

## 3. Semantic Class Axes
1. `determinism_class`: `deterministic` (seed assumption)
2. `volatility_class`: `nonvolatile` (seed assumption)
3. `host_interaction_class`: `none`
4. `thread_safety_class`: `safe_pure` (seed scope)
5. `arg_preparation_profile`: `values_only_pre_adapter`
6. `coercion_lift_profile`: `lookup_match_profile`
7. `kernel_signature_class`: `lookup_match`
8. `error_policy_class`: `mixed` (seed scope)
9. `compat_version_policy`: `version_scoped` (draft)
10. `function_adapter_fec_dependency_profile`: `none`
11. `surface_fec_dependency_profile`: `ref_only`
12. `fec_facility_tags`: `cap_reference_resolution`
13. `compile_eval_class`: `runtime_ref_dependent`

## 4. Seed Behavior Lanes (Declared Scope)
1. defaults lane:
   - `match_mode` default = `0` (exact),
   - `search_mode` default = `1` (first-to-last).
2. exact-match lane:
   - seed supports exact matching only.
3. search-order lane:
   - seed supports forward (`1`) and reverse (`-1`) scan.
4. mode-validation lane:
   - invalid modes error,
   - binary/wildcard modes are explicit unsupported-seed outcomes.
5. comparison lane (seed):
   - typed exact comparison for number/text/logical.
   - mixed-type lookup is non-match in seed scope.
6. boundary lane:
   - surface module resolves references pre-adapter (`values_only_pre_adapter`).
   - future selective-probe optimization path is deferred and would require function-controlled reference visibility (`refs_visible_in_adapter`) plus subset dereference support.
7. error lane:
   - lookup value and mode coercion errors propagate.
   - lookup-array candidate errors are skipped in exact scan (non-match behavior).
   - no-match returns explicit not-available lane.

## 5. Seed Out-of-Scope (Explicit)
1. wildcard semantics (`match_mode=2`) are not yet implemented.
2. binary search modes (`search_mode=2|-2`) are not yet implemented.
3. approximate-match semantics (`match_mode=1|-1`) are not yet implemented.
4. full Excel coercion parity for cross-type comparisons remains open.
5. full spill/range-shape semantics remain open.

## 6. Version Scope (Required Axes)
1. Excel application version/channel scope:
   - `unbounded_provisional` pending empirical matrix.
2. Workbook Compatibility Version scope:
   - `unbounded_provisional` pending compatibility replay.

## 7. Proof/Implementation Obligations
1. Lean obligations:
   - adapter/kernel module: `formal/lean/OxFunc/Functions/Xmatch.lean`
   - surface module: `formal/lean/OxFunc/Functions/XmatchSurface.lean`
   - seed profile theorem (`xmatchMeta_seed_profiles`)
   - seed defaults/order/not-found theorems
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
   - seed scaffold is implemented for declared scope,
   - empirical replay baseline is captured for declared lanes,
   - adapter-level wildcard/binary/approximate parity remains explicit follow-on.
