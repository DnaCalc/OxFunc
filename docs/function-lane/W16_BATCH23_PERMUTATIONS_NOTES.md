# W16 Batch 23 - Permutation Functions

Status: `in_progress-provisional`
Workset: `W16`
Evidence ID: `W16-BATCH23-PERMUTATIONS-20260315`

## Scope
1. `PERMUT`
2. `PERMUTATIONA`

## Native Excel Baseline
Probe artifact:
1. `.tmp/w16-batch23-permut-probe.csv`

Pinned lanes:
1. `PERMUT(10,3) -> 720`
2. `PERMUT(10.9,3.2) -> 720`
3. `PERMUT(3,4) -> #NUM!`
4. `PERMUT(-1,1) -> #NUM!`
5. `PERMUT(0,0) -> 1`
6. `PERMUTATIONA(3,2) -> 9`
7. `PERMUTATIONA(3.9,2.1) -> 9`
8. `PERMUTATIONA(0,0) -> 1`
9. `PERMUTATIONA(0,1) -> 0`
10. `PERMUTATIONA(-1,1) -> #NUM!`

## Current Implementation Notes
1. The batch reuses the existing binary-numeric values-only seam and the factorial/nonnegative truncation helpers from Batches 6 and 8.
2. `PERMUT` uses truncated nonnegative integers and returns `#NUM!` when `k > n`.
3. `PERMUTATIONA` uses truncated nonnegative integers and the current baseline matches `n^k`, including `0^0 -> 1`.
