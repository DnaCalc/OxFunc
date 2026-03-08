param(
    [Parameter(Mandatory = $true)]
    [string]$Manifest,

    [string]$OutDir = ".tmp",

    [string[]]$Lanes = @("XM6-A", "XM6-B", "XM6-C", "XM6-D", "XM6-E"),

    [string]$ArtifactRoot = ".tmp/xmatch-artifacts",

    [string]$WorkbookTemplate = "",

    [switch]$IncludeSeed,

    [string]$Baseline = ""
)

$ErrorActionPreference = "Stop"

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$runScript = Join-Path $scriptDir "run-xmatch-excel-baseline.ps1"
$analyzeScript = Join-Path $scriptDir "analyze-xmatch-results.ps1"
$newTemplateScript = Join-Path $scriptDir "new-xmatch-compat-template.ps1"

$outDirPath = [System.IO.Path]::GetFullPath($OutDir)
if (-not (Test-Path $outDirPath)) {
    New-Item -ItemType Directory -Path $outDirPath | Out-Null
}

$artifactRootPath = [System.IO.Path]::GetFullPath($ArtifactRoot)
if (-not (Test-Path $artifactRootPath)) {
    New-Item -ItemType Directory -Path $artifactRootPath | Out-Null
}

$defaultResultsPath = Join-Path $outDirPath "xmatch-results-default.csv"
$compatResultsPath = Join-Path $outDirPath "xmatch-results-compat.csv"
$resultsPath = Join-Path $outDirPath "xmatch-results-excel.csv"
$reportPath = Join-Path $outDirPath "xmatch-analysis-report.csv"
$summaryPath = Join-Path $outDirPath "xmatch-analysis-summary.json"

$defaultRunParams = @{
    Manifest = $Manifest
    Out = $defaultResultsPath
    Lanes = $Lanes
    ArtifactRoot = (Join-Path $artifactRootPath "default")
    RunLabel = "default"
}
if ($IncludeSeed) {
    $defaultRunParams["IncludeSeed"] = $true
}

& $runScript @defaultRunParams

$compatTemplatePath = ""
if (-not [string]::IsNullOrWhiteSpace($WorkbookTemplate)) {
    $compatTemplatePath = [System.IO.Path]::GetFullPath($WorkbookTemplate)
}
else {
    $compatTemplatePath = Join-Path $outDirPath "xmatch-compat-template.xls"
    & $newTemplateScript -Out $compatTemplatePath
}

$compatRunParams = @{
    Manifest = $Manifest
    Out = $compatResultsPath
    Lanes = $Lanes
    ArtifactRoot = (Join-Path $artifactRootPath "compat")
    WorkbookTemplate = $compatTemplatePath
    RunLabel = "compat_template"
}
if ($IncludeSeed) {
    $compatRunParams["IncludeSeed"] = $true
}

& $runScript @compatRunParams

$all = @()
if (Test-Path $defaultResultsPath) { $all += Import-Csv -Path $defaultResultsPath }
if (Test-Path $compatResultsPath) { $all += Import-Csv -Path $compatResultsPath }
$all | Export-Csv -Path $resultsPath -NoTypeInformation -Encoding UTF8

$analyzeParams = @{
    Results = $resultsPath
    OutReport = $reportPath
    OutSummaryJson = $summaryPath
    RequireDualRuns = $true
}
if (-not [string]::IsNullOrWhiteSpace($Baseline)) {
    $analyzeParams["Baseline"] = $Baseline
}

& $analyzeScript @analyzeParams

Write-Host "XMATCH suite complete."
Write-Host "Default: $defaultResultsPath"
Write-Host "Compat:  $compatResultsPath"
Write-Host "Results: $resultsPath"
Write-Host "Report: $reportPath"
Write-Host "Summary: $summaryPath"
