# W16 Batch 56 - Chi/F/T Distribution Family

Status: `in_progress-provisional`
Workset: `W16`
Evidence ID: `W16-BATCH56-CHI-F-T-20260316`

## Scope
1. `CHISQ.DIST`
2. `CHISQ.DIST.RT`
3. `CHISQ.INV`
4. `CHISQ.INV.RT`
5. `CHIDIST`
6. `CHIINV`
7. `F.DIST`
8. `F.DIST.RT`
9. `F.INV`
10. `F.INV.RT`
11. `FDIST`
12. `FINV`
13. `T.DIST`
14. `T.DIST.2T`
15. `T.DIST.RT`
16. `T.INV`
17. `T.INV.2T`
18. `TDIST`
19. `TINV`

## Current Batch Shape
1. The Rust family is integrated into OxFunc shared dispatch.
2. The family currently shares the `special_math_common` numerical substrate for incomplete gamma, beta, and inverse bisection helpers.
3. The current lane targets current-baseline parity for the seeded scalar numeric cases and compatibility aliases.

## Pinned Lanes
1. `CHISQ.DIST(2,4,TRUE)`, `CHISQ.DIST.RT(2,4)`, `CHISQ.INV(0.5,4)`, and `CHISQ.INV.RT(0.5,4)` are covered by the Rust test family.
2. `CHIDIST`/`CHIINV`, `FDIST`/`FINV`, and `TDIST`/`TINV` are treated as compatibility aliases over the modern kernels.
3. Negative or zero degrees-of-freedom lanes map to `#NUM!`.
4. Right-tail and two-tail `T` lanes currently require nonnegative `x` in the admitted slice, matching the seeded parity rows.

## Open Issues
1. Locale/version sweeps are not yet run for this batch.
2. Broader Excel replay artifacts for edge tails are still desirable even though the seeded Rust tests and integration are in place.
