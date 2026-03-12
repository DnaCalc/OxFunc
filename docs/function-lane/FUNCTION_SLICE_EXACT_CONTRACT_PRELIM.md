# Function Slice Contract (Prelim) - EXACT()

## 1. Slice Identity
1. `function_id`: `FUNC.EXACT`
2. `display_name`: `EXACT`
3. `owner_lane`: `OxFunc`
4. `status`: `provisional`

## 2. Signature and Admission Contract
1. arity:
   - minimum: `2`
   - maximum: `2`
2. admission policy:
   - admitted only for the binary shape `EXACT(text1, text2)`.

## 3. Semantic Class Axes
1. `determinism_class`: `deterministic`
2. `volatility_class`: `nonvolatile`
3. `host_interaction_class`: `none`
4. `thread_safety_class`: `safe_pure`
5. `arg_preparation_profile`: `values_only_pre_adapter`
6. `coercion_lift_profile`: `none`
7. `kernel_signature_class`: `custom`
8. `function_adapter_fec_dependency_profile`: `none`
9. `surface_fec_dependency_profile`: `ref_only`
10. `fec_facility_tags`: `cap_reference_resolution`

## 4. Pre-call Coercion Policy
1. surface preparation resolves references before adapter entry.
2. both arguments are coerced through the shared prepared-text path.
3. the current seed compares UTF-16 payloads exactly after scalar-to-text coercion.

## 5. Core Outcome Model
1. admitted call returns `TRUE` when both coerced text payloads match exactly.
2. comparison is case-sensitive and code-unit-sensitive in the current slice.
3. blank cells coerce to empty text before comparison in the current empirical baseline.
4. the current empirical baseline distinguishes precomposed and combining Unicode forms rather than normalizing them.
3. coercion failures terminate evaluation with error.

## 6. Post-call Adaptation Policy
1. successful evaluation returns a scalar logical `EvalValue`.
2. adapter errors map to worksheet-visible `#VALUE!` unless a worksheet error code is already carried through coercion.

## 7. Version Scope (Required Axes)
1. Excel application version/channel scope:
   - bounded local empirical baseline: Excel `16.0 (build 19725)`, channel `http://officecdn.microsoft.com/pr/492350f6-3a01-4f97-b9c0-c7c6ddf67d60`, locale `en-US`.
2. Workbook Compatibility Version scope:
   - bounded dual-run workbook lanes: `default` and `compat_template`.
   - `compat_template` is the `.xls` compatibility template emitted by `tools/w12-probe/new-w12-compat-template.ps1`.

## 8. Evidence Posture
1. `spec_anchor`:
   - packet conformance row `FDEF-037` in `EXCEL_FUNCTION_DEFINITION_PRELIM_CONFORMANCE.csv`
   - public reference ids linked there: `XLS-CF-FN-001`, `XLS-CF-FN-002`, `XLS-CF-FN-007`, `XLS-CF-TV-007`, `XLS-CF-TV-008`
2. `empirical_anchor`:
   - `W12-MODERATE-BL-20260309`
3. policy decision anchors:
   - `docs/function-lane/W12_PROFILE_SYSTEM_SIDE_NOTES.md` (note 5)
   - `docs/function-lane/TEXT_FUNCTION_EMPIRICAL_EXPANSION_NOTES.md`
   - `docs/function-lane/W12_EXECUTION_RECORD.md`
4. current status rationale:
   - function-phase-complete for the current reference Excel baseline,
   - case-sensitivity, numeric/logical textification, blank-as-empty, precomposed-versus-combining distinction, and identical surrogate-pair equality are now explicitly evidenced,
   - remaining locale/version expansion is orthogonal validation work rather than a known current-phase semantic gap.

## 9. W12 Seed Coverage
1. case-sensitive exact comparison is implemented.
2. scalar-to-text coercion is exercised for numeric and logical payloads.
3. blank-versus-empty behavior is exercised through dereferenced blank-cell replay.
4. Unicode comparison is explicitly pinned as code-unit-sensitive rather than normalization-aware in the current baseline.

## 10. Artifact Bindings
1. Rust: `crates/oxfunc_core/src/functions/exact_fn.rs`
2. Lean: `formal/lean/OxFunc/Functions/Exact.lean`
3. side-note linkage: `docs/function-lane/W12_PROFILE_SYSTEM_SIDE_NOTES.md` (note 5)
