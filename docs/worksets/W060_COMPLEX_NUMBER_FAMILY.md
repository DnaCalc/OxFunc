# WORKSET - Complex Number Family (W60)

## 1. Purpose
Promote the second ordinary successor packet from the normalized `W051` backlog by closing the complex-number family for the current reference Excel baseline.

This packet closes:
1. `COMPLEX`
2. `IMABS`
3. `IMAGINARY`
4. `IMARGUMENT`
5. `IMCONJUGATE`
6. `IMCOS`
7. `IMCOSH`
8. `IMCOT`
9. `IMCSC`
10. `IMCSCH`
11. `IMDIV`
12. `IMEXP`
13. `IMLN`
14. `IMLOG10`
15. `IMLOG2`
16. `IMPOWER`
17. `IMPRODUCT`
18. `IMREAL`
19. `IMSEC`
20. `IMSECH`
21. `IMSIN`
22. `IMSINH`
23. `IMSQRT`
24. `IMSUB`
25. `IMSUM`
26. `IMTAN`

## 2. Dependencies
1. `W058_GROUPED_ROW_NORMALIZATION_AND_HIDDEN_BACKLOG_SPLIT.md`
2. `docs/function-lane/W58_SUCCESSOR_PACKET_SPLIT.csv`
3. prior implementation/evidence lineage from:
   - `docs/function-lane/W16_BATCH52_COMPLEX_FAMILY_NOTES.md`
   - `crates/oxfunc_core/src/functions/complex_family.rs`
   - `formal/lean/OxFunc/Functions/ComplexFamily.lean`

## 3. Scope
In scope:
1. native Excel replay for the declared current-baseline complex-number slice,
2. closure-grade OxFunc runtime and dispatch evidence for all `26` rows,
3. Lean substrate/binding alignment for the complex family,
4. snapshot/export promotion so these rows stop reading as hidden `catalog_only` backlog,
5. `W051` removal/update for the `26` rows.

Out of scope:
1. locale/version sweeps beyond the declared current reference baseline,
2. any widening beyond the current complex-number text/value surface,
3. richer host-side replay or explain carriers beyond ordinary current-phase function closure.

## 4. Output Contract
This packet must produce:
1. this workset spec,
2. `docs/function-lane/FUNCTION_SLICE_COMPLEX_NUMBER_FAMILY_CONTRACT_PRELIM.md`,
3. `docs/function-lane/W60_SCENARIO_MANIFEST_SEED.csv`,
4. `docs/function-lane/W60_RUNTIME_REQUIREMENTS.md`,
5. `docs/function-lane/W60_SCOPE_RECONCILIATION.csv`,
6. `docs/function-lane/W60_EXECUTION_RECORD.md`,
7. `.tmp/w60-complex-family-results.csv`,
8. updated `W051` and downstream snapshot/labeling surfaces.

## 5. Gate Criteria
`W60` is complete when:
1. native Excel replay exists and matches the seeded complex-number scenarios,
2. targeted Rust tests pass for the family,
3. `lake build` passes,
4. the snapshot generator emits these `26` rows with real metadata rather than `catalog_only`,
5. the `26` rows are removed from active `W051` backlog counts.

## 6. Status
1. execution_state: `complete`
2. scope_completeness: `scope_complete`
3. target_completeness: `target_complete`
4. integration_completeness: `integrated`
5. open_lanes:
   - none in declared `W060` scope
