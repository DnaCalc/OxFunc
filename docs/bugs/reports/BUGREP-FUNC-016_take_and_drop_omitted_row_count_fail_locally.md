# BUGREP-FUNC-016: TAKE and DROP omitted row count fail locally

## Summary
- **Report id**: `BUGREP-FUNC-016`
- **Filed**: 2026-04-10
- **Status**: `triaged`
- **Canonical bug**: `BUG-FUNC-012`

## Intake
- **Source channel**: `OxFml handoff`
- **Reported against ref**: `2e818f03a71ba393690275a7fb437ddd9a6bf760`
- **Reported against kind**: `commit`
- **Report owner workset**: `W085`

## Prompt / Observation
1. `HANDOFF-OXFUNC-004` reported that a complex helper-bound lambda still
   fails after the OxFml-side invocation-environment fix.
2. The failing lanes are:
   - `TAKE(w,,COLUMNS(w)/2)`
   - `TAKE(w,,-COLUMNS(w)/2)`
   - `TAKE(Z,,COLUMNS(Z)/2)`
   - `TAKE(Z,,-COLUMNS(Z)/2)`
3. Local confirmation on 2026-04-10:
   - live Excel: `TAKE({1,2;3,4},,1) -> {1;3}`
   - live Excel: `TAKE({1,2;3,4},,-1) -> {2;4}`
   - live Excel: `DROP({1,2;3,4},,1) -> {2;4}`
   - live Excel: `DROP({1,2;3,4},,-1) -> {1;3}`
   - local OxFunc: `TAKE(array, MissingArg, 1) -> Err(Preparation(MissingArg))`
   - local OxFunc: `DROP(array, MissingArg, 1) -> Err(Preparation(MissingArg))`

## Initial Classification
- **Ownership guess**: `OxFunc-owned bug`
- **Duplicate of existing report?**: `no`
- **Needs canonical stream?**: `yes`

## Notes
1. This is not a helper-invocation bug on the OxFml side; the handoff’s
   upstream closure fix exposed a real local reshape gap.
2. The current local `TAKE` and `DROP` prepared paths always parse `args[1]` as
   required, so an omitted row-count never defaults to “all rows.”
3. The earlier `W39` seeded rows only covered explicit count forms such as
   `TAKE(...,2,-2)`, not omitted-leading-count column-slice forms.
