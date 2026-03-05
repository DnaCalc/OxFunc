param(
    [Parameter(Mandatory = $true)]
    [string]$Manifest,

    [Parameter(Mandatory = $true)]
    [string]$Out,

    [ValidateSet("dry-run", "prepare")]
    [string]$Mode = "dry-run"
)

$ErrorActionPreference = "Stop"

$manifestPathProject = Join-Path $PSScriptRoot "fp_probe_runner/Cargo.toml"
if (-not (Test-Path $manifestPathProject)) {
    Write-Error "Runner project not found: $manifestPathProject"
    exit 2
}

$manifestPath = Resolve-Path -Path $Manifest -ErrorAction SilentlyContinue
if (-not $manifestPath) {
    Write-Error "Manifest not found: $Manifest"
    exit 3
}

$outPath = [System.IO.Path]::GetFullPath($Out)
$outDir = Split-Path -Path $outPath -Parent
if ($outDir -and -not (Test-Path $outDir)) {
    New-Item -ItemType Directory -Path $outDir | Out-Null
}

cargo run --manifest-path $manifestPathProject -- --manifest $manifestPath --out $outPath --mode $Mode
if ($LASTEXITCODE -ne 0) {
    exit $LASTEXITCODE
}

Write-Host "fp-probe run complete: $outPath"
