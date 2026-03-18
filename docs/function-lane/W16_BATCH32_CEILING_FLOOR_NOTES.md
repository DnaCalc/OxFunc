# W16 Batch 32 - Ceiling and Floor Family

Status: `in_progress-provisional`
Workset: `W16`
Evidence ID: `W16-BATCH32-CEILING-FLOOR-20260315`

## Scope
1. `CEILING`
2. `CEILING.MATH`
3. `CEILING.PRECISE`
4. `ISO.CEILING`
5. `FLOOR`
6. `FLOOR.MATH`
7. `FLOOR.PRECISE`

## Native Excel Baseline
Probe artifacts:
1. `.tmp/w16-batch32-ceiling-floor-probe.csv`
2. `.tmp/w16-batch32-ceiling-floor-edge-probe.csv`
3. `.tmp/w16-batch32-ceiling-floor-defaults-probe.csv`

Pinned lanes:
1. `CEILING(4.3,2) -> 6`
2. `CEILING(-4.3,2) -> -4`
3. `CEILING(-4.3,-2) -> -6`
4. `CEILING(4.3,-2) -> #NUM!`
5. `CEILING(5,0) -> 0`
6. `FLOOR(4.3,2) -> 4`
7. `FLOOR(-4.3,2) -> -6`
8. `FLOOR(-4.3,-2) -> -4`
9. `FLOOR(4.3,-2) -> #NUM!`
10. `FLOOR(5,0) -> #DIV/0!`
11. `CEILING.MATH(-4.3,2) -> -4`
12. `CEILING.MATH(-4.3,2,1) -> -6`
13. `CEILING.MATH(4.3,-2) -> 6`
14. `CEILING.MATH(5,0) -> 0`
15. `FLOOR.MATH(-4.3,2) -> -6`
16. `FLOOR.MATH(-4.3,2,1) -> -4`
17. `FLOOR.MATH(4.3,-2) -> 4`
18. `FLOOR.MATH(5,0) -> 0`
19. `CEILING.PRECISE(-4.3,2) -> -4`
20. `FLOOR.PRECISE(-4.3,2) -> -6`
21. `ISO.CEILING(-4.3,2) -> -4`
22. `CEILING.MATH(4.3) -> 5`
23. `FLOOR.MATH(4.3) -> 4`
24. `CEILING.PRECISE(4.3) -> 5`
25. `FLOOR.PRECISE(4.3) -> 4`
26. `ISO.CEILING(4.3) -> 5`

## Current Implementation Notes
1. Legacy `CEILING` and `FLOOR` preserve their older sign-sensitive semantics.
2. `CEILING` returns `0` for zero significance, while legacy `FLOOR` returns `#DIV/0!`.
3. `CEILING.MATH`, `CEILING.PRECISE`, `ISO.CEILING`, `FLOOR.MATH`, and `FLOOR.PRECISE` all use `ABS(significance)` when significance is supplied.
4. `CEILING.PRECISE` and `ISO.CEILING` currently align with `CEILING.MATH(..., mode=0)` on the seeded baseline.
5. `FLOOR.PRECISE` currently aligns with `FLOOR.MATH(..., mode=0)` on the seeded baseline.
