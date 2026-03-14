param(
    [string[]]$Manifests = @(
        "docs/function-lane/W13_S1_SCENARIO_MANIFEST_SEED.csv",
        "docs/function-lane/W13_S2_SCENARIO_MANIFEST_SEED.csv"
    ),
    [string]$OutDir = ".tmp"
)

$ErrorActionPreference = "Stop"

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$baselineScript = Join-Path $scriptDir "..\coercion-probe\run-coercion-excel-baseline.ps1"
$newTemplateScript = Join-Path $scriptDir "..\w10-probe\new-w10-compat-template.ps1"

$outDirPath = [System.IO.Path]::GetFullPath($OutDir)
if (-not (Test-Path $outDirPath)) {
    New-Item -ItemType Directory -Path $outDirPath | Out-Null
}

$mergedManifestPath = Join-Path $outDirPath "w13-scenarios-manifest.csv"
$defaultResultsPath = Join-Path $outDirPath "w13-results-default.csv"
$compatResultsPath = Join-Path $outDirPath "w13-results-compat.csv"
$resultsPath = Join-Path $outDirPath "w13-results-excel.csv"
$summaryPath = Join-Path $outDirPath "w13-summary.json"
$compatTemplatePath = Join-Path $outDirPath "w13-compat-template.xls"

$allRows = New-Object System.Collections.Generic.List[object]
foreach ($manifest in $Manifests) {
    $rows = Import-Csv -Path ([System.IO.Path]::GetFullPath($manifest))
    foreach ($row in $rows) { $allRows.Add($row) }
}
$allRows | Export-Csv -Path $mergedManifestPath -NoTypeInformation -Encoding UTF8

& $newTemplateScript -Out $compatTemplatePath
& $baselineScript -Manifest $mergedManifestPath -Out $defaultResultsPath -Lanes @("W13-S1","W13-S2") -RunLabel "default"
& $baselineScript -Manifest $mergedManifestPath -Out $compatResultsPath -Lanes @("W13-S1","W13-S2") -RunLabel "compat_template" -WorkbookTemplate $compatTemplatePath

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

Write-Host "W13 suite complete."
Write-Host "Results: $resultsPath"
Write-Host "Summary: $summaryPath"
