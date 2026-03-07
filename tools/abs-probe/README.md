# ABS Probe Tooling (W5)

Purpose:
1. run reproducible Excel baseline scenarios for `ABS` behavior lanes,
2. produce expectation-aware verdicts and mismatch counts,
3. emit machine-readable artifacts for W5 conformance linkage.

## Primary scripts
1. `run-abs-excel-baseline.ps1`
   - executes one ABS baseline profile.
2. `run-abs-suite.ps1`
   - orchestrates `default` and `compat_template` runs and combined analysis output.
3. `analyze-abs-results.ps1`
   - summarizes expectation, dual-run, and drift metrics.
4. `new-abs-compat-template.ps1`
   - creates a compatibility-template workbook (`.xls`) when none is supplied.
5. `run-abs-entrypoint-baseline.ps1`
   - probes entrypoint-specific behavior (`Range.Formula`, `Evaluate`, `WorksheetFunction.Abs`).
6. `analyze-abs-entrypoint-results.ps1`
   - summarizes entrypoint probe expectation metrics.

## Inputs
1. scenario manifest: `docs/function-lane/ABS_SCENARIO_MANIFEST_SEED.csv`
2. entrypoint manifest: `docs/function-lane/ABS_ENTRYPOINT_SCENARIO_MANIFEST_SEED.csv`
2. output path(s) for CSV/JSON artifacts.
3. optional baseline CSV for drift comparison.

## Supported action lanes
1. `calculate`
2. `calculate_formula2`
3. `save_reopen_recalc`

## Baseline run example
```powershell
powershell -File tools/abs-probe/run-abs-excel-baseline.ps1 -Manifest docs/function-lane/ABS_SCENARIO_MANIFEST_SEED.csv -Out .tmp/abs-results-excel.csv
```

## Suite run example
```powershell
powershell -File tools/abs-probe/run-abs-suite.ps1 -Manifest docs/function-lane/ABS_SCENARIO_MANIFEST_SEED.csv -OutDir .tmp
```

Suite run with explicit compatibility template:
```powershell
powershell -File tools/abs-probe/run-abs-suite.ps1 -Manifest docs/function-lane/ABS_SCENARIO_MANIFEST_SEED.csv -OutDir .tmp -WorkbookTemplate .tmp/abs-compat-template.xls
```

## Entrypoint run example
```powershell
powershell -File tools/abs-probe/run-abs-entrypoint-baseline.ps1 -Manifest docs/function-lane/ABS_ENTRYPOINT_SCENARIO_MANIFEST_SEED.csv -Out .tmp/abs-entrypoint-results.csv
```

## Template
1. `results/ABS_RESULTS_TEMPLATE.csv`
2. `results/ABS_ENTRYPOINT_RESULTS_TEMPLATE.csv`
