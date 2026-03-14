# Function Slice Contract (Prelim) - TYPE()

## 1. Slice Identity
1. `function_id`: `FUNC.TYPE`
2. `display_name`: `TYPE`
3. `owner_lane`: `OxFunc`
4. `status`: `function-phase-complete`

## 2. Signature and Admission Contract
1. arity:
   - minimum: `1`
   - maximum: `1`
2. admission policy:
   - admitted for `TYPE(value)` over scalar, array, error, and dereferenced blank-cell lanes in the current slice.

## 3. Semantic Class Axes
1. `determinism_class`: `deterministic`
2. `volatility_class`: `nonvolatile`
3. `host_interaction_class`: `none`
4. `thread_safety_class`: `safe_pure`
5. `arg_preparation_profile`: `values_only_pre_adapter`
6. `coercion_lift_profile`: `custom`
7. `kernel_signature_class`: `custom`
8. `function_adapter_fec_dependency_profile`: `none`
9. `surface_fec_dependency_profile`: `ref_only`

## 4. Pre-call Coercion Policy
1. surface preparation resolves references before adapter entry.
2. true blank single-cell dereferences must arrive as prepared `empty_cell`, not as empty string or missing argument.
3. arrays remain arrays at the prepared boundary rather than being flattened before classification.

## 5. Core Outcome Model
1. numeric input returns type code `1`.
2. text input returns type code `2`.
3. logical input returns type code `4`.
4. error input returns type code `16`.
5. arrays return type code `64`.
6. prepared blank single-cell input returns type code `1` in the current baseline.

## 6. Post-call Adaptation Policy
1. successful evaluation returns a scalar numeric `EvalValue` carrying the type code.

## 7. Version Scope (Required Axes)
1. Excel application version/channel scope:
   - bounded local empirical baseline: Excel `16.0 (build 19725)`, channel `http://officecdn.microsoft.com/pr/492350f6-3a01-4f97-b9c0-c7c6ddf67d60`, current host locale.
2. Workbook Compatibility Version scope:
   - bounded dual-run workbook lanes: `default` and `compat_template`.

## 8. Evidence Posture
1. `spec_anchor`:
   - packet conformance row `FDEF-038` in `EXCEL_FUNCTION_DEFINITION_PRELIM_CONFORMANCE.csv`
2. `empirical_anchor`:
   - `W13-NONLOCALE-BL-20260314`
3. policy decision anchors:
   - `docs/function-lane/W13_EXECUTION_RECORD.md`
   - `docs/function-lane/LOCALE_AND_FORMAT_INTERFACE_OPTIONS.md`
4. current status rationale:
   - `TYPE()` is `function-phase-complete` for the current reference baseline,
   - the key seam is now explicit: a dereferenced blank single-cell input is classified as numeric-type `1`, while true array inputs remain type `64`,
   - Rust, Lean, and workbook replay all attend to that distinction.

## 9. Artifact Bindings
1. Rust: `crates/oxfunc_core/src/functions/type_fn.rs`
2. Lean: `formal/lean/OxFunc/Functions/Type.lean`
3. packet record: `docs/function-lane/W13_EXECUTION_RECORD.md`
