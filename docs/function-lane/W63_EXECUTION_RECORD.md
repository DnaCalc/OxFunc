# W63 Execution Record - Date Time And Business Day Family

Status: `complete`
Workset: `W063`
Evidence ID:
1. `W63-DATE-TIME-BUSINESS-DAY-BL-20260401`

## 1. Purpose
Record the closure of the fifth ordinary successor packet after `W058`: the date/time and business-day family.

## 2. Scope
Artifacts created or updated:
1. `docs/worksets/W063_DATE_TIME_AND_BUSINESS_DAY_FAMILY.md`
2. `docs/function-lane/FUNCTION_SLICE_DATE_TIME_AND_BUSINESS_DAY_FAMILY_CONTRACT_PRELIM.md`
3. `docs/function-lane/W63_SCENARIO_MANIFEST_SEED.csv`
4. `docs/function-lane/W63_RUNTIME_REQUIREMENTS.md`
5. `docs/function-lane/W63_SCOPE_RECONCILIATION.csv`
6. `docs/function-lane/W63_EXECUTION_RECORD.md`
7. `tools/w63-probe/run-w63-date-time-business-day-baseline.ps1`
8. `.tmp/w63-date-time-business-day-results.csv`
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
1. The date/time and business-day family is now closure-grade on the current reference Excel baseline across runtime, native replay, Lean alignment, and snapshot/export promotion.
2. The regenerated snapshot now exposes all `18` `W063` rows with real metadata as `function_meta_curated`.
3. No `W063` row remains `catalog_only` in the published snapshot export.
4. `W051` now drops by:
   - `18` snapshot-entry backlog rows (`79 -> 61`),
   - `18` normalized execution rows (`86 -> 68`).
5. `W063` introduces no new active `W054` Lean gap row.

## 5. Evidence Posture
Public-reference anchors:
1. `docs/function-lane/FUNCTION_CATALOG_CURRENT_BASELINE_LOCAL.csv`

Runtime / dispatch anchors:
1. `crates/oxfunc_core/src/functions/date_parts_family.rs`
2. `crates/oxfunc_core/src/functions/date_week_family.rs`
3. `crates/oxfunc_core/src/functions/workday_networkdays_family.rs`
4. `crates/oxfunc_core/src/functions/discount_bill_yearfrac_family.rs`
5. `crates/oxfunc_core/src/functions/surface_dispatch.rs`
6. `tools/xll-addin/oxfunc_xll/export_specs.csv`

Formal anchors:
1. `formal/lean/OxFunc/Functions/DatePartsFamily.lean`
2. `formal/lean/OxFunc/Functions/DateWeekFamily.lean`
3. `formal/lean/OxFunc/Functions/WorkdayNetworkdaysFamily.lean`
4. `formal/lean/OxFunc/Functions/DiscountBillYearfracFamily.lean`

Native replay anchors:
1. `docs/function-lane/W63_SCENARIO_MANIFEST_SEED.csv`
2. `docs/function-lane/W63_RUNTIME_REQUIREMENTS.md`
3. `tools/w63-probe/run-w63-date-time-business-day-baseline.ps1`
4. `.tmp/w63-date-time-business-day-results.csv`

Export-promotion anchors:
1. `tools/w44-probe/generate-w44-library-context-snapshot.ps1`
2. `docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1.csv`

## 6. Verification Runs
1. `powershell -ExecutionPolicy Bypass -File tools/w63-probe/run-w63-date-time-business-day-baseline.ps1`
2. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml date_parts_family -- --nocapture`
3. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml date_week_family -- --nocapture`
4. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml workday_networkdays_family -- --nocapture`
5. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml discount_bill_yearfrac_family -- --nocapture`
6. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml all_catalog_functions_have_at_least_one_export -- --nocapture`
7. `cargo check --manifest-path tools/xll-addin/oxfunc_xll/Cargo.toml`
8. `lake build`
9. `powershell -ExecutionPolicy Bypass -File tools/w44-probe/generate-w44-library-context-snapshot.ps1`

## 7. Standing
1. `W063` closes the date/time and business-day family for the declared current-baseline slice.
2. The remaining ordinary backlog now starts at `W064`.
3. No known semantic gap remains in the declared current-baseline scope for the `18` covered rows.
