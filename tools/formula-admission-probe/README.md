# Formula Admission Probe Tooling

Purpose:
1. characterize formula-admission behavior across entry/evaluation mechanisms,
2. capture whether admission-invalid formulas are rejected before worksheet evaluation,
3. record build/channel/compat metadata for replay.

## Command
```powershell
powershell -File tools/formula-admission-probe/run-formula-admission-baseline.ps1 -Out .tmp/formula-admission-results.csv
```

Optional artifact root for workbook mutation/open probes:
```powershell
powershell -File tools/formula-admission-probe/run-formula-admission-baseline.ps1 -Out .tmp/formula-admission-results.csv -ArtifactRoot C:/Temp/oxfunc_formula_admission_artifacts
```

## Covered Mechanisms
1. `Range.Formula`
2. `Range.Formula2`
3. `Application.Evaluate`
4. `Worksheet.Evaluate`
5. `Application.ExecuteExcel4Macro`
6. `Application.WorksheetFunction.Pi`
7. workbook open path for XML-mutated formulas.

## Output
CSV template:
1. `tools/formula-admission-probe/FORMULA_ADMISSION_RESULTS_TEMPLATE.csv`

Baseline output target:
1. `.tmp/formula-admission-results.csv`

Supplemental mapping artifacts (from baseline characterization run):
1. `.tmp/excel-com-error-mapping.csv`
2. `.tmp/excel-com-error-mapping-with-error-type.csv`
