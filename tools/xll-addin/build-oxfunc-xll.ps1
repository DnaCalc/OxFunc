param(
    [ValidateSet("debug", "release")]
    [string]$Profile = "release",

    [string]$OutDir = "",

    [switch]$SkipSpecSync
)

$ErrorActionPreference = "Stop"

$crateRoot = Resolve-Path -Path (Join-Path $PSScriptRoot "oxfunc_xll")
$manifestPath = Join-Path $crateRoot "Cargo.toml"
if (-not (Test-Path $manifestPath)) {
    throw "Missing Cargo manifest: $manifestPath"
}

if (-not $SkipSpecSync) {
    $syncScript = Join-Path $PSScriptRoot "sync-export-specs.ps1"
    if (-not (Test-Path $syncScript)) {
        throw "Missing sync script: $syncScript"
    }
    & $syncScript | Out-Null
}

$profileArgs = @()
$binSubdir = "debug"
if ($Profile -eq "release") {
    $profileArgs += "--release"
    $binSubdir = "release"
}

cargo build --manifest-path $manifestPath @profileArgs
$buildExit = $LASTEXITCODE

if ($buildExit -ne 0) {
    exit $buildExit
}

$dllPath = Join-Path $crateRoot "target\$binSubdir\oxfunc_xll.dll"
if (-not (Test-Path $dllPath)) {
    throw "Built DLL not found: $dllPath"
}

if ([string]::IsNullOrWhiteSpace($OutDir)) {
    $OutDir = Join-Path $PSScriptRoot "bin"
}
$outDirPath = [System.IO.Path]::GetFullPath($OutDir)
if (-not (Test-Path $outDirPath)) {
    New-Item -ItemType Directory -Path $outDirPath | Out-Null
}

$xllPath = Join-Path $outDirPath "OxFunc64.xll"
Copy-Item -Path $dllPath -Destination $xllPath -Force

Write-Host "Built XLL: $xllPath"
