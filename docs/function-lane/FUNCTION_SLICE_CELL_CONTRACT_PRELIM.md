# Function Slice Contract (Prelim) - CELL()

## 1. Slice Identity
1. `function_id`: `FUNC.CELL`
2. `display_name`: `CELL`
3. `owner_lane`: `OxFunc`
4. `status`: `provisional`

## 2. Signature and Admission Contract
1. arity:
   - minimum: `1`
   - maximum: `2`
2. admission policy:
   - admitted in this slice for `CELL(info_type)` and `CELL(info_type, reference)`.

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
2. when a reference argument is present, it must remain reference-bearing at adapter entry.
3. omitted-reference forms are admitted and delegated through the host-query seam because they depend on the active selected cell rather than only the formula cell.
4. explicit-reference local lanes still assume bounded A1 reference text in the current implementation surface.

## 5. Core Outcome Model
1. with an explicit reference argument, `address`, `row`, and `col` are computed from the parsed reference target.
2. with an explicit reference argument, `contents` and `type` resolve the reference through the resolver seam.
3. host/provider-backed lanes in the current baseline are:
   - `filename`
   - `format`
   - `color`
   - `parentheses`
   - `prefix`
   - `protect`
   - `width`
4. omitted-reference forms are routed through the typed host-query seam.
5. unsupported info types, invalid explicit reference text, and unavailable host/provider support return explicit worksheet-visible error outcomes.

## 6. Post-call Adaptation Policy
1. successful evaluation returns scalar text, scalar numeric/logical, or the native width array artifact depending on the selected info type.
2. invalid reference text and unresolved explicit-reference failures map to worksheet-visible `#REF!`; coercion, unsupported info types, and arity failures map to `#VALUE!` unless a worksheet error code is already carried through coercion.
3. `CELL("width", ref)` preserves the native two-item width artifact in the current baseline:
   - ordinary single-cell context shows the first numeric item,
   - `INDEX(...,2)` exposes the second boolean item,
   - `COLUMNS(...)` reports the intrinsic two-column width shape.

## 7. Version Scope (Required Axes)
1. Excel application version/channel scope:
   - bounded local empirical baseline: Excel `16.0 (build 19725)`, channel `http://officecdn.microsoft.com/pr/492350f6-3a01-4f97-b9c0-c7c6ddf67d60`, locale `en-US`.
2. Workbook Compatibility Version scope:
   - bounded dual-run workbook lanes: `default` and `compat_template`.
   - `compat_template` is the `.xls` compatibility template emitted by `tools/w10-probe/new-w10-compat-template.ps1`.

## 8. Evidence Posture
1. `spec_anchor`:
   - typed host-query seam row `FDEF-040` in `EXCEL_FUNCTION_DEFINITION_PRELIM_CONFORMANCE.csv`
   - antecedent packet row `FDEF-037` in `EXCEL_FUNCTION_DEFINITION_PRELIM_CONFORMANCE.csv`
   - public reference ids linked there: `XLS-CF-FN-001`, `XLS-CF-FN-002`, `XLS-CF-FN-007`, `XLS-CF-TV-007`, `XLS-CF-TV-008`
2. `empirical_anchor`:
   - `W12-CELL-PRE-20260309`
   - `W12-MODERATE-BL-20260309`
   - `W15-CELL-HOST-PRE-20260315`
   - `W15-XLL-BRIDGE-20260315`
3. policy decision anchors:
   - `docs/function-lane/W12_PROFILE_SYSTEM_SIDE_NOTES.md` (notes 7 and 8)
   - `docs/function-lane/W12_EXECUTION_RECORD.md`

## 9. Current W15 Coverage
1. explicit-reference local lanes:
   - `address`
   - `row`
   - `col`
   - `contents`
   - `type`
2. explicit-reference host/provider lanes:
   - `filename`
   - `format`
   - `color`
   - `parentheses`
   - `prefix`
   - `protect`
   - `width`
3. omitted-reference host/provider lanes are empirically pinned at least for:
   - `row`
   - `address`
   - `col`
   - `contents`
   - `type`
   - `filename`
   - `format`
   - `color`
   - `prefix`
   - `protect`
   - `width`
   - `parentheses`
4. current local target-gap status:
   - no known semantic gap remains in the admitted current-baseline slice after dual-run default/compat replay.
5. external integration lane:
   - cross-repo typed host-query seam acknowledgment is now recorded for `HO-FN-002`; no declared-scope integration gap remains for the current-baseline slice.

## 10. Artifact Bindings
1. Rust: `crates/oxfunc_core/src/functions/cell.rs`
2. Lean: `formal/lean/OxFunc/Functions/Cell.lean`
3. side-note linkage: `docs/function-lane/W12_PROFILE_SYSTEM_SIDE_NOTES.md` (notes 7 and 8)
4. active follow-on workset: `docs/worksets/W015_CELL_AND_INFO_HOST_QUERY_FUNCTIONS.md`
5. active seam note: `docs/function-lane/CELL_INFO_HOST_QUERY_SEAM_PRELIM.md`
