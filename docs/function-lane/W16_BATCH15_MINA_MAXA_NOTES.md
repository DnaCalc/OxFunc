# W16 Batch 15 - Inclusive Extremum Aggregate Functions

Status: `in_progress-provisional`
Workset: `W16`
Evidence ID: `W16-BATCH15-MINA-MAXA-20260315`

## Scope
1. `MINA`
2. `MAXA`

## Native Excel Baseline
Probe artifact:
1. `.tmp/w16-batch15-mina-maxa-probe.csv`

Pinned lanes:
1. `MAXA(2,3,4) -> 4`
2. `MAXA(TRUE,"2") -> 2`
3. `MAXA("x") -> #VALUE!`
4. `MAXA(F1:F3) -> 3`
5. `MAXA(G1:G2) -> 1`
6. `MAXA(G1:G3) -> #N/A`
7. `MINA(2,3,4) -> 2`
8. `MINA(TRUE,"2") -> 1`
9. `MINA("x") -> #VALUE!`
10. `MINA(F1:F3) -> 0`
11. `MINA(G1:G2) -> 0`
12. `MINA(G1:G3) -> #N/A`
