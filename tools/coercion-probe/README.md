# Coercion Probe Tooling (W4)

Purpose:
1. run reproducible Excel baseline scenarios for coercion/reference-resolution behavior,
2. produce expectation-aware verdicts and mismatch counts,
3. emit machine-readable artifacts for W4 conformance linkage.

## Primary scripts
1. `run-coercion-excel-baseline.ps1`
   - executes one coercion baseline profile.
2. `run-coercion-suite.ps1`
   - orchestrates baseline run and analysis output.
3. `analyze-coercion-results.ps1`
   - summarizes expectation and drift metrics.

## Inputs
1. scenario manifest: `docs/function-lane/COERCION_SCENARIO_MANIFEST_SEED.csv`
2. output path(s) for CSV/JSON artifacts.
3. optional baseline CSV for drift comparison.

## Supported action lanes
1. `calculate`
2. `calculate_formula2`
3. `save_reopen_recalc`
4. `csv_roundtrip_values`
5. `external_ref_open_state_compare` (generated workbook, closed-vs-open compare)

## Baseline run example
```powershell
powershell -File tools/coercion-probe/run-coercion-excel-baseline.ps1 -Manifest docs/function-lane/COERCION_SCENARIO_MANIFEST_SEED.csv -Out .tmp/coercion-results-excel.csv
```

Baseline run with compatibility template:
```powershell
powershell -File tools/coercion-probe/run-coercion-excel-baseline.ps1 -Manifest docs/function-lane/COERCION_SCENARIO_MANIFEST_SEED.csv -Out .tmp/coercion-results-compat.csv -WorkbookTemplate .tmp/abs-compat-template.xls -RunLabel compat_template
```

## Suite run example
```powershell
powershell -File tools/coercion-probe/run-coercion-suite.ps1 -Manifest docs/function-lane/COERCION_SCENARIO_MANIFEST_SEED.csv -OutDir .tmp
```

## Template
1. `results/COERCION_RESULTS_TEMPLATE.csv`
