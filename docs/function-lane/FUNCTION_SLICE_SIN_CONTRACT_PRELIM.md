# Function Slice Contract (Prelim) - SIN()

## 1. Slice Identity
1. `function_id`: `FUNC.SIN`
2. `display_name`: `SIN`
3. `owner_lane`: `OxFunc`
4. `status`: `function-phase-complete`

## 2. Signature and Admission Contract
1. arity:
   - minimum: `1`
   - maximum: `1`
2. admission policy:
   - admitted for `SIN(number)` under the ordinary worksheet numeric-coercion lane.
   - admitted array arguments lift elementwise across the current baseline slice.

## 3. Semantic Class Axes
1. `determinism_class`: `deterministic`
2. `volatility_class`: `nonvolatile`
3. `host_interaction_class`: `none`
4. `thread_safety_class`: `safe_pure`
5. `arg_preparation_profile`: `values_only_pre_adapter`
6. `coercion_lift_profile`: `unary_numeric_scalar_or_array_elementwise`
7. `kernel_signature_class`: `num_to_num`
8. `function_adapter_fec_dependency_profile`: `none`
9. `surface_fec_dependency_profile`: `ref_only`

## 4. Pre-call Coercion Policy
1. surface preparation resolves references before adapter entry.
2. scalar numeric text is admitted through the shared numeric-coercion path.
3. nonnumeric text becomes worksheet-visible `#VALUE!`.
4. array inputs lift elementwise, preserving per-element worksheet errors.

## 5. Core Outcome Model
1. admitted scalar numeric inputs return the ordinary trigonometric sine of the coerced numeric value.
2. admitted array inputs return a shape-preserving array of per-element results.
3. domain is unrestricted over finite numeric inputs in the current slice.

## 6. Post-call Adaptation Policy
1. successful scalar evaluation returns a numeric `EvalValue`.
2. successful array evaluation returns a shape-preserving array payload.
3. coercion failures map to worksheet-visible `#VALUE!` unless an existing worksheet error is already carried through preparation.

## 7. Version Scope (Required Axes)
1. Excel application version/channel scope:
   - bounded local empirical baseline: Excel `16.0 (build 19725)`, channel `http://officecdn.microsoft.com/pr/492350f6-3a01-4f97-b9c0-c7c6ddf67d60`, current host locale.
2. Workbook Compatibility Version scope:
   - bounded dual-run workbook lanes: `default` and `compat_template`.

## 8. Evidence Posture
1. `spec_anchor`:
   - packet conformance rows `FDEF-025` and `FDEF-038` in `EXCEL_FUNCTION_DEFINITION_PRELIM_CONFORMANCE.csv`
2. `empirical_anchor`:
   - `W13-NONLOCALE-BL-20260314`
3. policy decision anchors:
   - `docs/function-lane/W13_EXECUTION_RECORD.md`
4. current status rationale:
   - `SIN()` is `function-phase-complete` for the current reference baseline,
   - numeric-text admission, bad-text failure, and elementwise array-lift behavior are now aligned across Rust, Lean, and workbook replay,
   - broader locale/version sweeps remain orthogonal validation-phase work.

## 9. Artifact Bindings
1. Rust: `crates/oxfunc_core/src/functions/sin.rs`
2. Lean: `formal/lean/OxFunc/Functions/Sin.lean`
3. packet record: `docs/function-lane/W13_EXECUTION_RECORD.md`
