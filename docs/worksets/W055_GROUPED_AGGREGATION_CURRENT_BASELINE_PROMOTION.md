# WORKSET - Grouped Aggregation Current-Baseline Promotion (W55)

## 1. Purpose
Promote `GROUPBY` and `PIVOTBY` out of the residual preview cluster by recording the current OxFunc-owned closure packet for their admitted current-baseline grouped-aggregation slice.

This packet exists to remove stale "promotion/documentation" ambiguity once the following are already true:
1. OxFunc has real callable-backed runtime kernels for both functions.
2. OxFunc unit tests cover the current admitted option surface.
3. OxFml `W053` has an exercised adapter/fixture floor for inline `LAMBDA(...)`, bare built-in aggregation carriage, header, subtotal, filter, sort, and rejection lanes.
4. no current cross-repo interface-shape disagreement remains on the grouped-aggregation seam.
5. `W056` closes the previously missing native Excel replay and executable Lean evidence lanes for the admitted current-baseline slice.

## 2. Position and Dependencies
Program position:
1. post-`W054` packet (`W055`).

Dependencies:
1. `W038_FUNCTIONAL_LAMBDA_AND_HELPER_FAMILY.md`
2. `W042_DEFERRED_CALLABLE_SEAM_FIELD_LOCK_AND_HIGHER_ORDER_EVIDENCE.md`
3. `W047_TYPED_CONTEXT_AND_QUERY_BUNDLE_FREEZE.md`
4. `W048_RETURN_SURFACE_AND_PUBLICATION_HINT_FREEZE.md`
5. `W049_RUNTIME_LIBRARY_CONTEXT_PROVIDER_CONSUMER_MODEL.md`
6. `W051_IN_SCOPE_CURRENT_VERSION_NOT_COMPLETE_SURFACE.md`
7. OxFml `W053_grouped_aggregation_and_publication_class_adapter_expansion.md`
8. `W056_GROUPED_AGGREGATION_NATIVE_AND_FORMAL_BASELINE.md`

## 3. Scope
In scope for `W055`:
1. `GROUPBY`
2. `PIVOTBY`
3. current-baseline callable-backed grouped aggregation where:
   - the callable slot admits inline helper lambdas and bare built-in aggregation carriage already exercised through OxFml,
   - the current runtime covers totals, filters, sort, visible-header output, and the seeded rejection lanes,
   - the functions are admitted through the ordinary export/dispatch/catalog surface.
4. contract, evidence, and backlog-tracker promotion needed to remove both rows from `W051`.

Out of scope for `W055`:
1. a one-shot broad closure of every future `GROUPBY` / `PIVOTBY` option combination not already exercised on the current baseline,
2. locale sweeps or alternate Excel-version/channel sweeps,
3. any new callable-carrier redesign,
4. any future mismatch-driven grouped-aggregation seam reopen on the OxFml side.

## 4. Deliverables
1. updated `GROUPBY` contract note
2. updated `PIVOTBY` contract note
3. `W055` execution record
4. evidence-registry entry for the grouped-aggregation promotion packet
5. `W051` inventory and downstream count cleanup removing `GROUPBY` / `PIVOTBY` from preview status
6. integrated `W056` evidence posture in the promotion reading

## 5. Gate Model
### G1 - Runtime Floor
Pass when:
1. OxFunc unit tests cover default grouped-aggregation, visible-header, subtotal/filter/sort, and seeded rejection lanes for the admitted current-baseline slice.

### G2 - Adapter Floor
Pass when:
1. OxFml has deterministic grouped-aggregation adapter coverage through the live parser/binder/preparation/evaluation path for both `GROUPBY` and `PIVOTBY`.

### G3 - Contract Promotion
Pass when:
1. both function-slice contracts state the declared current-baseline scope, evidence anchors, and no-known-gap reading for that scope,
2. `W051` and downstream consumer docs no longer classify those rows as preview backlog.
3. the promotion packet explicitly consumes the native Excel and executable Lean evidence added in `W056`.

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
1. none for the declared current-phase packet scope.

## 7. Current Completion Reading
1. `GROUPBY` and `PIVOTBY` are now complete for declared current-phase OxFunc scope.
2. the prior `W051` residual was stale packet-tracking rather than a missing runtime, missing adapter floor, or unresolved seam-shape issue.
3. broader future widening remains possible, but no known semantic gap remains in the declared current-baseline packet scope promoted here.
4. this completion reading is carried by the combined `W055` + `W056` packet posture rather than the thinner original `W055` artifact alone.
