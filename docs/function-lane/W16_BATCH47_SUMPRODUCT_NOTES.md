# W16 Batch 47 - Sumproduct Family

Status: `in_progress-provisional`
Workset: `W16`
Evidence ID: `W16-BATCH47-SUMPRODUCT-20260316`

## Scope
1. `SUMPRODUCT`
2. `SUMX2MY2`
3. `SUMX2PY2`
4. `SUMXMY2`
5. `SERIESSUM`

## Native Excel Baseline
Probe artifacts:
1. `.tmp/w16-batch47-sumproduct-probe.csv`
2. `.tmp/w16-batch47-sumproduct-edge-probe.csv`

Pinned lanes:
1. `SUMPRODUCT({1,2},{3,4}) -> 11`
2. `SUMPRODUCT({1,TRUE},{3,4}) -> 3`
3. `SUMPRODUCT({1,"2"},{3,4}) -> 3`
4. `SUMPRODUCT({1,"x"},{3,4}) -> 3`
5. `SUMPRODUCT(5,6) -> 30`
6. `SUMPRODUCT(TRUE,6) -> 0`
7. Mixed scalar/array and mismatched-shape `SUMPRODUCT` calls produce `#VALUE!`.
8. `SUMX2MY2({1,2},{3,4}) -> -20`
9. `SUMX2PY2({1,2},{3,4}) -> 30`
10. `SUMXMY2({1,2},{3,4}) -> 8`
11. `SUMX2MY2({1,TRUE},{3,4}) -> -8`
12. `SUMX2MY2(TRUE,6) -> #DIV/0!`
13. Shape-mismatched `SUMX*` calls produce `#N/A`.
14. `SERIESSUM(2,1,2,{1,2,3}) -> 114`
15. `SERIESSUM("2",1,2,{1,2}) -> 18`
16. `SERIESSUM(TRUE,1,2,{1,2}) -> #VALUE!`
17. Non-numeric coefficients in `SERIESSUM` produce `#VALUE!`.

## Current Implementation Notes
1. The family keeps references visible in the adapter and resolves them before local shape/coercion handling.
2. `SUMPRODUCT` treats text, logical, and blank cells as zero after dereference, but propagates worksheet errors and rejects shape mismatch with `#VALUE!`.
3. `SUMX2MY2`, `SUMX2PY2`, and `SUMXMY2` require two equal-shaped operands, only accumulate positions where both sides are numeric, use `#N/A` for shape mismatch, and return `#DIV/0!` when no numeric pair survives.
4. `SERIESSUM` coerces only the scalar `x`, `n`, and `m` parameters through numeric-text parsing, flattens coefficient arrays row-major, and requires every coefficient cell to already be numeric.
