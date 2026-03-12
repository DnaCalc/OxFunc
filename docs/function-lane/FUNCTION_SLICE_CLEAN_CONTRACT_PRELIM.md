# Function Slice Contract (Prelim) - CLEAN()

## 1. Slice Identity
1. `function_id`: `FUNC.CLEAN`
2. `display_name`: `CLEAN`
3. `owner_lane`: `OxFunc`
4. `status`: `provisional`

## 2. Signature and Admission Contract
1. arity:
   - minimum: `1`
   - maximum: `1`
2. admission policy:
   - admitted only for the unary shape `CLEAN(text)`.

## 3. Semantic Class Axes
1. `determinism_class`: `deterministic`
2. `volatility_class`: `nonvolatile`
3. `host_interaction_class`: `none`
4. `thread_safety_class`: `safe_pure`
5. `arg_preparation_profile`: `values_only_pre_adapter`
6. `coercion_lift_profile`: `none`
7. `kernel_signature_class`: `text_to_text`
8. `function_adapter_fec_dependency_profile`: `none`
9. `surface_fec_dependency_profile`: `ref_only`
10. `fec_facility_tags`: `cap_reference_resolution`

## 4. Pre-call Coercion Policy
1. surface preparation resolves references before adapter entry.
2. the admitted argument is coerced through the shared prepared-text path.
3. scalar logical and numeric values therefore follow Excel text conversion before cleaning.

## 5. Core Outcome Model
1. admitted call removes UTF-16 code units below decimal `32`.
2. in the current empirical baseline, it also removes the observed extra C1 subset `129`, `141`, `143`, `144`, and `157`.
3. `CHAR(127)`, zero-width space (`U+200B`), and NBSP (`U+00A0`) are preserved in the current empirical baseline.
3. coercion failures terminate evaluation with error.

## 6. Post-call Adaptation Policy
1. successful evaluation returns a scalar text `EvalValue`.
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
   - `docs/function-lane/W12_PROFILE_SYSTEM_SIDE_NOTES.md` (note 5)
   - `docs/function-lane/TEXT_FUNCTION_EMPIRICAL_EXPANSION_NOTES.md`
   - `docs/function-lane/W12_EXECUTION_RECORD.md`
4. current status rationale:
   - function-phase-complete for the current reference Excel baseline,
   - ASCII control removal, the observed extra C1 removal subset, and the non-removal of `CHAR(127)` / `U+200B` / `U+00A0` are now explicitly evidenced,
   - current XLL bridge replay is now parity-closed on the extra C1 subset after rebuilding the bridge against the closeout core semantics,
   - remaining locale/version expansion is orthogonal validation work rather than a known current-phase semantic gap.

## 9. W12 Seed Coverage
1. low ASCII control-strip behavior is implemented and replayed directly.
2. prepared-text helper reuse is explicit in the slice.
3. the observed extra C1 removal subset (`129`, `141`, `143`, `144`, `157`) is now part of the admitted current-phase behavior claim.
4. `CHAR(127)`, zero-width space, and NBSP preservation are explicitly evidenced in the current baseline.

## 10. Artifact Bindings
1. Rust: `crates/oxfunc_core/src/functions/clean_fn.rs`
2. Lean: `formal/lean/OxFunc/Functions/Clean.lean`
3. side-note linkage: `docs/function-lane/W12_PROFILE_SYSTEM_SIDE_NOTES.md` (note 5)
