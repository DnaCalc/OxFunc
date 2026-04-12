# BUGREP-FUNC-017: Normal distribution functions show exact-value drift versus Excel

## Summary
- **Report id**: `BUGREP-FUNC-017`
- **Filed**: 2026-04-10
- **Status**: `triaged`
- **Canonical bug**: `BUG-FUNC-013`

## Intake
- **Source channel**: `user`
- **Reported against ref**: `2e818f03a71ba393690275a7fb437ddd9a6bf760`
- **Reported against kind**: `commit`
- **Report owner workset**: `W086`

## Prompt / Observation
1. User asked to confirm several small-variation differences between Excel and
   OxFunc and to suggest how to think about them.
2. Direct local replay on 2026-04-10 confirmed:
   - `NORM.DIST(0,0,1,TRUE) -> 0.49999998499999976`
   - `NORM.INV(0.975,0,1) -> 1.9599639471668913`
3. Live Excel `Value2` replay on 2026-04-10 confirmed:
   - `NORM.DIST(0,0,1,TRUE) -> 0.5`
   - `NORM.INV(0.975,0,1) -> 1.9599639845400536`
4. These are not display-policy differences; they are current local
   approximation-quality mismatches in the normal-distribution kernels.

## Initial Classification
- **Ownership guess**: `OxFunc-owned bug`
- **Duplicate of existing report?**: `no`
- **Needs canonical stream?**: `yes`

## Notes
1. The current local implementation in `normal_log_family.rs` uses a bounded
   approximation path:
   - `NORM.DIST` flows through `norm_cdf(...)`
   - `NORM.INV` flows through `inverse_standard_normal(...)`
2. Existing W062 evidence pinned these functions at rounded witness precision
   rather than exact current-baseline `Value2` parity, so the current mismatch
   was not blocked by the admitted witness floor.
3. This intake is intentionally separate from the corpus/display-policy review
   for tiny finance-last-digit rows.
