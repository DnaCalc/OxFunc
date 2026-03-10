param(
    [string]$Manifest = "docs/function-lane/TIME_FORMAT_HINT_SCENARIO_MANIFEST_SEED.csv",
    [string]$OutDir = ".tmp"
)

$ErrorActionPreference = "Stop"

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$baselineScript = Join-Path $scriptDir "run-time-format-hint-baseline.ps1"
$newTemplateScript = Join-Path $scriptDir "..\w10-probe\new-w10-compat-template.ps1"

$outDirPath = [System.IO.Path]::GetFullPath($OutDir)
if (-not (Test-Path $outDirPath)) {
    New-Item -ItemType Directory -Path $outDirPath | Out-Null
}

$defaultResultsPath = Join-Path $outDirPath "time-format-hint-results-default.csv"
$compatResultsPath = Join-Path $outDirPath "time-format-hint-results-compat.csv"
$resultsPath = Join-Path $outDirPath "time-format-hint-results.csv"
$summaryPath = Join-Path $outDirPath "time-format-hint-summary.json"
$compatTemplatePath = Join-Path $outDirPath "time-format-hint-compat-template.xls"

& $newTemplateScript -Out $compatTemplatePath
& $baselineScript -Manifest $Manifest -Out $defaultResultsPath -RunLabel "default"
& $baselineScript -Manifest $Manifest -Out $compatResultsPath -RunLabel "compat_template" -WorkbookTemplate $compatTemplatePath

$all = @()
if (Test-Path $defaultResultsPath) { $all += Import-Csv -Path $defaultResultsPath }
if (Test-Path $compatResultsPath) { $all += Import-Csv -Path $compatResultsPath }
$all | Export-Csv -Path $resultsPath -NoTypeInformation -Encoding UTF8

$matched = @($all | Where-Object { $_.expectation_status -eq "matched" }).Count
$mismatched = @($all | Where-Object { $_.expectation_status -eq "mismatched" }).Count
$failed = @($all | Where-Object { $_.execution_status -eq "failed" }).Count
$summary = [ordered]@{
    results_path = $resultsPath
    row_count = $all.Count
    matched = $matched
    mismatched = $mismatched
    failed = $failed
    dual_run_satisfied = (
        (@($all | Select-Object -ExpandProperty run_label -Unique) -contains "default") -and
        (@($all | Select-Object -ExpandProperty run_label -Unique) -contains "compat_template")
    )
    gate_status = if ($mismatched -eq 0 -and $failed -eq 0) { "green" } else { "needs_attention" }
}
$summary | ConvertTo-Json -Depth 4 | Set-Content -Path $summaryPath -Encoding UTF8

Write-Host "Time format-hint suite complete."
Write-Host "Default: $defaultResultsPath"
Write-Host "Compat:  $compatResultsPath"
Write-Host "Results: $resultsPath"
Write-Host "Summary: $summaryPath"
