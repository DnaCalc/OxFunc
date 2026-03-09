# W10 Probe Runtime Requirements (Seed)

Status: `provisional`
Workset: `W10`

## 1. Purpose
Define empirical replay requirements for W10 ten-function mixed-seam scenario packs.

## 2. Scenario Packs
1. `docs/function-lane/W10_S1_SCENARIO_MANIFEST_SEED.csv`
2. `docs/function-lane/W10_S2_SCENARIO_MANIFEST_SEED.csv`
3. `docs/function-lane/W10_S3_SCENARIO_MANIFEST_SEED.csv`
4. `docs/function-lane/W10_S4_SCENARIO_MANIFEST_SEED.csv`

## 3. Required Runtime Lanes
1. formula-only recalculation path (`Range.Formula` + calculate).
2. dynamic-array path (`Range.Formula2` + spill observation).
3. mixed reference/value input setup path.
4. volatile recalc observation path (`NOW`).

## 4. Expected Output Artifacts
1. `.tmp/w10-scenarios-manifest.csv`
2. `.tmp/w10-results-default.csv`
3. `.tmp/w10-results-compat.csv`
4. `.tmp/w10-results-excel.csv`
5. `.tmp/w10-analysis-report.csv`
6. `.tmp/w10-analysis-summary.json`
7. `.tmp/w10-results-default.csv.run-metadata.json`
8. `.tmp/w10-results-compat.csv.run-metadata.json`

## 5. Replay Command
1. `powershell -File tools/w10-probe/run-w10-suite.ps1 -OutDir .tmp`

## 6. Classification Rules
1. seed-scope unsupported lanes must be encoded as expected outcomes, not unexpected failures.
2. seam-level admission denials are boundary outcomes and must not be counted as function semantic failures.
3. volatile-value exact equality is not required for `NOW`; type/ordering/recalc delta checks are required.
