# Function Slice Contract (Prelim) - TEXT()

## 1. Slice Identity
1. `function_id`: `FUNC.TEXT`
2. `display_name`: `TEXT`
3. `owner_lane`: `OxFunc`
4. `status`: `function-phase-complete`

## 2. Signature and Admission Contract
1. arity:
   - minimum: `2`
   - maximum: `2`
2. admission policy:
   - admitted in this slice as `TEXT(value, format_text)` for the explicitly supported local format-code subset.

## 3. Semantic Class Axes
1. `determinism_class`: `deterministic`
2. `volatility_class`: `nonvolatile`
3. `host_interaction_class`: `none`
4. `thread_safety_class`: `safe_pure`
5. `arg_preparation_profile`: `values_only_pre_adapter`
6. `coercion_lift_profile`: `custom`
7. `kernel_signature_class`: `text_to_text`
8. `function_adapter_fec_dependency_profile`: `locale_profile`
9. `surface_fec_dependency_profile`: `composite`
10. `fec_facility_tags`: `cap_locale_parse_format`

## 4. Pre-call Coercion Policy
1. surface preparation resolves references before adapter entry.
2. `format_text` uses shared prepared-text coercion.
3. admitted scalar value lanes are numbers, logicals, text, worksheet errors, and blank cells.
4. text payloads first attempt locale-profile parsing; when that parse fails in the current slice, the original text is preserved.

## 5. Core Outcome Model
1. admitted render codes in the local seam are currently:
   - `0`
   - `0.00`
   - `0%`
   - `yyyy-mm-dd`
2. current-host seed rows pin:
   - rounding with `0`
   - fixed two-decimal rendering with `0.00`
   - percentage rendering with `0%`
   - date-serial rendering with `yyyy-mm-dd`
3. logical values return `TRUE`/`FALSE` text in the current slice.
4. nonnumeric text that is not parseable through the locale-profile parser is preserved unchanged in the current slice.
5. blank-cell input is treated as numeric zero before rendering in the current slice.

## 6. Post-call Adaptation Policy
1. successful evaluation returns a scalar text `EvalValue`.
2. unsupported render-code lanes and unsupported argument kinds currently map to worksheet-visible `#VALUE!`.

## 7. Version Scope (Required Axes)
1. Excel application version/channel scope:
   - bounded local empirical baseline: Excel `16.0 (build 19725)`, channel `http://officecdn.microsoft.com/pr/492350f6-3a01-4f97-b9c0-c7c6ddf67d60`, current host profile from `GET.WORKSPACE(37)` plus explicit `en-US` shim rows where admitted.
2. Workbook Compatibility Version scope:
   - current local seam closure is workbook-independent for the admitted render rows.

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
   - the local format-code render seam is now explicit in Rust and Lean and grounded in host-profile evidence,
   - and `TEXT()` is `function-phase-complete` for the current reference baseline because the admitted local render slice is now characterized, implemented, Lean-aligned, and empirically replayed; broader format-language and locale expansion remain orthogonal validation phases.

## 9. Artifact Bindings
1. Rust: `crates/oxfunc_core/src/functions/text_fn.rs`
2. Lean: `formal/lean/OxFunc/Functions/Text.lean`; `formal/lean/OxFunc/LocaleFormat.lean`
3. empirical manifests: `docs/function-lane/FORMAT_RENDER_SCENARIO_MANIFEST_SEED.csv`; `docs/function-lane/FORMAT_PROFILE_SEED.csv`

