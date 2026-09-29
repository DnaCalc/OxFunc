# Function Slice Contract (Prelim) - IFERROR()

## 1. Slice Identity
1. `function_id`: `FUNC.IFERROR`
2. `display_name`: `IFERROR`
3. `owner_lane`: `OxFunc`
4. `status`: `in_progress` (W111 expanded typed scope)

## 2. Signature and Admission Contract
1. arity:
   - minimum: `2`
   - maximum: `2`
2. admission policy:
   - admitted only for the binary shape `IFERROR(value, value_if_error)`.

## 3. Semantic Class Axes
1. `determinism_class`: `deterministic`
2. `volatility_class`: `nonvolatile`
3. `host_interaction_class`: `none`
4. `thread_safety_class`: `safe_pure`
5. `arg_preparation_profile`: `refs_visible_in_adapter`
6. `coercion_lift_profile`: `custom`
7. `kernel_signature_class`: `custom`
8. `function_adapter_fec_dependency_profile`: `none`
9. `surface_fec_dependency_profile`: `ref_only`
10. `fec_facility_tags`: `cap_reference_resolution`

## 4. Pre-call Coercion Policy
1. references stay visible to the adapter so fallback preparation can remain lazy.
2. the primary argument is prepared first under the shared values-only preparation helper.
3. scalar primaries prepare fallback only when needed. Array primaries include
   fallback shape even when existing primary cells need no fallback. This is
   prepared-value behavior, not proof of expression scheduling or effects.

## 5. Core Outcome Model
1. if the prepared primary argument is not an error, the primary value is returned.
2. if the prepared primary argument is an error, the prepared fallback value is returned.
3. selected missing or blank fallback and blank primary publish numeric zero.
4. array dimensions take the coordinatewise maxima of primary and fallback;
   singleton axes broadcast. Absent non-singleton primary coordinates produce
   catchable `#N/A`. Error selection is per cell, preserving unmatched values.

## 6. Post-call Adaptation Policy
1. successful evaluation returns the selected scalar, text, or error value directly as `EvalValue`.
2. preparation failures map to worksheet-visible `#VALUE!` unless a worksheet error code is already carried through coercion.

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
   - `docs/function-lane/W12_PROFILE_SYSTEM_SIDE_NOTES.md` (note 3)
   - `docs/function-lane/W12_EXECUTION_RECORD.md`

## 9. W12 Seed Coverage
1. historical W12 evidence covers selected scalar fallback preparation.
2. W111 corrects the earlier text: selected blank/missing fallbacks publish zero;
   non-error values pass through, and blank primaries publish zero.
3. the earlier phase claim is withdrawn for expanded array and missing-value
   scope. Current discovery and independent evidence are retained separately.

## 10. Artifact Bindings
1. Rust: `crates/oxfunc_core/src/functions/iferror.rs`
2. Lean: `formal/lean/OxFunc/Functions/IfError.lean`
3. side-note linkage: `docs/function-lane/W12_PROFILE_SYSTEM_SIDE_NOTES.md` (note 3)


## 11. W111 reference and evidence

Excel 16.0 build 20430, 64-bit, Workbook Compatibility Version 2; channel unverified.
The shared IF/IFERROR/IFNA packet and executable formal binding are retained in
`evidence/w111-broad-20260929/conditional/`. Frozen independent testing exposed
direct one-cell array identity loss; refinement and another holdout are active.
`scope_completeness=scope_partial`, `target_completeness=target_partial`,
`integration_completeness=partial`. Open lanes: refined shape replay, evaluator
scheduling/effects, reference-valued selection, uncommon reference/host forms,
HO-FN-023 receiving acknowledgment and integration.
