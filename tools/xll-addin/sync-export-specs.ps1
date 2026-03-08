param(
    [string]$OutPath = ""
)

$ErrorActionPreference = "Stop"

$repoRoot = Resolve-Path -Path (Join-Path $PSScriptRoot "..\..")
if ([string]::IsNullOrWhiteSpace($OutPath)) {
    $OutPath = Join-Path $PSScriptRoot "oxfunc_xll\export_specs.csv"
}
$outPathFull = [System.IO.Path]::GetFullPath($OutPath)
$outDir = Split-Path -Path $outPathFull -Parent
if ($outDir -and -not (Test-Path $outDir)) {
    New-Item -ItemType Directory -Path $outDir | Out-Null
}

$csv = cargo run --manifest-path (Join-Path $repoRoot "crates\oxfunc_core\Cargo.toml") --bin emit_xll_export_specs --quiet
if ($LASTEXITCODE -ne 0) {
    throw "Failed to generate export specs from oxfunc_core."
}

[System.IO.File]::WriteAllText($outPathFull, $csv, [System.Text.UTF8Encoding]::new($false))
Write-Host "Wrote export specs: $outPathFull"
