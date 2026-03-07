param(
    [Parameter(Mandatory = $true)]
    [string]$Manifest,

    [Parameter(Mandatory = $true)]
    [string]$Out,

    [string[]]$Lanes = @("ABS5-A", "ABS5-B", "ABS5-C", "ABS5-D", "ABS5-E"),

    [string]$ArtifactRoot = ".tmp/abs-artifacts",

    [string]$WorkbookTemplate = "",

    [string]$RunLabel = "default",

    [switch]$IncludeSeed
)

$ErrorActionPreference = "Stop"

$coercionRunner = Join-Path $PSScriptRoot "..\coercion-probe\run-coercion-excel-baseline.ps1"
$coercionRunnerPath = [System.IO.Path]::GetFullPath($coercionRunner)

$runParams = @{
    Manifest = $Manifest
    Out = $Out
    Lanes = $Lanes
    ArtifactRoot = $ArtifactRoot
    WorkbookTemplate = $WorkbookTemplate
    RunLabel = $RunLabel
}
if ($IncludeSeed) {
    $runParams["IncludeSeed"] = $true
}

& $coercionRunnerPath @runParams

Write-Host "ABS baseline run complete."
Write-Host "Output: $Out"
