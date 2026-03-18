# Function Slice Contract (Preliminary) - Regression / Forecast Family

Status: `provisional`
Workset: `W24`
Primary Functions: `GROWTH`, `TREND`, `LINEST`, `LOGEST`

## 1. Scope
1. close the admitted current-baseline slice for the regression/forecast family,
2. replace the old single-predictor-only packet note with the empirically pinned multivariate raw-result baseline,
3. bind the runtime and Lean substrate to a replayable native worksheet packet.

## 2. Admitted Current-Baseline Slice
1. `TREND`
   - numeric `known_y` vector only,
   - optional numeric `known_x` admitted as:
     - omitted default `1..n` sequence,
     - 1-D predictor vector,
     - full-rank multivariate matrix with one observation per row,
   - optional numeric `new_x` admitted as:
     - scalar, vector, or matrix for single-predictor cases,
     - row-shaped or multi-row predictor matrix with exact predictor column count for multivariate cases,
   - `const` optional, defaulting to `TRUE`.
2. `GROWTH`
   - same predictor-shape rules as `TREND`,
   - `known_y` values must be strictly positive,
   - multiplicative prediction is preserved across the admitted multivariate slice.
3. `LINEST`
   - numeric `known_y` vector,
   - optional numeric `known_x` with the same admitted vector/matrix rules,
   - `const` optional, defaulting to `TRUE`,
   - `stats` omitted or `FALSE`,
   - raw coefficient row only, with predictor coefficients returned in reverse predictor order and an intercept cell always present:
     - actual intercept when `const=TRUE`,
     - literal `0` when `const=FALSE`.
4. `LOGEST`
   - same admitted predictor rules as `LINEST`,
   - `known_y` must be strictly positive,
   - `stats` omitted or `FALSE`,
   - raw factor/base row only, with factors returned in reverse predictor order and a base cell always present:
     - actual base when `const=TRUE`,
     - literal `1` when `const=FALSE`.

## 3. Explicitly Out Of Slice
1. `LINEST(..., stats=TRUE)` and `LOGEST(..., stats=TRUE)` full statistics blocks.
2. Rank-deficient multivariate designs that require Excel-compatible pseudo-inverse or collinearity resolution policy.
3. Broader mixed-type coercion and legacy array-entry publication behavior beyond the current admitted numeric slice.

## 4. Metadata Shape
1. determinism: `deterministic`
2. volatility: `nonvolatile`
3. host_interaction: `none`
4. thread_safety: `safe_pure`
5. arg_preparation_profile: `refs_visible_in_adapter`
6. coercion_lift_profile: `custom`
7. fec_dependency_profile: `ref_only`
8. surface_fec_dependency_profile: `ref_only`

## 5. Evidence Basis
1. Rust runtime kernel and unit tests in `crates/oxfunc_core/src/functions/regression_forecast_family.rs`
2. Lean metadata/binding in `formal/lean/OxFunc/Functions/RegressionForecastFamily.lean`
3. Native worksheet packet in `docs/function-lane/W24_BATCH09_REGRESSION_FORECAST_SCENARIO_MANIFEST_SEED.csv`
4. Runtime harness in `tools/w24-probe/run-w24-batch09-regression-forecast-baseline.ps1`
5. Packet execution record in `docs/function-lane/W24_BATCH09_REGRESSION_FORECAST_EXECUTION_RECORD.md`

## 6. Scope Boundary
1. The closure is bounded to the admitted current-baseline regression/forecast slice above.
2. The family now includes the witnessed multivariate raw-result lanes and predictor-shape rules; it is no longer limited to the old single-predictor note.
3. Full stats-block semantics remain explicit future work rather than silent omission.
