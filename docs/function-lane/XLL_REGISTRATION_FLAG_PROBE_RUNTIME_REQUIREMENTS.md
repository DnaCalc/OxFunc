# XLL Registration Flag Probe Runtime Requirements

Status: `provisional`
Workset: `W11`

## 1. Purpose
Define replay requirements for volatile/thread-safe/macro-type XLL registration-flag evidence prior to profile-driven mapping.

## 2. Scenario Manifest
1. `docs/function-lane/XLL_REGISTRATION_FLAG_SCENARIO_MANIFEST_SEED.csv`

## 3. Experimental Registration Mode
1. Runtime alias registrations are enabled by default.
2. Optional override:
   - set `OXFUNC_XLL_ENABLE_FLAG_EXPERIMENTS=0` to disable alias registration.
3. Aliases are registered in `xlAutoOpen` without changing profile-to-signature generation.
4. Seed alias set:
   - `ox_NOW_F_BASE` vs `ox_NOW_F_VOL`
   - `ox_ABS_F_BASE` vs `ox_ABS_F_TS`
   - `ox_INDIRECT_F_BASE` vs `ox_INDIRECT_F_MACRO`

## 4. Required Runtime Lanes
1. repeated recalc lane (`calculate_twice`) for volatile checks.
2. scalar parity lane for thread-safe registration acceptance.
3. macro-type registration admission lane (seed-bounded while reference-return path remains partial).
4. dual-run replay lanes:
   - default workbook template,
   - compatibility template workbook.

## 5. Expected Output Artifacts
1. `.tmp/xll-registration-flags-results-default.csv`
2. `.tmp/xll-registration-flags-results-compat.csv`
3. `.tmp/xll-registration-flags-results-excel.csv`
4. `.tmp/xll-registration-flags-analysis-report.csv`
5. `.tmp/xll-registration-flags-analysis-summary.json`
6. `.tmp/xll-registration-flags-results-default.csv.run-metadata.json`
7. `.tmp/xll-registration-flags-results-compat.csv.run-metadata.json`

## 6. Replay Command
```powershell
powershell -File tools/xll-addin/run-registration-flag-suite.ps1 -OutDir .tmp -BuildIfMissing
```

## 7. Classification Rules
1. expectation mismatches must be classified by lane (`W11-VOL`, `W11-TS`, `W11-MAC`).
2. seed-bounded macro/reference-return limitations are recorded as bounded lanes, not silently ignored.
3. thread-safe evidence is not closed by scalar parity alone; multi-thread scheduling evidence remains required before mapping.
