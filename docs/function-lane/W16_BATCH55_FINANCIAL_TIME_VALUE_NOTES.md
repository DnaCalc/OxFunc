# W16 Batch 55 - Financial Time-Value Family

Status: `integrated-by-W24`
Workset: `W16`
Evidence ID: `W16-BATCH55-FINANCIAL-TIME-VALUE-20260316`

## Scope
1. `PV`
2. `FV`
3. `PMT`
4. `NPER`
5. `NPV`
6. `RATE`
7. `IPMT`
8. `PPMT`
9. `ISPMT`
10. `MIRR`
11. `FVSCHEDULE`
12. `PDURATION`
13. `RRI`
14. `NOMINAL`
15. `EFFECT`

## Current Standing
1. The family is now packet-evidenced by `W24 Batch 11`.
2. Canonical packet docs now live in:
   - `docs/function-lane/FUNCTION_SLICE_FINANCIAL_TIME_VALUE_FAMILY_CONTRACT_PRELIM.md`
   - `docs/function-lane/W24_BATCH11_FINANCIAL_TIME_VALUE_EXECUTION_RECORD.md`
3. The packet corrected the old local `ISPMT` note: on the current baseline `ISPMT(0.1,1,4,1000) -> -75`, not `-100`.

## Implemented Semantics
1. `PV`, `FV`, `PMT`, and `NPER` share one annuity identity with explicit zero-rate handling.
2. `RATE` is included with a secant-first iterative solver and a finite-difference fallback, capped at 20 iterations.
3. `IPMT` and `PPMT` are layered over `PMT` plus balance evolution from `FV`.
4. `ISPMT` uses the equal-principal schedule model with period indexing aligned to the documented Excel quirk that the schedule begins at zero internally.
5. `NPV`, `MIRR`, and `FVSCHEDULE` are implemented as cashflow/schedule kernels.
6. `PDURATION`, `RRI`, `NOMINAL`, and `EFFECT` are implemented as direct logarithmic/compound-rate transforms.

## Local Verification
1. Crate-level Rust tests cover annuity inversion consistency, a documented `RATE` sample, `NPV`, `IPMT`/`PPMT` partitioning, `ISPMT`, `MIRR`, `FVSCHEDULE`, `PDURATION`, `RRI`, `NOMINAL`, `EFFECT`, and representative domain errors.
2. `cargo check --manifest-path tools/xll-addin/oxfunc_xll/Cargo.toml`, `lake build`, and export-spec synchronization now pass with the family admitted on the shared surfaces.`r`n3. The matching Lean note records the current metadata profile for the batch.

## Open Issues
1. `RATE` remains the least settled lane because Excel convergence parity can depend on seed and iteration details; the current packet only claims the admitted sample slice.
2. Broader mixed cashflow/sequence breadth remains outside the admitted packet.
