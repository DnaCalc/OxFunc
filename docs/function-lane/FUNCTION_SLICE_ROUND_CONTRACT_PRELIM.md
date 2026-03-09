# Function Slice Contract (Prelim) - ROUND()

## 1. Slice Identity
1. `function_id`: `FUNC.ROUND`
2. `display_name`: `ROUND`
3. `owner_lane`: `OxFunc`
4. `status`: `provisional`

## 2. Signature and Admission Contract
1. arity:
   - minimum: `2`
   - maximum: `2`
2. admission policy:
   - admitted only for the binary shape `ROUND(number, num_digits)`.

## 3. Semantic Class Axes
1. `determinism_class`: `deterministic`
2. `volatility_class`: `nonvolatile`
3. `host_interaction_class`: `none`
4. `thread_safety_class`: `safe_pure`
5. `arg_preparation_profile`: `values_only_pre_adapter`
6. `coercion_lift_profile`: `unary_numeric_scalar_only`
7. `kernel_signature_class`: `nums_to_num`
8. `function_adapter_fec_dependency_profile`: `none`
9. `surface_fec_dependency_profile`: `ref_only`
10. `fec_facility_tags`: `cap_reference_resolution`

## 4. Pre-call Coercion Policy
1. surface preparation resolves references before adapter entry.
2. both arguments are coerced through the shared numeric path.
3. `num_digits` is truncated toward zero before kernel evaluation.

## 5. Core Outcome Model
1. admitted call applies half-away-from-zero rounding.
2. negative `num_digits` rounds on powers-of-ten to the left of the decimal point.
3. extreme `num_digits` values saturate to identity or signed zero under the bounded kernel.

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
   - `docs/function-lane/W12_PROFILE_SYSTEM_SIDE_NOTES.md` (note 4)
   - `docs/function-lane/W12_EXECUTION_RECORD.md`

## 9. W12 Seed Coverage
1. two-argument rounding is implemented.
2. half-away-from-zero kernel behavior is exercised.
3. richer floating-point edge correlation beyond the local seed remains open for later packetization.

## 10. Artifact Bindings
1. Rust: `crates/oxfunc_core/src/functions/round_fn.rs`
2. Lean: `formal/lean/OxFunc/Functions/Round.lean`
3. side-note linkage: `docs/function-lane/W12_PROFILE_SYSTEM_SIDE_NOTES.md` (note 4)
