# W16 Batch 40 - Helper and Concatenation Family

Status: `in_progress-provisional`
Workset: `W16`
Evidence ID: `W16-BATCH40-HELPER-CONCAT-20260316`

## Scope
1. `ISEVEN`
2. `ERROR.TYPE`
3. `IFNA`
4. `COUNTBLANK`
5. `CONCAT`
6. `CONCATENATE`

## Native Excel Baseline
Probe artifact:
1. `.tmp/w16-batch40-helper-concat-probe.csv`

Pinned lanes:
1. `ISEVEN(2.9) -> TRUE`
2. `ISEVEN(-1.2) -> FALSE`
3. `ISEVEN("2") -> TRUE`
4. `ISEVEN(D1) -> TRUE` when `D1` is blank
5. `ISEVEN(TRUE) -> #VALUE!`
6. `ERROR.TYPE(NA()) -> 7`
7. `ERROR.TYPE(1/0) -> 2`
8. `ERROR.TYPE(1) -> #N/A`
9. `ERROR.TYPE(D1) -> #N/A` when `D1` is blank
10. `IFNA(NA(),7) -> 7`
11. `IFNA(1/0,7) -> #DIV/0!`
12. `IFNA("x",7) -> "x"`
13. `COUNTBLANK(D1) -> 1`
14. `COUNTBLANK(D2) -> 1` when `D2` contains `""`
15. `COUNTBLANK(D1:D3) -> 2` when `D3="x"`
16. `CONCAT("a",1,TRUE) -> "a1TRUE"`
17. `CONCAT(D1:D3) -> "x"`
18. `CONCAT(D1,D2,D3) -> "x"`
19. `CONCATENATE("a",1,TRUE) -> "a1TRUE"`
20. `CONCATENATE(D1:D3) -> #VALUE!`
21. `CONCATENATE(D1,D2,D3) -> "x"`

## Current Implementation Notes
1. `ISEVEN` uses values-only preparation with a custom numeric policy: blank inputs are treated as zero, numeric text is accepted, and logicals are rejected with `#VALUE!`.
2. `ERROR.TYPE` returns the legacy `1..8` mapping only for the classic worksheet errors and falls back to `#N/A` for non-errors or newer non-classic error values.
3. `IFNA` shares the lazy branch-preparation seam with `IFERROR`, but it only traps `#N/A`.
4. `COUNTBLANK` counts both truly empty cells and zero-length text payloads, while propagating worksheet errors from the scanned arguments.
5. `CONCAT` flattens array-like arguments row-major and textifies each element, while `CONCATENATE` remains scalar-only and therefore rejects multi-cell ranges with `#VALUE!`.
