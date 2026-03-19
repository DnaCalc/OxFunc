# Function Slice Contract (Preliminary) - Forecast Compatibility Pair

Status: `provisional`
Workset: `W33`
Primary Functions: `FORECAST`, `FORECAST.LINEAR`

## 1. Scope
1. close the admitted current-baseline compatibility slice for `FORECAST` and `FORECAST.LINEAR`,
2. pin whether the two names are observably distinct on the current worksheet baseline,
3. bind the pair to the existing regression/forecast substrate instead of maintaining a duplicate kernel.

## 2. Admitted Current-Baseline Slice
1. arity is exact `3`: `x`, `known_y`, `known_x`.
2. `x`
   - admitted as a scalar numeric payload in the current slice.
3. `known_y` / `known_x`
   - admitted as numeric vectors only,
   - row vectors and column vectors are both accepted,
   - two-dimensional matrices remain outside this packet.
4. mismatched `known_y` / `known_x` vector lengths return `#N/A`.
5. direct reference-fed scalar/vector lanes behave like the equivalent array-literal lanes on the current baseline.
6. on the admitted current-baseline slice, `FORECAST` and `FORECAST.LINEAR` are semantically identical and both return the same scalar linear-trend prediction.

## 3. Metadata Shape
1. determinism: `deterministic`
2. volatility: `nonvolatile`
3. host_interaction: `none`
4. thread_safety: `safe_pure`
5. arg_preparation_profile: `refs_visible_in_adapter`
6. coercion_lift_profile: `custom`
7. fec_dependency_profile: `ref_only`
8. surface_fec_dependency_profile: `ref_only`

## 4. Compatibility Position
1. `FORECAST.LINEAR` is the ordinary current-baseline member name.
2. `FORECAST` is preserved as a compatibility surface.
3. no behavioral divergence was observed between the two names on the seeded current-baseline lanes in this packet.

## 5. Explicitly Out Of Slice
1. broader regression-family expansion beyond the scalar prediction pair,
2. mixed-type coercion outside the admitted numeric slice,
3. legacy array-entry publication behavior and broader compatibility-version sweeps.

## 6. Evidence Basis
1. Rust runtime/tests: `crates/oxfunc_core/src/functions/regression_forecast_family.rs`
2. Lean metadata/binding: `formal/lean/OxFunc/Functions/RegressionForecastFamily.lean`
3. native packet: `docs/function-lane/W33_SCENARIO_MANIFEST_SEED.csv`
4. runtime harness: `tools/w33-probe/run-w33-info-forecast-baseline.ps1`
5. packet execution record: `docs/function-lane/W33_EXECUTION_RECORD.md`
