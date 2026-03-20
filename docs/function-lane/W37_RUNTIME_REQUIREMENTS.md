# W37 Runtime Requirements

## 1. Purpose
Pin the current-baseline Excel-observed solver/publication behavior for the reopened large positive-root `XIRR` lane after `W32`.

## 2. Required Artifacts
1. `docs/function-lane/W37_SCENARIO_MANIFEST_SEED.csv`
2. `tools/w37-probe/run-w37-xirr-large-root-baseline.ps1`
3. `crates/oxfunc_core/examples/w37_xirr_large_root_probe.rs`
4. `.tmp/w37-xirr-large-root-results.csv`

## 3. Required Observable
1. for the seeded two-cashflow positive-root lane, OxFunc must match the current installed Excel observable for each seeded positive guess,
2. the replay must preserve that Excel's published `XIRR` result is a solver/publication observable rather than the exact mathematical closed-form root,
3. no regression may occur on the already repaired negative-root and negative-guess lanes covered by `W29` / `W32`.

## 4. Completion Gate
1. all seeded `W37` rows must match direct Excel,
2. the `W29` benchmark ledger must no longer contain an `all_diverge_or_inconclusive` lane for `XIRR`,
3. blocker `BLK-FN-011` must be resolved.
