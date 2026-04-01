# WORKSET - Grouped Aggregation Native And Formal Baseline (W56)

## 1. Purpose
Close the missing native Excel replay and stronger Lean executable-model lanes for the current grouped-aggregation packet that promotes `GROUPBY` and `PIVOTBY`.

This packet exists because the earlier `W055` promotion was runtime- and adapter-backed, but did not yet carry:
1. a native Excel replay manifest and emitted results for representative grouped-aggregation formulas, or
2. an executable Lean grouped-aggregation substrate beyond metadata-only theorems.

## 2. Position and Dependencies
Program position:
1. post-`W055` packet (`W056`).

Dependencies:
1. `W055_GROUPED_AGGREGATION_CURRENT_BASELINE_PROMOTION.md`
2. OxFml `W053_grouped_aggregation_and_publication_class_adapter_expansion.md`
3. `FORMALIZATION_STRATEGY_EXECUTABLE_SEMANTIC_MODEL.md`
4. `formal/lean/OxFunc/Functions/GroupedAggregation.lean`

## 3. Scope
In scope for `W056`:
1. native Excel replay for the admitted current-baseline grouped-aggregation slice of:
   - `GROUPBY`
   - `PIVOTBY`
2. sum-backed grouped-aggregation executable-model alignment in Lean for representative:
   - default grouping/pivoting,
   - hierarchical subtotal,
   - filter/sort-sensitive lanes,
   - row/column-total sort lanes.
3. propagation of that evidence into the function contracts and grouped-aggregation packet records.

Out of scope for `W056`:
1. full future `GROUPBY` / `PIVOTBY` option-matrix closure,
2. locale sweeps or alternate Excel-version/channel sweeps,
3. callable-carrier redesign,
4. publication/UI behavior beyond worksheet-observable grouped-aggregation results.

## 4. Deliverables
1. `W056` scenario manifest
2. `W056` runtime requirements
3. native Excel probe runner and emitted results
4. executable Lean grouped-aggregation substrate and function-binding examples
5. `W056` execution record
6. grouped-aggregation contract/evidence updates consuming the new artifacts

## 5. Gate Model
### G1 - Native Replay
Pass when:
1. native Excel replay runs through COM on the local host,
2. the seeded `GROUPBY` / `PIVOTBY` rows match expected worksheet-observable outputs.

### G2 - Executable Lean Model
Pass when:
1. Lean computes representative grouped-aggregation outcomes for the admitted sum-backed slice,
2. function-level `GROUPBY` / `PIVOTBY` bindings reference those executable examples rather than metadata alone.

### G3 - Packet Integration
Pass when:
1. `GROUPBY` and `PIVOTBY` contracts cite the native replay and executable Lean artifacts,
2. the grouped-aggregation promotion packet no longer lacks those two evidence lanes.
3. `W055` reads as supported only through the combined `W055` + `W056` evidence posture rather than the earlier thinner packet alone.

## 6. Current Status
Execution state:
1. `complete`

Claim confidence:
1. `provisional`

Assurance maturity:
1. `exercised`

Completeness axes:
1. `scope_completeness`: `scope_complete`
2. `target_completeness`: `target_complete`
3. `integration_completeness`: `integrated`

Open lanes:
1. none in declared `W056` scope.

## 7. Completion Reading
1. `W056` supplies the previously missing native Excel replay baseline for `GROUPBY` and `PIVOTBY` through the seeded `11`-scenario manifest and emitted COM-run results.
2. `W056` also supplies the previously missing executable Lean grouped-aggregation substrate and function-level example bindings for the admitted current-baseline sum-backed slice.
3. `W055` promotion claims are therefore now grounded in native Excel replay, OxFunc runtime tests, OxFml adapter coverage, and executable Lean alignment together.
