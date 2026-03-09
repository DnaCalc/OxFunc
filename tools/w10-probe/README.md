# W10 Probe Tooling

This folder contains replay tooling for the W10 ten-function mixed-seam empirical suite.

## Scripts
1. `run-w10-suite.ps1`
   - merges W10 S1..S4 manifests,
   - runs dual Excel baseline replays (`default` and `compat_template`),
   - emits combined results and analysis summary.
2. `analyze-w10-results.ps1`
   - summarizes W10 replay metrics and dual-run gate status.
3. `new-w10-compat-template.ps1`
   - creates a legacy `.xls` workbook template for compatibility-version runs.

## Example
```powershell
powershell -File tools/w10-probe/run-w10-suite.ps1 -OutDir .tmp
```

Outputs:
1. `.tmp/w10-scenarios-manifest.csv`
2. `.tmp/w10-results-default.csv`
3. `.tmp/w10-results-compat.csv`
4. `.tmp/w10-results-excel.csv`
5. `.tmp/w10-analysis-report.csv`
6. `.tmp/w10-analysis-summary.json`
