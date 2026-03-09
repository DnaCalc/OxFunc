param(
    [string[]]$Manifests = @(
        "docs/function-lane/W10_S1_SCENARIO_MANIFEST_SEED.csv",
        "docs/function-lane/W10_S2_SCENARIO_MANIFEST_SEED.csv",
        "docs/function-lane/W10_S3_SCENARIO_MANIFEST_SEED.csv",
        "docs/function-lane/W10_S4_SCENARIO_MANIFEST_SEED.csv"
    ),

    [string]$OutDir = ".tmp",

    [string[]]$Lanes = @("W10-S1", "W10-S2", "W10-S3", "W10-S4"),

    [string]$ArtifactRoot = ".tmp/w10-artifacts",

    [string]$WorkbookTemplate = "",

    [switch]$IncludeSeed,

    [string]$Baseline = ""
)

$ErrorActionPreference = "Stop"

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$runScript = Join-Path $scriptDir "..\coercion-probe\run-coercion-excel-baseline.ps1"
$analyzeScript = Join-Path $scriptDir "analyze-w10-results.ps1"
$newTemplateScript = Join-Path $scriptDir "new-w10-compat-template.ps1"

$outDirPath = [System.IO.Path]::GetFullPath($OutDir)
if (-not (Test-Path $outDirPath)) {
    New-Item -ItemType Directory -Path $outDirPath | Out-Null
}

$artifactRootPath = [System.IO.Path]::GetFullPath($ArtifactRoot)
if (-not (Test-Path $artifactRootPath)) {
    New-Item -ItemType Directory -Path $artifactRootPath | Out-Null
}

$mergedManifestPath = Join-Path $outDirPath "w10-scenarios-manifest.csv"
$defaultResultsPath = Join-Path $outDirPath "w10-results-default.csv"
$compatResultsPath = Join-Path $outDirPath "w10-results-compat.csv"
$resultsPath = Join-Path $outDirPath "w10-results-excel.csv"
$reportPath = Join-Path $outDirPath "w10-analysis-report.csv"
$summaryPath = Join-Path $outDirPath "w10-analysis-summary.json"

$mergedRows = New-Object System.Collections.Generic.List[object]

foreach ($manifest in $Manifests) {
    $manifestPath = [System.IO.Path]::GetFullPath($manifest)
    if (-not (Test-Path $manifestPath)) {
        throw "Manifest not found: $manifestPath"
    }

    $rows = Import-Csv -Path $manifestPath
    foreach ($row in $rows) {
        $mergedRows.Add($row)
    }
}

$mergedRows | Export-Csv -Path $mergedManifestPath -NoTypeInformation -Encoding UTF8

$defaultRunParams = @{
    Manifest = $mergedManifestPath
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
    $compatTemplatePath = Join-Path $outDirPath "w10-compat-template.xls"
    & $newTemplateScript -Out $compatTemplatePath
}

$compatRunParams = @{
    Manifest = $mergedManifestPath
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

Write-Host "W10 suite complete."
Write-Host "Manifest: $mergedManifestPath"
Write-Host "Default:  $defaultResultsPath"
Write-Host "Compat:   $compatResultsPath"
Write-Host "Results:  $resultsPath"
Write-Host "Report:   $reportPath"
Write-Host "Summary:  $summaryPath"
