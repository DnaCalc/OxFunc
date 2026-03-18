# Function Slice Contract (Preliminary) - Cashflow Rate Family

Status: `provisional`
Workset: `W24`
Primary Functions: `IRR`, `XNPV`, `XIRR`

## 1. Scope
1. close the admitted current-baseline numeric cashflow/date-vector slice for the cashflow rate family,
2. bind the integrated runtime and Lean substrate to a replayable native worksheet packet,
3. replace the old bounded-note-only standing with packet evidence for the seeded current baseline.

## 2. Admitted Current-Baseline Slice
1. `IRR`
   - 1-D numeric cashflow vector
   - optional scalar `guess`
2. `XNPV`
   - scalar discount rate
   - 1-D numeric cashflow vector
   - 1-D numeric Excel-serial date vector
3. `XIRR`
   - 1-D numeric cashflow vector
   - 1-D numeric Excel-serial date vector
   - optional scalar `guess`
4. Iterative solver policy:
   - hybrid secant/Newton path
   - default guess `0.1`
   - lower bound just above `-1`
   - finite root tolerance and bounded iterations
5. Date interpretation:
   - dates are truncated Excel serial numbers
   - `XNPV` and `XIRR` anchor discounting at the first supplied date
   - dates earlier than the first supplied date are rejected as `#NUM!`

## 3. Explicitly Out Of Slice
1. alternative iteration-path parity for difficult multi-root or ill-conditioned cases beyond the seeded packet,
2. mixed text/logical/blank cashflow or date coercions beyond the strict numeric vector slice,
3. multi-column matrix cashflow/date inputs,
4. locale/version sweeps and publication-format nuances.

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
1. Rust runtime kernel and unit tests in `crates/oxfunc_core/src/functions/cashflow_rate_family.rs`
2. Lean metadata/binding in `formal/lean/OxFunc/Functions/CashflowRateFamily.lean`
3. Native worksheet packet in `docs/function-lane/W24_BATCH12_CASHFLOW_RATE_SCENARIO_MANIFEST_SEED.csv`
4. Runtime harness in `tools/w24-probe/run-w24-batch12-cashflow-rate-baseline.ps1`
5. Packet execution record in `docs/function-lane/W24_BATCH12_CASHFLOW_RATE_EXECUTION_RECORD.md`

## 6. Scope Boundary
1. The closure is bounded to the admitted current-baseline numeric vector/date-vector slice above.
2. The packet now evidences the seeded success and error lanes directly instead of relying on the older bounded `W16` note alone.
3. Broader multi-root and alternate-solver-path parity remains a separate follow-on validation concern rather than an unacknowledged gap in this packet.
