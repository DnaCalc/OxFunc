# W56 Execution Record - Grouped Aggregation Native And Formal Baseline

Status: `complete`
Workset: `W56`
Evidence ID:
1. `W56-GROUPED-AGGREGATION-NATIVE-FORMAL-BL-20260331`

## 1. Purpose
Close the two evidence lanes that were still missing from the earlier grouped-aggregation promotion packet:
1. native Excel replay for representative `GROUPBY` / `PIVOTBY` formulas,
2. executable Lean grouped-aggregation alignment beyond metadata-only theorems.

## 2. Scope
Artifacts created or updated:
1. `docs/worksets/W056_GROUPED_AGGREGATION_NATIVE_AND_FORMAL_BASELINE.md`
2. `docs/function-lane/W56_GROUPED_AGGREGATION_SCENARIO_MANIFEST_SEED.csv`
3. `docs/function-lane/W56_GROUPED_AGGREGATION_RUNTIME_REQUIREMENTS.md`
4. `docs/function-lane/W56_EXECUTION_RECORD.md`
5. `tools/w56-probe/run-w56-grouped-aggregation-baseline.ps1`
6. `formal/lean/OxFunc/Functions/GroupedAggregation.lean`
7. `formal/lean/OxFunc/Functions/GroupBy.lean`
8. `formal/lean/OxFunc/Functions/PivotBy.lean`
9. `docs/function-lane/FUNCTION_SLICE_GROUPBY_CONTRACT_PRELIM.md`
10. `docs/function-lane/FUNCTION_SLICE_PIVOTBY_CONTRACT_PRELIM.md`
11. `docs/worksets/W055_GROUPED_AGGREGATION_CURRENT_BASELINE_PROMOTION.md`
12. `docs/function-lane/W55_EXECUTION_RECORD.md`
13. `docs/function-lane/FUNCTION_LANE_EVIDENCE_ID_REGISTRY.md`
14. `docs/worksets/README.md`
15. `docs/function-lane/README.md`

## 3. Completeness Axes
1. execution_state: `complete`
2. scope_completeness: `scope_complete`
3. target_completeness: `target_complete`
4. integration_completeness: `integrated`

## 4. Packet Result
1. Native Excel replay now exists for the admitted current-baseline grouped-aggregation slice through `11` seeded COM-driven scenarios.
2. All `11` seeded rows matched expected worksheet-observable outputs on the local Excel baseline (`Version 16.0`, product version `16.0.19822.20114`).
3. Lean now contains an executable grouped-aggregation substrate for ordered grouping, subtotal rendering, filter-sensitive sorting, and pivot-table rendering on the admitted sum-backed slice.
4. `GroupBy.lean` and `PivotBy.lean` now bind function-level example theorems to executable grouped-aggregation examples rather than metadata alone.
5. The earlier `W055` supported-status claim is now backed by native Excel replay and executable Lean alignment in addition to runtime and OxFml adapter evidence.

## 5. Evidence Posture
Public-reference anchors:
1. `docs/function-lane/FUNCTION_CATALOG_CURRENT_BASELINE_LOCAL.csv` rows for `GROUPBY` and `PIVOTBY`, including linked Microsoft support pages.

Deterministic replay and exercised-runtime anchors:
1. `docs/function-lane/W56_GROUPED_AGGREGATION_SCENARIO_MANIFEST_SEED.csv`
2. `docs/function-lane/W56_GROUPED_AGGREGATION_RUNTIME_REQUIREMENTS.md`
3. `tools/w56-probe/run-w56-grouped-aggregation-baseline.ps1`
4. `.tmp/w56-grouped-aggregation-results.csv`
5. `crates/oxfunc_core/src/functions/groupby_fn.rs`
6. `crates/oxfunc_core/src/functions/pivotby_fn.rs`
7. `crates/oxfunc_core/tests/oxfml_grouped_aggregation_adapter_integration.rs`
8. `..\OxFml\crates\oxfml_core\tests\w053_grouped_aggregation_adapter_tests.rs`
9. `..\OxFml\crates\oxfml_core\tests\fixtures\w053_grouped_aggregation_cases.json`

Formal-alignment anchors:
1. `formal/lean/OxFunc/Functions/GroupedAggregation.lean`
2. `formal/lean/OxFunc/Functions/GroupBy.lean`
3. `formal/lean/OxFunc/Functions/PivotBy.lean`

Verification seam qualifier:
1. `docs/function-lane/XLL_VERIFICATION_SEAM_LIMITATIONS.md`

## 6. Verification Runs
1. `powershell -ExecutionPolicy Bypass -File tools/w56-probe/run-w56-grouped-aggregation-baseline.ps1`
2. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml groupby_ -- --nocapture`
3. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml pivotby_ -- --nocapture`
4. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --test oxfml_grouped_aggregation_adapter_integration -- --nocapture`
5. `lake build`

## 7. Standing
1. `W056` closes the previously missing native Excel replay lane for the admitted grouped-aggregation slice.
2. `W056` closes the previously missing executable Lean substrate lane for the admitted grouped-aggregation slice.
3. `GROUPBY` and `PIVOTBY` may therefore continue to stand as `function-phase-complete` for the declared current reference baseline slice without doctrine drift on those two evidence axes.
