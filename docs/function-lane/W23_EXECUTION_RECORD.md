# W23 Execution Record - Host, Metadata, and Database Functions

Status: `complete`
Workset: `W23`
Evidence IDs:
1. `W23-DB-BL-20260321`
2. `W23-HOST-CLASS-BL-20260321`
3. `W23-ISF-BL-20260321`
4. `W23-STA-BL-20260321`
5. `W23-HI-VALMODEL-20260321`

## 1. Purpose
1. convert the remaining `W23` functions into honest OxFunc-side closure work where possible,
2. keep the residual host/publication seams explicit where current-baseline Excel behavior pressures them,
3. avoid treating host/publication/provider surfaces as ordinary pure function work.

## 2. Scope
Artifacts created or updated in this pass:
1. `docs/worksets/W023_DEFERRED_HOST_METADATA_AND_DATABASE_FUNCTIONS.md`
2. `docs/function-lane/W23_DEFERRED_HOST_METADATA_AND_DATABASE_INVENTORY.csv`
3. `docs/function-lane/W23_SCOPE_RECONCILIATION.csv`
4. `docs/function-lane/FUNCTION_SLICE_DATABASE_FAMILY_CONTRACT_PRELIM.md`
5. `docs/function-lane/FUNCTION_SLICE_ISFORMULA_CONTRACT_PRELIM.md`
6. `docs/function-lane/FUNCTION_SLICE_SUBTOTAL_AGGREGATE_CONTRACT_PRELIM.md`
7. `docs/function-lane/FUNCTION_SLICE_HYPERLINK_IMAGE_VALUE_MODEL_PRELIM.md`
8. `docs/function-lane/W23_DATABASE_SCENARIO_MANIFEST_SEED.csv`
9. `docs/function-lane/W23_DATABASE_RUNTIME_REQUIREMENTS.md`
10. `docs/function-lane/W23_HOST_PROVIDER_CLASSIFICATION_SCENARIO_MANIFEST_SEED.csv`
11. `docs/function-lane/W23_ISFORMULA_SCENARIO_MANIFEST_SEED.csv`
12. `docs/function-lane/W23_ISFORMULA_RUNTIME_REQUIREMENTS.md`
13. `docs/function-lane/W23_SUBTOTAL_AGGREGATE_SCENARIO_MANIFEST_SEED.csv`
14. `docs/function-lane/W23_SUBTOTAL_AGGREGATE_RUNTIME_REQUIREMENTS.md`
15. `docs/function-lane/W23_EXECUTION_RECORD.md`
16. `tools/w23-probe/run-w23-database-baseline.ps1`
17. `tools/w23-probe/run-w23-host-provider-classification.ps1`
18. `tools/w23-probe/run-w23-isformula-baseline.ps1`
19. `tools/w23-probe/run-w23-subtotal-aggregate-baseline.ps1`
20. `tools/w23-probe/run-w23-hyperlink-image-value-model-baseline.ps1`
21. `.tmp/w23-database-results.csv`
22. `.tmp/w23-host-provider-classification-results.csv`
23. `.tmp/w23-isformula-results.csv`
24. `.tmp/w23-subtotal-aggregate-results.csv`
25. `.tmp/w23-hyperlink-image-value-model-results.csv`
26. `crates/oxfunc_core/src/functions/database_family.rs`
27. `crates/oxfunc_core/src/functions/subtotal_aggregate_family.rs`
28. `crates/oxfunc_core/src/functions/misc_switch_info_family.rs`
29. `crates/oxfunc_core/src/functions/hyperlink_fn.rs`
30. `crates/oxfunc_core/src/host_info.rs`
31. `formal/lean/OxFunc/Functions/DatabaseFamily.lean`
32. `formal/lean/OxFunc/Functions/SubtotalAggregateFamily.lean`
33. `formal/lean/OxFunc/Functions/MiscSwitchInfoFamily.lean`
34. `formal/lean/OxFunc/HostInfoSeam.lean`

## 3. Completeness Axes
1. execution_state: `complete`
2. scope_completeness: `scope_complete`
3. target_completeness: `target_complete`
4. integration_completeness: `integrated`

