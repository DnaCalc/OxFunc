# Function Slice Contract (Prelim) - T()

## 1. Slice Identity
1. `function_id`: `FUNC.T`
2. `display_name`: `T`
3. `owner_lane`: `OxFunc`
4. `status`: `function-phase-complete`

## 2. Signature and Admission Contract
1. arity:
   - minimum: `1`
   - maximum: `1`
2. admission policy:
   - admitted for `T(value)` over scalar, array, error, and dereferenced blank-cell lanes in the current slice.

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
3. arrays are preserved as arrays and mapped elementwise.

## 5. Core Outcome Model
1. text passes through unchanged.
2. numbers, logicals, blank cells, and missing-like lanes become empty string text.
3. worksheet errors propagate unchanged.
4. arrays map elementwise with the same policy.

## 6. Post-call Adaptation Policy
1. successful scalar evaluation returns a text or error `EvalValue`.
2. successful array evaluation returns a shape-preserving array payload.

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
   - `T()` is `function-phase-complete` for the current reference baseline,
   - dereferenced blank single-cell input normalizes to prepared `empty_cell`, after which `T()` returns empty-string text,
   - array mapping and error propagation are pinned in Rust, Lean, and workbook replay.

## 9. Artifact Bindings
1. Rust: `crates/oxfunc_core/src/functions/t_fn.rs`
2. Lean: `formal/lean/OxFunc/Functions/T.lean`
3. packet record: `docs/function-lane/W13_EXECUTION_RECORD.md`
