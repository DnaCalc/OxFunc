# W40 Execution Record - Reference Metadata And Formula Visibility Functions

Status: `complete`
Workset: `W040`
Evidence IDs:
1. `W40-REFMETA-BL-20260321`

## 1. Purpose
1. close the admitted current-baseline packet for `ADDRESS`, `AREAS`, `FORMULATEXT`, `SHEET`, and `SHEETS`,
2. make the exact OxFunc-side host callback surface explicit,
3. keep pure rendering/reference semantics separate from host-owned sheet/formula metadata truth.

## 2. Scope
Artifacts created or updated in this packet:
1. `docs/worksets/W040_REFERENCE_METADATA_AND_FORMULA_VISIBILITY_FUNCTIONS.md`
2. `docs/function-lane/FUNCTION_SLICE_REFERENCE_METADATA_AND_FORMULA_VISIBILITY_CONTRACT_PRELIM.md`
3. `docs/function-lane/W40_SCENARIO_MANIFEST_SEED.csv`
4. `docs/function-lane/W40_RUNTIME_REQUIREMENTS.md`
5. `docs/function-lane/W40_EXECUTION_RECORD.md`
6. `docs/function-lane/W40_SCOPE_RECONCILIATION.csv`
7. `tools/w40-probe/run-w40-reference-metadata-baseline.ps1`
8. `.tmp/w40-reference-metadata-results.csv`
9. `crates/oxfunc_core/src/functions/reference_metadata_family.rs`
10. `crates/oxfunc_core/src/functions/mod.rs`
11. `crates/oxfunc_core/src/functions/surface_dispatch.rs`
12. `crates/oxfunc_core/src/host_info.rs`
13. `crates/oxfunc_core/src/xll_export_specs.rs`
14. `tools/xll-addin/oxfunc_xll/export_specs.csv`
15. `formal/lean/OxFunc/HostInfoSeam.lean`
16. `formal/lean/OxFunc/Functions/ReferenceMetadataFamily.lean`
17. `formal/lean/OxFunc.lean`
18. `docs/function-lane/EXCEL_FUNCTION_DEFINITION_PRELIM_CONFORMANCE.csv`

## 3. Completeness Axes
1. execution_state: `complete`
2. scope_completeness: `scope_complete`
3. target_completeness: `target_complete`
4. integration_completeness: `integrated`
5. open_lanes:
   - none in declared `W040` scope

## 4. Packet Result
1. all five functions now have an admitted current-baseline Rust runtime packet.
2. the exact first-pass host callback surface is pinned as:
   - `query_formula_text(reference)`
   - `query_sheet_index(CurrentSheet | Reference | SheetNameText)`
   - `query_sheet_count(Workbook | Reference)`
3. Lean now has an executable admitted-slice model for:
   - `ADDRESS` rendering,
   - `AREAS` union counting,
   - and the typed query surface needed by `FORMULATEXT`, `SHEET`, and `SHEETS`.
4. the packet is reconciled as `done` for all five inventory members in `W40_SCOPE_RECONCILIATION.csv`.

## 5. Main Findings
1. `ADDRESS` is pure OxFunc-owned rendering once scalar args are admitted; no host callback is needed.
2. `AREAS` is OxFunc-owned once multi-area reference shape is preserved.
3. `FORMULATEXT` cannot be reduced to value-only semantics; it needs stored formula text from the host/grid.
4. `SHEET` and `SHEETS` need sheet identity/topology truth, but OxFunc can still own argument admission and worksheet result projection.
5. the exact host callback surface is small and typed; OxFunc does not need workbook object access or a large callback API here.

## 6. Verification Runs
1. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml reference_metadata_family -- --nocapture`
2. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml all_catalog_functions_have_at_least_one_export -- --nocapture`
3. `cargo check --manifest-path tools/xll-addin/oxfunc_xll/Cargo.toml`
4. `powershell -ExecutionPolicy Bypass -File tools/xll-addin/sync-export-specs.ps1`
5. `powershell -ExecutionPolicy Bypass -File tools/w40-probe/run-w40-reference-metadata-baseline.ps1`
6. `lake build` from `formal/lean`

## 7. Standing
1. `W040` is no longer a planning-only or native-only packet.
2. all `5` reference-metadata rows are reconciled as `done` in `W40_SCOPE_RECONCILIATION.csv`.
3. `ISFORMULA` remains with `W023`; `CELL` / `INFO` remain with `W015`; `@` remains with `W014`.
