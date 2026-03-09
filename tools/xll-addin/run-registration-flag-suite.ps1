param(
    [string]$Manifest = "docs/function-lane/XLL_REGISTRATION_FLAG_SCENARIO_MANIFEST_SEED.csv",

    [string]$OutDir = ".tmp",

    [string]$WorkbookTemplate = "",

    [string]$XllPath = "",

    [switch]$BuildIfMissing,

    [string[]]$Lanes = @("W11-VOL", "W11-TS", "W11-MAC")
)

$ErrorActionPreference = "Stop"

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$runScript = Join-Path $scriptDir "run-registration-flag-evidence.ps1"
$analyzeScript = Join-Path $scriptDir "analyze-registration-flag-results.ps1"
$newTemplateScript = Join-Path $scriptDir "..\w10-probe\new-w10-compat-template.ps1"

$outDirPath = [System.IO.Path]::GetFullPath($OutDir)
if (-not (Test-Path $outDirPath)) {
    New-Item -ItemType Directory -Path $outDirPath | Out-Null
}

$manifestPath = [System.IO.Path]::GetFullPath($Manifest)
if (-not (Test-Path $manifestPath)) {
    throw "Manifest not found: $manifestPath"
}

$defaultResultsPath = Join-Path $outDirPath "xll-registration-flags-results-default.csv"
$compatResultsPath = Join-Path $outDirPath "xll-registration-flags-results-compat.csv"
$resultsPath = Join-Path $outDirPath "xll-registration-flags-results-excel.csv"
$reportPath = Join-Path $outDirPath "xll-registration-flags-analysis-report.csv"
$summaryPath = Join-Path $outDirPath "xll-registration-flags-analysis-summary.json"

$defaultParams = @{
    Manifest = $manifestPath
    Out = $defaultResultsPath
    Lanes = $Lanes
    RunLabel = "default"
}
if (-not [string]::IsNullOrWhiteSpace($XllPath)) {
    $defaultParams["XllPath"] = [System.IO.Path]::GetFullPath($XllPath)
}
if ($BuildIfMissing) {
    $defaultParams["BuildIfMissing"] = $true
}

& $runScript @defaultParams

$compatTemplatePath = ""
if (-not [string]::IsNullOrWhiteSpace($WorkbookTemplate)) {
    $compatTemplatePath = [System.IO.Path]::GetFullPath($WorkbookTemplate)
}
else {
    $compatTemplatePath = Join-Path $outDirPath "xll-registration-flags-compat-template.xls"
    & $newTemplateScript -Out $compatTemplatePath
}

$compatParams = @{
    Manifest = $manifestPath
    Out = $compatResultsPath
    Lanes = $Lanes
    WorkbookTemplate = $compatTemplatePath
    RunLabel = "compat_template"
}
if (-not [string]::IsNullOrWhiteSpace($XllPath)) {
    $compatParams["XllPath"] = [System.IO.Path]::GetFullPath($XllPath)
}
if ($BuildIfMissing) {
    $compatParams["BuildIfMissing"] = $true
}

& $runScript @compatParams

$all = @()
if (Test-Path $defaultResultsPath) { $all += Import-Csv -Path $defaultResultsPath }
if (Test-Path $compatResultsPath) { $all += Import-Csv -Path $compatResultsPath }
$all | Export-Csv -Path $resultsPath -NoTypeInformation -Encoding UTF8

& $analyzeScript -Results $resultsPath -OutReport $reportPath -OutSummaryJson $summaryPath -RequireDualRuns $true

Write-Host "XLL registration flag suite complete."
Write-Host "Default: $defaultResultsPath"
Write-Host "Compat:  $compatResultsPath"
Write-Host "Results: $resultsPath"
Write-Host "Report:  $reportPath"
Write-Host "Summary: $summaryPath"
