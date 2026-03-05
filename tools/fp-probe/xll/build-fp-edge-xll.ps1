param(
    [ValidateSet("debug", "release")]
    [string]$Profile = "release",

    [string]$OutDir = "",

    [string]$ExcelXllSdkDir = ""
)

$ErrorActionPreference = "Stop"

$crateRoot = Resolve-Path -Path (Join-Path $PSScriptRoot "fp_edge_xll")
$manifestPath = Join-Path $crateRoot "Cargo.toml"
if (-not (Test-Path $manifestPath)) {
    throw "Missing Cargo manifest: $manifestPath"
}

$profileArgs = @()
$binSubdir = "debug"
if ($Profile -eq "release") {
    $profileArgs += "--release"
    $binSubdir = "release"
}

$originalSdkEnv = $env:EXCEL_XLL_SDK_DIR
if (-not [string]::IsNullOrWhiteSpace($ExcelXllSdkDir)) {
    $env:EXCEL_XLL_SDK_DIR = (Resolve-Path -Path $ExcelXllSdkDir -ErrorAction Stop).Path
}

cargo build --manifest-path $manifestPath @profileArgs
$buildExit = $LASTEXITCODE

if ([string]::IsNullOrWhiteSpace($ExcelXllSdkDir)) {
    # keep environment untouched
}
elseif ($null -eq $originalSdkEnv) {
    Remove-Item Env:EXCEL_XLL_SDK_DIR -ErrorAction SilentlyContinue
}
else {
    $env:EXCEL_XLL_SDK_DIR = $originalSdkEnv
}

if ($buildExit -ne 0) {
    exit $buildExit
}

$dllPath = Join-Path $crateRoot "target\$binSubdir\fp_edge_xll.dll"
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

$xllPath = Join-Path $outDirPath "fp_edge_xll.xll"
Copy-Item -Path $dllPath -Destination $xllPath -Force

Write-Host "Built XLL: $xllPath"
