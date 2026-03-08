param(
    [Parameter(Mandatory = $true)]
    [string]$Manifest,

    [Parameter(Mandatory = $true)]
    [string]$Out,

    [string[]]$Lanes = @("XM6-A", "XM6-B", "XM6-C", "XM6-D", "XM6-E"),

    [string]$ArtifactRoot = ".tmp/xmatch-artifacts",

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

Write-Host "XMATCH baseline run complete."
Write-Host "Output: $Out"
