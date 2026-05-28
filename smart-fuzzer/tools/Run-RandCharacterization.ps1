[CmdletBinding(PositionalBinding = $false)]
param(
    [string] $RunId = "",
    [int]    $Samples = 50000
)

# RAND() characterization study (NOT a conformance/blessing test).
# Bulk-samples Excel RAND() and runs an identical statistical battery over
# Excel + several candidate Rust RNGs (rand_characterize), then reports
# whether the battery can distinguish them. See the findings note for the
# motivation: characterising Excel RAND() vs Rust implementations we might
# use, with the host-vs-OxFunc placement of RAND still undecided.

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$ScriptPath = Split-Path -Parent $MyInvocation.MyCommand.Path
$RepoRoot = (Resolve-Path (Join-Path $ScriptPath "..\..")).Path
Set-Location $RepoRoot

if ([string]::IsNullOrWhiteSpace($RunId)) {
    $RunId = "rand-characterization-" + (Get-Date).ToUniversalTime().ToString("yyyyMMddTHHmmssZ")
}
$RunRoot = Join-Path $RepoRoot "smart-fuzzer\runs\$RunId"
if (Test-Path $RunRoot) { throw "run directory already exists: $RunRoot" }
[void](New-Item -ItemType Directory -Path $RunRoot -Force)
$SamplesPath = Join-Path $RunRoot "excel-rand-samples.txt"
$ReportPath  = Join-Path $RunRoot "rand-characterization-report.json"

# --------------------------------------------------------------------------
# Bulk-sample Excel RAND() into a column, one recalc, read with round-trip
# precision so the Rust side parses the identical f64 bits.
# --------------------------------------------------------------------------
$inv = [System.Globalization.CultureInfo]::InvariantCulture
Write-Host "Sampling Excel RAND() x $Samples (bulk column recalc)..."
$excel = $null; $workbook = $null
try {
    $excel = New-Object -ComObject Excel.Application
    $excel.Visible = $false
    $excel.DisplayAlerts = $false
    $excel.ScreenUpdating = $false
    $workbook = $excel.Workbooks.Add()
    $ws = $workbook.Worksheets.Item(1)
    $env = [ordered]@{ version = [string]$excel.Version; build = [string]$excel.Build }

    $addr = "A1:A$Samples"
    $ws.Range($addr).Formula2 = "=RAND()"
    $ws.Calculate() | Out-Null
    $block = $ws.Range($addr).Value2   # object[,] 1-based [row,1]

    $sb = New-Object System.Text.StringBuilder
    for ($r = 1; $r -le $Samples; $r++) {
        $v = [double]$block.GetValue($r, 1)
        [void]$sb.AppendLine($v.ToString("R", $inv))
    }
    [System.IO.File]::WriteAllText($SamplesPath, $sb.ToString())
    Write-Host "  wrote $Samples samples to $SamplesPath (Excel $($env.version) build $($env.build))"
} finally {
    if ($null -ne $workbook) { $workbook.Close($false) | Out-Null }
    if ($null -ne $excel) { $excel.Quit() | Out-Null }
    [System.Runtime.InteropServices.Marshal]::FinalReleaseComObject($excel) | Out-Null 2>$null
    [GC]::Collect(); [GC]::WaitForPendingFinalizers()
}

# --------------------------------------------------------------------------
# Run the identical battery over Excel + Rust RNGs.
# --------------------------------------------------------------------------
$manifest = Join-Path $RepoRoot "smart-fuzzer\tools\pmt_ppmt_local_eval\Cargo.toml"
& cargo run --quiet --manifest-path $manifest --bin rand_characterize -- --excel $SamplesPath --out $ReportPath
if ($LASTEXITCODE -ne 0) { throw "rand_characterize failed ($LASTEXITCODE)" }

# --------------------------------------------------------------------------
# Print comparison table.
# --------------------------------------------------------------------------
$report = Get-Content -Raw $ReportPath | ConvertFrom-Json
Write-Host ""
Write-Host ("{0,-16} {1,8} {2,9} {3,9} {4,9} {5,11} {6,13} {7,11}" -f "source","mean","variance","chi2_64","ks_unif","autocorr1","grid2^32","ks_vs_excel")
Write-Host ("-" * 96)
$ksx = $report.two_sample_ks_vs_excel
foreach ($name in $report.sources.PSObject.Properties.Name) {
    $s = $report.sources.$name
    $ks = if ($name -eq "excel_rand") { "" } else { ("{0:F5}" -f [double]$ksx.$name) }
    Write-Host ("{0,-16} {1,8:F4} {2,9:F5} {3,9:F1} {4,9:F4} {5,11:F5} {6,13:F4} {7,11}" -f `
        $name, [double]$s.mean, [double]$s.variance, [double]$s.chi2_uniform_64, [double]$s.ks_vs_uniform, [double]$s.lag1_autocorr, [double]$s.frac_on_2pow32_grid, $ks)
}
Write-Host ""
Write-Host "chi2_64 reference: df=63, ~uniform if roughly 40-90 (critical ~82.5 at p=0.05)."
Write-Host "theoretical uniform: mean=0.5 var=0.08333 skew=0 excess_kurt=-1.2"
Write-Host "report: $ReportPath"
