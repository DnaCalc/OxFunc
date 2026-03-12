# Function Slice Contract (Prelim) - DATE()

## 1. Slice Identity
1. `function_id`: `FUNC.DATE`
2. `display_name`: `DATE`
3. `owner_lane`: `OxFunc`
4. `status`: `provisional`

## 2. Signature and Admission Contract
1. arity:
   - minimum: `3`
   - maximum: `3`
2. admission policy:
   - admitted only for the ternary shape `DATE(year, month, day)`.

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
2. year, month, and day are coerced numerically and truncated toward zero.
3. years in `[0, 1899]` are normalized by adding `1900` in the current seed.

## 5. Core Outcome Model
1. admitted call returns an Excel serial date number.
2. month overflow is normalized through year-month arithmetic before serial conversion.
3. the bounded kernel preserves the Excel 1900 leap-year bug seed (`1900-02-29 -> 60`) and returns `#NUM!` for bounded numeric-domain failures.

## 6. Post-call Adaptation Policy
1. successful evaluation returns a scalar numeric `EvalValue`.
2. numeric-domain failures map to worksheet-visible `#NUM!`; coercion and arity failures map to `#VALUE!` unless a worksheet error code is already carried through coercion.

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
   - `docs/function-lane/W12_PROFILE_SYSTEM_SIDE_NOTES.md` (note 4)
   - `docs/function-lane/W12_EXECUTION_RECORD.md`

## 9. W12 Seed Coverage
1. Excel serial conversion is implemented with year/month/day truncation and year-in-`[0,1899]` normalization.
2. the serial-zero boundary (`DATE(1900,1,0)=0`), month-zero rejection, month overflow, short-year normalization, and the `1900-02-29 -> 60` leap-bug lane are pinned directly.
3. no known current-phase semantic gap remains in the admitted ternary lane, so this slice is `function-phase-complete` for the current reference baseline.

## 10. Artifact Bindings
1. Rust: `crates/oxfunc_core/src/functions/date_fn.rs`
2. Lean: `formal/lean/OxFunc/Functions/Date.lean`
3. side-note linkage: `docs/function-lane/W12_PROFILE_SYSTEM_SIDE_NOTES.md` (note 4)
