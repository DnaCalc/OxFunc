# WORKSET - Statistical Distributions And Compat A (W61)

## 1. Purpose
Promote the third ordinary successor packet from the normalized `W051` backlog by closing the first statistical-distribution wave for the current reference Excel baseline.

This packet closes:
1. `BETA.DIST`
2. `BETA.INV`
3. `BETADIST`
4. `BETAINV`
5. `BINOM.DIST`
6. `BINOM.DIST.RANGE`
7. `BINOM.INV`
8. `BINOMDIST`
9. `CHIDIST`
10. `CHIINV`
11. `CHISQ.DIST`
12. `CHISQ.DIST.RT`
13. `CHISQ.INV`
14. `CHISQ.INV.RT`
15. `CONFIDENCE`
16. `CONFIDENCE.NORM`
17. `CRITBINOM`
18. `EXPON.DIST`
19. `EXPONDIST`
20. `F.DIST`
21. `F.DIST.RT`
22. `F.INV`
23. `F.INV.RT`
24. `FDIST`
25. `FINV`
26. `GAMMA.DIST`
27. `GAMMA.INV`
28. `GAMMADIST`
29. `GAMMAINV`

## 2. Dependencies
1. `W057_HIDDEN_ORDINARY_BACKLOG_SYSTEMATIC_COMPLETION_PLAN.md`
2. `W058_GROUPED_ROW_NORMALIZATION_AND_HIDDEN_BACKLOG_SPLIT.md`
3. `docs/function-lane/W58_SUCCESSOR_PACKET_SPLIT.csv`
4. `W054_LEAN_FORMALIZATION_GAP_RECONCILIATION.md`
5. prior implementation/evidence lineage from:
   - `crates/oxfunc_core/src/functions/beta_gamma_stats_family.rs`
   - `crates/oxfunc_core/src/functions/discrete_dist_family.rs`
   - `crates/oxfunc_core/src/functions/chi_f_t_family.rs`
   - `crates/oxfunc_core/src/functions/normal_log_family.rs`
   - `formal/lean/OxFunc/Functions/DiscreteDistFamily.lean`
   - `formal/lean/OxFunc/Functions/ChiFTFamily.lean`
   - `formal/lean/OxFunc/Functions/NormalLogFamily.lean`

## 3. Scope
In scope:
1. native Excel replay for the declared current-baseline statistical-distribution slice,
2. closure-grade OxFunc runtime and dispatch evidence for all `29` rows,
3. Lean substrate/binding alignment for the beta/gamma and chi/F subfamilies plus the already-covered discrete/normal rows,
4. snapshot/export promotion so these rows stop reading as hidden `catalog_only` backlog,
5. `W051` removal/update for the `29` rows.

Out of scope:
1. locale/version sweeps beyond the declared current reference baseline,
2. the `W062` statistical wave (`T.*`, normal/lognormal legacy rows, percentile/rank, moments, and residual aliases),
3. broader distribution-family refactors beyond the admitted current-baseline closure surface.

## 4. Output Contract
This packet must produce:
1. this workset spec,
2. `docs/function-lane/FUNCTION_SLICE_STATISTICAL_DISTRIBUTIONS_AND_COMPAT_A_CONTRACT_PRELIM.md`,
3. `docs/function-lane/W61_SCENARIO_MANIFEST_SEED.csv`,
4. `docs/function-lane/W61_RUNTIME_REQUIREMENTS.md`,
5. `docs/function-lane/W61_SCOPE_RECONCILIATION.csv`,
6. `docs/function-lane/W61_EXECUTION_RECORD.md`,
7. `.tmp/w61-statistical-distributions-compat-a-results.csv`,
8. updated `W054`, `W051`, and downstream snapshot/labeling surfaces.

## 5. Gate Criteria
`W61` is complete when:
1. native Excel replay exists and matches the seeded statistical-distribution scenarios,
2. targeted Rust tests pass for the covered families,
3. `lake build` passes with the new Lean family coverage,
4. the snapshot generator emits these `29` rows with real metadata rather than `catalog_only`,
5. the `29` rows are removed from active `W051` backlog counts,
6. the corresponding `W054` missing-lean rows are reconciled out of the active gap inventory.

## 6. Status
1. execution_state: `complete`
2. scope_completeness: `scope_complete`
3. target_completeness: `target_complete`
4. integration_completeness: `integrated`
5. open_lanes:
   - none in declared `W061` scope
