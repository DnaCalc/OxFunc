# BUGREP-FUNC-014: SORT omitted middle argument defaults to #VALUE! locally

## Summary
- **Report id**: `BUGREP-FUNC-014`
- **Filed**: 2026-04-10
- **Status**: `triaged`
- **Canonical bug**: `BUG-FUNC-010`

## Intake
- **Source channel**: `user`
- **Reported against ref**: `2e818f03a71ba393690275a7fb437ddd9a6bf760`
- **Reported against kind**: `commit`
- **Report owner workset**: `W083`

## Prompt / Observation
1. User asked to check `=SORT({2;3;7;5},,-1)`.
2. Direct OxFunc-side local replay on 2026-04-10 using an explicit array and
   `CallArgValue::MissingArg` for the middle argument observed:
   - `eval_sort_surface([array, MissingArg, -1], NoResolver)`
   - result: `Err(Preparation(MissingArg))`
   - worksheet-surface outcome: `#VALUE!`
3. The explicit non-omitted equivalent succeeds:
   - `SORT({2;3;7;5},1,-1) -> {7;5;3;2}`

## Initial Classification
- **Ownership guess**: `OxFunc-owned bug`
- **Duplicate of existing report?**: `no`
- **Needs canonical stream?**: `yes`

## Notes
1. The local divergence is not in descending sort semantics themselves.
2. The failure is an omitted-optional-argument defaulting gap on the
   values-only prepared surface.
3. Initial adjacent-risk scan shows `SORTBY(..., by_array,)` hits the same
   helper path for omitted sort-order defaulting and belongs in the same
   canonical stream.
