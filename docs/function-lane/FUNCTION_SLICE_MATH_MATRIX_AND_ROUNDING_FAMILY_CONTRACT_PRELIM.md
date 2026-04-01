# Function Slice - Math, Matrix, And Rounding Family Contract (Prelim)

Status: `active`
Owner lane: `OxFunc`
Workset: `W067`

## 1. Purpose
Define the current-phase contract for the `W067` rounding, matrix, and sumproduct-family wave.

## 2. Covered Surface
1. `CEILING.MATH`
2. `CEILING.PRECISE`
3. `FLOOR`
4. `FLOOR.MATH`
5. `FLOOR.PRECISE`
6. `ISO.CEILING`
7. `MDETERM`
8. `MINVERSE`
9. `MMULT`
10. `MUNIT`
11. `SERIESSUM`
12. `SUMPRODUCT`
13. `SUMX2MY2`
14. `SUMX2PY2`
15. `SUMXMY2`

## 3. Rounding Contract
1. the rounding rows use the ordinary values-only pre-adapter seam.
2. `CEILING.MATH`, `CEILING.PRECISE`, `ISO.CEILING`, `FLOOR.MATH`, and `FLOOR.PRECISE` use absolute significance when significance is supplied.
3. `CEILING.MATH` and `FLOOR.MATH` default omitted significance to `1` and omitted mode to `0`.
4. `CEILING.PRECISE` and `ISO.CEILING` align with the current-baseline `CEILING.MATH(..., mode = 0)` lane.
5. `FLOOR.PRECISE` aligns with the current-baseline `FLOOR.MATH(..., mode = 0)` lane.
6. legacy `FLOOR` preserves the older sign-sensitive semantics, including `#DIV/0!` on zero significance.

## 4. Matrix Contract
1. `MDETERM`, `MINVERSE`, `MMULT`, and `MUNIT` use the refs-visible adapter seam on the admitted current baseline.
2. matrix operands preserve array/reference shape and admit only numeric cells; text, logical, and blank matrix cells produce `#VALUE!`.
3. `MDETERM`, `MINVERSE`, and `MMULT` treat scalar numeric inputs as `1x1` matrices on the admitted slice.
4. `MUNIT` follows separate scalar numeric coercion, admitting numeric text and truncating fractional sizes toward zero before validation.
5. non-square matrix lanes publish `#VALUE!`, singular inverse lanes publish `#NUM!`, and multiplication shape mismatch publishes `#VALUE!`.

## 5. Sumproduct / Series Contract
1. `SUMPRODUCT`, `SUMX2MY2`, `SUMX2PY2`, `SUMXMY2`, and `SERIESSUM` use the refs-visible adapter seam on the admitted current baseline.
2. `SUMPRODUCT` dereferences ranges before local shape/coercion handling, treats text/logical/blank cells as zero, propagates worksheet errors, and uses `#VALUE!` for shape mismatch.
3. `SUMX2MY2`, `SUMX2PY2`, and `SUMXMY2` require equal-shaped operands, accumulate only positions where both sides are numeric, use `#N/A` for shape mismatch, and publish `#DIV/0!` when no numeric pair survives.
4. `SERIESSUM` coerces only the scalar `x`, `n`, and `m` inputs through numeric-text parsing, flattens coefficient arrays row-major, and requires every coefficient cell to already be numeric.

## 6. Runtime / Formal Anchors
Runtime anchors:
1. `crates/oxfunc_core/src/functions/ceiling_floor_family.rs`
2. `crates/oxfunc_core/src/functions/matrix_family.rs`
3. `crates/oxfunc_core/src/functions/sumproduct_family.rs`
4. `crates/oxfunc_core/src/functions/surface_dispatch.rs`

Formal anchors:
1. `formal/lean/OxFunc/Functions/CeilingFloorFamily.lean`
2. `formal/lean/OxFunc/Functions/MatrixFamily.lean`
3. `formal/lean/OxFunc/Functions/SumproductFamily.lean`

Native replay anchors:
1. `docs/function-lane/W67_SCENARIO_MANIFEST_SEED.csv`
2. `tools/w67-probe/run-w67-math-matrix-rounding-baseline.ps1`
3. `.tmp/w67-math-matrix-rounding-results.csv`

Provenance anchors:
1. `docs/function-lane/W16_BATCH32_CEILING_FLOOR_NOTES.md`
2. `docs/function-lane/W16_BATCH45_MATRIX_FAMILY_NOTES.md`
3. `docs/function-lane/W16_BATCH47_SUMPRODUCT_NOTES.md`
