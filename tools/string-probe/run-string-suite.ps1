param(
    [Parameter(Mandatory = $true)]
    [string]$Manifest,

    [string]$OutDir = ".tmp",

    [string]$ArtifactRoot = ".tmp/string-artifacts",

    [string]$WorkbookTemplate = "",

    [string]$Baseline = ""
)

$ErrorActionPreference = "Stop"

$outRoot = [System.IO.Path]::GetFullPath($OutDir)
if (-not (Test-Path $outRoot)) {
    New-Item -ItemType Directory -Path $outRoot | Out-Null
}

$artifactRootFull = [System.IO.Path]::GetFullPath($ArtifactRoot)
if (-not (Test-Path $artifactRootFull)) {
    New-Item -ItemType Directory -Path $artifactRootFull | Out-Null
}

$manifestPath = (Resolve-Path -Path $Manifest -ErrorAction Stop).Path

$defaultOut = Join-Path $outRoot "string-results-default.csv"
$compatOut = Join-Path $outRoot "string-results-compat.csv"
$combinedOut = Join-Path $outRoot "string-results-all.csv"
$analysisReport = Join-Path $outRoot "string-analysis-report.csv"
$analysisSummary = Join-Path $outRoot "string-analysis-summary.json"

$runner = Join-Path $PSScriptRoot "run-string-excel-baseline.ps1"
$analyzer = Join-Path $PSScriptRoot "analyze-string-results.ps1"

$defaultArtifactRoot = Join-Path $artifactRootFull "default"
$compatArtifactRoot = Join-Path $artifactRootFull "compat"

& $runner -Manifest $manifestPath -Out $defaultOut -ArtifactRoot $defaultArtifactRoot -RunLabel "default"

$hasCompatRun = $false
if (-not [string]::IsNullOrWhiteSpace($WorkbookTemplate)) {
    & $runner -Manifest $manifestPath -Out $compatOut -ArtifactRoot $compatArtifactRoot -WorkbookTemplate $WorkbookTemplate -RunLabel "compat_template"
    $hasCompatRun = $true
}
else {
    Write-Host "WorkbookTemplate not provided; compatibility-template run skipped."
}

$all = @()
if (Test-Path $defaultOut) { $all += Import-Csv -Path $defaultOut }
if ($hasCompatRun -and (Test-Path $compatOut)) { $all += Import-Csv -Path $compatOut }
$all | Export-Csv -Path $combinedOut -NoTypeInformation -Encoding UTF8

$analysisArgs = @{
    Results = $combinedOut
    OutReport = $analysisReport
    OutSummaryJson = $analysisSummary
}
if (-not [string]::IsNullOrWhiteSpace($Baseline)) {
    $analysisArgs.Baseline = $Baseline
}

& $analyzer @analysisArgs

Write-Host "String suite complete."
Write-Host "  Default run: $defaultOut"
if ($hasCompatRun) { Write-Host "  Compat run:  $compatOut" }
Write-Host "  Combined:    $combinedOut"
Write-Host "  Analysis:    $analysisReport"
Write-Host "  Summary:     $analysisSummary"
