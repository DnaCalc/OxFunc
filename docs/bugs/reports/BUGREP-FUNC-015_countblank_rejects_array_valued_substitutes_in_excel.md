# BUGREP-FUNC-015: COUNTBLANK rejects array-valued substitutes in Excel

## Summary
- **Report id**: `BUGREP-FUNC-015`
- **Filed**: 2026-04-10
- **Status**: `triaged`
- **Canonical bug**: `BUG-FUNC-011`

## Intake
- **Source channel**: `user`
- **Reported against ref**: `2e818f03a71ba393690275a7fb437ddd9a6bf760`
- **Reported against kind**: `commit`
- **Report owner workset**: `W084`

## Prompt / Observation
1. User asked whether `COUNTBLANK` should only accept a range and not an array
   for Excel compatibility.
2. Live Excel replay on 2026-04-10 pinned the key split:
   - `=LET(d,{"";1},COUNTBLANK(d)) -> #VALUE!`
   - `=COUNTBLANK(A1:A3) -> 2` when `A1=""`, `A2=1`, `A3` empty
3. The current local OxFunc implementation in `countblank_fn.rs` expands direct
   aggregate arguments through `expand_aggregate_arg(...)`, so inline arrays are
   currently treated as though they were range-like inputs rather than being
   rejected.

## Initial Classification
- **Ownership guess**: `OxFunc-owned bug`
- **Duplicate of existing report?**: `no`
- **Needs canonical stream?**: `yes`

## Notes
1. Live Excel contrast controls on 2026-04-10 show this is not a blanket
   aggregate-family rule:
   - `COUNT({"2",TRUE}) -> 0`
   - `COUNTA({"";1}) -> 2`
   - `ROWS({1;2;3}) -> 3`
   - `COLUMNS({1,2,3}) -> 3`
2. The nearest same-direction policy neighbors are the reference-sensitive
   rejection functions:
   - `AREAS`
   - `ISFORMULA`
   - `FORMULATEXT`
   - `SUBTOTAL`
   - `AGGREGATE`
3. The initial repair hypothesis is therefore function-local `COUNTBLANK`
   tightening, not a blanket adapter-level change to `COUNT` / `COUNTA` /
   `ROWS` / `COLUMNS`.
