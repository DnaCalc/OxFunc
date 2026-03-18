# W16 Batch 14 - Extremum Aggregate Functions

Status: `in_progress-provisional`
Workset: `W16`
Evidence ID: `W16-BATCH14-MIN-MAX-20260315`

## Scope
1. `MIN`
2. `MAX`

## Native Excel Baseline
Probe artifact:
1. `.tmp/w16-batch14-min-max-probe.csv`

Pinned lanes:
1. `MAX(2,3,4) -> 4`
2. `MAX(TRUE,"2") -> 2`
3. `MAX("x") -> #VALUE!`
4. `MAX(F1:F3) -> 3`
5. `MAX(G1:G2) -> 0`
6. `MAX(G1:G3) -> #N/A`
7. `MIN(2,3,4) -> 2`
8. `MIN(TRUE,"2") -> 1`
9. `MIN("x") -> #VALUE!`
10. `MIN(F1:F3) -> 0`
11. `MIN(G1:G2) -> 0`
12. `MIN(G1:G3) -> #N/A`
