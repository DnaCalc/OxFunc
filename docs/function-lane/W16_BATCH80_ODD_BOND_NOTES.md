# W16 Batch 80 - Odd Bond Functions

Superseded in part by `W27`.
This historical `W16` note records the original bounded odd-bond packet, but the current admitted current-baseline odd-bond position is now owned by:
- `docs/worksets/W027_DEFERRED_ADVANCED_BOND_AND_ODD_BOND_HARDENING.md`
- `docs/function-lane/FUNCTION_SLICE_ODD_BOND_FAMILY_CONTRACT_PRELIM.md`
- `docs/function-lane/W27_EXECUTION_RECORD.md`

Status: `in_progress-provisional`
Workset: `W16`
Evidence ID: `W16-BATCH80-ODD-BOND-20260316`

## Scope
1. `ODDFPRICE`
2. `ODDFYIELD`
3. `ODDLPRICE`
4. `ODDLYIELD`

## Bounded Semantic Slice
1. The family is scalar-only and self-contained.
2. `basis` is truncated to integer and admitted only for `0..4`; `frequency` is admitted only for `1`, `2`, or `4`.
3. `ODDF*` is currently bounded to short odd-first coupons only:
   issue date must be strictly after the previous regular coupon date implied by `first_coupon` and `frequency`.
4. `ODDF*` also currently requires maturity to align exactly to the regular schedule implied by `first_coupon` and `frequency`.
5. The older `ODDL*` segmented quasi-coupon summary here is no longer authoritative. `W27` replaces it with the normalized odd-last quasi-coupon accumulation that matches direct Excel parity on the admitted blocker lane.
6. `ODDFYIELD` remains inversion-based on the admitted slice. `ODDLYIELD` is no longer accurately described here as just a generic bounded bisection path; `W27` now pins the direct odd-last parity lane explicitly.

## Pinned Baselines
1. `ODDFPRICE(...)` bounded example lane -> `113.597717474079`
2. `ODDFYIELD(...)` inversion lane -> `0.0625`
3. `ODDLPRICE(...)` bounded example lane was originally recorded here as `99.8948395136953`, but this is superseded by the direct Excel parity value pinned in `W27`: `99.87828601472134`
4. `ODDLYIELD(...)` inversion lane -> `0.0405`

## Open Lanes
1. Long odd-first coupons are not yet modeled.
2. `ODDF*` cases where maturity is not regular relative to `first_coupon` are not yet modeled.
3. Broader parity sweeps against native Excel odd-bond edge cases, especially basis `1` compatibility details, remain open.
4. Direct current-baseline odd-bond closure is no longer tracked here; use `W27`.
