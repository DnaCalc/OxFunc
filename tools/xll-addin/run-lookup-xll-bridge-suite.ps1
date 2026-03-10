param(
    [string]$Manifest = "docs/function-lane/LOOKUP_XLL_BRIDGE_SCENARIO_MANIFEST_SEED.csv",

    [string]$OutDir = ".tmp",

    [string]$XllPath = "",

    [switch]$BuildIfMissing
)

$ErrorActionPreference = "Stop"

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$runScript = Join-Path $scriptDir "run-oxfunc-xll-bridge-baseline.ps1"

$outDirPath = [System.IO.Path]::GetFullPath($OutDir)
if (-not (Test-Path $outDirPath)) {
    New-Item -ItemType Directory -Path $outDirPath | Out-Null
}

$resultsPath = Join-Path $outDirPath "lookup-xll-bridge-results.csv"

$params = @{
    Manifest = $Manifest
    Out = $resultsPath
}
if (-not [string]::IsNullOrWhiteSpace($XllPath)) {
    $params["XllPath"] = $XllPath
}
if ($BuildIfMissing) {
    $params["BuildIfMissing"] = $true
}

& $runScript @params

Write-Host "Lookup XLL bridge suite complete."
Write-Host "Results: $resultsPath"
