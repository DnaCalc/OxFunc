# Function Slice Contract (Prelim) - OFFSET()

## 1. Slice Identity
1. `function_id`: `FUNC.OFFSET`
2. `display_name`: `OFFSET`
3. `owner_lane`: `OxFunc`
4. `status`: `provisional`

## 2. Signature and Admission Contract
1. arity:
   - minimum: `3`
   - maximum: `5`
2. admission policy:
   - admitted in this slice only for `OFFSET(reference, rows, cols[, height[, width]])`.

## 3. Semantic Class Axes
1. `determinism_class`: `deterministic`
2. `volatility_class`: `volatile_contextual`
3. `host_interaction_class`: `workbook_state`
4. `thread_safety_class`: `host_serialized`
5. `arg_preparation_profile`: `refs_visible_in_adapter`
6. `coercion_lift_profile`: `custom`
7. `kernel_signature_class`: `custom`
8. `function_adapter_fec_dependency_profile`: `caller_context`
9. `surface_fec_dependency_profile`: `caller_context`
10. `fec_facility_tags`: `cap_caller_context`

## 4. Pre-call Coercion Policy
1. the first argument must remain reference-bearing at adapter entry.
2. row, column, height, and width arguments are coerced numerically inside the adapter and truncated toward zero.
3. optional height and width must be positive when supplied.

## 5. Core Outcome Model
1. admitted call parses the base A1 reference and applies the requested row and column offsets.
2. height and width resize the returned reference area when present.
3. invalid reference text, missing reference identity, or out-of-bounds dimensions return bounded error outcomes.

## 6. Post-call Adaptation Policy
1. successful evaluation returns a reference `EvalValue` with `A1` or `Area` shape depending on the resulting extent.
2. invalid reference text and invalid dimensions map to worksheet-visible `#REF!`; coercion and arity failures map to `#VALUE!` unless a worksheet error code is already carried through coercion.

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
   - `docs/function-lane/W12_PROFILE_SYSTEM_SIDE_NOTES.md` (notes 3 and 7)
   - `docs/function-lane/W12_EXECUTION_RECORD.md`

## 9. W12 Seed Coverage
1. preserved-reference argument handling is implemented for bounded A1 targets.
2. reference shifting and explicit resize arguments are exercised.
3. broader caller-context, non-A1, and full macro-sensitive reference-return lanes remain explicit target bounds.

## 10. Artifact Bindings
1. Rust: `crates/oxfunc_core/src/functions/offset.rs`
2. Lean: `formal/lean/OxFunc/Functions/Offset.lean`
3. side-note linkage: `docs/function-lane/W12_PROFILE_SYSTEM_SIDE_NOTES.md` (notes 3 and 7)
