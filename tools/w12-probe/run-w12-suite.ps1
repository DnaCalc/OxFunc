param(
    [string[]]$Manifests = @(
        "docs/function-lane/W12_S1_SCENARIO_MANIFEST_SEED.csv",
        "docs/function-lane/W12_S2_SCENARIO_MANIFEST_SEED.csv",
        "docs/function-lane/W12_S3_SCENARIO_MANIFEST_SEED.csv",
        "docs/function-lane/W12_S4_SCENARIO_MANIFEST_SEED.csv",
        "docs/function-lane/W12_S5_SCENARIO_MANIFEST_SEED.csv",
        "docs/function-lane/W12_S6_SCENARIO_MANIFEST_SEED.csv"
    ),

    [string]$OutDir = ".tmp",

    [string[]]$Lanes = @("W12-S1", "W12-S2", "W12-S3", "W12-S4", "W12-S5", "W12-S6"),

    [string]$ArtifactRoot = ".tmp/w12-artifacts",

    [string]$WorkbookTemplate = "",

    [switch]$IncludeSeed,

    [string]$Baseline = ""
)

$ErrorActionPreference = "Stop"

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$runScript = Join-Path $scriptDir "..\coercion-probe\run-coercion-excel-baseline.ps1"
$analyzeScript = Join-Path $scriptDir "analyze-w12-results.ps1"
$newTemplateScript = Join-Path $scriptDir "new-w12-compat-template.ps1"

$outDirPath = [System.IO.Path]::GetFullPath($OutDir)
if (-not (Test-Path $outDirPath)) {
    New-Item -ItemType Directory -Path $outDirPath | Out-Null
}

$artifactRootPath = [System.IO.Path]::GetFullPath($ArtifactRoot)
if (-not (Test-Path $artifactRootPath)) {
    New-Item -ItemType Directory -Path $artifactRootPath | Out-Null
}

$mergedManifestPath = Join-Path $outDirPath "w12-scenarios-manifest.csv"
$defaultResultsPath = Join-Path $outDirPath "w12-results-default.csv"
$compatResultsPath = Join-Path $outDirPath "w12-results-compat.csv"
$resultsPath = Join-Path $outDirPath "w12-results-excel.csv"
$reportPath = Join-Path $outDirPath "w12-analysis-report.csv"
$summaryPath = Join-Path $outDirPath "w12-analysis-summary.json"
$isolatedScenarioIds = @(
    "W12S3-005",
    "W12S5-001",
    "W12S5-002",
    "W12S5-003",
    "W12S5-004",
    "W12S5-005",
    "W12S6-004"
)

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

function Invoke-CoercionRun {
    param([hashtable]$RunParams)

    function Quote-PwshString {
        param([string]$Value)
        return "'" + ($Value -replace "'", "''") + "'"
    }

    $commandParts = @(
        "& " + (Quote-PwshString $runScript),
        "-Manifest " + (Quote-PwshString ([string]$RunParams["Manifest"])),
        "-Out " + (Quote-PwshString ([string]$RunParams["Out"])),
        "-ArtifactRoot " + (Quote-PwshString ([string]$RunParams["ArtifactRoot"])),
        "-RunLabel " + (Quote-PwshString ([string]$RunParams["RunLabel"]))
    )

    if ($RunParams.ContainsKey("WorkbookTemplate")) {
        $commandParts += "-WorkbookTemplate " + (Quote-PwshString ([string]$RunParams["WorkbookTemplate"]))
    }
    if ($RunParams.ContainsKey("IncludeSeed") -and $RunParams["IncludeSeed"]) {
        $commandParts += "-IncludeSeed"
    }
    if ($RunParams.ContainsKey("Lanes")) {
        $laneTerms = @($RunParams["Lanes"] | ForEach-Object { Quote-PwshString ([string]$_) })
        $commandParts += "-Lanes @(" + ($laneTerms -join ",") + ")"
    }

    $commandText = $commandParts -join " "
    & powershell -Command $commandText
    if ($LASTEXITCODE -ne 0) {
        throw "Coercion runner failed with exit code $LASTEXITCODE."
    }
}

function Invoke-IsolatedReruns {
    param(
        [object[]]$Rows,
        [hashtable]$RunParams,
        [string]$OutPath,
        [string]$RunLabel
    )

    if ($Rows.Count -eq 0) {
        return
    }

    $existing = @()
    if (Test-Path $OutPath) {
        $existing = @(Import-Csv -Path $OutPath)
    }

    $rerunResults = New-Object System.Collections.Generic.List[object]
    foreach ($scenarioId in $isolatedScenarioIds) {
        $row = $Rows | Where-Object { $_.scenario_id -eq $scenarioId } | Select-Object -First 1
        if ($null -eq $row) {
            continue
        }

        $singleManifestPath = Join-Path $outDirPath ("w12-isolated-" + $RunLabel + "-" + $scenarioId + ".csv")
        @($row) | Export-Csv -Path $singleManifestPath -NoTypeInformation -Encoding UTF8

        $singleResultPath = Join-Path $outDirPath ("w12-isolated-" + $RunLabel + "-" + $scenarioId + "-results.csv")
        $singleArtifactRoot = Join-Path $artifactRootPath ("isolated-" + $RunLabel + "-" + $scenarioId)

        $singleParams = @{
            Manifest = $singleManifestPath
            Out = $singleResultPath
            Lanes = @([string]$row.lane)
            ArtifactRoot = $singleArtifactRoot
            RunLabel = $RunLabel
        }
        if ($RunParams.ContainsKey("WorkbookTemplate")) {
            $singleParams["WorkbookTemplate"] = $RunParams["WorkbookTemplate"]
        }
        if ($IncludeSeed) {
            $singleParams["IncludeSeed"] = $true
        }

        Invoke-CoercionRun -RunParams $singleParams
        $singleRows = Import-Csv -Path $singleResultPath
        foreach ($singleRow in $singleRows) {
            $rerunResults.Add($singleRow)
        }
    }

    $rerunArray = @($rerunResults.ToArray())
    $rerunKeys = @($rerunArray | ForEach-Object { ([string]$_.scenario_id) + "|" + ([string]$_.run_label) })
    $filtered = @(
        $existing | Where-Object {
            $key = ([string]$_.scenario_id) + "|" + ([string]$_.run_label)
            $key -notin $rerunKeys
        }
    )
    $merged = @($filtered) + $rerunArray
    $merged | Export-Csv -Path $OutPath -NoTypeInformation -Encoding UTF8
}

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

Invoke-CoercionRun -RunParams $defaultRunParams
Invoke-IsolatedReruns -Rows $mergedRows.ToArray() -RunParams $defaultRunParams -OutPath $defaultResultsPath -RunLabel "default"

$compatTemplatePath = ""
if (-not [string]::IsNullOrWhiteSpace($WorkbookTemplate)) {
    $compatTemplatePath = [System.IO.Path]::GetFullPath($WorkbookTemplate)
}
else {
    $compatTemplatePath = Join-Path $outDirPath "w12-compat-template.xls"
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

Invoke-CoercionRun -RunParams $compatRunParams
Invoke-IsolatedReruns -Rows $mergedRows.ToArray() -RunParams $compatRunParams -OutPath $compatResultsPath -RunLabel "compat_template"

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

Write-Host "W12 suite complete."
Write-Host "Manifest: $mergedManifestPath"
Write-Host "Default:  $defaultResultsPath"
Write-Host "Compat:   $compatResultsPath"
Write-Host "Results:  $resultsPath"
Write-Host "Report:   $reportPath"
Write-Host "Summary:  $summaryPath"
