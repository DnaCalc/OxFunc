param(
    [string]$OutDir = ".tmp",
    [string]$WorkbookTemplate = "",
    [switch]$IncludeSeed
)

$ErrorActionPreference = "Stop"

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$runScript = Join-Path $scriptDir "run-w12-suite.ps1"

& $runScript `
    -Manifests @("docs/function-lane/W12_CELL_PRE_SCENARIO_MANIFEST_SEED.csv") `
    -Lanes @("W12-CELL-PRE") `
    -ArtifactRoot (Join-Path $OutDir "w12-cell-pre-artifacts") `
    -OutDir $OutDir `
    -WorkbookTemplate $WorkbookTemplate `
    -IncludeSeed:$IncludeSeed

if (Test-Path (Join-Path $OutDir "w12-results-default.csv")) {
    Move-Item -Force (Join-Path $OutDir "w12-results-default.csv") (Join-Path $OutDir "w12-cell-pre-results-default.csv")
}
if (Test-Path (Join-Path $OutDir "w12-results-compat.csv")) {
    Move-Item -Force (Join-Path $OutDir "w12-results-compat.csv") (Join-Path $OutDir "w12-cell-pre-results-compat.csv")
}
if (Test-Path (Join-Path $OutDir "w12-results-excel.csv")) {
    Move-Item -Force (Join-Path $OutDir "w12-results-excel.csv") (Join-Path $OutDir "w12-cell-pre-results-excel.csv")
}
if (Test-Path (Join-Path $OutDir "w12-analysis-report.csv")) {
    Move-Item -Force (Join-Path $OutDir "w12-analysis-report.csv") (Join-Path $OutDir "w12-cell-pre-analysis-report.csv")
}
if (Test-Path (Join-Path $OutDir "w12-analysis-summary.json")) {
    Move-Item -Force (Join-Path $OutDir "w12-analysis-summary.json") (Join-Path $OutDir "w12-cell-pre-analysis-summary.json")
}
