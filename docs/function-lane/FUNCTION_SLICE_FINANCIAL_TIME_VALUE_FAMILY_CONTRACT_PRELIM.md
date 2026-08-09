# Function Slice Contract (Preliminary) - Financial Time-Value Family

Status: `provisional_w109_aligned`
Workset: `W24`, amended by `W109`
Primary Functions: `PV`, `FV`, `PMT`, `NPER`, `NPV`, `RATE`, `IPMT`, `PPMT`, `ISPMT`, `MIRR`, `FVSCHEDULE`, `PDURATION`, `RRI`, `NOMINAL`, `EFFECT`

## 1. Scope
1. close the admitted current-baseline scalar and numeric-sequence slice for the financial time-value family,
2. bind the integrated runtime and Lean substrate to a replayable native worksheet packet,
3. replace the old incorrect `ISPMT` period-index note with the empirically observed baseline.

## 2. Admitted Current-Baseline Slice
1. scalar annuity functions:
   - `PV`, `FV`, `PMT`, `NPER`
   - optional `fv` and `type` defaults preserved
2. iterative rate lane:
   - `RATE` with the current solver surface and admitted scalar sample lane
3. partitioned payment family:
   - `IPMT`, `PPMT`
   - standard scalar annuity arguments only
4. equal-principal schedule lane:
   - `ISPMT`
   - current observed linear schedule formula, including period-zero and beyond-schedule scalar lanes
5. sequence-backed numeric kernels:
   - `NPV`, `MIRR`, `FVSCHEDULE`
   - numeric arrays/references in the admitted sequence slice
6. direct logarithmic/compound transforms:
   - `PDURATION`, `RRI`, `NOMINAL`, `EFFECT`

### 2.1 W109 exact-publication alignment for EFFECT, RRI, and NOMINAL

On the current Excel 16.0 build-20228 x64 / workbook CompatibilityVersion 2
reference profile, the admitted scalar slice additionally requires:

1. `EFFECT` truncates `npery`, constructs the base with x87-double-rounded
   divide/add operations, uses an x87-double-rounded LSB-first binary-power
   loop while the truncated count is below `u32::MAX`, switches at exactly
   `u32::MAX` to the raw stored-LN/product/x87-EXP chain, and publishes the
   final subtraction through the x87 double-rounding boundary.
2. `NOMINAL` truncates `npery`, stores the x87-double-rounded `1+effect` base,
   uses one register-continuous FYL2X/F2XM1/FSCALE power for truncated counts
   `1` and `2`, uses the raw stored-LN/product/x87-EXP chain for counts `>=3`,
   stores the completed power, and evaluates `n*(power-1)` in that order.
3. `RRI` rejects periods below binary64 `MIN_NORMAL`; DAZ-normalizes present
   and future values; publishes `+0` on normalized equality before sign guards;
   DAZ-normalizes its x87-double-rounded quotient; publishes `-1` for a zero
   quotient; uses the quotient directly when `periods==1`; otherwise uses the
   raw reciprocal-first stored-LN/product/x87-EXP chain; and publishes the final
   subtraction through x87 double rounding. Nonfinite intermediates/results
   publish `#NUM!` at their identified checks.

These are executable-publication requirements, not merely mathematical
identities. The reusable x87 primitives and route dispatches are part of the
function's primary semantic substrate for this admitted profile.

## 3. Explicitly Out Of Slice
1. broader cross-build `RATE` convergence parity beyond the admitted packet sample lanes.
2. richer cashflow-shape and mixed-type sequence breadth beyond the admitted numeric packet.
3. locale/version sweeps and publication-format nuances.

## 4. Metadata Shape
1. determinism: `deterministic`
2. volatility: `nonvolatile`
3. host_interaction: `none`
4. thread_safety: `safe_pure`
5. arg_preparation_profile: `refs_visible_in_adapter`
6. coercion_lift_profile: `custom`
7. fec_dependency_profile: `none`
8. surface_fec_dependency_profile: `ref_only`

## 5. Evidence Basis
1. Rust runtime kernel and unit tests in `crates/oxfunc_core/src/functions/financial_time_value_family.rs`
2. Lean metadata/binding in `formal/lean/OxFunc/Functions/FinancialTimeValueFamily.lean`
3. Native worksheet packet in `docs/function-lane/W24_BATCH11_FINANCIAL_TIME_VALUE_SCENARIO_MANIFEST_SEED.csv`
4. Runtime harness in `tools/w24-probe/run-w24-batch11-financial-time-value-baseline.ps1`
5. Packet execution record in `docs/function-lane/W24_BATCH11_FINANCIAL_TIME_VALUE_EXECUTION_RECORD.md`
6. W109 exact graph/evidence record in
   `docs/function-lane/W109_EFFECT_RRI_NOMINAL_IDENTIFICATION_20260809.md`
7. Deterministic scorer and edge-artifact generator in
   `smart-fuzzer/tools/calc_graph_racer/src/bin/race_effect_rri_check.rs`

## 6. Scope Boundary
1. The closure is bounded to the admitted current-baseline scalar/sequence slice above.
2. The packet now evidences `ISPMT` directly instead of relying on the older incorrect local note.
3. Broader `RATE` convergence parity remains a separate follow-on validation concern rather than an unacknowledged gap in this packet.
4. This amendment does not promote the whole financial family: PMT, RATE, and
   other cataloged family members retain known W109 semantic lanes. The family
   therefore remains `scope_partial` even though the current-reference
   EFFECT/RRI/NOMINAL publication graphs are identified and exercised.
