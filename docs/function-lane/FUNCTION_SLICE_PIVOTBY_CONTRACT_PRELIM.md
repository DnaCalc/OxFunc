# Function Slice Contract (Prelim) - PIVOTBY()

## 1. Slice Identity
1. `function_id`: `FUNC.PIVOTBY`
2. `display_name`: `PIVOTBY`
3. `owner_lane`: `OxFunc`
4. `status`: `in_progress`

## 2. Signature and Admission Contract
1. arity:
   - minimum: `4`
   - maximum: `255`
2. admission policy:
   - admitted for `PIVOTBY(row_fields, col_fields, values, function[, field_headers[, row_total_depth[, row_sort_order[, col_total_depth[, col_sort_order[, filter_array[, relative_to]]]]]]])`.
   - the fourth argument (`function`) is a callable (LAMBDA or built-in aggregation function).

## 3. Semantic Class Axes
1. `determinism_class`: `deterministic`
2. `volatility_class`: `nonvolatile`
3. `host_interaction_class`: `none`
4. `thread_safety_class`: `safe_pure`
5. `arg_preparation_profile`: `values_only_pre_adapter`
6. `coercion_lift_profile`: `custom`
7. `kernel_signature_class`: `custom`
8. `function_adapter_fec_dependency_profile`: `none`
9. `surface_fec_dependency_profile`: `none`
10. `fec_facility_tags`: none
11. `compile_eval_class`: `not_const_foldable`

## 4. Core Outcome Model
1. `PIVOTBY` groups `values` by unique entries in `row_fields` and `col_fields`, producing a two-dimensional pivot table by applying `function` to each row/column intersection group.
2. optional arguments control field headers, row and column total depths, sort orders, filtering, and relative-to computation.
3. the result is a dynamic array `EvalValue::Array` containing the pivoted and aggregated data.
4. OxFunc now has callable-backed runtime pivoting on an admitted current-baseline slice when a prepared callable value is supplied at the function boundary.
5. Wider completion work remains open around broader option-matrix characterization, promotion doctrine, and seam-owned callable transport.

## 5. Version Scope (Required Axes)
1. Excel application version/channel scope:
   - Microsoft 365 current channel (reference baseline).
2. Workbook Compatibility Version scope:
   - `default`.

## 6. Proof/Implementation Obligations
1. Lean obligations:
   - arity validation theorem for four-through-255-argument cases.
   - argument parsing theorem for optional parameters.
   - executable alignment for the admitted callable-backed pivot slice and its seeded empirical lanes.
2. Rust obligations:
   - arity validation and argument parsing for all admitted arities.
   - callable-backed pivoting, total rows/columns, filtering, and seeded sort/header lanes for the admitted current-baseline slice.
   - unit tests for the admitted empirical slice mapped in the local correlation/evidence packet.

## 7. Artifact Bindings
1. Rust: `crates/oxfunc_core/src/functions/pivotby_fn.rs`
2. Lean: `formal/lean/OxFunc/Functions/Pivotby.lean`
3. XLL: exported through the ordinary catalog surface; callable transport remains seam-owned.

## 8. Evidence Posture
1. `spec_anchor`: to be attached from public references.
2. `empirical_anchor`: required for validated status.
3. current status rationale:
   - `PIVOTBY()` is no longer a stub; OxFunc now has a real callable-backed runtime implementation for the admitted current-baseline slice,
   - the function remains `in_progress` because broader completion promotion work, formal alignment expansion, and final seam-owned callable transport remain open.
