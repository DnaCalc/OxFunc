# W16 Batch 45 - Matrix Family

Status: `in_progress-provisional`
Workset: `W16`
Evidence ID: `W16-BATCH45-MATRIX-FAMILY-20260316`

## Scope
1. `MDETERM`
2. `MINVERSE`
3. `MMULT`
4. `MUNIT`

## Native Excel Baseline
Probe artifacts:
1. `.tmp/w16-batch45-matrix-probe.csv`
2. `.tmp/w16-batch45-matrix-edge-probe.csv`

Pinned lanes:
1. `MDETERM({1,2;3,4}) -> -2`
2. `MINVERSE({1,2;3,4}) -> {-2,1;1.5,-0.5}`
3. `MMULT({1,2;3,4},{5;6}) -> {17;39}`
4. `MUNIT(3) -> identity(3)`
5. `MDETERM(5) -> 5`
6. `MINVERSE(5) -> 0.2`
7. `MMULT(5,2) -> 10`
8. `MUNIT(2.9) -> identity(2)`
9. `MUNIT("2") -> identity(2)`
10. `MDETERM({1,2,3;4,5,6}) -> #VALUE!`
11. `MINVERSE({1,2;2,4}) -> #NUM!`
12. `MMULT({1,2;3,4},{5;6;7}) -> #VALUE!`
13. matrix operands reject text, logical, and blank cells with `#VALUE!`

## Current Implementation Notes
1. Matrix operands preserve array/reference shape in the adapter and only admit numeric cells.
2. Scalar numeric arguments are treated as `1x1` matrices for `MDETERM`, `MINVERSE`, and `MMULT`.
3. `MUNIT` follows separate scalar-number coercion, so text numerics and truncated fractional sizes are admitted there even though text matrix operands are not.
4. `MINVERSE` uses Gauss-Jordan elimination with partial pivoting and maps singular matrices to `#NUM!`.
