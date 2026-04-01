# W62 Execution Record - Statistical Distributions And Compat B

Status: `complete`
Workset: `W062`
Evidence ID:
1. `W62-STAT-DIST-COMPAT-B-BL-20260401`

## 1. Purpose
Record the closure of the fourth ordinary successor packet after `W058`: the second statistical-distribution and compatibility wave.

## 2. Scope
Artifacts created or updated:
1. `docs/worksets/W062_STATISTICAL_DISTRIBUTIONS_AND_COMPAT_B.md`
2. `docs/function-lane/FUNCTION_SLICE_STATISTICAL_DISTRIBUTIONS_AND_COMPAT_B_CONTRACT_PRELIM.md`
3. `docs/function-lane/W62_SCENARIO_MANIFEST_SEED.csv`
4. `docs/function-lane/W62_RUNTIME_REQUIREMENTS.md`
5. `docs/function-lane/W62_SCOPE_RECONCILIATION.csv`
6. `docs/function-lane/W62_EXECUTION_RECORD.md`
7. `tools/w62-probe/run-w62-statistical-distributions-compat-b-baseline.ps1`
8. `.tmp/w62-statistical-distributions-compat-b-results.csv`
9. `formal/lean/OxFunc/Functions/ChiFTFamily.lean`
10. `docs/function-lane/W54_LEAN_FORMALIZATION_GAP_INVENTORY.csv`
11. `docs/worksets/W054_LEAN_FORMALIZATION_GAP_RECONCILIATION.md`
12. `tools/w44-probe/generate-w44-library-context-snapshot.ps1`
13. `docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1.csv`
14. `docs/function-lane/W51_HIDDEN_NON_DEFERRED_BACKLOG_CURRENT.csv`
15. `docs/function-lane/W51_NORMALIZED_ORDINARY_BACKLOG_CURRENT.csv`
16. `docs/worksets/W051_IN_SCOPE_CURRENT_VERSION_NOT_COMPLETE_SURFACE.md`
17. `docs/function-lane/OXFUNC_SURFACE_ADMISSION_AND_LABELING_POLICY.md`
18. `docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1_README.md`
19. `docs/IN_PROGRESS_FEATURE_WORKLIST.md`
20. `docs/worksets/README.md`
21. `docs/worksets/W057_HIDDEN_ORDINARY_BACKLOG_SYSTEMATIC_COMPLETION_PLAN.md`
22. `docs/function-lane/FUNCTION_LANE_EVIDENCE_ID_REGISTRY.md`
23. `docs/function-lane/README.md`

## 3. Completeness Axes
1. execution_state: `complete`
2. scope_completeness: `scope_complete`
3. target_completeness: `target_complete`
4. integration_completeness: `integrated`

## 4. Packet Result
1. The second statistical-distribution and compatibility wave is now closure-grade on the current reference Excel baseline across runtime, native replay, Lean alignment, and snapshot/export promotion.
2. The regenerated snapshot now exposes all `35` `W062` rows with real metadata as `function_meta_curated`.
3. No `W062` row remains `catalog_only` in the published snapshot export.
4. `W051` now drops by:
   - `35` snapshot-entry backlog rows (`114 -> 79`),
   - `35` normalized execution rows (`121 -> 86`).
5. `W054` now drops the corresponding active Lean gap rows from `15` to `10`.

## 5. Evidence Posture
Public-reference anchors:
1. `docs/function-lane/FUNCTION_CATALOG_CURRENT_BASELINE_LOCAL.csv`

Runtime / dispatch anchors:
1. `crates/oxfunc_core/src/functions/chi_f_t_family.rs`
2. `crates/oxfunc_core/src/functions/discrete_dist_family.rs`
3. `crates/oxfunc_core/src/functions/normal_log_family.rs`
4. `crates/oxfunc_core/src/functions/legacy_stats_alias_family.rs`
5. `crates/oxfunc_core/src/functions/moment_stats_family.rs`
6. `crates/oxfunc_core/src/functions/surface_dispatch.rs`
7. `tools/xll-addin/oxfunc_xll/export_specs.csv`

Formal anchors:
1. `formal/lean/OxFunc/Functions/ChiFTFamily.lean`
2. `formal/lean/OxFunc/Functions/DiscreteDistFamily.lean`
3. `formal/lean/OxFunc/Functions/NormalLogFamily.lean`
4. `formal/lean/OxFunc/Functions/LegacyStatsAliasFamily.lean`
5. `formal/lean/OxFunc/Functions/MomentStatsFamily.lean`

Native replay anchors:
1. `docs/function-lane/W62_SCENARIO_MANIFEST_SEED.csv`
2. `docs/function-lane/W62_RUNTIME_REQUIREMENTS.md`
3. `tools/w62-probe/run-w62-statistical-distributions-compat-b-baseline.ps1`
4. `.tmp/w62-statistical-distributions-compat-b-results.csv`

Export-promotion anchors:
1. `tools/w44-probe/generate-w44-library-context-snapshot.ps1`
2. `docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1.csv`

## 6. Verification Runs
1. `powershell -ExecutionPolicy Bypass -File tools/w62-probe/run-w62-statistical-distributions-compat-b-baseline.ps1`
2. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml chi_f_t_family -- --nocapture`
3. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml normal_log_family -- --nocapture`
4. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml discrete_dist_family -- --nocapture`
5. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml legacy_stats_alias_family -- --nocapture`
6. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml moment_stats_family -- --nocapture`
7. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml all_catalog_functions_have_at_least_one_export -- --nocapture`
8. `cargo check --manifest-path tools/xll-addin/oxfunc_xll/Cargo.toml`
9. `cargo fmt --manifest-path crates/oxfunc_core/Cargo.toml --all --check`
10. `lake build`
11. `powershell -ExecutionPolicy Bypass -File tools/w44-probe/generate-w44-library-context-snapshot.ps1`

## 7. Standing
1. `W062` closes the second statistical-distribution and compatibility wave for the declared current-baseline slice.
2. The remaining ordinary backlog now starts at `W063`.
3. No known semantic gap remains in the declared current-baseline scope for the `35` covered rows.
