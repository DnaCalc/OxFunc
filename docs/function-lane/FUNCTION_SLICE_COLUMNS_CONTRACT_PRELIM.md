# Function Slice Contract (Prelim) - COLUMNS()

## 1. Slice Identity
1. `function_id`: `FUNC.COLUMNS`
2. `display_name`: `COLUMNS`
3. `owner_lane`: `OxFunc`
4. `status`: `function-phase-complete`

## 2. Signature and Admission Contract
1. arity:
   - minimum: `1`
   - maximum: `1`
2. admission policy:
   - admitted for `COLUMNS(reference_or_array)`.
   - the argument must be a reference or array; non-reference/non-array arguments are admission-invalid at worksheet ingress.

## 3. Semantic Class Axes
1. `determinism_class`: `deterministic`
2. `volatility_class`: `nonvolatile`
3. `host_interaction_class`: `workbook_state`
4. `thread_safety_class`: `host_serialized`
5. `arg_preparation_profile`: `refs_visible_in_adapter`
6. `coercion_lift_profile`: `custom`
7. `kernel_signature_class`: `custom`
8. `function_adapter_fec_dependency_profile`: `ref_only`
9. `surface_fec_dependency_profile`: `ref_only`
10. `fec_facility_tags`: `cap_reference_resolution`
11. `compile_eval_class`: `not_const_foldable`

## 4. Core Outcome Model
1. `COLUMNS(single_cell_reference)` returns `1`.
2. `COLUMNS(area_reference)` returns the number of columns in the referenced area as a scalar number.
3. `COLUMNS(array_constant)` returns the column count of the array.
4. the result is always a scalar numeric `EvalValue`.

## 5. Version Scope (Required Axes)
1. Excel application version/channel scope:
   - Microsoft 365 current channel (reference baseline).
2. Workbook Compatibility Version scope:
   - `default`.

## 6. Proof/Implementation Obligations
1. Lean obligations:
   - admitted result theorem for single-argument case.
   - determinism theorem.
   - scalar-return shape theorem.
2. Rust obligations:
   - correct column-count extraction from reference and array arguments.
   - arity rejection for zero-argument and multi-argument calls.
   - unit tests mapped in correlation ledger.

## 7. Artifact Bindings
1. Rust: `crates/oxfunc_core/src/functions/columns_fn.rs`
2. Lean: `formal/lean/OxFunc/Functions/Columns.lean`
3. XLL: not yet exported.

## 8. Evidence Posture
1. `spec_anchor`: to be attached from public references.
2. `empirical_anchor`: required for validated status.
3. current status rationale:
   - `COLUMNS()` is `function-phase-complete` for the current reference baseline,
   - scalar column-count extraction from references and arrays is aligned across Rust and Lean,
   - cross-build/channel and compatibility-variant replay remain validation-phase follow-up.
