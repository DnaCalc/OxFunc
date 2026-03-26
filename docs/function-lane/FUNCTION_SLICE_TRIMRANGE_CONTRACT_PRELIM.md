# Function Slice Contract (Prelim) - TRIMRANGE()

## 1. Slice Identity
1. `function_id`: `FUNC.TRIMRANGE`
2. `display_name`: `TRIMRANGE`
3. `owner_lane`: `OxFunc`
4. `status`: `function-phase-complete`

## 2. Signature and Admission Contract
1. arity:
   - minimum: `1`
   - maximum: `4`
2. admission policy:
   - admitted for `TRIMRANGE(range[, trim_rows_type[, trim_cols_type[, headers_count]]])`.
   - `trim_rows_type` defaults to `1`, `trim_cols_type` defaults to `1`, `headers_count` defaults to `0`.

## 3. Semantic Class Axes
1. `determinism_class`: `deterministic`
2. `volatility_class`: `nonvolatile`
3. `host_interaction_class`: `workbook_state`
4. `thread_safety_class`: `host_serialized`
5. `arg_preparation_profile`: `values_only_pre_adapter`
6. `coercion_lift_profile`: `custom`
7. `kernel_signature_class`: `custom`
8. `function_adapter_fec_dependency_profile`: `none`
9. `surface_fec_dependency_profile`: `ref_only`
10. `fec_facility_tags`: none
11. `compile_eval_class`: `not_const_foldable`

## 4. Core Outcome Model
1. `TRIMRANGE(range)` trims trailing blank rows and columns from the input range and returns the reduced array.
2. `trim_rows_type` controls row trimming: `0` = no row trimming, `1` = trim trailing blank rows, `2` = trim leading and trailing blank rows.
3. `trim_cols_type` controls column trimming with the same semantics as `trim_rows_type` applied to columns.
4. `headers_count` specifies the number of leading rows to protect from trimming (header rows are never removed).
5. if trimming removes all data rows/columns, the function returns `#CALC!`.
6. invalid trim-type values (not in `{0, 1, 2}`) surface `#VALUE!`.
7. the result is a dynamic array `EvalValue::Array` containing the trimmed range contents.

## 5. Version Scope (Required Axes)
1. Excel application version/channel scope:
   - Microsoft 365 current channel (reference baseline).
2. Workbook Compatibility Version scope:
   - `default`.

## 6. Proof/Implementation Obligations
1. Lean obligations:
   - admitted result theorem for one-through-four-argument cases.
   - trim-type validation theorem.
   - `#CALC!` on empty result theorem.
   - headers_count preservation theorem.
2. Rust obligations:
   - correct trailing, leading, and combined blank trimming.
   - headers_count row protection.
   - trim-type validation and `#VALUE!` generation.
   - empty-result `#CALC!` generation.
   - unit tests mapped in correlation ledger.

## 7. Artifact Bindings
1. Rust: `crates/oxfunc_core/src/functions/trimrange_fn.rs`
2. Lean: `formal/lean/OxFunc/Functions/Trimrange.lean`
3. XLL: not yet exported.

## 8. Evidence Posture
1. `spec_anchor`: to be attached from public references.
2. `empirical_anchor`: required for validated status.
3. current status rationale:
   - `TRIMRANGE()` is `function-phase-complete` for the current reference baseline,
   - row and column trimming with header protection and empty-result error semantics are aligned across Rust and Lean,
   - cross-build/channel and compatibility-variant replay remain validation-phase follow-up.
