[CmdletBinding()]
param([string]$Out = ".tmp/excel-decomposition-microprobes.csv")

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$repo = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path
Import-Module (Join-Path $PSScriptRoot "CellRefBatch.psm1") -Force

$excel = New-Object -ComObject Excel.Application
$excel.Visible = $false; $excel.DisplayAlerts = $false
$wb = $excel.Workbooks.Add(); $ws = $wb.Worksheets.Item(1)
$probes = @()
function Add-Probe([string]$group,[string]$variant,[int]$row,[string]$formula) {
    $script:ws.Cells.Item($row,1).Formula2 = $formula
    $script:probes += [pscustomobject]@{group=$group;variant=$variant;row=$row;formula=$formula}
}
try {
    # ACOTH identity family. Inputs are cell values so reciprocal rounding is explicit.
    $ws.Cells.Item(1,3).Value2 = [double]5; $ws.Cells.Item(2,3).Value2 = [double]10
    Add-Probe "acoth-5" "ACOTH" 1 "=ACOTH(C1)"
    Add-Probe "acoth-5" "ATANH-recip" 2 "=ATANH(1/C1)"
    Add-Probe "acoth-5" "half-log-ratio" 3 "=0.5*LN((C1+1)/(C1-1))"
    Add-Probe "acoth-5" "half-log-difference" 4 "=0.5*(LN(C1+1)-LN(C1-1))"
    Add-Probe "acoth-10" "ACOTH" 5 "=ACOTH(C2)"
    Add-Probe "acoth-10" "ATANH-recip" 6 "=ATANH(1/C2)"
    Add-Probe "acoth-10" "half-log-ratio" 7 "=0.5*LN((C2+1)/(C2-1))"
    Add-Probe "acoth-10" "half-log-difference" 8 "=0.5*(LN(C2+1)-LN(C2-1))"

    # Exact geometric GROWTH control and equivalent graph shapes.
    @(2,4,8,16) | ForEach-Object -Begin {$c=3} -Process {$ws.Cells.Item(12,$c++).Value2=[double]$_}
    @(1,2,3,4) | ForEach-Object -Begin {$c=3} -Process {$ws.Cells.Item(13,$c++).Value2=[double]$_}
    $ws.Cells.Item(14,3).Value2 = [double]5
    Add-Probe "growth-geometric" "GROWTH" 12 "=GROWTH(C12:F12,C13:F13,C14)"
    Add-Probe "growth-geometric" "log-domain-regression" 13 "=EXP(INTERCEPT(LN(C12:F12),C13:F13)+SLOPE(LN(C12:F12),C13:F13)*C14)"
    Add-Probe "growth-geometric" "exp-ln-direct" 14 "=EXP(LN(C12)*C14)"
    Add-Probe "growth-geometric" "power-direct" 15 "=POWER(C12,C14)"

    # Exact-line FORECAST control and two common regression publication graphs.
    @(2,4,6,8) | ForEach-Object -Begin {$c=3} -Process {$ws.Cells.Item(20,$c++).Value2=[double]$_}
    @(1,2,3,4) | ForEach-Object -Begin {$c=3} -Process {$ws.Cells.Item(21,$c++).Value2=[double]$_}
    $ws.Cells.Item(22,3).Value2 = [double]5
    Add-Probe "forecast-linear" "FORECAST" 20 "=FORECAST(C22,C20:F20,C21:F21)"
    Add-Probe "forecast-linear" "centered-slope" 21 "=AVERAGE(C20:F20)+SLOPE(C20:F20,C21:F21)*(C22-AVERAGE(C21:F21))"
    Add-Probe "forecast-linear" "intercept-plus-slope" 22 "=INTERCEPT(C20:F20,C21:F21)+SLOPE(C20:F20,C21:F21)*C22"

    # Fractional-year XNPV and equivalent term graphs.
    @(-1000,500,600) | ForEach-Object -Begin {$c=3} -Process {$ws.Cells.Item(30,$c++).Value2=[double]$_}
    @(43831,44013,44562) | ForEach-Object -Begin {$c=3} -Process {$ws.Cells.Item(31,$c++).Value2=[double]$_}
    $ws.Cells.Item(32,3).Value2 = [double]0.05
    $p2 = "D30/POWER(1+C32,(D31-C31)/365)"
    $p3 = "E30/POWER(1+C32,(E31-C31)/365)"
    $e2 = "D30/EXP(((D31-C31)/365)*LN(1+C32))"
    $e3 = "E30/EXP(((E31-C31)/365)*LN(1+C32))"
    Add-Probe "xnpv-fractional" "XNPV" 30 "=XNPV(C32,C30:E30,C31:E31)"
    Add-Probe "xnpv-fractional" "power-forward" 31 "=C30+$p2+$p3"
    Add-Probe "xnpv-fractional" "power-reverse-group" 32 "=$p3+($p2+C30)"
    Add-Probe "xnpv-fractional" "exp-ln-forward" 33 "=C30+$e2+$e3"

    # ATANH identities through Excel's now-characterized worksheet LN path.
    $ws.Cells.Item(40,3).Value2 = [double]0.1; $ws.Cells.Item(41,3).Value2 = [double]0.2
    Add-Probe "atanh-0.1" "ATANH" 40 "=ATANH(C40)"
    Add-Probe "atanh-0.1" "half-log-ratio" 41 "=0.5*LN((1+C40)/(1-C40))"
    Add-Probe "atanh-0.1" "half-log-difference" 42 "=0.5*(LN(1+C40)-LN(1-C40))"
    Add-Probe "atanh-0.1" "half-log1p-shape" 43 "=0.5*LN(1+2*C40/(1-C40))"
    Add-Probe "atanh-0.2" "ATANH" 44 "=ATANH(C41)"
    Add-Probe "atanh-0.2" "half-log-ratio" 45 "=0.5*LN((1+C41)/(1-C41))"
    Add-Probe "atanh-0.2" "half-log-difference" 46 "=0.5*(LN(1+C41)-LN(1-C41))"
    Add-Probe "atanh-0.2" "half-log1p-shape" 47 "=0.5*LN(1+2*C41/(1-C41))"

    $excel.CalculateFull()
    $rows = foreach ($probe in $probes) {
        $v = $ws.Cells.Item($probe.row,1).Value2
        [pscustomobject]@{
            group=$probe.group; variant=$probe.variant; value=[double]$v
            bits_hex=Get-F64BitsHex ([double]$v); formula=$probe.formula
        }
    }
    $outPath = if ([IO.Path]::IsPathRooted($Out)) {$Out} else {Join-Path $repo $Out}
    $parent = Split-Path -Parent $outPath; if ($parent) {New-Item -ItemType Directory -Force -Path $parent | Out-Null}
    $rows | Export-Csv -NoTypeInformation -Encoding utf8 -Path $outPath
    $rows | Format-Table group,variant,bits_hex -AutoSize | Out-Host
    Write-Host "Excel $($excel.Version) build $($excel.Build); output -> $outPath" -ForegroundColor Cyan
}
finally {
    $wb.Close($false); $excel.Quit()
    [void][Runtime.InteropServices.Marshal]::FinalReleaseComObject($ws)
    [void][Runtime.InteropServices.Marshal]::FinalReleaseComObject($wb)
    [void][Runtime.InteropServices.Marshal]::FinalReleaseComObject($excel)
}
