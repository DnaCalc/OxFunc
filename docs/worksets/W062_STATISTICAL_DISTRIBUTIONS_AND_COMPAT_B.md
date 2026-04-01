# WORKSET - Statistical Distributions And Compat B (W62)

## 1. Purpose
Promote the fourth ordinary successor packet from the normalized `W051` backlog by closing the second statistical-distribution wave for the current reference Excel baseline.

This packet closes:
1. `COVAR`
2. `HYPGEOM.DIST`
3. `HYPGEOMDIST`
4. `KURT`
5. `LOGINV`
6. `LOGNORM.DIST`
7. `LOGNORM.INV`
8. `LOGNORMDIST`
9. `MODE`
10. `NEGBINOM.DIST`
11. `NEGBINOMDIST`
12. `NORM.DIST`
13. `NORM.INV`
14. `NORM.S.DIST`
15. `NORM.S.INV`
16. `NORMDIST`
17. `NORMINV`
18. `NORMSDIST`
19. `NORMSINV`
20. `PERCENTILE`
21. `PERCENTRANK`
22. `POISSON`
23. `POISSON.DIST`
24. `QUARTILE`
25. `SKEW`
26. `SKEW.P`
27. `STEYX`
28. `T.DIST`
29. `T.DIST.2T`
30. `T.DIST.RT`
31. `T.INV`
32. `T.INV.2T`
33. `TDIST`
34. `TINV`
35. `TRIMMEAN`

## 2. Dependencies
1. `W057_HIDDEN_ORDINARY_BACKLOG_SYSTEMATIC_COMPLETION_PLAN.md`
2. `W058_GROUPED_ROW_NORMALIZATION_AND_HIDDEN_BACKLOG_SPLIT.md`
3. `docs/function-lane/W58_SUCCESSOR_PACKET_SPLIT.csv`
4. `W054_LEAN_FORMALIZATION_GAP_RECONCILIATION.md`
5. prior implementation/evidence lineage from:
   - `crates/oxfunc_core/src/functions/chi_f_t_family.rs`
   - `crates/oxfunc_core/src/functions/discrete_dist_family.rs`
   - `crates/oxfunc_core/src/functions/normal_log_family.rs`
   - `crates/oxfunc_core/src/functions/legacy_stats_alias_family.rs`
   - `crates/oxfunc_core/src/functions/moment_stats_family.rs`
   - `formal/lean/OxFunc/Functions/ChiFTFamily.lean`
   - `formal/lean/OxFunc/Functions/DiscreteDistFamily.lean`
   - `formal/lean/OxFunc/Functions/NormalLogFamily.lean`
   - `formal/lean/OxFunc/Functions/LegacyStatsAliasFamily.lean`
   - `formal/lean/OxFunc/Functions/MomentStatsFamily.lean`

## 3. Scope
In scope:
1. native Excel replay for the declared current-baseline statistical-distribution and residual compatibility slice,
2. closure-grade OxFunc runtime and dispatch evidence for all `35` rows,
3. Lean substrate/binding alignment for the remaining T-family ids plus the already-covered normal/log, discrete-distribution, legacy-alias, and moment-statistics rows,
4. snapshot/export promotion so these rows stop reading as hidden `catalog_only` backlog,
5. `W051` removal/update for the `35` rows.

Out of scope:
1. locale/version sweeps beyond the declared current reference baseline,
2. the `W063` date/time and business-day wave,
3. broader statistical-family refactors beyond the admitted current-baseline closure surface.

## 4. Output Contract
This packet must produce:
1. this workset spec,
2. `docs/function-lane/FUNCTION_SLICE_STATISTICAL_DISTRIBUTIONS_AND_COMPAT_B_CONTRACT_PRELIM.md`,
3. `docs/function-lane/W62_SCENARIO_MANIFEST_SEED.csv`,
4. `docs/function-lane/W62_RUNTIME_REQUIREMENTS.md`,
5. `docs/function-lane/W62_SCOPE_RECONCILIATION.csv`,
6. `docs/function-lane/W62_EXECUTION_RECORD.md`,
7. `.tmp/w62-statistical-distributions-compat-b-results.csv`,
8. updated `W054`, `W051`, and downstream snapshot/labeling surfaces.

## 5. Gate Criteria
`W62` is complete when:
1. native Excel replay exists and matches the seeded statistical-distribution and residual-compatibility scenarios,
2. targeted Rust tests pass for the covered families,
3. `lake build` passes with the remaining T-family Lean coverage,
4. the snapshot generator emits these `35` rows with real metadata rather than `catalog_only`,
5. the `35` rows are removed from active `W051` backlog counts,
6. the corresponding `W054` missing-lean rows are reconciled out of the active gap inventory.

## 6. Status
1. execution_state: `complete`
2. scope_completeness: `scope_complete`
3. target_completeness: `target_complete`
4. integration_completeness: `integrated`
5. open_lanes:
   - none in declared `W062` scope
