# Function Slice - Statistical Distributions And Compat B Contract (Prelim)

Status: `active`
Owner lane: `OxFunc`
Workset: `W062`

## 1. Purpose
Define the current-phase contract for the `W062` statistical-distribution and compatibility wave.

## 2. Covered Surface
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

## 3. T / Normal / Log Contract
1. `T.DIST`, `T.DIST.2T`, `T.DIST.RT`, `T.INV`, and `T.INV.2T` use the ordinary values-only pre-adapter seam over the current chi/F/T substrate.
2. `TDIST` follows the two-tail `T` distribution lane and `TINV` follows the two-tail inverse lane on the current baseline.
3. `NORMDIST` / `NORMINV` are compatibility aliases of `NORM.DIST` / `NORM.INV`, and `NORMSDIST` / `NORMSINV` are compatibility aliases of `NORM.S.DIST` / `NORM.S.INV`.
4. `LOGINV` is the compatibility alias of `LOGNORM.INV`, and `LOGNORMDIST` follows the cumulative lane of `LOGNORM.DIST`.
5. probability-domain and scale-domain violations across the T / normal / log rows return `#NUM!`.

## 4. Discrete Distribution Contract
1. `POISSON` and `POISSON.DIST` share the same current-baseline Poisson substrate.
2. `HYPGEOMDIST` follows the point-probability lane of `HYPGEOM.DIST`.
3. `NEGBINOMDIST` follows the point-probability lane of `NEGBINOM.DIST`.
4. invalid discrete-distribution parameter lanes return `#NUM!`.

## 5. Legacy Alias / Moment Contract
1. `COVAR` follows the `COVARIANCE.P` kernel on the current baseline.
2. `MODE`, `PERCENTILE`, `PERCENTRANK`, and `QUARTILE` follow the current `.SNGL` / `.INC` compatibility lanes used by the repo.
3. `KURT`, `SKEW`, `SKEW.P`, and `TRIMMEAN` use the aggregate direct-and-range dual policy over ordinary scalar numeric publication.
4. `STEYX` follows the paired regression residual lane with the current-baseline small-negative-residual clamp to zero.
5. `MODE` no-match publication returns `#N/A`.

## 6. Runtime / Formal Anchors
Runtime anchors:
1. `crates/oxfunc_core/src/functions/chi_f_t_family.rs`
2. `crates/oxfunc_core/src/functions/discrete_dist_family.rs`
3. `crates/oxfunc_core/src/functions/normal_log_family.rs`
4. `crates/oxfunc_core/src/functions/legacy_stats_alias_family.rs`
5. `crates/oxfunc_core/src/functions/moment_stats_family.rs`

Formal anchors:
1. `formal/lean/OxFunc/Functions/ChiFTFamily.lean`
2. `formal/lean/OxFunc/Functions/DiscreteDistFamily.lean`
3. `formal/lean/OxFunc/Functions/NormalLogFamily.lean`
4. `formal/lean/OxFunc/Functions/LegacyStatsAliasFamily.lean`
5. `formal/lean/OxFunc/Functions/MomentStatsFamily.lean`

Native replay anchors:
1. `docs/function-lane/W62_SCENARIO_MANIFEST_SEED.csv`
2. `tools/w62-probe/run-w62-statistical-distributions-compat-b-baseline.ps1`
3. `.tmp/w62-statistical-distributions-compat-b-results.csv`
