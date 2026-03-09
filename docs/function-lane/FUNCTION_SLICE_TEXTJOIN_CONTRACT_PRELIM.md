# Function Slice Contract (Prelim) - TEXTJOIN()

## 1. Slice Identity
1. `function_id`: `FUNC.TEXTJOIN`
2. `display_name`: `TEXTJOIN`
3. `owner_lane`: `OxFunc`
4. `status`: `provisional`

## 2. Signature and Admission Contract
1. arity:
   - minimum: `3`
   - maximum: `255`
2. admission policy:
   - admitted in this slice as `TEXTJOIN(delimiter, ignore_empty, text1, ...)`.

## 3. Semantic Class Axes
1. `determinism_class`: `deterministic`
2. `volatility_class`: `nonvolatile`
3. `host_interaction_class`: `none`
4. `thread_safety_class`: `safe_pure`
5. `arg_preparation_profile`: `values_only_pre_adapter`
6. `coercion_lift_profile`: `custom`
7. `kernel_signature_class`: `text_to_text`
8. `function_adapter_fec_dependency_profile`: `none`
9. `surface_fec_dependency_profile`: `ref_only`
10. `fec_facility_tags`: `cap_reference_resolution`

## 4. Pre-call Coercion Policy
1. surface preparation resolves references before adapter entry.
2. delimiter and payload values use the shared prepared-text coercion path.
3. `ignore_empty` is parsed numerically and treated as true when non-zero.

## 5. Core Outcome Model
1. admitted call concatenates payload arguments with the admitted delimiter.
2. empty and missing payload lanes are skipped when `ignore_empty` is true.
3. current seed is scalar/value-only; array flattening is not claimed.

## 6. Post-call Adaptation Policy
1. successful evaluation returns a scalar text `EvalValue`.
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
   - `docs/function-lane/W12_EXECUTION_RECORD.md`

## 9. W12 Seed Coverage
1. scalar delimiter and payload joining is implemented.
2. ignore-empty behavior is exercised for empty cells and empty strings.
3. array flattening and richer formatting/text coercion remain explicit target follow-up.

## 10. Artifact Bindings
1. Rust: `crates/oxfunc_core/src/functions/textjoin.rs`
2. Lean: `formal/lean/OxFunc/Functions/TextJoin.lean`
3. side-note linkage: `docs/function-lane/W12_PROFILE_SYSTEM_SIDE_NOTES.md` (note 5)
