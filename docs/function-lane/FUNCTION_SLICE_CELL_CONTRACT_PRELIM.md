# Function Slice Contract (Prelim) - CELL()

## 1. Slice Identity
1. `function_id`: `FUNC.CELL`
2. `display_name`: `CELL`
3. `owner_lane`: `OxFunc`
4. `status`: `provisional`

## 2. Signature and Admission Contract
1. arity:
   - minimum: `2`
   - maximum: `2`
2. admission policy:
   - admitted in this slice only for the bounded two-argument form `CELL(info_type, reference)`.

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
10. `fec_facility_tags`: `cap_caller_context`; `cap_reference_resolution`

## 4. Pre-call Coercion Policy
1. `info_type` is prepared through the shared text coercion path and normalized case-insensitively.
2. the reference argument must remain reference-bearing at adapter entry.
3. only bounded A1 reference text is admitted in the current seed.

## 5. Core Outcome Model
1. admitted `address`, `row`, and `col` info types are computed from the parsed reference target.
2. admitted `contents` and `type` info types resolve the reference through the resolver seam.
3. unsupported info types, missing reference identity, and invalid reference text return bounded error outcomes.

## 6. Post-call Adaptation Policy
1. successful evaluation returns either scalar text or scalar numeric `EvalValue` depending on the selected info type.
2. invalid reference text and unresolved reference failures map to worksheet-visible `#REF!`; coercion, unsupported info types, and arity failures map to `#VALUE!` unless a worksheet error code is already carried through coercion.

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
   - `W12-CELL-PRE-20260309`
   - `W12-MODERATE-BL-20260309`
3. policy decision anchors:
   - `docs/function-lane/W12_PROFILE_SYSTEM_SIDE_NOTES.md` (notes 7 and 8)
   - `docs/function-lane/W12_EXECUTION_RECORD.md`

## 9. W12 Seed Coverage
1. required pre-implementation probe selected bounded info-type support: `address`, `row`, `col`, `contents`, `type`.
2. preserved-reference argument handling is implemented for bounded A1 targets.
3. one-argument caller-context, filename, format, protection metadata, and broader host-sensitive lanes remain explicit target bounds.

## 10. Artifact Bindings
1. Rust: `crates/oxfunc_core/src/functions/cell.rs`
2. Lean: `formal/lean/OxFunc/Functions/Cell.lean`
3. side-note linkage: `docs/function-lane/W12_PROFILE_SYSTEM_SIDE_NOTES.md` (notes 7 and 8)
