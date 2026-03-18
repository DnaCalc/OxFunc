# Function Slice Contract (Prelim) - Date Value Family (`DATEVALUE`, `TIMEVALUE`, `DAYS360`, `DATEDIF`)

## 1. Slice Identity
1. `function_ids`:
   - `FUNC.DATEVALUE`
   - `FUNC.TIMEVALUE`
   - `FUNC.DAYS360`
   - `FUNC.DATEDIF`
2. `display_family_name`: `Date Value Family`
3. `owner_lane`: `OxFunc`
4. `status`: `provisional`

## 2. Signature and Admission Contract
1. admitted signatures:
   - `DATEVALUE(date_text)`
   - `TIMEVALUE(time_text)`
   - `DAYS360(start_date, end_date, [method])`
   - `DATEDIF(start_date, end_date, unit)`
2. arity:
   - `DATEVALUE`: exact `1`
   - `TIMEVALUE`: exact `1`
   - `DAYS360`: min `2`, max `3`
   - `DATEDIF`: exact `3`

## 3. Semantic Class Axes
1. `determinism_class`: `deterministic`
2. `volatility_class`: `nonvolatile`
3. `host_interaction_class`: `none`
4. `thread_safety_class`: `safe_pure`
5. `arg_preparation_profile`: `values_only_pre_adapter`
6. `coercion_lift_profile`: `custom`
7. `kernel_signature_class`: `custom`
8. `function_adapter_fec_dependency_profile`:
   - `DATEVALUE`, `TIMEVALUE`: `locale_profile`
   - `DAYS360`, `DATEDIF`: `none`
9. `surface_fec_dependency_profile`:
   - `DATEVALUE`, `TIMEVALUE`: `composite`
   - `DAYS360`, `DATEDIF`: `ref_only`
10. `fec_facility_tags`:
   - `cap_locale_parse_format`
   - `cap_reference_resolution`

## 4. Pre-call Coercion Policy
1. `DATEVALUE` and `TIMEVALUE` are values-only text/profile functions.
2. current admitted text subset is the empirically pinned host-like profile:
   - ISO date text `yyyy-mm-dd`
   - `d-MMM-yyyy`
   - optional trailing `h:mm[:ss] [AM|PM]`
   - pure time text for `TIMEVALUE`
3. slash-date text such as `1/2/2024` remains rejected in the current admitted slice.
4. `DAYS360` and `DATEDIF` operate over serial dates after ordinary numeric coercion.

## 5. Core Outcome Model
1. `DATEVALUE` returns the date serial only and ignores any admitted time suffix.
2. `TIMEVALUE` returns only the time fraction and ignores any admitted date prefix.
3. `DATEVALUE("6:35 AM")` returns `0` in the current baseline.
4. the family preserves the current `1900` serial baseline including:
   - serial `0 -> 1900-01-00`
   - fake serial `60 -> 1900-02-29`
5. `DAYS360` supports both:
   - NASD/US method when omitted or `FALSE`
   - European method when `TRUE`
6. `DATEDIF` admitted units in this slice:
   - `Y`
   - `M`
   - `D`
   - `YM`
   - `YD`
   - `MD`
7. invalid `DATEDIF` unit text yields worksheet `#NUM!`.
8. current-baseline `DATEDIF("MD")` quirks are preserved exactly as empirically pinned, including negative outputs in the seeded lanes.

## 6. Post-call Adaptation Policy
1. successful evaluation returns scalar numeric serials or day-count values.
2. invalid date-text or time-text in the admitted current slice maps to worksheet `#VALUE!`.
3. invalid `DATEDIF` unit or invalid date order maps to worksheet `#NUM!`.
4. no family-specific XLL limitation is currently known for the admitted current-baseline slice.

## 7. Version Scope (Required Axes)
1. Excel application version/channel scope:
   - bounded local empirical baseline: Excel `16.0 (build 19725)`, channel `http://officecdn.microsoft.com/pr/492350f6-3a01-4f97-b9c0-c7c6ddf67d60`, locale `en-US`.
2. Workbook Compatibility Version scope:
   - current workset evidence is pinned on the local default workbook baseline.
   - broader compatibility-template replay remains orthogonal validation work because no current-baseline semantic divergence is known for this family.

## 8. Evidence Posture
1. `spec_anchor`:
   - `FDEF-043` in `EXCEL_FUNCTION_DEFINITION_PRELIM_CONFORMANCE.csv`
2. empirical anchors:
   - `W16-BATCH48-DATEVALUE-20260316`
   - `W24-B02-DATEVALUE-20260318`
3. policy decision anchors:
   - `docs/function-lane/W16_BATCH48_DATEVALUE_NOTES.md`
   - `docs/function-lane/W24_BATCH02_DATEVALUE_EXECUTION_RECORD.md`

## 9. Current W24 Coverage
1. native replay pins the admitted text subset for `DATEVALUE` and `TIMEVALUE`,
2. native replay pins both NASD/US and European `DAYS360` February-end divergences,
3. native replay pins the admitted `DATEDIF` unit set plus the quirky `MD` lanes,
4. Rust runtime and Lean alignment already exist for the family substrate,
5. no known semantic gap remains in the declared current-baseline slice for these four functions.

## 10. Artifact Bindings
1. Rust: `crates/oxfunc_core/src/functions/date_value_family.rs`
2. Lean: `formal/lean/OxFunc/Functions/DateValueFamily.lean`
3. workset: `docs/worksets/W024_ORDINARY_FUNCTIONS_MEGA_BATCH_EXECUTION_PLAN.md`
4. scenario manifest: `docs/function-lane/W24_BATCH02_DATEVALUE_SCENARIO_MANIFEST_SEED.csv`
5. runtime requirements: `docs/function-lane/W24_BATCH02_DATEVALUE_RUNTIME_REQUIREMENTS.md`
6. execution record: `docs/function-lane/W24_BATCH02_DATEVALUE_EXECUTION_RECORD.md`
7. runner: `tools/w24-probe/run-w24-batch02-datevalue-baseline.ps1`
