# Function Slice Contract (Prelim) - GROUPBY()

## 1. Slice Identity
1. `function_id`: `FUNC.GROUPBY`
2. `display_name`: `GROUPBY`
3. `owner_lane`: `OxFunc`
4. `status`: `function-phase-complete`

## 2. Signature and Admission Contract
1. arity:
   - minimum: `3`
   - maximum: `255`
2. admission policy:
   - admitted for `GROUPBY(row_fields, values, function[, field_headers[, total_depth[, sort_order[, filter_array[, field_relationship]]]]])`.
   - the third argument (`function`) is a callable (LAMBDA or built-in aggregation function).

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
1. `GROUPBY` groups `values` by unique entries in `row_fields`, then applies `function` to each group to produce an aggregated output array.
2. optional arguments control field headers, total depth, sort order, filtering, and field relationships.
3. the result is a dynamic array `EvalValue::Array` containing grouped and aggregated data.
4. OxFunc now has callable-backed runtime grouping on the declared current-baseline slice when a prepared callable value is supplied at the function boundary.
5. Current-baseline grouped aggregation includes the exercised totals, headers, filter, sort, and rejection lanes promoted in `W055`.

## 5. Version Scope (Required Axes)
1. Excel application version/channel scope:
   - Microsoft 365 current channel (reference baseline).
2. Workbook Compatibility Version scope:
   - `default`.

## 6. Proof/Implementation Obligations
1. Lean obligations:
   - metadata/alignment theorems for the grouped-aggregation surface profile.
   - executable grouped-aggregation substrate alignment for the admitted callable-backed grouping slice and its seeded exercised lanes.
2. Rust obligations:
   - arity validation and argument parsing for all admitted arities.
   - callable-backed grouping, total rows, filtering, subtotal handling, and sort/header lanes for the admitted current-baseline slice.
   - unit tests and adapter tests for the exercised grouped-aggregation lanes mapped in the local evidence packet.

## 7. Artifact Bindings
1. Rust: `crates/oxfunc_core/src/functions/groupby_fn.rs`
2. Lean: `formal/lean/OxFunc/Functions/GroupBy.lean`
3. grouped-aggregation substrate: `formal/lean/OxFunc/Functions/GroupedAggregation.lean`
4. historical packet chain:
   - `docs/HISTORY.md`
   - git tag `OxFunc_V1`
5. XLL: exported through the ordinary catalog surface.

## 8. Evidence Posture
1. public-reference anchors:
   - `docs/function-lane/FUNCTION_CATALOG_CURRENT_BASELINE_LOCAL.csv` row `203`
   - linked Microsoft support page `https://support.microsoft.com/en-us/office/groupby-function-5e08ae8c-6800-4b72-b623-c41773611505`
2. deterministic replay and exercised-runtime anchors:
   - `docs/function-lane/FUNCTION_LANE_EVIDENCE_ID_REGISTRY.md`
   - `docs/HISTORY.md`
   - `tools/w56-probe/run-w56-grouped-aggregation-baseline.ps1`
   - `crates/oxfunc_core/src/functions/groupby_fn.rs`
   - `crates/oxfunc_core/tests/oxfml_grouped_aggregation_adapter_integration.rs`
   - `..\OxFml\crates\oxfml_core\tests\w053_grouped_aggregation_adapter_tests.rs`
   - `..\OxFml\crates\oxfml_core\tests\fixtures\w053_grouped_aggregation_cases.json`
3. packet evidence anchor:
   - `W55-GROUPED-AGGREGATION-PROMOTION-20260331`
   - `W56-GROUPED-AGGREGATION-NATIVE-FORMAL-BL-20260331`
4. current status rationale:
   - `GROUPBY()` is no longer a stub or preview-only runtime lane,
   - the current reference-baseline callable-backed grouping slice is exercised across native Excel replay, OxFunc runtime tests, and OxFml adapter fixtures,
   - the Lean layer now includes an executable grouped-aggregation substrate with function-level example bindings for seeded admitted lanes,
   - no known semantic gap remains in the declared current-phase slice promoted by `W055`.

## 9. Current Coverage
1. OxFunc runtime coverage includes:
   - default callable-backed grouping,
   - visible-header output,
   - hierarchical subtotals,
   - filter-sensitive output,
   - descending aggregate sort,
   - tabular subtotal rejection.
2. OxFml adapter coverage includes:
   - inline `LAMBDA(x,SUM(x))` callable carriage,
   - bare built-in aggregation callable carriage via `SUM`,
   - visible-header, subtotal, filter, and sort-sensitive grouped-aggregation lanes,
   - grouped-aggregation runtime rejection for tabular subtotals.
3. Lean coverage for the declared current-phase slice includes the executable grouped-aggregation substrate in `formal/lean/OxFunc/Functions/GroupedAggregation.lean` plus function-level example bindings in `formal/lean/OxFunc/Functions/GroupBy.lean`.
4. no known semantic gap remains in the declared current-baseline slice for `GROUPBY`.
