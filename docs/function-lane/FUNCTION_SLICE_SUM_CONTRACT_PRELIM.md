# Function Slice Contract (Prelim) - SUM()

## 1. Slice Identity
1. `function_id`: `FUNC.SUM`
2. `display_name`: `SUM`
3. `owner_lane`: `OxFunc`
4. `status`: `provisional`

## 2. Signature and Admission Contract
1. arity:
   - minimum: `1`
   - maximum: `255`
2. admission policy:
   - admitted as a variadic aggregate fold with value-only preparation and argument-structure-sensitive coercion.

## 3. Semantic Class Axes
1. `determinism_class`: `deterministic`
2. `volatility_class`: `nonvolatile`
3. `host_interaction_class`: `none`
4. `thread_safety_class`: `safe_pure`
5. `arg_preparation_profile`: `values_only_pre_adapter`
6. `coercion_lift_profile`: `aggregate_direct_and_range_dual_policy`
7. `kernel_signature_class`: `nums_to_num`
8. `function_adapter_fec_dependency_profile`: `none`
9. `surface_fec_dependency_profile`: `ref_only`

## 4. Pre-call Coercion Policy
1. OxFunc prepares `SUM` arguments as value-only aggregate items rather than plain scalar values.
2. direct-scalar items use direct-argument aggregate coercion.
3. array-like items use array-scan/range-scan aggregate coercion.
4. the current OxFunc input-structure classes for `SUM` are:
   - `direct_scalar`
   - `direct_array_literal`
   - `reference_derived`
   - `opaque_array_value`
5. the current reference Excel baseline does not show a worksheet-semantic difference between `direct_array_literal` and `reference_derived` for `SUM`; the observed distinction is `direct_scalar` versus `array_like`.
6. when only an evaluated array reaches the OxFunc surface without stronger upstream source information, the explicit OxFunc fallback is `opaque_array_value`.

## 5. Core Outcome Model
1. direct scalar arguments coerce numeric text and logicals through the worksheet aggregate direct-argument policy.
2. missing arguments and empty cells in direct-scalar position contribute zero.
3. array-scan inputs use worksheet scan policy:
   - numeric cells contribute,
   - text and logical cells are ignored,
   - blank cells are ignored,
   - worksheet errors propagate.
4. `SUM` currently applies the same scan policy to all array-like origins; the explicit classes exist to preserve direct-scalar versus array-like behavior and to avoid silently inventing a more specific source class after evaluation.
5. the `SUM` kernel itself does not consume raw references; dereference and array-like classification happen before numeric fold evaluation.

## 6. Post-call Adaptation Policy
1. successful evaluation returns a numeric `EvalValue`.
2. worksheet errors discovered during aggregate scan propagate as carried worksheet errors.
3. non-worksheet coercion failures map to worksheet-visible `#VALUE!`.

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
   - `docs/function-lane/W10_PROFILE_SYSTEM_SIDE_NOTES.md` (note 1)
   - `docs/function-lane/W10_EXECUTION_RECORD.md`
   - `docs/upstream/NOTES_FOR_OXFML.md`
4. current status rationale:
   - function-phase-complete and locally verified for the current reference Excel baseline,
   - direct-scalar versus array-scan behavior is now empirically pinned for `SUM("2",TRUE)`, `SUM({"2",TRUE})`, and `SUM(range-with-text-and-logical)`,
   - the OxFunc seam now models the required direct-scalar versus array-like distinction explicitly and defines the fallback for evaluated arrays whose upstream source class is no longer visible,
   - remaining OxFml work is to preserve whatever richer upstream structure later functions actually need, not a remaining OxFunc function-semantic gap for `SUM`.

## 9. W10 Coverage
1. deterministic numeric fold baseline.
2. direct scalar coercion of numeric text and logicals.
3. direct non-numeric text rejection.
4. direct missing/empty treatment.
5. direct scalar worksheet-error propagation.
6. reference-derived scan behavior for text/logical/blank.
7. direct-array-literal scan behavior.
8. explicit opaque-array fallback behavior for the OxFunc surface seam.

## 10. Artifact Bindings
1. Rust: `crates/oxfunc_core/src/functions/sum.rs`
2. Lean: `formal/lean/OxFunc/Functions/Sum.lean`
3. side-note linkage: `docs/function-lane/W10_PROFILE_SYSTEM_SIDE_NOTES.md` (note 1)
