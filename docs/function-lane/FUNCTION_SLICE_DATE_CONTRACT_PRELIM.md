# Function Slice Contract (Prelim) - DATE()

## 1. Slice Identity
1. `function_id`: `FUNC.DATE`
2. `display_name`: `DATE`
3. `owner_lane`: `OxFunc`
4. `status`: `in_progress`, W111 current-reference review; `scope_partial`.

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
2. Year, month, and day use numerical coercion, with explicit missing/blank slots
   prepared as zero and logicals as zero/one. Coercion errors are evaluated from
   left to right. Arrays lift through the ordinary broadcast adapter.
3. Integer preparation uses the observed tolerant floor, not truncation. Positive
   year values are capped at 10,000 before signed 16-bit conversion; the short-year
   offset is selected from that converted value. Month and day width rules and
   exact boundary witnesses are recorded in the W111 evidence linked below.

## 5. Core Outcome Model
1. admitted call returns an Excel serial date number.
2. month overflow is normalized through year-month arithmetic before serial conversion.
3. The month-start serial preserves the fictitious 1900 leap day before adding
   the day offset: `DATE(1900,2,29)=60`, `DATE(1900,2,30)=61`. The normalized
   month must lie in years 1900–9999 and the final serial in 0–2,958,465.

## 6. Post-call Adaptation Policy
1. Scalar inputs return a numeric value; array inputs broadcast and retain per-cell results/errors.
2. numeric-domain failures map to worksheet-visible `#NUM!`; coercion and arity failures map to `#VALUE!` unless a worksheet error code is already carried through coercion.

## 7. Version Scope (Required Axes)
1. Excel application version/channel scope:
   - W111 reference: Excel `16.0`, build `20430`, 64-bit, channel unverified.
   - Historical W12 reference: build `19725`, channel `http://officecdn.microsoft.com/pr/492350f6-3a01-4f97-b9c0-c7c6ddf67d60`, locale `en-US`.
2. Workbook Compatibility Version scope:
   - W111: Compatibility Version `2`, 1900 date system, precision-as-displayed disabled.
   - Historical dual-run workbook lanes: `default` and `compat_template`.
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

## 9. W111 Review and Remaining Scope
1. The earlier W12 completion claim is withdrawn for the current reference:
   fresh probes found rounding, integer-width, overflow and array gaps.
2. [Retained date evidence](evidence/w111-broad-20260929/dates/README.md) links
   exact public-interface observations, failed candidate holdouts, Rust replay and
   the exercised Lean preparation/calendar bindings.
3. `scope_completeness=scope_partial`, `target_completeness=target_partial`,
   `integration_completeness=partial`. Open lanes: fresh final-candidate holdout,
   locale/date-text preparation, shared decimal conversion and combined validation.
4. XLL recreation of workbook locale/date-system context remains a verification
   seam limitation. HO-FN-022 records the receiving evaluator's context dependency;
   filing it does not establish integration.

## 10. Artifact Bindings
1. Rust: `crates/oxfunc_core/src/functions/date_fn.rs`
2. Lean: `formal/lean/OxFunc/Functions/Date.lean`
3. side-note linkage: `docs/function-lane/W12_PROFILE_SYSTEM_SIDE_NOTES.md` (note 4)
