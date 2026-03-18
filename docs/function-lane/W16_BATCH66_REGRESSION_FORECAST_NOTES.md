# W16 Batch 66 - Regression Forecast Family

Status: `integrated-by-W24`
Workset: `W16`
Function lanes: `GROWTH`, `TREND`, `LINEST`, `LOGEST`

## Current Standing
1. The old bounded single-predictor note is superseded by `W24 Batch 09`.
2. The family is now packet-evidenced for the admitted current-baseline multivariate raw-result slice:
   - multivariate `known_x` matrices are admitted,
   - `TREND` / `GROWTH` preserve single-predictor matrix `new_x` shape,
   - multivariate `new_x` is row-oriented with exact predictor column count,
   - `LINEST` / `LOGEST` keep a trailing intercept/base cell even when `const=FALSE`,
   - raw coefficient/factor rows return predictor terms in reverse predictor order.
3. Canonical packet docs now live in:
   - `docs/function-lane/FUNCTION_SLICE_REGRESSION_FORECAST_FAMILY_CONTRACT_PRELIM.md`
   - `docs/function-lane/W24_BATCH09_REGRESSION_FORECAST_EXECUTION_RECORD.md`

## Explicitly Out Of Slice
1. `LINEST(...,stats=TRUE)` and `LOGEST(...,stats=TRUE)` output blocks.
2. Rank-deficient multivariate design policy.
3. Broader mixed-type and legacy publication breadth outside the admitted packet.

## Evidence
1. Rust unit tests in `regression_forecast_family.rs`.
2. Native worksheet packet in `docs/function-lane/W24_BATCH09_REGRESSION_FORECAST_SCENARIO_MANIFEST_SEED.csv`.
3. Replay artifact `.tmp/w24-batch09-regression-forecast-results.csv`.
