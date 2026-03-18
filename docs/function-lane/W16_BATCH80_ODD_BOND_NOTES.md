# W16 Batch 80 - Odd Bond Functions

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
5. `ODDL*` uses a segmented quasi-coupon schedule from `last_interest` forward, with prorated final coupon and clean-price accrued-interest subtraction across elapsed quasi periods.
6. Yield functions invert the bounded price kernels with nonnegative-yield bisection.

## Pinned Baselines
1. `ODDFPRICE(...)` bounded example lane -> `113.597717474079`
2. `ODDFYIELD(...)` inversion lane -> `0.0625`
3. `ODDLPRICE(...)` bounded example lane -> `99.8948395136953`
4. `ODDLYIELD(...)` inversion lane -> `0.0405`

## Open Lanes
1. Long odd-first coupons are not yet modeled.
2. `ODDF*` cases where maturity is not regular relative to `first_coupon` are not yet modeled.
3. Broader parity sweeps against native Excel odd-bond edge cases, especially basis `1` compatibility details, remain open.
