# W16 Batch 38 - Date Serial Parts

Status: `in_progress-provisional`
Workset: `W16`
Evidence ID: `W16-BATCH38-DATE-PARTS-20260316`

## Scope
1. `DAY`
2. `MONTH`
3. `YEAR`
4. `DAYS`

## Native Excel Baseline
Probe artifacts:
1. `.tmp/w16-date-part-probe.csv`
2. `.tmp/w16-date-part-blank-probe.csv`

Pinned lanes:
1. `DAY(1) -> 1`
2. `MONTH(1) -> 1`
3. `YEAR(1) -> 1900`
4. `DAY(60) -> 29`
5. `MONTH(60) -> 2`
6. `YEAR(60) -> 1900`
7. `DAY(61.9) -> 1`
8. `MONTH(61.9) -> 3`
9. `YEAR(61.9) -> 1900`
10. `DAY(0) -> 0`
11. `DAY(-1) -> #NUM!`
12. `DAYS(61,60) -> 1`
13. `DAYS(1,2) -> -1`
14. `DAYS(61.9,60.1) -> 1`
15. `DAYS(TRUE,1) -> 0`
16. `DAYS(,1) -> -1`
17. `DAY(blank_ref) -> 0`
18. `MONTH(blank_ref) -> 1`
19. `YEAR(blank_ref) -> 1900`
20. `DAYS(blank_ref,1) -> -1`

## Current Implementation Notes
1. The batch preserves Excel's 1900 date system quirks, including serial `0 -> 1900-01-00` and the fake leap-day `1900-02-29` at serial `60`.
2. `DAY`, `MONTH`, and `YEAR` truncate numeric serials toward zero before extracting parts.
3. Negative serials produce `#NUM!` for all four functions.
4. `DAYS` performs simple truncated-serial subtraction after the same numeric admission policy.
5. Missing arguments and blank-cell references are admitted as zero for the current baseline slice, matching the pinned worksheet probe rows.
