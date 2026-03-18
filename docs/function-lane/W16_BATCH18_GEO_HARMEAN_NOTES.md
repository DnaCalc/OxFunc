# W16 Batch 18 - Geometric and Harmonic Mean

Status: `in_progress-provisional`
Workset: `W16`
Evidence ID: `W16-BATCH18-GEO-HARMEAN-20260315`

## Scope
1. `GEOMEAN`
2. `HARMEAN`

## Native Excel Baseline
Probe artifact:
1. `.tmp/w16-batch18-geo-harmean-probe.csv`

Pinned lanes:
1. `GEOMEAN(2,8) -> 4`
2. `GEOMEAN(TRUE,"2") -> 1.41421356237309`
3. `GEOMEAN("x") -> #VALUE!`
4. `GEOMEAN(F1:F2) -> 4`
5. `GEOMEAN(F1:F3) -> #NUM!`
6. `GEOMEAN(G1:G2) -> #NUM!`
7. `GEOMEAN(G1:G3) -> #N/A`
8. `GEOMEAN(TRUE) -> 1`
9. `GEOMEAN(FALSE) -> #NUM!`
10. `GEOMEAN("") -> #VALUE!`
11. `HARMEAN(2,8) -> 3.2`
12. `HARMEAN(TRUE,"2") -> 1.33333333333333`
13. `HARMEAN("x") -> #VALUE!`
14. `HARMEAN(F1:F2) -> 3.2`
15. `HARMEAN(F1:F3) -> #NUM!`
16. `HARMEAN(G1:G2) -> #N/A`
17. `HARMEAN(G1:G3) -> #N/A`
18. `HARMEAN(TRUE) -> 1`
19. `HARMEAN(FALSE) -> #NUM!`
20. `HARMEAN("") -> #VALUE!`
