# W12 Probe Runtime Requirements (Seed)

Status: `provisional`
Workset: `W12`

## 1. Purpose
Define empirical replay requirements for the W12 moderate fifteen-function packet and its required `CELL` pre-implementation probe.

## 2. Scenario Packs
1. `docs/function-lane/W12_CELL_PRE_SCENARIO_MANIFEST_SEED.csv`
2. `docs/function-lane/W12_S1_SCENARIO_MANIFEST_SEED.csv`
3. `docs/function-lane/W12_S2_SCENARIO_MANIFEST_SEED.csv`
4. `docs/function-lane/W12_S3_SCENARIO_MANIFEST_SEED.csv`
5. `docs/function-lane/W12_S4_SCENARIO_MANIFEST_SEED.csv`
6. `docs/function-lane/W12_S5_SCENARIO_MANIFEST_SEED.csv`
7. `docs/function-lane/W12_S6_SCENARIO_MANIFEST_SEED.csv`

## 3. Required Runtime Lanes
1. formula-only recalculation path for scalar/value-return functions.
2. formula2/dynamic-array path for `HSTACK`.
3. save/reopen/recalc path for volatile date/time lanes.
4. mixed reference/value setup path for `OFFSET` and bounded `CELL` info types.

## 4. Expected Output Artifacts
1. `.tmp/w12-results-default.csv`
2. `.tmp/w12-results-compat.csv`
3. `.tmp/w12-results-excel.csv`
4. `.tmp/w12-analysis-report.csv`
5. `.tmp/w12-analysis-summary.json`
6. `.tmp/w12-results-default.csv.run-metadata.json`
7. `.tmp/w12-results-compat.csv.run-metadata.json`
8. `.tmp/w12-cell-pre-results-default.csv`
9. `.tmp/w12-cell-pre-results-compat.csv`
10. `.tmp/w12-cell-pre-results-excel.csv`
11. `.tmp/w12-cell-pre-analysis-report.csv`
12. `.tmp/w12-cell-pre-analysis-summary.json`

## 5. Replay Commands
1. `powershell -File tools/w12-probe/run-w12-cell-preprobe.ps1 -OutDir .tmp`
2. `powershell -File tools/w12-probe/run-w12-suite.ps1 -OutDir .tmp`

## 6. Classification Rules
1. `CELL` preprobe rows establish the bounded W12 runtime scope before promotion claims for `CELL`.
2. volatile rows (`TODAY`, `RAND`) are expected observed-value lanes, not exact-equality claims; W11 follow-back handles positive/control volatility evidence.
3. reference-return built-in baselines (`OFFSET`, `CELL`) are evidence inputs for bounded runtime admission and W11 macro/caller-context candidate planning.
