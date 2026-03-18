# W16 Batch 17 - Rank Order Statistics

Status: `in_progress-provisional`
Workset: `W16`
Evidence ID: `W16-BATCH17-LARGE-SMALL-20260315`

## Scope
1. `LARGE`
2. `SMALL`

## Native Excel Baseline
Probe artifact:
1. `.tmp/w16-batch17-large-small-probe.csv`

Pinned lanes:
1. `LARGE(F1:F3,1) -> 3`
2. `LARGE(F1:F3,2) -> 2`
3. `LARGE({TRUE,"2"},1) -> #NUM!`
4. `LARGE(G1:G2,1) -> #NUM!`
5. `LARGE(G1:G3,1) -> #N/A`
6. `LARGE(F1:F3,4) -> #NUM!`
7. `LARGE(F1:F3,1.9) -> 2`
8. `LARGE(F1:F3,0) -> #NUM!`
9. `SMALL(F1:F3,1) -> 0`
10. `SMALL(F1:F3,2) -> 2`
11. `SMALL({TRUE,"2"},1) -> #NUM!`
12. `SMALL(G1:G2,1) -> #NUM!`
13. `SMALL(G1:G3,1) -> #N/A`
14. `SMALL(F1:F3,4) -> #NUM!`
15. `SMALL(F1:F3,1.9) -> 0`
16. `SMALL(F1:F3,0) -> #NUM!`
