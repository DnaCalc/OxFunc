# Function Slice Contract (Preliminary) - Financial Time-Value Family

Status: `provisional`
Workset: `W24`
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

## 6. Scope Boundary
1. The closure is bounded to the admitted current-baseline scalar/sequence slice above.
2. The packet now evidences `ISPMT` directly instead of relying on the older incorrect local note.
3. Broader `RATE` convergence parity remains a separate follow-on validation concern rather than an unacknowledged gap in this packet.
