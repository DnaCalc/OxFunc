# W16 Batch 16 - Logical Negation and Median

Status: `in_progress-provisional`
Workset: `W16`
Evidence ID: `W16-BATCH16-NOT-MEDIAN-20260315`

## Scope
1. `NOT`
2. `MEDIAN`

## Native Excel Baseline
Probe artifact:
1. `.tmp/w16-batch16-not-median-probe.csv`

Pinned lanes:
1. `NOT(TRUE) -> FALSE`
2. `NOT(0) -> TRUE`
3. `NOT(2) -> FALSE`
4. `NOT("x") -> #VALUE!`
5. `NOT(F3) -> TRUE`
6. `NOT(G2) -> FALSE`
7. `NOT(G1) -> #VALUE!`
8. `NOT(G3) -> #N/A`
9. `MEDIAN(2,3,4) -> 3`
10. `MEDIAN(TRUE,"2") -> 1.5`
11. `MEDIAN("x") -> #VALUE!`
12. `MEDIAN(F1:F3) -> 2`
13. `MEDIAN(G1:G2) -> #NUM!`
14. `MEDIAN(G1:G3) -> #N/A`
15. `MEDIAN(F1:F2) -> 2.5`
