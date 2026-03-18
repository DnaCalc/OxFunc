# W16 Batch 70 - Cumulative Finance Functions

Status: `in_progress-provisional`
Workset: `W16`
Evidence ID: `W16-BATCH70-CUMULATIVE-FINANCE-20260316`

## Scope
1. `CUMIPMT`
2. `CUMPRINC`

## Native Excel Baseline
Pinned Microsoft seed lanes:
1. `CUMIPMT(0.09/12,30*12,125000,13,24,0) -> -11135.23213`
2. `CUMIPMT(0.09/12,30*12,125000,1,1,0) -> -937.5`
3. `CUMPRINC(0.09/12,30*12,125000,13,24,0) -> -934.1071234`
4. `CUMPRINC(0.09/12,30*12,125000,1,1,0) -> -68.27827118`

## Current Implementation Notes
1. This batch is intentionally bounded to scalar numeric arguments and does not widen into any shared range-scan or shared-dispatch work.
2. The kernels reuse the standard amortization substrate locally: fixed payment, per-period interest, per-period principal, then inclusive accumulation across the requested period window.
3. `type` is accepted only as exact `0` or `1`; other numeric values return `#NUM!`.
4. `nper`, `start_period`, and `end_period` are currently truncated toward zero before validation and accumulation.
5. The current slice returns `#NUM!` when `rate <= 0`, `nper <= 0`, `pv <= 0`, `start_period < 1`, `end_period < 1`, `start_period > end_period`, or `end_period > nper`.
6. Beginning-of-period handling is explicitly covered for the first payment lane where cumulative interest is `0`.
