# Function Slice Contract (Prelim) - ISNUMBER()

## 1. Slice Identity
1. `function_id`: `FUNC.ISNUMBER`
2. `display_name`: `ISNUMBER`
3. `owner_lane`: `OxFunc`
4. `status`: `provisional`

## 2. Signature and Admission Contract
1. arity:
   - minimum: `1`
   - maximum: `1`
2. admission policy:
   - unary only.

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

## 4. Pre-call Coercion Policy
1. unary input is prepared through the values-only pre-adapter seam.
2. no scalar coercion is attempted beyond ordinary reference resolution and value preparation.

## 5. Core Outcome Model
1. prepared numeric payload returns logical `TRUE`.
2. prepared text, logical, blank-like, and error payloads return logical `FALSE`.
3. non-unary shapes are admission failures in the current slice.

## 6. Post-call Adaptation Policy
1. successful evaluation returns a logical `EvalValue`.
2. preparation failures map to worksheet-visible `#VALUE!` unless a worksheet error code is already carried through the preparation seam.

## 7. Version Scope (Required Axes)
1. Excel application version/channel scope:
   - bounded local empirical baseline: Excel `16.0 (build 19725)`, channel `http://officecdn.microsoft.com/pr/492350f6-3a01-4f97-b9c0-c7c6ddf67d60`, locale `en-US`.
2. Workbook Compatibility Version scope:
   - bounded dual-run workbook lanes: `default` and `compat_template`.
   - `compat_template` is the `.xls` compatibility template emitted by `tools/w10-probe/new-w10-compat-template.ps1`.

## 8. Evidence Posture
1. `spec_anchor`:
   - packet conformance row `FDEF-035` in `EXCEL_FUNCTION_DEFINITION_PRELIM_CONFORMANCE.csv`
   - public reference ids linked there: `XLS-CF-FN-001`, `XLS-CF-FN-002`, `XLS-CF-FN-007`, `XLS-CF-TV-007`, `XLS-CF-TV-008`
2. `empirical_anchor`:
   - `W10-TENMIX-SEED-20260308`
3. policy decision anchors:
   - `docs/function-lane/W10_EXECUTION_RECORD.md`
4. current status rationale:
   - function-phase-complete and locally verified for the current reference Excel baseline,
   - numeric, text, error, and reference-fed lanes are exercised in Rust tests and W10 dual-run workbook replay,
   - remaining locale/version expansion is orthogonal validation-phase work rather than a current-phase function-semantic gap.

## 9. W10 Coverage
1. returns true only for numeric prepared value payload.
2. text, logical, and error lanes return false.
3. reference arguments are supported through pre-adapter preparation.

## 10. Artifact Bindings
1. Rust: `crates/oxfunc_core/src/functions/isnumber.rs`
2. Lean: `formal/lean/OxFunc/Functions/IsNumber.lean`
