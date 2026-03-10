# Function Slice Contract (Prelim) - NOW()

## 1. Slice Identity
1. `function_id`: `FUNC.NOW`
2. `display_name`: `NOW`
3. `owner_lane`: `OxFunc`
4. `status`: `provisional`

## 2. Signature and Admission Contract
1. arity:
   - minimum: `0`
   - maximum: `0`
2. admission policy:
   - nullary only.

## 3. Semantic Class Axes
1. `determinism_class`: `time_dependent`
2. `volatility_class`: `volatile_full`
3. `host_interaction_class`: `application_state`
4. `thread_safety_class`: `host_serialized`
5. `arg_preparation_profile`: `values_only_pre_adapter`
6. `coercion_lift_profile`: `none`
7. `kernel_signature_class`: `custom`
8. `function_adapter_fec_dependency_profile`: `time_provider`
9. `surface_fec_dependency_profile`: `time_provider`

## 4. W10 Seed Coverage
1. explicit provider seam (`NowProvider`) is implemented.
2. runtime adapter rejects non-finite provider payloads.
3. no argument-admission ambiguity in current seed.

## 5. Post-call Adaptation Policy
1. successful evaluation returns a scalar numeric serial value.
2. worksheet-surface semantics also include a post-evaluation format hint:
   - when `NOW()` is entered into a cell previously formatted as `General`, built-in Excel changes the caller cell to a date-time number format in the observed baseline.
3. this format hint is part of semantic characterization for `NOW`, but application of the hint is an engine/FEC/F3E responsibility rather than a pure kernel obligation.
4. current XLL verification does not require reproducing caller-cell format application; that seam limitation is tracked in `docs/function-lane/XLL_VERIFICATION_SEAM_LIMITATIONS.md`.

## 6. Version Scope (Required Axes)
1. Excel application version/channel scope:
   - bounded local empirical baseline: Excel `16.0 (build 19725)`, channel `http://officecdn.microsoft.com/pr/492350f6-3a01-4f97-b9c0-c7c6ddf67d60`, locale `en-US`.
2. Workbook Compatibility Version scope:
   - bounded dual-run workbook lanes: `default` and `compat_template`.
   - `compat_template` is the `.xls` compatibility template emitted by `tools/w10-probe/new-w10-compat-template.ps1`.

## 7. Evidence Posture
1. `spec_anchor`:
   - packet conformance row `FDEF-035` in `EXCEL_FUNCTION_DEFINITION_PRELIM_CONFORMANCE.csv`
   - public reference ids linked there: `XLS-CF-FN-001`, `XLS-CF-FN-002`, `XLS-CF-FN-007`, `XLS-CF-TV-007`, `XLS-CF-TV-008`
2. `empirical_anchor`:
   - `W10-TENMIX-SEED-20260308`
   - `W11-XLL-FLAGS-BL-20260309`
   - `W10-W12-TFMT-20260310`
3. policy decision anchors:
   - `docs/function-lane/W10_PROFILE_SYSTEM_SIDE_NOTES.md` (notes 4 and 8)
   - `docs/function-lane/W10_EXECUTION_RECORD.md`
   - `docs/function-lane/XLL_REGISTRATION_FLAG_EXECUTION_RECORD.md`
   - `docs/function-lane/TIME_FORMAT_HINT_EXECUTION_RECORD.md`
4. current status rationale:
   - function-phase-complete and locally verified for the current reference Excel baseline,
   - provider-value, nullary admission, recalc sensitivity, and caller-cell format-hinting are now explicitly evidenced,
   - remaining locale/version expansion and broader experimental XLL control-alias investigation are orthogonal validation/seam work rather than current-phase function-semantic gaps.

## 8. Artifact Bindings
1. Rust: `crates/oxfunc_core/src/functions/now_fn.rs`
2. Lean: `formal/lean/OxFunc/Functions/Now.lean`
3. side-note linkage: `docs/function-lane/W10_PROFILE_SYSTEM_SIDE_NOTES.md` (notes 4 and 8)
