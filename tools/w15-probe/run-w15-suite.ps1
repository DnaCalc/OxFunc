param(
    [string]$OutDir = ".tmp",
    [string]$WorkbookTemplate = ""
)

$ErrorActionPreference = "Stop"

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$outDirPath = [System.IO.Path]::GetFullPath($OutDir)
if (-not (Test-Path $outDirPath)) {
    New-Item -ItemType Directory -Path $outDirPath | Out-Null
}

$compatTemplatePath = ""
if (-not [string]::IsNullOrWhiteSpace($WorkbookTemplate)) {
    $compatTemplatePath = [System.IO.Path]::GetFullPath($WorkbookTemplate)
} else {
    $compatTemplatePath = Join-Path $outDirPath "w15-compat-template.xls"
    & (Join-Path $scriptDir "..\\w10-probe\\new-w10-compat-template.ps1") -Out $compatTemplatePath
}

& (Join-Path $scriptDir "run-w15-info-preprobe.ps1") `
    -Manifest "docs/function-lane/W15_INFO_PRE_SCENARIO_MANIFEST_SEED.csv" `
    -Out (Join-Path $OutDir "w15-info-pre-results.csv") `
    -RunLabel "default"

& (Join-Path $scriptDir "run-w15-info-preprobe.ps1") `
    -Manifest "docs/function-lane/W15_INFO_PRE_SCENARIO_MANIFEST_SEED.csv" `
    -Out (Join-Path $OutDir "w15-info-pre-results-compat.csv") `
    -WorkbookTemplate $compatTemplatePath `
    -RunLabel "compat_template"

& (Join-Path $scriptDir "run-w15-cell-host-preprobe.ps1") `
    -Manifest "docs/function-lane/W15_CELL_HOST_PRE_SCENARIO_MANIFEST_SEED.csv" `
    -Out (Join-Path $OutDir "w15-cell-host-pre-results.csv") `
    -RunLabel "default"

& (Join-Path $scriptDir "run-w15-cell-host-preprobe.ps1") `
    -Manifest "docs/function-lane/W15_CELL_HOST_PRE_SCENARIO_MANIFEST_SEED.csv" `
    -Out (Join-Path $OutDir "w15-cell-host-pre-results-compat.csv") `
    -WorkbookTemplate $compatTemplatePath `
    -RunLabel "compat_template"

& (Join-Path $scriptDir "run-w15-xll-bridge.ps1") `
    -Manifest "docs/function-lane/W15_XLL_BRIDGE_SCENARIO_MANIFEST_SEED.csv" `
    -Out (Join-Path $OutDir "w15-xll-bridge-results.csv") `
    -BuildIfMissing `
    -RunLabel "default"

& (Join-Path $scriptDir "run-w15-xll-bridge.ps1") `
    -Manifest "docs/function-lane/W15_XLL_BRIDGE_SCENARIO_MANIFEST_SEED.csv" `
    -Out (Join-Path $OutDir "w15-xll-bridge-results-compat.csv") `
    -BuildIfMissing `
    -WorkbookTemplate $compatTemplatePath `
    -RunLabel "compat_template"
