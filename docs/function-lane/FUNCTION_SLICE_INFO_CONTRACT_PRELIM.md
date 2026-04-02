# Function Slice Contract (Prelim) - INFO()

## 1. Slice Identity
1. `function_id`: `FUNC.INFO`
2. `display_name`: `INFO`
3. `owner_lane`: `OxFunc`
4. `status`: `provisional`

## 2. Signature and Admission Contract
1. arity:
   - minimum: `1`
   - maximum: `1`
2. admission policy:
   - admitted only in the one-argument text form `INFO(type_text)` for the current slice.

## 3. Semantic Class Axes
1. `determinism_class`: `deterministic`
2. `volatility_class`: `volatile_contextual`
3. `host_interaction_class`: `workbook_state`
4. `thread_safety_class`: `host_serialized`
5. `arg_preparation_profile`: `values_only_pre_adapter`
6. `coercion_lift_profile`: `custom`
7. `kernel_signature_class`: `custom`
8. `function_adapter_fec_dependency_profile`: `composite`
9. `surface_fec_dependency_profile`: `composite`
10. `fec_facility_tags`: `cap_workbook_info`; `cap_application_info`; `cap_environment_info`

## 4. Pre-call Coercion Policy
1. `type_text` is prepared through the shared scalar-to-text path.
2. normalization is case-insensitive and trimmed.
3. unsupported or unavailable `type_text` values are not guessed; they must map through explicit observed policy.

## 5. Core Outcome Model
1. `INFO` is modeled as a typed host-query function, not a pure local kernel.
2. OxFunc owns:
   - `type_text` normalization,
   - query-kind classification,
   - expected result-shape policy,
   - worksheet-visible error mapping.
3. FEC/F3E or a local test provider owns the actual host facts:
   - workbook directory and workbook-count style facts,
   - application calculation mode and release/system information,
   - any environment-sensitive or availability-sensitive lanes.

## 6. Post-call Adaptation Policy
1. successful evaluation returns scalar text or scalar numeric `EvalValue` according to the selected query kind.
2. unsupported `type_text` values map to worksheet-visible `#VALUE!` unless empirical evidence shows a different host outcome.
3. host-unavailable lanes must preserve the observed error/value outcome instead of being normalized ad hoc.

## 7. Version Scope (Required Axes)
1. Excel application version/channel scope:
   - bounded local empirical baseline: Excel `16.0 (build 19725)`, channel `http://officecdn.microsoft.com/pr/492350f6-3a01-4f97-b9c0-c7c6ddf67d60`, locale `en-US`.
2. Workbook Compatibility Version scope:
   - bounded dual-run workbook lanes: `default` and `compat_template`.
   - `compat_template` is the `.xls` compatibility template emitted by `tools/w10-probe/new-w10-compat-template.ps1`.

## 8. Evidence Posture
1. `spec_anchor`:
   - `FDEF-040` in `EXCEL_FUNCTION_DEFINITION_PRELIM_CONFORMANCE.csv`
   - Microsoft support page for `INFO`
2. `empirical_anchor`:
   - `W15-INFO-PRE-20260315`
   - `W15-XLL-BRIDGE-20260315`
3. related local reference seams:
   - `docs/function-lane/FORMAT_SHIM_AND_GET_INFO_REFERENCE_NOTES.md`
   - `docs/function-lane/XLL_GET_INFO_EXECUTION_RECORD.md`

## 9. Current Admitted Evidence Slice
Observed and seeded in the current baseline:
1. `directory`
2. `numfile`
3. `origin`
4. `osversion`
5. `recalc`
6. `release`
7. `system`
8. `memavail`
9. `memused`
10. `totmem`
11. generated `ox_INFO(...)` XLL bridge parity with native `INFO(...)` for the ten seeded lanes

Observed current-baseline outcomes on `2026-03-15`:
1. `directory` -> text
2. `numfile` -> numeric `1`
3. `origin` -> text `$A:$A$1`
4. `osversion` -> text `Windows (64-bit) NT 10.00`
5. `recalc` -> text `Automatic`
6. `release` -> text `16.0`
7. `system` -> text `pcdos`
8. `memavail` / `memused` / `totmem` -> `#N/A`
9. generated `ox_INFO(...)` exports matched native `INFO(...)` across all ten seeded lanes in both `default` and `compat_template` replays from `W15-XLL-BRIDGE-20260315`
10. cross-repo typed host-query seam acknowledgment is now recorded for `HO-FN-002`; no declared-scope integration gap remains for the current-baseline slice

## 10. Artifact Bindings
1. parked host-query provenance: `docs/HISTORY.md` (`W015` wave-1 archive entry)
2. seam note: `docs/function-lane/CELL_INFO_HOST_QUERY_SEAM_PRELIM.md`
3. scenario manifest: `docs/function-lane/W15_INFO_PRE_SCENARIO_MANIFEST_SEED.csv`
4. execution record: `docs/function-lane/W15_EXECUTION_RECORD.md`
5. runner: `tools/w15-probe/run-w15-info-preprobe.ps1`
6. Rust: `crates/oxfunc_core/src/host_info.rs`; `crates/oxfunc_core/src/functions/info_fn.rs`
7. Lean: `formal/lean/OxFunc/HostInfoSeam.lean`; `formal/lean/OxFunc/Functions/Info.lean`
