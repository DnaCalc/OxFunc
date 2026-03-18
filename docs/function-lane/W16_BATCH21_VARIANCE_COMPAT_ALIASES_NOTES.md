# W16 Batch 21 - Variance Compatibility Aliases

Status: `in_progress-provisional`
Workset: `W16`
Evidence ID: `W16-BATCH21-VARIANCE-COMPAT-ALIASES-20260315`

## Scope
1. `STDEV`
2. `STDEVP`
3. `VAR`
4. `VARP`

## Native Excel Baseline
Probe artifact:
1. `.tmp/w16-batch21-stdev-var-compat-probe.csv`

Pinned lanes:
1. `STDEV(2,3,4) -> 1`
2. `STDEV(TRUE,"2") -> 0.7071067811865476`
3. `STDEV(F1:F3) -> 1.5275252316519465`
4. `STDEV(G1:G2) -> #DIV/0!`
5. `STDEV(G1:G3) -> #N/A`
6. `STDEVP(2,3,4) -> 0.816496580927726`
7. `STDEVP(TRUE,"2") -> 0.5`
8. `STDEVP(F1:F3) -> 1.247219128924647`
9. `STDEVP(G1:G2) -> #DIV/0!`
10. `STDEVP(G1:G3) -> #N/A`
11. `VAR(2,3,4) -> 1`
12. `VAR(TRUE,"2") -> 0.5`
13. `VAR(F1:F3) -> 2.333333333333333`
14. `VAR(G1:G2) -> #DIV/0!`
15. `VAR(G1:G3) -> #N/A`
16. `VARP(2,3,4) -> 0.6666666666666666`
17. `VARP(TRUE,"2") -> 0.25`
18. `VARP(F1:F3) -> 1.5555555555555556`
19. `VARP(G1:G2) -> #DIV/0!`
20. `VARP(G1:G3) -> #N/A`
