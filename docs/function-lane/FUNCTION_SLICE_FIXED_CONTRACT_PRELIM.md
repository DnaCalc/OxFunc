# Function Slice Contract (Prelim) - FIXED()

## 1. Slice Identity
1. `function_id`: `FUNC.FIXED`
2. `display_name`: `FIXED`
3. `owner_lane`: `OxFunc`
4. `status`: `provisional`

## 2. Signature and Admission Contract
1. arity:
   - minimum: `1`
   - maximum: `3`
2. admission policy:
   - admitted in this slice as `FIXED(number, [decimals], [no_commas])` through the local locale-format render seam.

## 3. Semantic Class Axes
1. `determinism_class`: `deterministic`
2. `volatility_class`: `nonvolatile`
3. `host_interaction_class`: `none`
4. `thread_safety_class`: `safe_pure`
5. `arg_preparation_profile`: `values_only_pre_adapter`
6. `coercion_lift_profile`: `custom`
7. `kernel_signature_class`: `custom`
8. `function_adapter_fec_dependency_profile`: `locale_profile`
9. `surface_fec_dependency_profile`: `composite`
10. `fec_facility_tags`: `cap_locale_parse_format`

## 4. Pre-call Coercion Policy
1. surface preparation resolves references before adapter entry.
2. value and optional decimals are coerced through the locale-profile numeric parser.
3. `no_commas` is boolean-ish in the current slice:
   - logicals are admitted directly,
   - numeric non-zero is treated as true,
   - zero is treated as false.
4. blank cells coerce to zero in the current slice.

## 5. Core Outcome Model
1. successful evaluation delegates to the locale-profile fixed-format renderer.
2. current-host seed rows pin:
   - grouped rendering
   - `no_commas=TRUE` rendering
   - negative rendering
   - text-numeric admission
   - logical admission
   - blank-cell zeroing
3. current-host renderer currently uses space grouping and `.` decimal separator.

## 6. Post-call Adaptation Policy
1. successful evaluation returns a scalar text `EvalValue`.
2. parse/coercion/render failures map to worksheet-visible `#VALUE!` unless a worksheet error is already carried through coercion.

## 7. Version Scope (Required Axes)
1. Excel application version/channel scope:
   - bounded local empirical baseline: Excel `16.0 (build 19725)`, channel `http://officecdn.microsoft.com/pr/492350f6-3a01-4f97-b9c0-c7c6ddf67d60`, current host profile from `GET.WORKSPACE(37)`.
2. Workbook Compatibility Version scope:
   - current local seam closure is workbook-independent for the admitted rows.

## 8. Evidence Posture
1. `spec_anchor`:
   - packet conformance row `FDEF-038` in `EXCEL_FUNCTION_DEFINITION_PRELIM_CONFORMANCE.csv`
2. `empirical_anchor`:
   - `W13-LOCALE-SHIM-20260314`
3. policy decision anchors:
   - `docs/function-lane/LOCALE_AND_FORMAT_INTERFACE_OPTIONS.md`
   - `docs/function-lane/FORMAT_SHIM_AND_GET_INFO_REFERENCE_NOTES.md`
   - `docs/function-lane/LOCALE_FORMAT_SEAM_EXECUTION_RECORD.md`
   - `docs/function-lane/W13_EXECUTION_RECORD.md`
4. current status rationale:
   - the local fixed-format render seam is now explicit in Rust and Lean and grounded in the current host profile,
   - but `FIXED` is not yet `function-phase-complete` because broader locale/profile variation and the full Excel formatting environment remain outside the currently admitted shim subset.

## 9. Artifact Bindings
1. Rust: `crates/oxfunc_core/src/functions/fixed_fn.rs`
2. Lean: `formal/lean/OxFunc/Functions/Fixed.lean`; `formal/lean/OxFunc/LocaleFormat.lean`
3. empirical manifests: `docs/function-lane/FORMAT_RENDER_SCENARIO_MANIFEST_SEED.csv`; `docs/function-lane/FORMAT_PROFILE_SEED.csv`
