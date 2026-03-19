# W33 Execution Record - Information Predicates And Forecast Compatibility

Status: `complete`
Workset: `W33`
Evidence ID: `W33-INFO-FORECAST-20260319`

## 1. Purpose
1. close the newly promoted catalog members that form one clean ordinary packet:
   - the information predicates `ISBLANK`, `ISERR`, `ISERROR`, `ISLOGICAL`, `ISNA`, `ISNONTEXT`, `ISODD`, `ISREF`, `ISTEXT`,
   - the forecast compatibility pair `FORECAST`, `FORECAST.LINEAR`.

## 2. Scope
Artifacts created or updated:
1. `docs/function-lane/FUNCTION_SLICE_INFORMATION_PREDICATES_FAMILY_CONTRACT_PRELIM.md`
2. `docs/function-lane/FUNCTION_SLICE_FORECAST_COMPATIBILITY_PAIR_CONTRACT_PRELIM.md`
3. `docs/function-lane/W33_SCENARIO_MANIFEST_SEED.csv`
4. `docs/function-lane/W33_RUNTIME_REQUIREMENTS.md`
5. `docs/function-lane/W33_EXECUTION_RECORD.md`
6. `tools/w33-probe/run-w33-info-forecast-baseline.ps1`
7. `docs/function-lane/EXCEL_FUNCTION_DEFINITION_PRELIM_CONFORMANCE.csv`
8. `docs/function-lane/FUNCTION_LANE_EVIDENCE_ID_REGISTRY.md`
9. `crates/oxfunc_core/src/functions/is_predicates_family.rs`
10. `crates/oxfunc_core/src/functions/regression_forecast_family.rs`
11. `crates/oxfunc_core/src/functions/surface_dispatch.rs`
12. `crates/oxfunc_core/src/functions/mod.rs`
13. `crates/oxfunc_core/src/xll_export_specs.rs`
14. `formal/lean/OxFunc/Functions/IsPredicatesFamily.lean`
15. `formal/lean/OxFunc/Functions/RegressionForecastFamily.lean`
16. `formal/lean/OxFunc.lean`

## 3. Completeness Axes
1. execution_state: `complete`
2. scope_completeness: `scope_complete`
3. target_completeness: `target_complete`
4. integration_completeness: `integrated`
5. open_lanes:
   - none in declared `W33` scope

## 4. Empirical Findings
From `.tmp/w33-info-forecast-results.csv`:
1. `ISBLANK` distinguishes true blank cells from `""` and formulas returning `""`.
2. `ISERR`, `ISERROR`, and `ISNA` preserve the expected Excel split between general errors and `#N/A`.
3. `ISNONTEXT` treats blank references as non-text.
4. `ISODD` accepts numeric text but rejects direct logicals with `#VALUE!`, matching the seeded `ISEVEN`-style coercion seam.
5. `ISREF` returns `TRUE` for direct references and reference-returning functions (`INDIRECT`, `OFFSET`, `INDEX(...,0,0)`), but `FALSE` for array values such as `HSTACK(...)`.
6. `FORECAST` and `FORECAST.LINEAR` are semantically identical on the admitted current-baseline scalar/vector packet.
7. Mismatched `known_y` / `known_x` lengths return `#N/A` for both forecast names.

## 5. Implementation Result
1. `is_predicates_family.rs` now carries the nine-function substrate, with an explicit values-only/ref-visible split.
2. `regression_forecast_family.rs` now includes `FORECAST` and `FORECAST.LINEAR` as a compatibility pair over the existing linear prediction substrate rather than as a duplicate kernel.
3. XLL export generation and Lean metadata/binding are integrated for all eleven functions.

## 6. Verification Runs
1. `powershell -ExecutionPolicy Bypass -File tools/w33-probe/run-w33-info-forecast-baseline.ps1`
2. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml is_predicates_family`
3. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml regression_forecast_family`
4. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml all_catalog_functions_have_at_least_one_export`
5. `cargo check --manifest-path tools/xll-addin/oxfunc_xll/Cargo.toml`
6. `powershell -ExecutionPolicy Bypass -File tools/xll-addin/sync-export-specs.ps1`
7. `lake build`

## 7. Standing
1. All eleven `W33` members are now function-phase-complete for the admitted current reference baseline.
2. The packet is intentionally bounded to the admitted slices in the two family contracts.
3. Locale/version sweeps remain orthogonal validation work rather than a current `W33` semantic gap.
