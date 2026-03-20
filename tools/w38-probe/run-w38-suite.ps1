param(
    [string]$OutDir = ".tmp"
)

$root = (Resolve-Path (Join-Path $PSScriptRoot "..\\..")).Path
$outRoot = Join-Path $root $OutDir

if (-not (Test-Path $outRoot)) {
    New-Item -ItemType Directory -Force -Path $outRoot | Out-Null
}

powershell -ExecutionPolicy Bypass -File (Join-Path $PSScriptRoot "run-w38-lambda-helper-stage1-baseline.ps1") `
    -Out (Join-Path $OutDir "w38-lambda-helper-stage1-results.csv")

powershell -ExecutionPolicy Bypass -File (Join-Path $PSScriptRoot "run-w38-map-reduce-scan-stage2-baseline.ps1") `
    -Out (Join-Path $OutDir "w38-map-reduce-scan-stage2-results.csv")

powershell -ExecutionPolicy Bypass -File (Join-Path $PSScriptRoot "run-w38-stage3-byrow-bycol-makearray-defined-names-baseline.ps1") `
    -Out (Join-Path $OutDir "w38-stage3-byrow-bycol-makearray-defined-names-results.csv")
