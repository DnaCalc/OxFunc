# BUGREP-FUNC-013: User observation: RATE(360,-1073.64,200000) returns #NUM! locally

## Summary
- **Report id**: `BUGREP-FUNC-013`
- **Filed**: 2026-04-10
- **Status**: `triaged`
- **Canonical bug**: `BUG-FUNC-009`

## Intake
- **Source channel**: `user`
- **Reported against ref**: `5d54d7f4ab2cdde6458272292d15ae1b317a0fef`
- **Reported against kind**: `commit`
- **Report owner workset**: `W081`

## Prompt / Observation
1. User asked to inspect `=RATE(360,-1073.64,200000)`.
2. Live Excel COM replay on 2026-04-10 showed:
   - displayed text under General: `0%`
   - underlying `Value2`: `0.004166644536345589`
3. Direct local OxFunc observation on the same date showed:
   - `rate(360,-1073.64,200000,0,EndOfPeriod,None) -> NoConvergence`
   - the surface therefore publishes `#NUM!`
4. Bounded local guess scan showed the failure is not total unsolvability:
   - `guess=None` and `guess=0.1` -> `NoConvergence`
   - `guess=0.01`, `0.005`, `0.004`, and `0.001` all converge locally to
     approximately `0.0041666445363455x`

## Initial Classification
- **Ownership guess**: `OxFunc-owned bug`
- **Duplicate of existing report?**: `no`
- **Needs canonical stream?**: `yes`

## Notes
1. This is not a display-only issue. Excel's displayed `0%` hides a nonzero
   underlying rate, while OxFunc currently fails the omitted-guess lane and
   surfaces `#NUM!`.
2. The intake is filed as `BUG-FUNC-009` and bounded owner `W081`.
