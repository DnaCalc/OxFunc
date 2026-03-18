# W16 Batch 26 - Percentile and Quartile Functions

Status: `in_progress-provisional`
Workset: `W16`
Evidence ID: `W16-BATCH26-PERCENTILE-QUARTILE-20260315`

## Scope
1. `PERCENTILE.INC`
2. `PERCENTILE.EXC`
3. `QUARTILE.INC`
4. `QUARTILE.EXC`

## Native Excel Baseline
Probe artifact:
1. `.tmp/w16-batch26-percentile-quartile-probe.csv`

Pinned lanes:
1. `PERCENTILE.INC(F1:F5,0) -> 1`
2. `PERCENTILE.INC(F1:F5,0.25) -> 2`
3. `PERCENTILE.INC(F1:F5,0.3) -> 2.2`
4. `PERCENTILE.INC(F1:F5,1) -> 5`
5. `PERCENTILE.INC(F1:F5,-0.1) -> #NUM!`
6. `PERCENTILE.INC(G1:G2,0.5) -> #NUM!`
7. `PERCENTILE.INC(G1:G3,0.5) -> #N/A`
8. `PERCENTILE.EXC(F1:F5,0.25) -> 1.5`
9. `PERCENTILE.EXC(F1:F5,0.3) -> 1.8`
10. `PERCENTILE.EXC(F1:F5,0.75) -> 4.5`
11. `PERCENTILE.EXC(F1:F5,0.1) -> #NUM!`
12. `PERCENTILE.EXC(F1:F5,1) -> #NUM!`
13. `QUARTILE.INC(F1:F5,0) -> 1`
14. `QUARTILE.INC(F1:F5,1) -> 2`
15. `QUARTILE.INC(F1:F5,2) -> 3`
16. `QUARTILE.INC(F1:F5,3) -> 4`
17. `QUARTILE.INC(F1:F5,4) -> 5`
18. `QUARTILE.INC(F1:F5,5) -> #NUM!`
19. `QUARTILE.EXC(F1:F5,1) -> 1.5`
20. `QUARTILE.EXC(F1:F5,2) -> 3`
21. `QUARTILE.EXC(F1:F5,3) -> 4.5`
22. `QUARTILE.EXC(F1:F5,0) -> #NUM!`
23. `QUARTILE.EXC(F1:F5,4) -> #NUM!`

## Current Implementation Notes
1. The batch uses a numeric-only survivor policy:
   - numbers survive,
   - text/logical/blank cells are ignored,
   - worksheet errors propagate.
2. `PERCENTILE.INC` uses inclusive interpolation over `1 + k*(n-1)`.
3. `PERCENTILE.EXC` uses exclusive interpolation over `k*(n+1)` and returns `#NUM!` outside the admitted interior rank band.
4. `QUARTILE.INC` and `QUARTILE.EXC` are implemented as constrained percentile wrappers over quartile index `q/4`.
