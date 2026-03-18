# Function Slice Contract (Preliminary) - Special Distribution Family

Status: `provisional`
Workset: `W24`
Primary Functions: `ERF`, `ERF.PRECISE`, `ERFC`, `ERFC.PRECISE`, `GAMMA`, `GAMMALN`, `GAMMALN.PRECISE`, `WEIBULL`, `WEIBULL.DIST`

## 1. Scope
1. close the admitted current-baseline slice for the special-distribution family,
2. bind the integrated runtime and Lean substrate to a replayable native worksheet packet,
3. make the pinned domain/error lanes explicit, especially for `GAMMA` poles and `WEIBULL.DIST` at `x = 0`.

## 2. Admitted Current-Baseline Slice
1. `ERF`
   - one-argument and interval forms.
2. `ERF.PRECISE`
   - one-argument form.
3. `ERFC`, `ERFC.PRECISE`
   - one-argument form.
4. `GAMMA`
   - positive inputs and negative non-integers,
   - `#NUM!` on poles and overflow.
5. `GAMMALN`, `GAMMALN.PRECISE`
   - positive inputs only,
   - `#NUM!` at `0`.
6. `WEIBULL`, `WEIBULL.DIST`
   - shared cumulative/density kernel with final boolean flag,
   - `x = 0` returns `0` for both cumulative and density lanes in the admitted baseline,
   - invalid negative `x`, nonpositive `alpha`, and nonpositive `beta` return `#NUM!`.

## 3. Metadata Shape
1. determinism: `deterministic`
2. volatility: `nonvolatile`
3. host_interaction: `none`
4. thread_safety: `safe_pure`
5. arg_preparation_profile: `values_only_pre_adapter`
6. coercion_lift_profile: `custom`
7. fec_dependency_profile: `none`
8. surface_fec_dependency_profile: `ref_only`

## 4. Evidence Basis
1. Rust runtime kernel and unit tests in `crates/oxfunc_core/src/functions/special_dist_family.rs`
2. Lean metadata/binding in `formal/lean/OxFunc/Functions/SpecialDistFamily.lean`
3. Native worksheet packet in `docs/function-lane/W24_BATCH06_SPECIAL_DIST_SCENARIO_MANIFEST_SEED.csv`
4. Runtime harness in `tools/w24-probe/run-w24-batch06-special-dist-baseline.ps1`
5. Packet execution record in `docs/function-lane/W24_BATCH06_SPECIAL_DIST_EXECUTION_RECORD.md`

## 5. Scope Boundary
1. The packet proves the admitted current-baseline numeric slice only.
2. Broader statistical-family harmonization remains outside this packet.