## 4. Packet Result
1. the database family now has bounded Rust runtime, Lean metadata alignment, and a native Excel baseline packet.
2. `ISFORMULA` now has a typed host-query contract, native Excel replay, Lean seam alignment, and an existing XLL host-info bridge path.
3. `SUBTOTAL` and `AGGREGATE` now have a typed row-visibility callback seam plus current-baseline reference-form runtime and replay evidence.
4. `HYPERLINK` is now characterized as ordinary text value plus a first-pass `style=hyperlink` presentation hint on the current baseline, with actual publication/clickability still owned above OxFunc.
5. `IMAGE` is now characterized as a typed image-query/runtime lane with semantic `_webimage` rich-value carriage and separate published worksheet fallback, and the OxFml-side exercised evaluator/host/adapter floor is now in-tree.
6. `COPILOT`, `GETPIVOTDATA`, `DETECTLANGUAGE`, `PHONETIC`, `CALL`, and `REGISTER.ID` are no longer treated as unresolved `W23` semantics; they now belong to successor packets or are already closed for current phase.

## 5. Main Findings
1. the `D*` family is materially easier than the rest of `W23` because once prepared database/criteria grids are available, the admitted slice is fully OxFunc-owned.
2. `ISFORMULA` is not a difficult semantic function once the host callback exists; it is the same kind of typed information seam as the already-closed `CELL` / `INFO` work.
3. `SUBTOTAL` always ignores filtered rows and nested aggregate cells on the admitted slice, and the `101..111` codes also ignore manually hidden rows.
4. on the admitted reference-form slice, `AGGREGATE` options `0..3` ignore nested aggregate cells, while options `4..7` keep nested aggregate values and only split hidden/filter/error handling.
5. `HYPERLINK` behaves like plain text at the value boundary: `TYPE(...) = 2`, `CELL("contents", ...)` is the displayed text, and a referencing cell receives the same plain text while losing the hyperlink-style underline/publication treatment.
6. `IMAGE` does not currently behave like an ordinary scalar value: the support-example lane preserves a non-ordinary payload across reference with `TYPE = 128`, which pressures an extended/rich host-managed value or publication-object model.

## 6. Verification Runs
1. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml database_family -- --nocapture`
2. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml subtotal_aggregate_family -- --nocapture`
3. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml misc_switch_info_family -- --nocapture`
4. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml hyperlink_fn -- --nocapture`
5. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml all_catalog_functions_have_at_least_one_export -- --nocapture`
6. `cargo check --manifest-path tools/xll-addin/oxfunc_xll/Cargo.toml`
7. `powershell -ExecutionPolicy Bypass -File tools/xll-addin/sync-export-specs.ps1`
8. `powershell -ExecutionPolicy Bypass -File tools/w23-probe/run-w23-database-baseline.ps1`
9. `powershell -ExecutionPolicy Bypass -File tools/w23-probe/run-w23-host-provider-classification.ps1`
10. `powershell -ExecutionPolicy Bypass -File tools/w23-probe/run-w23-isformula-baseline.ps1`
11. `powershell -ExecutionPolicy Bypass -File tools/w23-probe/run-w23-subtotal-aggregate-baseline.ps1`
12. `powershell -ExecutionPolicy Bypass -File tools/w23-probe/run-w23-hyperlink-image-value-model-baseline.ps1`
13. `lake build`

## 7. Standing
1. `DAVERAGE`, `DCOUNT`, `DCOUNTA`, `DGET`, `DMAX`, `DMIN`, `DPRODUCT`, `DSTDEV`, `DSTDEVP`, `DSUM`, `DVAR`, and `DVARP` are closure-grade on the admitted current-baseline slice inside `W23`.
2. `ISFORMULA`, `SUBTOTAL`, and `AGGREGATE` are now closure-grade on the admitted current-baseline OxFunc side inside `W23`.
3. `HYPERLINK` and `IMAGE` are now also complete for the declared current-phase OxFunc scope.
4. wider future host/publication model growth remains above this packet's declared scope.
