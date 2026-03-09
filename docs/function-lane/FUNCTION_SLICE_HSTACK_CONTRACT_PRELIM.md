# Function Slice Contract (Prelim) - HSTACK()

## 1. Slice Identity
1. `function_id`: `FUNC.HSTACK`
2. `display_name`: `HSTACK`
3. `owner_lane`: `OxFunc`
4. `status`: `provisional`

## 2. Signature and Admission Contract
1. arity:
   - minimum: `1`
   - maximum: `255`
2. admission policy:
   - admitted in this slice as a variadic horizontal array-assembly function.

## 3. Semantic Class Axes
1. `determinism_class`: `deterministic`
2. `volatility_class`: `nonvolatile`
3. `host_interaction_class`: `none`
4. `thread_safety_class`: `safe_pure`
5. `arg_preparation_profile`: `values_only_pre_adapter`
6. `coercion_lift_profile`: `custom`
7. `kernel_signature_class`: `custom`
8. `function_adapter_fec_dependency_profile`: `none`
9. `surface_fec_dependency_profile`: `ref_only`
10. `fec_facility_tags`: `cap_reference_resolution`

## 4. Pre-call Coercion Policy
1. surface preparation resolves references before adapter entry.
2. prepared array-shape payloads are consumed structurally rather than elementwise.
3. scalar, blank, and missing arguments contribute `1x1` shape units in the current seed.

## 5. Core Outcome Model
1. admitted call returns an array-shape payload whose row count is the maximum of input row counts and whose column count is the sum of input column counts.
2. current seed is shape-only and does not materialize payload cells.
3. preparation failures terminate evaluation with error.

## 6. Post-call Adaptation Policy
1. successful evaluation returns an array-shape `EvalValue`.
2. preparation and arity failures map to worksheet-visible `#VALUE!` unless a worksheet error code is already carried through preparation.

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
   - `docs/function-lane/W12_PROFILE_SYSTEM_SIDE_NOTES.md` (note 8)
   - `docs/function-lane/W12_EXECUTION_RECORD.md`

## 9. W12 Seed Coverage
1. shape-only horizontal composition is implemented over scalar and array-shape inputs.
2. scalar, blank, and missing arguments currently contribute `1x1` shape units.
3. full payload fill, padding rules, and richer dynamic-array materialization remain explicit target bounds.

## 10. Artifact Bindings
1. Rust: `crates/oxfunc_core/src/functions/hstack.rs`
2. Lean: `formal/lean/OxFunc/Functions/Hstack.lean`
3. side-note linkage: `docs/function-lane/W12_PROFILE_SYSTEM_SIDE_NOTES.md` (note 8)
