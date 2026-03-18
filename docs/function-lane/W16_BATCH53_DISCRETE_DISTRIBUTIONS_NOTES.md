# W16 Batch 53 - Discrete And Count Distributions

Status: `in_progress-provisional`
Workset: `W16`

## Scope
1. `BINOM.DIST`
2. `BINOM.DIST.RANGE`
3. `BINOM.INV`
4. `BINOMDIST`
5. `POISSON`
6. `POISSON.DIST`
7. `HYPGEOM.DIST`
8. `HYPGEOMDIST`
9. `NEGBINOM.DIST`
10. `NEGBINOMDIST`
11. `CRITBINOM`
12. `EXPON.DIST`
13. `EXPONDIST`

## Current Slice
1. All functions are modeled as values-only, nonvolatile pure kernels with `surface_fec_dependency_profile = refOnly`.
2. Integer count arguments are truncated before domain validation in the current implementation.
3. Compatibility aliases are implemented as surface aliases over the same modern kernels, except for the legacy no-cumulative signatures (`HYPGEOMDIST`, `NEGBINOMDIST`).

## Pinned Lanes
1. `BINOM.DIST(2,4,0.25,FALSE) -> 0.2109375`
2. `BINOM.DIST(2,4,0.25,TRUE) -> 0.94921875`
3. `BINOM.DIST.RANGE(4,0.25,2,3) -> 0.2578125`
4. `BINOM.INV(6,0.5,0.7) -> 4`
5. `POISSON.DIST(3,2,FALSE) -> 0.1804470443154836`
6. `POISSON.DIST(3,2,TRUE) -> 0.857123460498547`
7. `HYPGEOM.DIST(2,5,4,10,FALSE) -> 0.47619047619047616`
8. `HYPGEOM.DIST(2,5,4,10,TRUE) -> 0.7380952380952381`
9. `NEGBINOM.DIST(3,2,0.5,FALSE) -> 0.125`
10. `NEGBINOM.DIST(3,2,0.5,TRUE) -> 0.8125`
11. `EXPON.DIST(2,1.5,FALSE) -> 0.07468060255179591`
12. `EXPON.DIST(2,1.5,TRUE) -> 0.950212931632136`

## Notes
1. `BINOM.DIST`, `POISSON.DIST`, `HYPGEOM.DIST`, `NEGBINOM.DIST`, and `EXPON.DIST` return `#NUM!` on out-of-domain numeric inputs.
2. `BINOM.DIST.RANGE` defaults the optional upper bound to the lower bound.
3. Surface evaluators use the standard values-only coercion path, so direct numeric text and logical cumulative flags are admitted through existing adapter rules.
4. Shared dispatch/XLL integration is intentionally untouched in this task.
