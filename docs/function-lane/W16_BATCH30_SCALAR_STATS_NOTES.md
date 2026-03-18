# W16 Batch 30 - Scalar Statistical Transform Functions

Status: `in_progress-provisional`
Workset: `W16`
Evidence ID: `W16-BATCH30-SCALAR-STATS-20260315`

## Scope
1. `FISHER`
2. `FISHERINV`
3. `PHI`
4. `GAUSS`
5. `STANDARDIZE`

## Native Excel Baseline
Probe artifact:
1. `.tmp/w16-batch30-scalar-stats-probe.csv`

Pinned lanes:
1. `FISHER(0.5) -> 0.549306144334055`
2. `FISHER(1) -> #NUM!`
3. `FISHER(-1) -> #NUM!`
4. `FISHERINV(0.549306144334055) -> 0.5`
5. `PHI(0) -> 0.398942280401433`
6. `PHI(1) -> 0.241970724519143`
7. `GAUSS(0) -> 0`
8. `GAUSS(1) -> 0.341344746068543`
9. `STANDARDIZE(42,40,1.5) -> 1.33333333333333`
10. `STANDARDIZE(42,40,0) -> #NUM!`
11. `STANDARDIZE("42",40,1.5) -> 1.33333333333333`

## Current Implementation Notes
1. `FISHER` enforces the open interval `(-1,1)` and returns `#NUM!` at or beyond the boundary.
2. `FISHERINV` uses the standard logistic-tanh identity `((e^(2y))-1)/((e^(2y))+1)`.
3. `PHI` and `GAUSS` share a common normal-distribution helper substrate.
4. `GAUSS(0)` is normalized to exact worksheet zero rather than leaving a tiny approximation residue.
5. `STANDARDIZE` uses ordinary values-only numeric coercion and returns `#NUM!` when `standard_dev <= 0`.
