# W16 Batch 50 - Normal Lognormal Family

Status: `in_progress-provisional`
Workset: `W16`
Evidence ID: `W16-BATCH50-NORMAL-LOGNORM-20260316`

## Scope
1. `CONFIDENCE`
2. `CONFIDENCE.NORM`
3. `NORM.DIST`
4. `NORM.INV`
5. `NORM.S.DIST`
6. `NORM.S.INV`
7. `NORMDIST`
8. `NORMINV`
9. `NORMSDIST`
10. `NORMSINV`
11. `LOGNORM.DIST`
12. `LOGNORM.INV`
13. `LOGNORMDIST`

## Native Excel Baseline
Probe artifacts:
1. `.tmp/w16-batch50-normal-lognorm-probe.csv`
2. `.tmp/w16-batch50-normal-lognorm-edge-probe.csv`

Pinned lanes:
1. `NORM.S.DIST(1,TRUE) -> 0.841344746068543`
2. `NORM.S.DIST(1,FALSE) -> 0.241970724519143`
3. `NORM.S.INV(0.841344746068543) -> 1`
4. `NORM.DIST(42,40,1.5,TRUE) -> 0.908788780274132`
5. `NORM.DIST(42,40,1.5,FALSE) -> 0.109340049783996`
6. `NORM.INV(0.908788780274132,40,1.5) -> 42`
7. `NORMSDIST(1) -> 0.841344746068543`
8. `NORMSINV(0.841344746068543) -> 1`
9. `NORMDIST(42,40,1.5,TRUE) -> 0.908788780274132`
10. `NORMINV(0.908788780274132,40,1.5) -> 42`
11. `LOGNORM.DIST(EXP(2),2,0.5,TRUE) -> 0.5`
12. `LOGNORM.DIST(EXP(2),2,0.5,FALSE) -> 0.107981933026376`
13. `LOGNORM.INV(0.841344746068543,2,0.5) -> 12.1824939607035`
14. `LOGNORMDIST(EXP(2),2,0.5) -> 0.5`
15. `CONFIDENCE.NORM(0.05,2.5,100) -> 0.489990996135013`
16. `CONFIDENCE(0.05,2.5,100) -> 0.489990996135013`
17. `NORM.S.INV(0) -> #NUM!`
18. `NORM.DIST(1,0,0,TRUE) -> #NUM!`
19. `LOGNORM.DIST(0,2,0.5,TRUE) -> #NUM!`
20. `CONFIDENCE.NORM(0.05,2.5,0) -> #NUM!`
21. numeric cumulative flags treat nonzero values as `TRUE`, so `NORM.S.DIST(1,2) -> 0.841344746068543`

## Current Implementation Notes
1. `NORM.S.DIST` uses the existing Gaussian density substrate plus the same `erf`-based cumulative path already used for `GAUSS`.
2. `NORM.S.INV` uses a rational inverse-normal approximation over the open interval `(0,1)`.
3. `NORM.DIST` and `NORM.INV` are affine transforms over the standard-normal kernels and reject `stdev <= 0` with `#NUM!`.
4. `LOGNORM.DIST` and `LOGNORM.INV` are implemented as transforms through the normal kernels and reject `x <= 0` or `stdev <= 0` with `#NUM!`.
5. `CONFIDENCE` and `CONFIDENCE.NORM` share the same current-baseline semantics.
6. Compatibility aliases are wired as semantic aliases, not separate kernels.