# W61 Execution Record - Statistical Distributions And Compat A

Status: `complete`
Workset: `W061`
Evidence ID:
1. `W61-STAT-DIST-COMPAT-A-BL-20260401`

## 1. Purpose
Record the closure of the third ordinary successor packet after `W058`: the first statistical-distribution and compatibility wave.

## 2. Scope
Artifacts created or updated:
1. `docs/worksets/W061_STATISTICAL_DISTRIBUTIONS_AND_COMPAT_A.md`
2. `docs/function-lane/FUNCTION_SLICE_STATISTICAL_DISTRIBUTIONS_AND_COMPAT_A_CONTRACT_PRELIM.md`
3. `docs/function-lane/W61_SCENARIO_MANIFEST_SEED.csv`
4. `docs/function-lane/W61_RUNTIME_REQUIREMENTS.md`
5. `docs/function-lane/W61_SCOPE_RECONCILIATION.csv`
6. `docs/function-lane/W61_EXECUTION_RECORD.md`
7. `tools/w61-probe/run-w61-statistical-distributions-compat-a-baseline.ps1`
8. `.tmp/w61-statistical-distributions-compat-a-results.csv`
9. `formal/lean/OxFunc/Functions/BetaGammaStatsFamily.lean`
10. `formal/lean/OxFunc/Functions/ChiFTFamily.lean`
11. `formal/lean/OxFunc.lean`
12. `docs/function-lane/W54_LEAN_FORMALIZATION_GAP_INVENTORY.csv`
13. `docs/worksets/W054_LEAN_FORMALIZATION_GAP_RECONCILIATION.md`
14. `tools/w44-probe/generate-w44-library-context-snapshot.ps1`
15. `docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1.csv`
16. `docs/function-lane/W51_HIDDEN_NON_DEFERRED_BACKLOG_CURRENT.csv`
17. `docs/function-lane/W51_NORMALIZED_ORDINARY_BACKLOG_CURRENT.csv`
18. `docs/worksets/W051_IN_SCOPE_CURRENT_VERSION_NOT_COMPLETE_SURFACE.md`
19. `docs/function-lane/OXFUNC_SURFACE_ADMISSION_AND_LABELING_POLICY.md`
20. `docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1_README.md`
21. `docs/IN_PROGRESS_FEATURE_WORKLIST.md`
22. `docs/worksets/README.md`
23. `docs/worksets/W057_HIDDEN_ORDINARY_BACKLOG_SYSTEMATIC_COMPLETION_PLAN.md`
24. `docs/function-lane/FUNCTION_LANE_EVIDENCE_ID_REGISTRY.md`
25. `docs/function-lane/README.md`

## 3. Completeness Axes
1. execution_state: `complete`
2. scope_completeness: `scope_complete`
3. target_completeness: `target_complete`
4. integration_completeness: `integrated`

## 4. Packet Result
1. The first statistical-distribution and compatibility wave is now closure-grade on the current reference Excel baseline across runtime, native replay, Lean alignment, and snapshot/export promotion.
2. The regenerated snapshot now exposes all `29` `W061` rows with real metadata as `function_meta_curated`.
3. No `W061` row remains `catalog_only` in the published snapshot export.
4. `W051` now drops by:
   - `29` snapshot-entry backlog rows (`143 -> 114`),
   - `29` normalized execution rows (`150 -> 121`).
5. `W054` now drops the corresponding active Lean gap rows from `32` to `15`.

## 5. Evidence Posture
Public-reference anchors:
1. `docs/function-lane/FUNCTION_CATALOG_CURRENT_BASELINE_LOCAL.csv`

Runtime / dispatch anchors:
1. `crates/oxfunc_core/src/functions/beta_gamma_stats_family.rs`
2. `crates/oxfunc_core/src/functions/discrete_dist_family.rs`
3. `crates/oxfunc_core/src/functions/chi_f_t_family.rs`
4. `crates/oxfunc_core/src/functions/normal_log_family.rs`
5. `crates/oxfunc_core/src/functions/surface_dispatch.rs`
6. `tools/xll-addin/oxfunc_xll/export_specs.csv`

Formal anchors:
1. `formal/lean/OxFunc/Functions/BetaGammaStatsFamily.lean`
2. `formal/lean/OxFunc/Functions/DiscreteDistFamily.lean`
3. `formal/lean/OxFunc/Functions/ChiFTFamily.lean`
4. `formal/lean/OxFunc/Functions/NormalLogFamily.lean`

Native replay anchors:
1. `docs/function-lane/W61_SCENARIO_MANIFEST_SEED.csv`
2. `docs/function-lane/W61_RUNTIME_REQUIREMENTS.md`
3. `tools/w61-probe/run-w61-statistical-distributions-compat-a-baseline.ps1`
4. `.tmp/w61-statistical-distributions-compat-a-results.csv`

Export-promotion anchors:
1. `tools/w44-probe/generate-w44-library-context-snapshot.ps1`
2. `docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1.csv`

## 6. Verification Runs
1. `powershell -ExecutionPolicy Bypass -File tools/w61-probe/run-w61-statistical-distributions-compat-a-baseline.ps1`
2. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml beta_gamma_stats_family -- --nocapture`
3. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml discrete_dist_family -- --nocapture`
4. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml chi_f_t_family -- --nocapture`
5. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml normal_log_family -- --nocapture`
6. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml all_catalog_functions_have_at_least_one_export -- --nocapture`
7. `cargo check --manifest-path tools/xll-addin/oxfunc_xll/Cargo.toml`
8. `lake build`
9. `powershell -ExecutionPolicy Bypass -File tools/w44-probe/generate-w44-library-context-snapshot.ps1`

## 7. Standing
1. `W061` closes the first statistical-distribution and compatibility wave for the declared current-baseline slice.
2. The remaining ordinary backlog now starts at `W062`.
3. No known semantic gap remains in the declared current-baseline scope for the `29` covered rows.
