# Function Slice Contract (Prelim) - COUNT()

## 1. Slice Identity
1. `function_id`: `FUNC.COUNT`
2. `display_name`: `COUNT`
3. `owner_lane`: `OxFunc`
4. `status`: `provisional`

## 2. Signature and Admission Contract
1. arity:
   - minimum: `1`
   - maximum: `255`
2. admission policy:
   - admitted in this slice as a direct-argument counting fold over prepared values.

## 3. Semantic Class Axes
1. `determinism_class`: `deterministic`
2. `volatility_class`: `nonvolatile`
3. `host_interaction_class`: `none`
4. `thread_safety_class`: `safe_pure`
5. `arg_preparation_profile`: `values_only_pre_adapter`
6. `coercion_lift_profile`: `aggregate_direct_and_range_dual_policy` (declared target profile)
7. `kernel_signature_class`: `nums_to_num`
8. `function_adapter_fec_dependency_profile`: `none`
9. `surface_fec_dependency_profile`: `ref_only`
10. `fec_facility_tags`: `cap_reference_resolution`

## 4. Pre-call Coercion Policy
1. surface preparation resolves references before adapter entry.
2. prepared values are tested through the shared numeric coercion path.
3. missing args, empty cells, and non-numeric text are non-counting in the current seed; worksheet errors still propagate.

## 5. Core Outcome Model
1. admitted call returns the count of prepared arguments that coerce numerically.
2. numeric text and logicals count when numeric coercion succeeds.
3. worksheet errors and non-ignorable preparation failures terminate the fold with error.

## 6. Post-call Adaptation Policy
1. successful evaluation returns a scalar numeric `EvalValue`.
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
   - `docs/function-lane/W12_PROFILE_SYSTEM_SIDE_NOTES.md` (notes 1 and 2)
   - `docs/function-lane/W12_EXECUTION_RECORD.md`

## 9. W12 Seed Coverage
1. counts direct numeric, logical, and numeric-text arguments.
2. ignores array-like text/logical lanes and propagates worksheet errors from admitted inputs.
3. current empirical baseline does not require any finer source-class split than direct-scalar versus array-like input, so this slice is `function-phase-complete` for the current reference baseline.

## 10. Artifact Bindings
1. Rust: `crates/oxfunc_core/src/functions/count.rs`
2. Lean: `formal/lean/OxFunc/Functions/Count.lean`
3. side-note linkage: `docs/function-lane/W12_PROFILE_SYSTEM_SIDE_NOTES.md` (notes 1 and 2)
