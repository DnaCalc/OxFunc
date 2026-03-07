param(
    [Parameter(Mandatory = $true)]
    [string]$Manifest,

    [string]$OutDir = ".tmp",

    [string[]]$Lanes = @("CO4-A", "CO4-B", "CO4-C", "CO4-D", "CO4-E", "CO4-F"),

    [switch]$IncludeSeed,

    [string]$Baseline = ""
)

$ErrorActionPreference = "Stop"

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$runScript = Join-Path $scriptDir "run-coercion-excel-baseline.ps1"
$analyzeScript = Join-Path $scriptDir "analyze-coercion-results.ps1"

$outDirPath = [System.IO.Path]::GetFullPath($OutDir)
if (-not (Test-Path $outDirPath)) {
    New-Item -ItemType Directory -Path $outDirPath | Out-Null
}

$resultsPath = Join-Path $outDirPath "coercion-results-excel.csv"
$reportPath = Join-Path $outDirPath "coercion-analysis-report.csv"
$summaryPath = Join-Path $outDirPath "coercion-analysis-summary.json"

$runParams = @{
    Manifest = $Manifest
    Out = $resultsPath
    Lanes = $Lanes
}
if ($IncludeSeed) {
    $runParams["IncludeSeed"] = $true
}

& $runScript @runParams

$analyzeParams = @{
    Results = $resultsPath
    OutReport = $reportPath
    OutSummaryJson = $summaryPath
}
if (-not [string]::IsNullOrWhiteSpace($Baseline)) {
    $analyzeParams["Baseline"] = $Baseline
}

& $analyzeScript @analyzeParams

Write-Host "Coercion suite complete."
Write-Host "Results: $resultsPath"
Write-Host "Report: $reportPath"
Write-Host "Summary: $summaryPath"
