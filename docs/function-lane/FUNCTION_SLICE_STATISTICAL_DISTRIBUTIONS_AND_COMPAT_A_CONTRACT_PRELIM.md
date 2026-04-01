# Function Slice - Statistical Distributions And Compat A Contract (Prelim)

Status: `active`
Owner lane: `OxFunc`
Workset: `W061`

## 1. Purpose
Define the current-phase contract for the `W061` statistical-distribution and compatibility wave.

## 2. Covered Surface
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

## 3. Beta / Gamma Contract
1. the beta/gamma modern and legacy rows use the ordinary values-only pre-adapter seam and custom numeric kernels,
2. `BETADIST` / `BETAINV` follow the modern beta substrate with implicit bounds `A = 0`, `B = 1`,
3. `GAMMADIST` / `GAMMAINV` follow the same current-baseline scalar substrate as `GAMMA.DIST` / `GAMMA.INV`,
4. invalid probability, shape, or range lanes return `#NUM!`, while non-finite numeric inputs reject with the worksheet error lane recorded by the current baseline.

## 4. Binomial / Exponential Contract
1. `BINOM.DIST`, `BINOM.DIST.RANGE`, `BINOM.INV`, `BINOMDIST`, `CRITBINOM`, `EXPON.DIST`, and `EXPONDIST` use the ordinary values-only pre-adapter seam,
2. boolean cumulative flags follow Excel’s ordinary nonzero-truthy numeric coercion,
3. `BINOMDIST` and `CRITBINOM` are compatibility aliases of the modern binomial kernels,
4. `EXPONDIST` is the compatibility alias of `EXPON.DIST`,
5. out-of-domain numeric lanes return `#NUM!`.

## 5. Chi / F / Confidence Contract
1. `CHIDIST` maps to the right-tail chi-square lane and `CHIINV` maps to the right-tail inverse lane on the current baseline,
2. `FDIST` maps to the right-tail `F` distribution lane and `FINV` maps to the right-tail inverse lane,
3. `CHISQ.*` and `F.*` degrees-of-freedom arguments follow the current-baseline positive-integer truncation rule,
4. `CONFIDENCE` and `CONFIDENCE.NORM` share the same current-baseline normal-confidence kernel,
5. invalid numeric domains across the chi/F/confidence rows return `#NUM!`.

## 6. Runtime / Formal Anchors
Runtime anchors:
1. `crates/oxfunc_core/src/functions/beta_gamma_stats_family.rs`
2. `crates/oxfunc_core/src/functions/discrete_dist_family.rs`
3. `crates/oxfunc_core/src/functions/chi_f_t_family.rs`
4. `crates/oxfunc_core/src/functions/normal_log_family.rs`

Formal anchors:
1. `formal/lean/OxFunc/Functions/BetaGammaStatsFamily.lean`
2. `formal/lean/OxFunc/Functions/DiscreteDistFamily.lean`
3. `formal/lean/OxFunc/Functions/ChiFTFamily.lean`
4. `formal/lean/OxFunc/Functions/NormalLogFamily.lean`

Native replay anchors:
1. `docs/function-lane/W61_SCENARIO_MANIFEST_SEED.csv`
2. `tools/w61-probe/run-w61-statistical-distributions-compat-a-baseline.ps1`
3. `.tmp/w61-statistical-distributions-compat-a-results.csv`
