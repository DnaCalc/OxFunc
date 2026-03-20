# W39 Execution Record - Dynamic Array Shaping And Reshaping Family

Status: `complete`
Workset: `W39`
Evidence IDs:
1. `W39-RESHAPE-BL-20260320`

## 1. Purpose
1. take ownership of the dynamic-array shaping and reshaping family as one packet,
2. close the family across runtime, export, Lean, and native worksheet evidence for the admitted current-baseline slice,
3. keep spill-shape semantics separate from the helper/callable and implicit-intersection seams.

## 2. Scope
Artifacts created or updated in this packet:
1. `docs/worksets/W039_DYNAMIC_ARRAY_SHAPING_AND_RESHAPING_FAMILY.md`
2. `docs/function-lane/FUNCTION_SLICE_DYNAMIC_ARRAY_SHAPING_AND_RESHAPING_FAMILY_CONTRACT_PRELIM.md`
3. `docs/function-lane/W39_SCENARIO_MANIFEST_SEED.csv`
4. `docs/function-lane/W39_RUNTIME_REQUIREMENTS.md`
5. `docs/function-lane/W39_EXECUTION_RECORD.md`
6. `tools/w39-probe/run-w39-dynamic-array-reshape-baseline.ps1`
7. `.tmp/w39-dynamic-array-reshape-results.csv`
8. `crates/oxfunc_core/src/functions/dynamic_array_reshape_family.rs`
9. `crates/oxfunc_core/src/functions/mod.rs`
10. `crates/oxfunc_core/src/functions/surface_dispatch.rs`
11. `crates/oxfunc_core/src/xll_export_specs.rs`
12. `tools/xll-addin/oxfunc_xll/export_specs.csv`
13. `formal/lean/OxFunc/Functions/DynamicArrayReshapeFamily.lean`
14. `formal/lean/OxFunc.lean`

## 3. Completeness Axes
1. execution_state: `complete`
2. scope_completeness: `scope_complete`
3. target_completeness: `target_complete`
4. integration_completeness: `integrated`
5. open_lanes:
   - none in declared `W39` scope

## 4. Packet Result
1. all fifteen functions now have a shared Rust runtime family and surface evaluators.
2. the admitted slice is exported through the ordinary dispatch/export surfaces.
3. Lean now has an executable admitted-slice model for the family substrate.
4. the native Excel packet ran green for all `15` seeded rows in `.tmp/w39-dynamic-array-reshape-results.csv`.

## 5. Main Findings
1. selector order and duplication are core semantics for `CHOOSECOLS` / `CHOOSEROWS`.
2. `TAKE` and `DROP` are best modeled as axis-span operations, not cell-wise deletions.
3. `EXPAND`, `VSTACK`, `WRAPCOLS`, and `WRAPROWS` all materially depend on explicit `#N/A` padding behavior.
4. `TOCOL` and `TOROW` are flattening/orientation functions first, and ignore-mode functions second.
5. `FILTER`, `SORT`, `SORTBY`, and `UNIQUE` can be admitted honestly on a seeded deterministic scalar/key slice without claiming the whole future matrix of collation and complex-key behavior.

## 6. Verification Runs
1. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml dynamic_array_reshape_family -- --nocapture`
2. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml all_catalog_functions_have_at_least_one_export -- --nocapture`
3. `cargo check --manifest-path tools/xll-addin/oxfunc_xll/Cargo.toml`
4. `powershell -ExecutionPolicy Bypass -File tools/xll-addin/sync-export-specs.ps1`
5. `powershell -ExecutionPolicy Bypass -File tools/w39-probe/run-w39-dynamic-array-reshape-baseline.ps1`
6. `lake build` from `formal/lean`

## 7. Standing
1. `W39` is no longer a planning-only packet.
2. all `15` reshaping-family rows are reconciled as `done` in `W39_SCOPE_RECONCILIATION.csv`.
3. helper/callable interactions remain with `W38`, and implicit-intersection / legacy CSE interactions remain outside this packet.
