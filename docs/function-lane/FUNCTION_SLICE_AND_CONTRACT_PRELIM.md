# Function Slice Contract (Prelim) - AND()

## 1. Slice Identity
1. `function_id`: `FUNC.AND`
2. `display_name`: `AND`
3. `owner_lane`: `OxFunc`
4. `status`: `provisional`

## 2. Signature and Admission Contract
1. arity:
   - minimum: `1`
   - maximum: `255`
2. admission policy:
   - admitted in this slice as a variadic logical fold over prepared values.

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
2. Numeric zero is false; other finite numeric values are true. Logical values retain their truth value.
3. Direct text contributes only when it is ASCII-case-insensitive `TRUE` or `FALSE`, without whitespace trimming. Other direct text, and all text inside arrays or references, is ignored. Blank cells and omitted items are ignored.

## 5. Core Outcome Model
1. All arguments are evaluated eagerly. A false value does not suppress a later worksheet error.
2. The first worksheet error in argument order wins; array and range cells are visited in row-major order.
3. Without an error, any contributing false value yields `FALSE`; otherwise at least one contributing true value yields `TRUE`.
4. If every item is ignored, the outcome is `#VALUE!`.

## 6. Post-call Adaptation Policy
1. successful evaluation returns a scalar logical `EvalValue`.
2. adapter errors map to worksheet-visible `#VALUE!` unless a worksheet error code is already carried through coercion.

## 7. Version Scope (Required Axes)
1. Excel application version/channel scope:
   - bounded local empirical baseline: Excel `16.0 (build 19725)`, channel `http://officecdn.microsoft.com/pr/492350f6-3a01-4f97-b9c0-c7c6ddf67d60`, locale `en-US`.
2. Workbook Compatibility Version scope:
   - bounded dual-run workbook lanes: `default` and `compat_template`.
   - `compat_template` is the `.xls` compatibility template emitted by `tools/w12-probe/new-w12-compat-template.ps1`.
3. Current corroborating replay: Excel 16.0 build 20430, 64-bit, Compatibility Version 2, 1900 date system, 2026-09-29. Update channel is unverified. This does not extend the observations to every locale or alternate build.

## 8. Evidence Posture
1. `spec_anchor`:
   - packet conformance row `FDEF-037` in `EXCEL_FUNCTION_DEFINITION_PRELIM_CONFORMANCE.csv`
   - public reference ids linked there: `XLS-CF-FN-001`, `XLS-CF-FN-002`, `XLS-CF-FN-007`, `XLS-CF-TV-007`, `XLS-CF-TV-008`
2. `empirical_anchor`:
   - `W12-MODERATE-BL-20260309`
   - `evidence/w111-broad-20260929/logical/`: 540 typed AND/OR/XOR observations with exact dispatch replay, plus spelling and error-precedence observations.
3. policy decision anchors:
   - `docs/function-lane/W12_PROFILE_SYSTEM_SIDE_NOTES.md` (note 4)
   - `docs/function-lane/W12_EXECUTION_RECORD.md`

## 9. Current Evidence and Review State
1. The earlier text-rejection and short-circuit descriptions were stale. The runtime repair predates the 2026-09-29 campaign; its direct-text rule is now exercised by the retained typed replay and the Lean logical-fold bindings.
2. Current campaign state is `scope_partial`, `target_partial`, integration `partial`. Open lanes: canonical status reconciliation, the required completion checklist and self-audit, and combined repository validation. Passing this corpus is not a new function-phase completion claim.

## 10. Artifact Bindings
1. Rust: `crates/oxfunc_core/src/functions/and_fn.rs`
2. Lean: `formal/lean/OxFunc/Functions/AndFn.lean`
3. side-note linkage: `docs/function-lane/W12_PROFILE_SYSTEM_SIDE_NOTES.md` (note 4)
