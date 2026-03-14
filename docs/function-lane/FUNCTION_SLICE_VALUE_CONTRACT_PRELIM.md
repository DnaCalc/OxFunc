# Function Slice Contract (Prelim) - VALUE()

## 1. Slice Identity
1. `function_id`: `FUNC.VALUE`
2. `display_name`: `VALUE`
3. `owner_lane`: `OxFunc`
4. `status`: `function-phase-complete`

## 2. Signature and Admission Contract
1. arity:
   - minimum: `1`
   - maximum: `1`
2. admission policy:
   - admitted in this slice as `VALUE(text)` plus already-numeric passthrough at the prepared-argument boundary.

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
2. numeric inputs pass through unchanged.
3. text inputs are delegated to the locale-profile parser seam.
4. logical, blank, missing, and unsupported aggregate/reference payload kinds are `#VALUE!` in the current slice.

## 5. Core Outcome Model
1. successful admitted text parsing returns the parsed numeric/date serial value.
2. current empirically pinned current-host lanes include:
   - plain decimal numeric text
   - space-grouped numeric text
   - currency text with local symbol
   - percent text
   - ISO `yyyy-mm-dd` date text
3. current empirically pinned rejection lanes include current-host slash date text (`"1/2/2024"`) and blank/whitespace-only text.

## 6. Post-call Adaptation Policy
1. successful evaluation returns a scalar numeric `EvalValue`.
2. parser/coercion failures map to worksheet-visible `#VALUE!` unless a worksheet error is already carried through coercion.

## 7. Version Scope (Required Axes)
1. Excel application version/channel scope:
   - bounded local empirical baseline: Excel `16.0 (build 19725)`, channel `http://officecdn.microsoft.com/pr/492350f6-3a01-4f97-b9c0-c7c6ddf67d60`, local host profile from `GET.WORKSPACE(37)` plus explicit `en-US` shim rows.
2. Workbook Compatibility Version scope:
   - current local seam closure is workbook-independent for the admitted parser rows.

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
   - the local locale-parse seam is now explicit in Rust and Lean and grounded in machine-readable host-profile evidence,
   - and `VALUE()` is `function-phase-complete` for the current reference baseline because the admitted local parser slice is now characterized, implemented, Lean-aligned, and empirically replayed; broader locale/version expansion remains an orthogonal validation phase.

## 9. Artifact Bindings
1. Rust: `crates/oxfunc_core/src/functions/value_fn.rs`
2. Lean: `formal/lean/OxFunc/Functions/Value.lean`; `formal/lean/OxFunc/LocaleFormat.lean`
3. empirical manifests: `docs/function-lane/VALUE_PARSE_SCENARIO_MANIFEST_SEED.csv`; `docs/function-lane/FORMAT_PROFILE_SEED.csv`

