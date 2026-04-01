# W66 Execution Record - Text Core And Compatibility Family

Status: `complete`
Workset: `W066`
Evidence ID:
1. `W66-TEXT-CORE-COMPAT-BL-20260401`

## 1. Purpose
Record the closure of the eighth ordinary successor packet after `W058`: the text core and compatibility family promotion packet.

## 2. Scope
Artifacts created or updated:
1. `docs/worksets/W066_TEXT_CORE_AND_COMPATIBILITY_FAMILY.md`
2. `docs/function-lane/FUNCTION_SLICE_TEXT_CORE_AND_COMPATIBILITY_FAMILY_CONTRACT_PRELIM.md`
3. `docs/function-lane/W66_SCENARIO_MANIFEST_SEED.csv`
4. `docs/function-lane/W66_RUNTIME_REQUIREMENTS.md`
5. `docs/function-lane/W66_SCOPE_RECONCILIATION.csv`
6. `docs/function-lane/W66_EXECUTION_RECORD.md`
7. `tools/w66-probe/run-w66-text-core-compat-baseline.ps1`
8. `.tmp/w66-text-core-compat-results.csv`
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
1. `W066` promotes the already-evidenced text core and compatibility family into the ordinary-backlog closure program for the current reference Excel baseline.
2. Native Excel replay matched all `35/35` seeded `W066` scenarios on Excel `16.0.19822.20114`.
3. The regenerated `W44` snapshot now reports all `16` `W066` hidden snapshot entries as `function_meta_curated`; `0` `W066` rows remain `catalog_only`.
4. `W051` now drops to `18` hidden snapshot-entry backlog rows and `18` normalized execution backlog rows.
5. Current consumer-facing counts now read `534` published rows, `499` supported rows, `0` preview rows, and `17` deferred rows.

## 5. Evidence Posture
Public-reference anchors:
1. `docs/function-lane/FUNCTION_CATALOG_CURRENT_BASELINE_LOCAL.csv`

Runtime / dispatch anchors:
1. `crates/oxfunc_core/src/functions/text_scalar_misc.rs`
2. `crates/oxfunc_core/src/functions/concat_family.rs`
3. `crates/oxfunc_core/src/functions/text_slice_family.rs`
4. `crates/oxfunc_core/src/functions/text_search_replace_family.rs`
5. `crates/oxfunc_core/src/functions/text_b_compat_family.rs`
6. `crates/oxfunc_core/src/functions/text_unicode_fn.rs`
7. `crates/oxfunc_core/src/functions/surface_dispatch.rs`

Formal anchors:
1. `formal/lean/OxFunc/Functions/CodeFn.lean`
2. `formal/lean/OxFunc/Functions/LowerFn.lean`
3. `formal/lean/OxFunc/Functions/UpperFn.lean`
4. `formal/lean/OxFunc/Functions/TrimFn.lean`
5. `formal/lean/OxFunc/Functions/ReptFn.lean`
6. `formal/lean/OxFunc/Functions/ConcatFamily.lean`
7. `formal/lean/OxFunc/Functions/TextSliceFamily.lean`
8. `formal/lean/OxFunc/Functions/TextSearchReplaceFamily.lean`
9. `formal/lean/OxFunc/Functions/TextBCompatFamily.lean`
10. `formal/lean/OxFunc/Functions/TextUnicodeFn.lean`

Native replay anchors:
1. `docs/function-lane/W66_SCENARIO_MANIFEST_SEED.csv`
2. `docs/function-lane/W66_RUNTIME_REQUIREMENTS.md`
3. `tools/w66-probe/run-w66-text-core-compat-baseline.ps1`
4. `.tmp/w66-text-core-compat-results.csv`

Provenance anchors:
1. `docs/function-lane/W16_BATCH31_TEXT_SCALAR_MISC_NOTES.md`
2. `docs/function-lane/W16_BATCH35_TEXT_UNICODE_NOTES.md`
3. `docs/function-lane/W16_BATCH36_TEXT_SLICE_NOTES.md`
4. `docs/function-lane/W16_BATCH40_HELPER_CONCAT_NOTES.md`
5. `docs/function-lane/W16_BATCH42_TEXT_SEARCH_REPLACE_NOTES.md`
6. `docs/function-lane/W16_BATCH73_TEXT_B_COMPAT_NOTES.md`

Export-promotion anchors:
1. `tools/w44-probe/generate-w44-library-context-snapshot.ps1`
2. `docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1.csv`

## 6. Verification Runs
1. `powershell -ExecutionPolicy Bypass -File tools/w66-probe/run-w66-text-core-compat-baseline.ps1`
2. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib text_scalar_misc -- --nocapture`
3. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib concat_family -- --nocapture`
4. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib text_slice_family -- --nocapture`
5. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib text_search_replace_family -- --nocapture`
6. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib text_b_compat_family -- --nocapture`
7. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib text_unicode_fn -- --nocapture`
8. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib all_catalog_functions_have_at_least_one_export -- --nocapture`
9. `cargo fmt --manifest-path crates/oxfunc_core/Cargo.toml --all --check`
10. `cargo check --manifest-path tools/xll-addin/oxfunc_xll/Cargo.toml`
11. `lake build`
12. `powershell -ExecutionPolicy Bypass -File tools/w44-probe/generate-w44-library-context-snapshot.ps1`

## 7. Standing
1. `W066` is complete for declared current-baseline scope.
2. The packet relied on existing `W16` runtime/formal substrate evidence and repinned the same admitted slice through a dedicated `W066` replay/closure bundle rather than reopening the semantics.
