# W16 Batch 19 - Inclusive Average and Deviation Statistics

Status: `in_progress-provisional`
Workset: `W16`
Evidence ID: `W16-BATCH19-AVERAGEA-AVEDEV-DEVSQ-20260315`

## Scope
1. `AVERAGEA`
2. `AVEDEV`
3. `DEVSQ`

## Native Excel Baseline
Probe artifact:
1. `.tmp/w16-batch19-averagea-avedev-devsq-probe.csv`

Pinned lanes:
1. `AVERAGEA(2,3,4) -> 3`
2. `AVERAGEA(TRUE,"2") -> 1.5`
3. `AVERAGEA("x") -> #VALUE!`
4. `AVERAGEA(F1:F3) -> 1.6666666666666667`
5. `AVERAGEA(G1:G2) -> 0.5`
6. `AVERAGEA(G1:G3) -> #N/A`
7. `AVERAGEA(TRUE) -> 1`
8. `AVERAGEA(FALSE) -> 0`
9. `AVERAGEA("") -> #VALUE!`
10. `AVEDEV(2,3,4) -> 0.6666666666666666`
11. `AVEDEV(TRUE,"2") -> 0.5`
12. `AVEDEV("x") -> #VALUE!`
13. `AVEDEV(F1:F3) -> 1.111111111111111`
14. `AVEDEV(G1:G2) -> #NUM!`
15. `AVEDEV(G1:G3) -> #N/A`
16. `AVEDEV(TRUE) -> 0`
17. `AVEDEV(FALSE) -> 0`
18. `AVEDEV("") -> #VALUE!`
19. `DEVSQ(2,3,4) -> 2`
20. `DEVSQ(TRUE,"2") -> 0.5`
21. `DEVSQ("x") -> #VALUE!`
22. `DEVSQ(F1:F3) -> 4.666666666666667`
23. `DEVSQ(G1:G2) -> #NUM!`
24. `DEVSQ(G1:G3) -> #N/A`
25. `DEVSQ(TRUE) -> 0`
26. `DEVSQ(FALSE) -> 0`
27. `DEVSQ("") -> #VALUE!`

## Current Batch Reading
1. `AVERAGEA` uses the inclusive aggregate provenance seam already exercised by `MINA` and `MAXA`.
2. Direct logical scalars are counted as `1` and `0`, direct numeric text is parsed, and direct non-numeric text stays `#VALUE!`.
3. Reference-derived logicals and text participate through the existing dual-policy aggregate walk, which is why `AVERAGEA(G1:G2) -> 0.5` while `AVERAGEA(G1:G3) -> #N/A` once the traversed error is encountered.
4. `AVEDEV` and `DEVSQ` share the same admission policy but retain stricter survivor-count postconditions, so an empty surviving numeric set produces `#NUM!` rather than a numeric result.

## Formal Support
1. Lean metadata bindings for this batch live in:
   - `formal/lean/OxFunc/Functions/AverageAFn.lean`
   - `formal/lean/OxFunc/Functions/AveDevFn.lean`
   - `formal/lean/OxFunc/Functions/DevSqFn.lean`
2. `formal/lean/OxFunc.lean` imports those bindings into the aggregate W16 surface.
3. The current formal slice fixes the shared aggregate metadata only:
   - `argPreparationProfile = valuesOnlyPreAdapter`
   - `coercionLiftProfile = aggregateDirectAndRangeDualPolicy`
   - `surfaceFecDependencyProfile = refOnly`
4. This pass is docs/formal support only for Batch 19; runtime scope is intentionally untouched.
