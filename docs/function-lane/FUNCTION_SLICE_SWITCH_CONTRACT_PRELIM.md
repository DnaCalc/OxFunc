# Function Slice Contract (Prelim) - SWITCH()

## 1. Slice Identity
1. `function_id`: `FUNC.SWITCH`
2. `display_name`: `SWITCH`
3. `owner_lane`: `OxFunc`
4. `status`: `provisional`

## 2. Signature and Admission Contract
1. arity:
   - minimum: `3`
   - maximum: `255`
2. admission policy:
   - admitted form is `SWITCH(expression, value1, result1, [value2, result2]..., [default])`.
   - the first argument is the expression.
   - the remaining arguments are alternating candidate/result pairs with an optional trailing default.

## 3. Semantic Class Axes
1. `determinism_class`: `deterministic`
2. `volatility_class`: `nonvolatile`
3. `host_interaction_class`: `none`
4. `thread_safety_class`: `safe_pure`
5. `arg_preparation_profile`: `refs_visible_in_adapter`
6. `coercion_lift_profile`: `custom`
7. `kernel_signature_class`: `custom`
8. `function_adapter_fec_dependency_profile`: `none`
9. `surface_fec_dependency_profile`: `ref_only`
10. `fec_facility_tags`: `cap_reference_resolution`
11. `compile_eval_class`: `runtime_ref_dependent`

## 4. Pre-call Coercion Policy
1. `SWITCH` remains adapter-visible because result branches must stay lazy until a match is chosen.
2. candidate comparison is typed:
   - text matches text case-insensitively in the current baseline,
   - numeric text does not match numeric values,
   - logicals match by logical value.
3. missing or empty current-surface cases follow the existing prepared-argument semantics.

## 5. Core Outcome Model
1. candidates are tested left-to-right.
2. the first matching candidate selects its paired result and later branches are not forced.
3. if no candidate matches and a default exists, the default is returned.
4. if no candidate matches and no default exists, the result is worksheet `#N/A`.
5. if an earlier candidate expression errors before a match is found, that error propagates.

## 6. Post-call Adaptation Policy
1. the selected result branch is returned as-is through the ordinary evaluation surface.
2. selected scalar and error results preserve their worksheet-visible form.
3. no `SWITCH`-specific XLL seam limitation is currently known for the admitted current-baseline slice.

## 7. Version Scope (Required Axes)
1. Excel application version/channel scope:
   - bounded local empirical baseline: Excel `16.0 (build 19725)`, channel `http://officecdn.microsoft.com/pr/492350f6-3a01-4f97-b9c0-c7c6ddf67d60`, locale `en-US`.
2. Workbook Compatibility Version scope:
   - current workset evidence is pinned on the local default workbook baseline.
   - broader compatibility-template replay remains orthogonal validation work because no current-baseline semantic divergence is known for this function.

## 8. Evidence Posture
1. `spec_anchor`:
   - `FDEF-042` in `EXCEL_FUNCTION_DEFINITION_PRELIM_CONFORMANCE.csv`
2. empirical anchors:
   - `W16-BATCH49-SWITCH-20260316`
   - `W24-B01-SWITCH-20260318`
3. policy decision anchors:
   - `docs/function-lane/W16_BATCH49_SWITCH_NOTES.md`
   - `docs/function-lane/W24_BATCH01_SWITCH_EXECUTION_RECORD.md`
4. verification seam qualifier:
   - `docs/function-lane/XLL_VERIFICATION_SEAM_LIMITATIONS.md`

## 9. Current W24 Coverage
1. current-baseline replay pins:
   - first-match selection,
   - optional-default fallback,
   - no-default `#N/A`,
   - logical and text matching,
   - case-insensitive text comparison,
   - typed non-match between numeric `2` and text `"2"`,
   - lazy skipping of later result errors,
   - propagation of an earlier selected result error.
2. Rust runtime coverage exists in `misc_switch_info_family.rs`.
3. Lean coverage for the admitted current-phase slice remains the metadata/alignment substrate in `MiscSwitchInfoFamily.lean`.
4. no known semantic gap remains in the declared current-baseline slice for `SWITCH`.

## 10. Artifact Bindings
1. Rust: `crates/oxfunc_core/src/functions/misc_switch_info_family.rs`
2. Lean: `formal/lean/OxFunc/Functions/MiscSwitchInfoFamily.lean`
3. workset: `docs/worksets/W024_ORDINARY_FUNCTIONS_MEGA_BATCH_EXECUTION_PLAN.md`
4. scenario manifest: `docs/function-lane/W24_BATCH01_SWITCH_SCENARIO_MANIFEST_SEED.csv`
5. runtime requirements: `docs/function-lane/W24_BATCH01_SWITCH_RUNTIME_REQUIREMENTS.md`
6. execution record: `docs/function-lane/W24_BATCH01_SWITCH_EXECUTION_RECORD.md`
7. runner: `tools/w24-probe/run-w24-batch01-switch-baseline.ps1`
