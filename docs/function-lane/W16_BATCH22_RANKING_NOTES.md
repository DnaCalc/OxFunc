# W16 Batch 22 - Ranking Functions

Status: `in_progress-provisional`
Workset: `W16`
Evidence ID: `W16-BATCH22-RANKING-20260315`

## Scope
1. `RANK.EQ`
2. `RANK.AVG`
3. `RANK`

## Native Excel Baseline
Probe artifacts:
1. `.tmp/w16-batch22-rank-probe.csv`
2. `.tmp/w16-batch22-rank-edge-probe.csv`
3. `.tmp/w16-batch22-rank-order-probe.csv`

Pinned lanes:
1. `RANK.EQ(20,F1:F4) -> 2`
2. `RANK.EQ(20,F1:F4,1) -> 2`
3. `RANK.EQ(25,F1:F4) -> #N/A`
4. `RANK.EQ(TRUE,F1:F4) -> #N/A`
5. `RANK.EQ("2",F1:F4) -> #N/A`
6. `RANK.EQ(20,G1:G2) -> #N/A`
7. `RANK.AVG(20,F1:F4) -> 2.5`
8. `RANK.AVG(20,F1:F4,1) -> 2.5`
9. `RANK.AVG(25,F1:F4) -> #N/A`
10. `RANK.AVG(TRUE,F1:F4) -> #N/A`
11. `RANK.AVG("2",F1:F4) -> #N/A`
12. `RANK.AVG(20,G1:G2) -> #N/A`
13. `RANK(20,F1:F4) -> 2`
14. `RANK(20,F1:F4,1) -> 2`
15. `RANK(25,F1:F4) -> #N/A`
16. `RANK(20,G1:G2) -> #N/A`
17. `RANK.EQ(20,F1:F4,"1") -> #VALUE!`
18. `RANK.EQ(20,F1:F4,2) -> 2` when the ranked range contains no worksheet error

## Current Implementation Notes
1. The ranking family reuses the existing ordered-numeric survivor policy from `MEDIAN` / `LARGE` / `SMALL`:
   - numbers survive,
   - reference-derived text/logical/blank cells are ignored,
   - errors propagate.
2. The first argument is not coerced from direct logical or text to numeric in the current baseline; nonnumeric direct lookup values yield `#N/A`.
3. The optional order argument is numeric-only and uses `0 => descending`, nonzero => ascending after truncation.
4. `RANK` is treated as the legacy compatibility alias of `RANK.EQ` for the current baseline.
