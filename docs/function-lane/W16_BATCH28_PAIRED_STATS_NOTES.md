# W16 Batch 28 - Paired Statistics Functions

Status: `in_progress-provisional`
Workset: `W16`
Evidence ID: `W16-BATCH28-PAIRED-STATS-20260315`

## Scope
1. `CORREL`
2. `PEARSON`
3. `RSQ`
4. `SLOPE`
5. `INTERCEPT`
6. `COVARIANCE.P`
7. `COVARIANCE.S`

## Native Excel Baseline
Probe artifact:
1. `.tmp/w16-batch28-paired-stats-probe.csv`

Pinned lanes:
1. `CORREL(F1:F4,G1:G4) -> 1`
2. `PEARSON(F1:F4,G1:G4) -> 1`
3. `RSQ(F1:F4,G1:G4) -> 1`
4. `SLOPE(G1:G4,F1:F4) -> 2`
5. `INTERCEPT(G1:G4,F1:F4) -> 0`
6. `COVARIANCE.P(F1:F4,G1:G4) -> 2.5`
7. `COVARIANCE.S(F1:F4,G1:G4) -> 3.33333333333333`
8. `CORREL(H1:H2,F1:F2) -> #DIV/0!`
9. `CORREL(F1:F2,F1:F3) -> #N/A`
10. `SLOPE(H1:H2,F1:F2) -> 0`
11. `INTERCEPT(H1:H2,F1:F2) -> 1`
12. `COVARIANCE.P(H1:H2,F1:F2) -> 0`
13. `COVARIANCE.S(H1:H2,F1:F2) -> 0`
14. `SLOPE(F1:F2,H1:H2) -> #DIV/0!`
15. `INTERCEPT(F1:F2,H1:H2) -> #DIV/0!`
16. `CORREL(I1:I3,F1:F3) -> 1`
17. `SLOPE(I1:I3,F1:F3) -> 1`
18. `INTERCEPT(I1:I3,F1:F3) -> 0`
19. `COVARIANCE.P(I1:I3,F1:F3) -> 0.25`
20. `COVARIANCE.S(I1:I3,F1:F3) -> 0.5`
21. `CORREL(1,F1) -> #DIV/0!`
22. `COVARIANCE.P(1,F1) -> 0`
23. `CORREL(1,F1:F2) -> #N/A`
24. `COVARIANCE.P(1,F1:F2) -> #N/A`
25. `COVARIANCE.P(1,2) -> 0`
26. `COVARIANCE.P(TRUE,1) -> #DIV/0!`
27. `COVARIANCE.P("1",1) -> #DIV/0!`
28. `COVARIANCE.P("x",1) -> #DIV/0!`

## Current Implementation Notes
1. The batch uses a shared paired-statistics substrate that first enforces equal expanded cardinality and then filters pairwise to rows where both sides are numeric.
2. Text, logical, blank, and missing values are ignored rather than coerced on this substrate, even for direct scalar arguments.
3. Shape/cardinality mismatch produces `#N/A` before pairwise numeric filtering.
4. `CORREL` and `PEARSON` share the same correlation kernel.
5. `RSQ` is the squared correlation result over the same pairwise substrate.
6. `SLOPE` and `INTERCEPT` depend only on variance in the `known_x` side; constant `known_y` yields `0` slope and finite intercept, while constant `known_x` yields `#DIV/0!`.
7. `COVARIANCE.P` admits a single surviving pair and returns `0` for scalar/scalar lanes, while `COVARIANCE.S` requires at least two surviving pairs.
