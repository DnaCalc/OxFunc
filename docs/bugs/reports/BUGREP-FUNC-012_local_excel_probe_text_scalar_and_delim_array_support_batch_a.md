# BUGREP-FUNC-012: Local Excel probe on text scalar and delimiter array-support batch A

## Summary
- **Report id**: `BUGREP-FUNC-012`
- **Filed**: 2026-04-09
- **Status**: `triaged`
- **Canonical bug**: `BUG-FUNC-008`

## Intake
- **Source channel**: `local empirical probe`
- **Reported against ref**: `5d54d7f4ab2cdde6458272292d15ae1b317a0fef`
- **Reported against kind**: `commit`
- **Report owner workset**: `W080`

## Prompt / Observation
1. Systematic `W080` batch-A replay checked adjacent ordinary text functions for
   array-expansion behavior using live Excel on 2026-04-09.
2. Confirmed spill lanes:
   - `CHAR(SEQUENCE(3)+64) -> {A;B;C}`
   - `CODE({"A","B"}) -> {65,66}`
   - `LOWER({"A","B"}) -> {"a","b"}`
   - `UPPER({"a","b"}) -> {"A","B"}`
   - `TRIM({"  a  "," b "}) -> {"a","b"}`
   - `REPT("x",SEQUENCE(3)) -> {"x";"xx";"xxx"}`
   - `REPT({"a","b"},2) -> {"aa","bb"}`
   - `TEXTAFTER("a-b-c","-",SEQUENCE(3)) -> {"b-c";"c";#N/A}`
   - `TEXTBEFORE("a-b-c","-",SEQUENCE(3)) -> {"a";"a-b";#N/A}`
   - `TEXTAFTER({"a-b","c-d"},"-") -> {"b","d"}`
   - `TEXTBEFORE({"a-b","c-d"},"-") -> {"a","c"}`
3. Non-lift note from the same quick replay:
   - `TEXTAFTER("a-b,c-d",{"-",","})` did not open a spill lane in the simple
     probe and therefore should not be widened speculatively.

## Initial Classification
- **Ownership guess**: `OxFunc-owned bug`
- **Duplicate of existing report?**: `no`
- **Needs canonical stream?**: `yes`

## Notes
1. This report opens `BUG-FUNC-008` for the first bounded `W080` batch beyond
   the earlier text-slice `LEFT` / `RIGHT` / `MID` correction.
2. The observed pattern is still bounded: spill is admitted for the tested text
   unary lanes, for `REPT` on a single array-valued text/count argument, and
   for `TEXTAFTER` / `TEXTBEFORE` on text and `instance_num`, not for every
   possible argument position.
