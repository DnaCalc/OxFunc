# Function Slice Contract (Prelim) - RAND()

## 1. Slice Identity
1. `function_id`: `FUNC.RAND`
2. `display_name`: `RAND`
3. `owner_lane`: `OxFunc`
4. `status`: `provisional`

## 2. Signature and Admission Contract
1. arity:
   - minimum: `0`
   - maximum: `0`
2. admission policy:
   - nullary only.

## 3. Semantic Class Axes
1. `determinism_class`: `pseudo_random`
2. `volatility_class`: `volatile_full`
3. `host_interaction_class`: `application_state`
4. `thread_safety_class`: `host_serialized`
5. `arg_preparation_profile`: `values_only_pre_adapter`
6. `coercion_lift_profile`: `none`
7. `kernel_signature_class`: `custom`
8. `function_adapter_fec_dependency_profile`: `random_provider`
9. `surface_fec_dependency_profile`: `random_provider`
10. `fec_facility_tags`: `cap_random_provider`

## 4. Pre-call Coercion Policy
1. no user arguments are admitted in the current seed.
2. value production depends on the declared random-provider seam supplied by the host surface.

## 5. Core Outcome Model
1. admitted call returns the provider value when it is finite and in `[0, 1)`.
2. provider values outside that range are rejected.
3. nonzero arity is an admission failure in the current seed.

## 6. Post-call Adaptation Policy
1. successful evaluation returns a scalar numeric `EvalValue`.
2. provider and arity failures map to worksheet-visible `#VALUE!` in the current runtime seam.

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
   - `docs/function-lane/W12_PROFILE_SYSTEM_SIDE_NOTES.md` (note 6)
   - `docs/function-lane/W12_EXECUTION_RECORD.md`

## 9. W12 Seed Coverage
1. explicit provider seam is implemented and restricted to finite values in `[0,1)`.
2. nullary admission and the worksheet-visible range contract (`0 <= RAND() < 1`) are pinned directly.
3. volatility-registration follow-back remains a W11 seam concern, not a core `RAND` semantic gap, so this slice is `function-phase-complete` for the current reference baseline.

## 10. Artifact Bindings
1. Rust: `crates/oxfunc_core/src/functions/rand_fn.rs`
2. Lean: `formal/lean/OxFunc/Functions/Rand.lean`
3. side-note linkage: `docs/function-lane/W12_PROFILE_SYSTEM_SIDE_NOTES.md` (note 6)
