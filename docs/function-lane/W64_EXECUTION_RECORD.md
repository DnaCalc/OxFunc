# W64 Execution Record - Financial Core Misc Family

Status: `complete`
Workset: `W064`
Evidence ID:
1. `W64-FINANCIAL-CORE-MISC-BL-20260401`

## 1. Purpose
Record the closure of the sixth ordinary successor packet after `W058`: the financial core miscellaneous family.

## 2. Scope
Artifacts created or updated:
1. `docs/worksets/W064_FINANCIAL_CORE_MISC_FAMILY.md`
2. `docs/function-lane/FUNCTION_SLICE_FINANCIAL_CORE_MISC_FAMILY_CONTRACT_PRELIM.md`
3. `docs/function-lane/W64_SCENARIO_MANIFEST_SEED.csv`
4. `docs/function-lane/W64_RUNTIME_REQUIREMENTS.md`
5. `docs/function-lane/W64_SCOPE_RECONCILIATION.csv`
6. `docs/function-lane/W64_EXECUTION_RECORD.md`
7. `tools/w64-probe/run-w64-financial-core-misc-baseline.ps1`
8. `.tmp/w64-financial-core-misc-results.csv`
9. `tools/w44-probe/generate-w44-library-context-snapshot.ps1`
10. `docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1.csv`
11. `docs/function-lane/W51_HIDDEN_NON_DEFERRED_BACKLOG_CURRENT.csv`
12. `docs/function-lane/W51_NORMALIZED_ORDINARY_BACKLOG_CURRENT.csv`
13. `docs/worksets/W051_IN_SCOPE_CURRENT_VERSION_NOT_COMPLETE_SURFACE.md`
14. `docs/function-lane/OXFUNC_SURFACE_ADMISSION_AND_LABELING_POLICY.md`
15. `docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1_README.md`
16. `docs/IN_PROGRESS_FEATURE_WORKLIST.md`
17. `docs/worksets/README.md`
18. `docs/worksets/W057_HIDDEN_ORDINARY_BACKLOG_SYSTEMATIC_COMPLETION_PLAN.md`
19. `docs/function-lane/FUNCTION_LANE_EVIDENCE_ID_REGISTRY.md`
20. `docs/function-lane/README.md`

## 3. Completeness Axes
1. execution_state: `complete`
2. scope_completeness: `scope_complete`
3. target_completeness: `target_complete`
4. integration_completeness: `integrated`

## 4. Packet Result
1. The financial core miscellaneous family is now closure-grade on the current reference Excel baseline across runtime, native replay, Lean alignment, and snapshot/export promotion.
2. The regenerated snapshot now exposes all `15` `W064` rows with real metadata as `function_meta_curated`.
3. No `W064` row remains `catalog_only` in the published snapshot export.
4. `W051` now drops by:
   - `15` snapshot-entry backlog rows (`61 -> 46`),
   - `15` normalized execution rows (`68 -> 53`).
5. `W064` introduces no new active `W054` Lean gap row.

## 5. Evidence Posture
Public-reference anchors:
1. `docs/function-lane/FUNCTION_CATALOG_CURRENT_BASELINE_LOCAL.csv`

Runtime / dispatch anchors:
1. `crates/oxfunc_core/src/functions/cumulative_finance_family.rs`
2. `crates/oxfunc_core/src/functions/depreciation_family.rs`
3. `crates/oxfunc_core/src/functions/discount_bill_yearfrac_family.rs`
4. `crates/oxfunc_core/src/functions/dollar_fraction_family.rs`
5. `crates/oxfunc_core/src/functions/surface_dispatch.rs`
6. `tools/xll-addin/oxfunc_xll/export_specs.csv`

Formal anchors:
1. `formal/lean/OxFunc/Functions/CumulativeFinanceFamily.lean`
2. `formal/lean/OxFunc/Functions/DepreciationFamily.lean`
3. `formal/lean/OxFunc/Functions/DiscountBillYearfracFamily.lean`
4. `formal/lean/OxFunc/Functions/DollarFractionFamily.lean`

Native replay anchors:
1. `docs/function-lane/W64_SCENARIO_MANIFEST_SEED.csv`
2. `docs/function-lane/W64_RUNTIME_REQUIREMENTS.md`
3. `tools/w64-probe/run-w64-financial-core-misc-baseline.ps1`
4. `.tmp/w64-financial-core-misc-results.csv`

Export-promotion anchors:
1. `tools/w44-probe/generate-w44-library-context-snapshot.ps1`
2. `docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1.csv`

## 6. Verification Runs
1. `powershell -ExecutionPolicy Bypass -File tools/w64-probe/run-w64-financial-core-misc-baseline.ps1`
2. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml cumulative_finance_family -- --nocapture`
3. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml depreciation_family -- --nocapture`
4. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml discount_bill_yearfrac_family -- --nocapture`
5. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml dollar_fraction_family -- --nocapture`
6. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml all_catalog_functions_have_at_least_one_export -- --nocapture`
7. `cargo fmt --manifest-path crates/oxfunc_core/Cargo.toml --all --check`
8. `cargo check --manifest-path tools/xll-addin/oxfunc_xll/Cargo.toml`
9. `lake build`
10. `powershell -ExecutionPolicy Bypass -File tools/w44-probe/generate-w44-library-context-snapshot.ps1`

## 7. Standing
1. `W064` is complete for the declared current-baseline packet scope after native replay, targeted Rust tests, Lean build, export promotion, and backlog reconciliation all passed.
2. `W064` closes the financial core miscellaneous family for the declared current-baseline slice.
3. The remaining ordinary backlog now starts at `W065`.
4. No known semantic gap remains in the declared current-baseline scope for the `15` covered rows.
