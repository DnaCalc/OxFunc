# W55 Execution Record - Grouped Aggregation Current-Baseline Promotion

Status: `complete`
Workset: `W55`
Evidence ID:
1. `W55-GROUPED-AGGREGATION-PROMOTION-20260331`

## 1. Purpose
Record the OxFunc-side promotion packet that removes `GROUPBY` and `PIVOTBY` from the residual preview cluster after the callable seam freeze, OxFml `W053` adapter expansion, and the follow-on `W056` native/formal hardening pass.

## 2. Scope
Artifacts created or updated:
1. `docs/worksets/W055_GROUPED_AGGREGATION_CURRENT_BASELINE_PROMOTION.md`
2. `docs/function-lane/W55_EXECUTION_RECORD.md`
3. `docs/function-lane/FUNCTION_SLICE_GROUPBY_CONTRACT_PRELIM.md`
4. `docs/function-lane/FUNCTION_SLICE_PIVOTBY_CONTRACT_PRELIM.md`
5. `docs/function-lane/FUNCTION_LANE_EVIDENCE_ID_REGISTRY.md`
6. `docs/function-lane/W51_IN_SCOPE_CURRENT_VERSION_NOT_COMPLETE_INVENTORY.csv`
7. `docs/function-lane/W51_INTERESTING_POST_FREEZE_LOCAL_WORK.csv`
8. `docs/worksets/W051_IN_SCOPE_CURRENT_VERSION_NOT_COMPLETE_SURFACE.md`
9. `docs/function-lane/OXFUNC_SURFACE_ADMISSION_AND_LABELING_POLICY.md`
10. `docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1_README.md`
11. `docs/IN_PROGRESS_FEATURE_WORKLIST.md`
12. `docs/worksets/README.md`
13. `CURRENT_BLOCKERS.md`
14. `docs/worksets/W056_GROUPED_AGGREGATION_NATIVE_AND_FORMAL_BASELINE.md`
15. `docs/function-lane/W56_EXECUTION_RECORD.md`
16. `docs/function-lane/W56_GROUPED_AGGREGATION_SCENARIO_MANIFEST_SEED.csv`
17. `docs/function-lane/W56_GROUPED_AGGREGATION_RUNTIME_REQUIREMENTS.md`
18. `tools/w56-probe/run-w56-grouped-aggregation-baseline.ps1`
19. `formal/lean/OxFunc/Functions/GroupedAggregation.lean`

## 3. Completeness Axes
1. execution_state: `complete`
2. scope_completeness: `scope_complete`
3. target_completeness: `target_complete`
4. integration_completeness: `integrated`

## 4. Packet Result
1. `GROUPBY` now has a closure-grade OxFunc runtime floor for the declared current-baseline slice:
   - callable-backed default grouping,
   - hierarchical subtotals,
   - visible-header output,
   - filter-sensitive and sort-sensitive lanes,
   - tabular-subtotal rejection.
2. `PIVOTBY` now has a closure-grade OxFunc runtime floor for the declared current-baseline slice:
   - callable-backed default pivoting,
   - visible-header band output,
   - zero-total/filter-sensitive lanes,
   - row/column-total sort lanes.
3. OxFml `W053` now provides deterministic adapter evidence for:
   - inline `LAMBDA(x,SUM(x))` carriage,
   - bare built-in aggregation carriage via `SUM`,
   - grouped-aggregation bind/reject and runtime-reject consequences,
   - end-to-end parser/binder/preparation/evaluation coverage through the live seam.
4. the previous `W051` status for `GROUPBY` / `PIVOTBY` is now resolved as packet-tracking lag rather than a live semantic or seam blocker.
5. `W056` now adds native Excel replay and executable Lean grouped-aggregation evidence for the same admitted current-baseline slice.
6. `GROUPBY` and `PIVOTBY` are therefore promoted out of the preview cluster and into the supported bucket for the current reference baseline.

## 5. Evidence Posture
Public-reference anchors:
1. `docs/function-lane/FUNCTION_CATALOG_CURRENT_BASELINE_LOCAL.csv` rows for `GROUPBY` and `PIVOTBY`, including the linked Microsoft support pages.

Deterministic replay and exercised-runtime anchors:
1. OxFunc runtime tests in `crates/oxfunc_core/src/functions/groupby_fn.rs`
2. OxFunc runtime tests in `crates/oxfunc_core/src/functions/pivotby_fn.rs`
3. OxFunc adapter integration tests in `crates/oxfunc_core/tests/oxfml_grouped_aggregation_adapter_integration.rs`
4. OxFml grouped-aggregation adapter corpus in `..\OxFml\crates\oxfml_core\tests\w053_grouped_aggregation_adapter_tests.rs`
5. OxFml grouped-aggregation fixture corpus in `..\OxFml\crates\oxfml_core\tests\fixtures\w053_grouped_aggregation_cases.json`
6. OxFml local owner packet `..\OxFml\docs\worksets\W053_grouped_aggregation_and_publication_class_adapter_expansion.md`
7. native Excel replay manifest in `docs/function-lane/W56_GROUPED_AGGREGATION_SCENARIO_MANIFEST_SEED.csv`
8. native Excel replay runner in `tools/w56-probe/run-w56-grouped-aggregation-baseline.ps1`
9. native Excel replay results in `.tmp/w56-grouped-aggregation-results.csv`

Formal-alignment anchors:
1. `formal/lean/OxFunc/Functions/GroupedAggregation.lean`
2. `formal/lean/OxFunc/Functions/GroupBy.lean`
3. `formal/lean/OxFunc/Functions/PivotBy.lean`

Verification seam qualifier:
1. `docs/function-lane/XLL_VERIFICATION_SEAM_LIMITATIONS.md`

## 6. Verification Runs
1. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml groupby_ -- --nocapture`
2. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml pivotby_ -- --nocapture`
3. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --test oxfml_grouped_aggregation_adapter_integration -- --nocapture`
4. `cargo test --manifest-path ..\OxFml\crates\oxfml_core\Cargo.toml --test w053_grouped_aggregation_adapter_tests -- --nocapture`
5. `cargo test --manifest-path ..\OxFml\crates\oxfml_core\Cargo.toml --test w053_grouped_aggregation_fixture_tests -- --nocapture`
6. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml all_catalog_functions_have_at_least_one_export -- --nocapture`
7. `lake build`
8. `powershell -ExecutionPolicy Bypass -File tools/w56-probe/run-w56-grouped-aggregation-baseline.ps1`

## 7. Standing
1. `GROUPBY` and `PIVOTBY` are now function-phase-complete for the declared current reference Excel baseline on the admitted current-phase grouped-aggregation slice.
2. no known semantic gap remains in that declared slice.
3. no live OxFml/OxFunc interface-shape disagreement remains on the grouped-aggregation seam.
4. native Excel replay and executable Lean grouped-aggregation evidence are now part of the supporting closure packet rather than missing qualifiers.
5. future grouped-aggregation widening is now mismatch-driven follow-on work rather than a blocker to current supported status.
