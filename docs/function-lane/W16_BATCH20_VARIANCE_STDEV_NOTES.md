# W16 Batch 20 - Variance and Standard Deviation Families

Status: `in_progress-provisional`
Workset: `W16`
Evidence ID: `W16-BATCH20-VARIANCE-STDEV-20260315`

## Scope
1. `STDEV.S`
2. `STDEV.P`
3. `STDEVA`
4. `STDEVPA`
5. `STDEV`
6. `STDEVP`
7. `VAR.S`
8. `VAR.P`
9. `VARA`
10. `VARPA`
11. `VAR`
12. `VARP`

## Native Excel Baseline
Probe artifact:
1. `.tmp/w16-batch20-stdev-var-probe.csv`

Pinned lanes:
1. `STDEV.S(2,3,4) -> 1`
2. `STDEV.S(TRUE,"2") -> 0.7071067811865476`
3. `STDEV.S("x") -> #VALUE!`
4. `STDEV.S(F1:F3) -> 1.5275252316519465`
5. `STDEV.S(G1:G2) -> #DIV/0!`
6. `STDEV.S(G1:G3) -> #N/A`
7. `STDEV.S(TRUE) -> #DIV/0!`
8. `STDEV.P(2,3,4) -> 0.816496580927726`
9. `STDEV.P(TRUE,"2") -> 0.5`
10. `STDEV.P(G1:G2) -> #DIV/0!`
11. `STDEV.P(TRUE) -> 0`
12. `STDEV.P(FALSE) -> 0`
13. `STDEVA(TRUE,"2") -> 0.7071067811865476`
14. `STDEVA(G1:G2) -> 0.7071067811865476`
15. `STDEVA(TRUE) -> #DIV/0!`
16. `STDEVPA(TRUE,"2") -> 0.5`
17. `STDEVPA(G1:G2) -> 0.5`
18. `STDEVPA(TRUE) -> 0`
19. `STDEV(2,3,4) -> 1`
20. `STDEVP(2,3,4) -> 0.816496580927726`
21. `VAR.S(2,3,4) -> 1`
22. `VAR.S(TRUE,"2") -> 0.5`
23. `VAR.S(G1:G2) -> #DIV/0!`
24. `VAR.P(2,3,4) -> 0.6666666666666666`
25. `VAR.P(TRUE,"2") -> 0.25`
26. `VAR.P(G1:G2) -> #DIV/0!`
27. `VAR.P(TRUE) -> 0`
28. `VARA(TRUE,"2") -> 0.5`
29. `VARA(G1:G2) -> 0.5`
30. `VARPA(TRUE,"2") -> 0.25`
31. `VARPA(G1:G2) -> 0.25`
32. `VARPA(TRUE) -> 0`
33. `VARPA("") -> #VALUE!`
34. `VAR(2,3,4) -> 1`
35. `VARP(2,3,4) -> 0.6666666666666666`
