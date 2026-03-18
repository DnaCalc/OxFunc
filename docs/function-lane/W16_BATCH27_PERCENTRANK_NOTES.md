# W16 Batch 27 - Percentage Rank Functions

Status: `in_progress-provisional`
Workset: `W16`
Evidence ID: `W16-BATCH27-PERCENTRANK-20260315`

## Scope
1. `PERCENTRANK.INC`
2. `PERCENTRANK.EXC`

## Native Excel Baseline
Probe artifacts:
1. `.tmp/w16-batch27-percentrank-probe.csv`
2. `.tmp/w16-batch27-percentrank-sig-probe.csv`

Pinned lanes:
1. `PERCENTRANK.INC(F1:F5,1) -> 0`
2. `PERCENTRANK.INC(F1:F5,3) -> 0.5`
3. `PERCENTRANK.INC(F1:F5,3.5) -> 0.625`
4. `PERCENTRANK.INC(F1:F5,5) -> 1`
5. `PERCENTRANK.INC(F1:F5,0) -> #N/A`
6. `PERCENTRANK.INC(TRUE,"2") -> #N/A`
7. `PERCENTRANK.INC(G1:G2,1) -> #N/A`
8. `PERCENTRANK.INC(G1:G3,1) -> #N/A`
9. `PERCENTRANK.EXC(F1:F5,1) -> 0.166`
10. `PERCENTRANK.EXC(F1:F5,3) -> 0.5`
11. `PERCENTRANK.EXC(F1:F5,3.5) -> 0.583`
12. `PERCENTRANK.EXC(F1:F5,5) -> 0.833`
13. `PERCENTRANK.EXC(F1:F5,0) -> #N/A`
14. `PERCENTRANK.EXC(TRUE,"2") -> #N/A`
15. `PERCENTRANK.EXC(G1:G2,1) -> #N/A`
16. `PERCENTRANK.EXC(G1:G3,1) -> #N/A`
17. `PERCENTRANK.INC(F1:F5,3.5,6) -> 0.625`
18. `PERCENTRANK.INC(F1:F5,3.5,1.9) -> 0.6`
19. `PERCENTRANK.INC(F1:F5,3.5,0) -> #NUM!`
20. `PERCENTRANK.EXC(F1:F5,3.5,6) -> 0.583333`
21. `PERCENTRANK.EXC(F1:F5,3.5,1.9) -> 0.5`
22. `PERCENTRANK.EXC(F1:F5,3.5,0) -> #NUM!`

## Current Implementation Notes
1. The batch uses the same ordered numeric-only survivor policy as Batch 26.
2. `PERCENTRANK.INC` uses inclusive exact/interpolated positions over `(index)/(n-1)`.
3. `PERCENTRANK.EXC` uses exclusive exact/interpolated positions over `(index+1)/(n+1)`.
4. The optional significance argument is truncated to an integer and interpreted as significant digits, not decimal places.
