# Function Slice Contract (Prelim) - TODAY()

## 1. Slice Identity
1. `function_id`: `FUNC.TODAY`
2. `display_name`: `TODAY`
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
10. `fec_facility_tags`: `cap_time_provider`

## 4. Pre-call Coercion Policy
1. no user arguments are admitted in the current seed.
2. value production depends on the declared time-provider seam supplied by the host surface.

## 5. Core Outcome Model
1. admitted call requests a provider serial and floors it to an integral date serial.
2. non-finite provider payloads are rejected.
3. nonzero arity is an admission failure in the current seed.

## 6. Post-call Adaptation Policy
1. successful evaluation returns a scalar numeric `EvalValue`.
2. provider and arity failures map to worksheet-visible `#VALUE!` in the current runtime seam.
3. worksheet-surface semantics also include a post-evaluation format hint:
   - when `TODAY()` is entered into a cell previously formatted as `General`, built-in Excel changes the caller cell to a date format in the observed/documented baseline.
4. this format hint is part of semantic characterization for `TODAY`, but application of the hint is an engine/FEC/F3E responsibility rather than a pure kernel obligation.
5. current XLL verification does not require reproducing caller-cell format application; that seam limitation is tracked in `docs/function-lane/XLL_VERIFICATION_SEAM_LIMITATIONS.md`.

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
   - `W11-XLL-FLAGS-BL-20260309`
   - `W10-W12-TFMT-20260310`
3. policy decision anchors:
   - `docs/function-lane/W12_PROFILE_SYSTEM_SIDE_NOTES.md` (note 6)
   - `docs/function-lane/W12_EXECUTION_RECORD.md`
   - `docs/function-lane/XLL_REGISTRATION_FLAG_EXECUTION_RECORD.md`
   - `docs/function-lane/TIME_FORMAT_HINT_EXECUTION_RECORD.md`
4. current status rationale:
   - function-phase-complete and locally verified for the current reference Excel baseline,
   - provider floor semantics, recalc sensitivity, and caller-cell format-hinting are now explicitly evidenced,
   - remaining locale/version expansion and broader experimental XLL control-alias investigation are orthogonal validation/seam work rather than current-phase function-semantic gaps.

## 9. W12 Seed Coverage
1. explicit provider seam is implemented and floors the provider serial to an integral date value.
2. no argument-admission ambiguity remains in the current seed.
3. ordinary user-facing volatile behavior is evidenced through the W11 follow-back lane; broader experimental control-alias investigation remains external seam work.

## 10. Artifact Bindings
1. Rust: `crates/oxfunc_core/src/functions/today_fn.rs`
2. Lean: `formal/lean/OxFunc/Functions/Today.lean`
3. side-note linkage: `docs/function-lane/W12_PROFILE_SYSTEM_SIDE_NOTES.md` (note 6)
