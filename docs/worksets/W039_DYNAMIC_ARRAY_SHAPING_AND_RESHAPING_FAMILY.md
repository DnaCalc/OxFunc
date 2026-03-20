# WORKSET - Dynamic Array Shaping And Reshaping Family (W39)

## 1. Purpose
Open the dedicated packet for the remaining high-interest dynamic-array shaping and reshaping family.

This packet isolates spill-shape, orientation, reshaping, and selection semantics that are richer than ordinary scalar kernels but less helper-coupled than the lambda family.

## 2. Provenance
Opened after the interesting-function review identified the dynamic-array shaping cluster as the best next interesting in-core family after the lambda/helper packet.

Relevant supporting context:
1. `docs/function-lane/INTERESTING_FUNCTIONS_INITIAL_CLASSIFICATION.csv`
2. `docs/function-lane/FUNCTION_CATALOG_CURRENT_BASELINE_LOCAL.csv`
3. `docs/worksets/W012_MODERATE_FUNCTION_EXPANSION.md`
4. `docs/function-lane/FUNCTION_SLICE_HSTACK_CONTRACT_PRELIM.md`

## 3. Scope
Machine-readable inventory:
1. `docs/function-lane/W39_DYNAMIC_ARRAY_SHAPING_AND_RESHAPING_INVENTORY.csv`

Current total:
1. `15` functions.

Members:
1. `CHOOSECOLS`
2. `CHOOSEROWS`
3. `DROP`
4. `EXPAND`
5. `FILTER`
6. `SORT`
7. `SORTBY`
8. `TAKE`
9. `TOCOL`
10. `TOROW`
11. `TRANSPOSE`
12. `UNIQUE`
13. `VSTACK`
14. `WRAPCOLS`
15. `WRAPROWS`

## 4. Why This Packet Matters
1. These functions are central to modern Excel array semantics and spill-shape behavior.
2. They pressure omission/default handling, row/column orientation, array publication shape, and spill-aware errors.
3. They offer a high-value semantic packet without immediately requiring the full helper/lambda seam.
4. `HSTACK` is already closed and provides a useful precedent and comparison point for this family.

## 5. In Scope
1. empirical characterization of admitted current-baseline semantics for the family,
2. shared contract work for array shaping, selection, orientation, and publication rules,
3. runtime family implementation and dispatch/export wiring where missing,
4. Lean/formal alignment for the admitted array-shaping substrate,
5. scenario manifests, runtime requirements, execution record, and evidence registry rows.

## 6. Out Of Scope
1. helper/lambda-family functions from `W038`,
2. implicit intersection/operator placement from `W014`,
3. provider-bound data functions,
4. host/query and metadata functions.

## 7. Gate Criteria
This workset can only be reported `scope_complete` when:
1. all fifteen members have reproducible native worksheet evidence for the admitted slice,
2. the shared shaping/reshaping contract states the admitted row/column/orientation/default rules clearly,
3. runtime and export integration are complete for the admitted slice,
4. Lean/formal alignment for the primary array-shaping substrate is integrated,
5. no known function-semantic gap remains in declared current-baseline scope for the admitted slice.

## 8. Initial Status
1. execution_state: `in_progress`
2. scope_completeness: `scope_partial`
3. target_completeness: `target_partial`
4. integration_completeness: `partial`
5. open_lanes:
   - no packet-specific scenario manifest yet
   - no shaping/reshaping family contract yet
   - spill/publication edge matrix still unstarted
