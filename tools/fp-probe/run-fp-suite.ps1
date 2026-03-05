param(
    [Parameter(Mandatory = $true)]
    [string]$Manifest,

    [string]$OutDir = ".tmp",

    [string]$WorkbookTemplate = ""
)

$ErrorActionPreference = "Stop"

$outRoot = [System.IO.Path]::GetFullPath($OutDir)
if (-not (Test-Path $outRoot)) {
    New-Item -ItemType Directory -Path $outRoot | Out-Null
}

$manifestPath = (Resolve-Path $Manifest).Path
$excelAbdOut = Join-Path $outRoot "fp-results-excel-abd.csv"
$excelCOut = Join-Path $outRoot "fp-results-excel-c.csv"
$excelCombinedOut = Join-Path $outRoot "fp-results-excel-all.csv"
$leanOut = Join-Path $outRoot "fp-results-lean.csv"

$excelRunner = Join-Path $PSScriptRoot "run-fp-excel-baseline.ps1"
$xllBuilder = Join-Path $PSScriptRoot "xll/build-fp-edge-xll.ps1"
$leanRunner = Join-Path $PSScriptRoot "run-fp-lean-baseline.ps1"
$stitcher = Join-Path $PSScriptRoot "stitch-fp-deviation-ledger.ps1"

if ([string]::IsNullOrWhiteSpace($WorkbookTemplate)) {
    & $excelRunner -Manifest $manifestPath -Out $excelAbdOut -Lanes @("FP-A", "FP-B", "FP-D")
}
else {
    & $excelRunner -Manifest $manifestPath -Out $excelAbdOut -Lanes @("FP-A", "FP-B", "FP-D") -WorkbookTemplate $WorkbookTemplate
}

& $xllBuilder -Profile release

$xllPath = Join-Path $PSScriptRoot "xll/bin/fp_edge_xll.xll"
if ([string]::IsNullOrWhiteSpace($WorkbookTemplate)) {
    & $excelRunner -Manifest $manifestPath -Out $excelCOut -Lanes @("FP-C") -XllPath $xllPath
}
else {
    & $excelRunner -Manifest $manifestPath -Out $excelCOut -Lanes @("FP-C") -WorkbookTemplate $WorkbookTemplate -XllPath $xllPath
}

$excelAll = @()
if (Test-Path $excelAbdOut) { $excelAll += Import-Csv $excelAbdOut }
if (Test-Path $excelCOut) { $excelAll += Import-Csv $excelCOut }
$excelAll | Export-Csv -Path $excelCombinedOut -NoTypeInformation -Encoding UTF8

& $leanRunner -Manifest $manifestPath -Out $leanOut

& $stitcher -ExcelResults $excelCombinedOut -LeanResults $leanOut

Write-Host "FP suite complete."
Write-Host "Excel (A/B/D): $excelAbdOut"
Write-Host "Excel (C):     $excelCOut"
Write-Host "Excel (all):   $excelCombinedOut"
Write-Host "Lean:          $leanOut"
