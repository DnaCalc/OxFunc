# W24 Batch 09 Execution Record - Regression / Forecast Family

Status: `complete-provisional`
Workset: `W24`
Evidence ID: `W24-B09-REGRESSION-FORECAST-20260318`

## 1. Purpose
Record the regression/forecast family closure packet inside the `W24` ordinary mega-batch.

## 2. Scope
1. close `GROWTH`, `TREND`, `LINEST`, and `LOGEST` for the admitted current reference baseline,
2. replace the older single-predictor-only packet note with an evidenced multivariate/raw-result slice,
3. bind the runtime and Lean substrate to a replayable native worksheet packet.

## 3. Completeness Axes
1. execution_state: `complete`
2. scope_completeness: `scope_complete`
3. target_completeness: `target_complete`
4. integration_completeness: `integrated`
5. open_lanes:
   - `LINEST(...,stats=TRUE)` and `LOGEST(...,stats=TRUE)` remain outside this packet.
   - rank-deficient multivariate design policy remains outside this packet.

## 4. Executed Scope
Artifacts created or updated:
1. `docs/function-lane/FUNCTION_SLICE_REGRESSION_FORECAST_FAMILY_CONTRACT_PRELIM.md`
2. `docs/function-lane/W24_BATCH09_REGRESSION_FORECAST_SCENARIO_MANIFEST_SEED.csv`
3. `docs/function-lane/W24_BATCH09_REGRESSION_FORECAST_RUNTIME_REQUIREMENTS.md`
4. `docs/function-lane/W24_BATCH09_REGRESSION_FORECAST_EXECUTION_RECORD.md`
5. `tools/w24-probe/run-w24-batch09-regression-forecast-baseline.ps1`
6. `docs/function-lane/EXCEL_FUNCTION_DEFINITION_PRELIM_CONFORMANCE.csv`
7. `docs/function-lane/FUNCTION_LANE_EVIDENCE_ID_REGISTRY.md`
8. `docs/function-lane/W24_ORDINARY_FUNCTIONS_MEGA_BATCH_CHECKLIST.csv`
9. `docs/function-lane/W16_BATCH66_REGRESSION_FORECAST_NOTES.md`
10. `formal/lean/OxFunc/Functions/RegressionForecastFamily.lean`
11. `crates/oxfunc_core/src/functions/regression_forecast_family.rs`

## 5. Empirical Findings
From `.tmp/w24-batch09-regression-forecast-results.csv`:
1. `TREND({2;4;6})` defaults `known_x` to the `1..n` sequence and preserves the original row/column publication shape.
2. Single-predictor `new_x` preserves scalar, vector, and full matrix shape, for example `ARRAYTOTEXT(TREND({2;4;6},{1;2;3},{1,2;3,4}),1)` -> `{2,4;6,8}`.
3. Multivariate `new_x` is row-oriented: `ARRAYTOTEXT(TREND({6;8;9;11;10},{1,1;2,1;1,2;2,2;3,1},{4,1;4,2}),1)` -> `{12;15}`.
4. Multivariate `new_x` with the wrong predictor column count returns `#REF!`.
5. `GROWTH` follows the same predictor-shape policy and rejects nonpositive `known_y` with `#NUM!`.
6. `LINEST(...,FALSE)` still returns a trailing intercept cell of `0`, not a shortened row.
7. `LOGEST(...,FALSE)` still returns a trailing base cell of `1`, not a shortened row.
8. Multivariate `LINEST` and `LOGEST` return predictor coefficients/factors in reverse predictor order, then the intercept/base cell.

## 6. Implementation Result
1. The kernel now accepts the admitted multivariate predictor matrices instead of rejecting them as out of slice.
2. `TREND` and `GROWTH` now distinguish predictor-shape admission for `known_x` versus `new_x`, matching the observed row-oriented multivariate `new_x` rule.
3. `LINEST` and `LOGEST` raw-result publication now matches the current baseline for `const=FALSE`, keeping a trailing intercept/base cell.
4. The Lean artifact remains a metadata/binding alignment layer for the admitted slice rather than a second full numeric implementation.

## 7. Verification Runs
1. `powershell -ExecutionPolicy Bypass -File tools/w24-probe/run-w24-batch09-regression-forecast-baseline.ps1`
2. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml regression_forecast_family`
3. `cargo check --manifest-path tools/xll-addin/oxfunc_xll/Cargo.toml`
4. `lake build`

## 8. Standing
1. `GROWTH`, `TREND`, `LINEST`, and `LOGEST` are now function-phase-complete for the admitted current reference baseline.
2. The closure is bounded to the admitted regression/forecast slice above.
3. The next work in `W24` continues with the locale/text-profile family.
