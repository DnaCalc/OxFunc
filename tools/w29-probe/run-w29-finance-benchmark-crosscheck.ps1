[CmdletBinding()]
param(
    [string]$OxFuncOut = ".tmp/w29-finance-oxfunc-results.csv",
    [string]$FSharpOut = ".tmp/w29-finance-fsharp-results.csv",
    [string]$ExcelOut = ".tmp/w29-finance-excel-results.csv",
    [string]$LedgerOut = "docs/function-lane/W29_FINANCE_BENCHMARK_DISCREPANCY_LEDGER.csv",
    [string]$SummaryOut = ".tmp/w29-finance-summary.json",
    [string]$FSharpProject = ".tmp/ExcelFinancialFunctions/src/ExcelFinancialFunctions/ExcelFinancialFunctions.fsproj",
    [string]$FSharpDll = ".tmp/ExcelFinancialFunctions/src/ExcelFinancialFunctions/bin/Debug/netstandard2.0/ExcelFinancialFunctions.dll"
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Ensure-ParentDirectory {
    param([string]$Path)
    $parent = Split-Path -Parent $Path
    if ($parent -and -not (Test-Path $parent)) {
        New-Item -ItemType Directory -Path $parent -Force | Out-Null
    }
}

function Normalize-Observable {
    param($Value)
    if ($null -eq $Value) { return "" }
    if ($Value -is [double] -or $Value -is [float] -or $Value -is [decimal]) {
        return ([double]$Value).ToString("G17", [System.Globalization.CultureInfo]::InvariantCulture)
    }
    if ($Value -is [int] -or $Value -is [long]) {
        return $Value.ToString()
    }
    return [string]$Value
}

function Is-ErrorObservable {
    param([string]$Observable)
    $Observable.StartsWith("#")
}

function Observables-Match {
    param(
        [string]$A,
        [string]$B,
        [double]$Tolerance = 1e-6
    )
    if ($A -eq $B) { return $true }
    if ((Is-ErrorObservable $A) -or (Is-ErrorObservable $B)) { return $false }
    $aNum = 0.0
    $bNum = 0.0
    $okA = [double]::TryParse($A, [System.Globalization.NumberStyles]::Float, [System.Globalization.CultureInfo]::InvariantCulture, [ref]$aNum)
    $okB = [double]::TryParse($B, [System.Globalization.NumberStyles]::Float, [System.Globalization.CultureInfo]::InvariantCulture, [ref]$bNum)
    if (-not ($okA -and $okB)) { return $false }
    return [Math]::Abs($aNum - $bNum) -le $Tolerance
}

function Invoke-OxFuncProbe {
    $raw = cargo run --manifest-path crates/oxfunc_core/Cargo.toml --example w29_finance_probe
    $rows = $raw | ConvertFrom-Csv
    $rows | Export-Csv -Path $OxFuncOut -NoTypeInformation -Encoding UTF8
    return $rows
}

function Invoke-FSharpBuild {
    dotnet build $FSharpProject -v minimal | Out-Null
}

function Invoke-FSharpProbe {
    $resolvedDll = (Resolve-Path $FSharpDll).Path
    $fsiScript = @"
#r @"$resolvedDll"
open System
open Excel.FinancialFunctions

let p caseId value =
    printfn "%s,fsharp,%.15f" caseId value

let coupDays = Financial.CoupDays(DateTime(2012,1,1), DateTime(2016,2,29), Frequency.SemiAnnual, DayCountBasis.ActualActual)
let coupDaysBS = Financial.CoupDaysBS(DateTime(2012,1,1), DateTime(2016,2,29), Frequency.SemiAnnual, DayCountBasis.ActualActual)
let coupDaysNC = Financial.CoupDaysNC(DateTime(2012,1,1), DateTime(2016,2,29), Frequency.SemiAnnual, DayCountBasis.ActualActual)
p "coupon_leap_coupdays" coupDays
p "coupon_leap_coupdaybs" coupDaysBS
p "coupon_leap_coupdaysnc" coupDaysNC
p "coupon_leap_diff" (coupDays - coupDaysBS - coupDaysNC)

let dates = [| DateTime(2000,2,29); DateTime(2000,3,31) |]
let values1 = [| 206101714.849377; -156650972.54265 |]
let values2 = [| 15108163.3840923; -75382259.6628424 |]
p "xnpv_negative_rate_case1" (Financial.XNpv(-0.960452195, values1, dates))
p "xnpv_negative_rate_case2" (Financial.XNpv(-0.960452189, values1, dates))
p "xirr_negative_rate_case1_guess_neg" (Financial.XIrr(values1, dates, -0.1))
p "xirr_negative_rate_case2_guess_neg" (Financial.XIrr(values2, dates, -0.1))
p "xirr_negative_rate_case2_guess_pos" (Financial.XIrr(values2, dates, 0.1))

let ratePmt = Financial.Pmt(0.01, 48.0, 8000.0, 0.0, PaymentDue.EndOfPeriod)
p "rate_seed_sample" (Financial.Rate(48.0, ratePmt, 8000.0, 0.0, PaymentDue.EndOfPeriod, 0.1))

p "pricemat_basis1_seed" (Financial.PriceMat(DateTime(2024,6,15), DateTime(2025,12,31), DateTime(2024,1,1), 0.0525, 0.061, DayCountBasis.ActualActual))
p "yieldmat_basis1_seed" (Financial.YieldMat(DateTime(2024,6,15), DateTime(2025,12,31), DateTime(2024,1,1), 0.0525, 98.59811340546048, DayCountBasis.ActualActual))
p "oddlprice_seed" (Financial.OddLPrice(DateTime(2008,2,7), DateTime(2008,6,15), DateTime(2007,10,15), 0.0375, 0.0405, 100.0, Frequency.SemiAnnual, DayCountBasis.UsPsa30_360))
p "oddlyield_seed" (Financial.OddLYield(DateTime(2008,2,7), DateTime(2008,6,15), DateTime(2007,10,15), 0.0375, 99.87828601472134, 100.0, Frequency.SemiAnnual, DayCountBasis.UsPsa30_360))
p "oddfyield_seed" (Financial.OddFYield(DateTime(2008,11,11), DateTime(2021,3,1), DateTime(2008,10,15), DateTime(2009,3,1), 0.0785, 113.597717474079, 100.0, Frequency.SemiAnnual, DayCountBasis.ActualActual))
"@
    $fsiScriptPath = ".tmp/w29-fsharp-probe.fsx"
    Set-Content -Path $fsiScriptPath -Value $fsiScript -Encoding UTF8
    $raw = dotnet fsi $fsiScriptPath
    if ($LASTEXITCODE -ne 0) {
        throw "dotnet fsi probe failed."
    }
    $rows = $raw | ConvertFrom-Csv -Header case_id,source,observable
    $rows | Export-Csv -Path $FSharpOut -NoTypeInformation -Encoding UTF8
    return $rows
}

function Get-ExcelCellObservable {
    param($Range)
    $text = [string]$Range.Text
    if ($text.StartsWith('#')) { return $text }
    Normalize-Observable $Range.Value2
}

function Invoke-ExcelProbe {
    $excel = New-Object -ComObject Excel.Application
    $excel.Visible = $false
    $excel.DisplayAlerts = $false
    $wb = $excel.Workbooks.Add()
    $ws = $wb.Worksheets.Item(1)
    try {
        $ws.Range('B1').Value2 = 206101714.849377
        $ws.Range('B2').Value2 = -156650972.54265
        $ws.Range('C1').Formula = '=DATE(2000,2,29)'
        $ws.Range('C2').Formula = '=DATE(2000,3,31)'
        $ws.Range('B4').Value2 = 15108163.3840923
        $ws.Range('B5').Value2 = -75382259.6628424
        $ws.Range('C4').Formula = '=DATE(2000,2,29)'
        $ws.Range('C5').Formula = '=DATE(2000,3,31)'

        $rows = @()
        $ws.Range('A1').Formula = '=COUPDAYS(DATE(2012,1,1),DATE(2016,2,29),2,1)'
        $ws.Range('A2').Formula = '=COUPDAYBS(DATE(2012,1,1),DATE(2016,2,29),2,1)'
        $ws.Range('A3').Formula = '=COUPDAYSNC(DATE(2012,1,1),DATE(2016,2,29),2,1)'
        $rows += [pscustomobject]@{ case_id='coupon_leap_coupdays'; source='excel'; observable=(Get-ExcelCellObservable $ws.Range('A1')) }
        $rows += [pscustomobject]@{ case_id='coupon_leap_coupdaybs'; source='excel'; observable=(Get-ExcelCellObservable $ws.Range('A2')) }
        $rows += [pscustomobject]@{ case_id='coupon_leap_coupdaysnc'; source='excel'; observable=(Get-ExcelCellObservable $ws.Range('A3')) }
        $rows += [pscustomobject]@{ case_id='coupon_leap_diff'; source='excel'; observable=Normalize-Observable ([double]$ws.Range('A1').Value2 - [double]$ws.Range('A2').Value2 - [double]$ws.Range('A3').Value2) }

        $ws.Range('A4').Formula = '=XNPV(-0.960452195,B1:B2,C1:C2)'
        $ws.Range('A5').Formula = '=XNPV(-0.960452189,B1:B2,C1:C2)'
        $ws.Range('A6').Formula = '=XIRR(B1:B2,C1:C2,-0.1)'
        $ws.Range('A7').Formula = '=XIRR(B4:B5,C4:C5,-0.1)'
        $ws.Range('A8').Formula = '=XIRR(B4:B5,C4:C5,0.1)'
        $rows += [pscustomobject]@{ case_id='xnpv_negative_rate_case1'; source='excel'; observable=(Get-ExcelCellObservable $ws.Range('A4')) }
        $rows += [pscustomobject]@{ case_id='xnpv_negative_rate_case2'; source='excel'; observable=(Get-ExcelCellObservable $ws.Range('A5')) }
        $rows += [pscustomobject]@{ case_id='xirr_negative_rate_case1_guess_neg'; source='excel'; observable=(Get-ExcelCellObservable $ws.Range('A6')) }
        $rows += [pscustomobject]@{ case_id='xirr_negative_rate_case2_guess_neg'; source='excel'; observable=(Get-ExcelCellObservable $ws.Range('A7')) }
        $rows += [pscustomobject]@{ case_id='xirr_negative_rate_case2_guess_pos'; source='excel'; observable=(Get-ExcelCellObservable $ws.Range('A8')) }

        $ws.Range('A9').Formula = '=RATE(48,PMT(0.01,48,8000),8000)'
        $ws.Range('A10').Formula = '=PRICEMAT(DATE(2024,6,15),DATE(2025,12,31),DATE(2024,1,1),0.0525,0.061,1)'
        $ws.Range('A11').Formula = '=YIELDMAT(DATE(2024,6,15),DATE(2025,12,31),DATE(2024,1,1),0.0525,98.59811340546048,1)'
        $ws.Range('A12').Formula = '=ODDLPRICE(DATE(2008,2,7),DATE(2008,6,15),DATE(2007,10,15),0.0375,0.0405,100,2,0)'
        $ws.Range('A13').Formula = '=ODDLYIELD(DATE(2008,2,7),DATE(2008,6,15),DATE(2007,10,15),0.0375,99.87828601472134,100,2,0)'
        $ws.Range('A14').Formula = '=ODDFYIELD(DATE(2008,11,11),DATE(2021,3,1),DATE(2008,10,15),DATE(2009,3,1),0.0785,113.597717474079,100,2,1)'
        $rows += [pscustomobject]@{ case_id='rate_seed_sample'; source='excel'; observable=(Get-ExcelCellObservable $ws.Range('A9')) }
        $rows += [pscustomobject]@{ case_id='pricemat_basis1_seed'; source='excel'; observable=(Get-ExcelCellObservable $ws.Range('A10')) }
        $rows += [pscustomobject]@{ case_id='yieldmat_basis1_seed'; source='excel'; observable=(Get-ExcelCellObservable $ws.Range('A11')) }
        $rows += [pscustomobject]@{ case_id='oddlprice_seed'; source='excel'; observable=(Get-ExcelCellObservable $ws.Range('A12')) }
        $rows += [pscustomobject]@{ case_id='oddlyield_seed'; source='excel'; observable=(Get-ExcelCellObservable $ws.Range('A13')) }
        $rows += [pscustomobject]@{ case_id='oddfyield_seed'; source='excel'; observable=(Get-ExcelCellObservable $ws.Range('A14')) }

        $rows | Export-Csv -Path $ExcelOut -NoTypeInformation -Encoding UTF8
        return $rows
    }
    finally {
        $wb.Close($false)
        $excel.Quit()
        [System.Runtime.Interopservices.Marshal]::ReleaseComObject($ws) | Out-Null
        [System.Runtime.Interopservices.Marshal]::ReleaseComObject($wb) | Out-Null
        [System.Runtime.Interopservices.Marshal]::ReleaseComObject($excel) | Out-Null
    }
}

function Get-ComparisonClass {
    param(
        [string]$OxFunc,
        [string]$FSharp,
        [string]$Excel
    )
    $oxExcel = Observables-Match $OxFunc $Excel
    $fsExcel = Observables-Match $FSharp $Excel
    $oxFs = Observables-Match $OxFunc $FSharp

    if ($oxExcel -and $fsExcel) { return 'aligned_all_three' }
    if ($oxExcel -and -not $fsExcel) { return 'oxfunc_matches_excel_fsharp_differs' }
    if ($fsExcel -and -not $oxExcel) { return 'fsharp_matches_excel_oxfunc_differs' }
    if ($oxFs -and -not $oxExcel) { return 'oxfunc_matches_fsharp_not_excel' }
    return 'all_diverge_or_inconclusive'
}

Ensure-ParentDirectory $OxFuncOut
Ensure-ParentDirectory $FSharpOut
Ensure-ParentDirectory $ExcelOut
Ensure-ParentDirectory $LedgerOut
Ensure-ParentDirectory $SummaryOut

Invoke-FSharpBuild
$oxRows = Invoke-OxFuncProbe
$fsRows = Invoke-FSharpProbe
$excelRows = Invoke-ExcelProbe

$caseMeta = @{
    'coupon_leap_coupdays' = @{ family='coupon_family'; note='Actual/actual leap-year coupon-period size from the public F# compatibility note' }
    'coupon_leap_diff' = @{ family='coupon_family'; note='CoupDays - CoupDaysBS - CoupDaysNC leap-year identity lane from the public F# compatibility note' }
    'xnpv_negative_rate_case1' = @{ family='cashflow_rate_family'; note='Negative-rate XNPV lane from the public F# compatibility note' }
    'xnpv_negative_rate_case2' = @{ family='cashflow_rate_family'; note='Negative-rate XNPV companion lane from the public F# compatibility note' }
    'xirr_negative_rate_case1_guess_neg' = @{ family='cashflow_rate_family'; note='Negative-rate XIRR lane from the public F# compatibility note' }
    'xirr_negative_rate_case2_guess_neg' = @{ family='cashflow_rate_family'; note='Negative-root XIRR lane from the public F# compatibility note' }
    'xirr_negative_rate_case2_guess_pos' = @{ family='cashflow_rate_family'; note='Large-root XIRR lane from the public F# compatibility note' }
    'rate_seed_sample' = @{ family='financial_time_value_family'; note='Representative seeded RATE inversion lane from W24 Batch 11' }
    'pricemat_basis1_seed' = @{ family='bond_core_family'; note='Repaired W27 basis-1 maturity-security lane' }
    'yieldmat_basis1_seed' = @{ family='bond_core_family'; note='Repaired W27 basis-1 maturity-security inversion lane' }
    'oddlprice_seed' = @{ family='odd_bond_family'; note='Repaired W27 odd-last price lane' }
    'oddlyield_seed' = @{ family='odd_bond_family'; note='Repaired W27 odd-last yield lane' }
    'oddfyield_seed' = @{ family='odd_bond_family'; note='Seeded odd-first inversion lane; public F# docs mention qualitative root-finding differences' }
}

$oxMap = @{}
foreach ($row in $oxRows) { $oxMap[$row.case_id] = $row.observable }
$fsMap = @{}
foreach ($row in $fsRows) { $fsMap[$row.case_id] = $row.observable }
$excelMap = @{}
foreach ($row in $excelRows) { $excelMap[$row.case_id] = $row.observable }

$ledger = foreach ($caseId in $caseMeta.Keys | Sort-Object) {
    $ox = [string]$oxMap[$caseId]
    $fs = [string]$fsMap[$caseId]
    $ex = [string]$excelMap[$caseId]
    [pscustomobject]@{
        case_id = $caseId
        family = $caseMeta[$caseId].family
        oxfunc_observable = $ox
        fsharp_observable = $fs
        excel_observable = $ex
        comparison_class = Get-ComparisonClass -OxFunc $ox -FSharp $fs -Excel $ex
        note = $caseMeta[$caseId].note
    }
}

$ledger | Export-Csv -Path $LedgerOut -NoTypeInformation -Encoding UTF8

$summary = [pscustomobject]@{
    captured_at_utc = [DateTime]::UtcNow.ToString("o")
    case_count = ($ledger | Measure-Object).Count
    aligned_all_three = ($ledger | Where-Object comparison_class -eq 'aligned_all_three' | Measure-Object).Count
    oxfunc_matches_excel_fsharp_differs = ($ledger | Where-Object comparison_class -eq 'oxfunc_matches_excel_fsharp_differs' | Measure-Object).Count
    fsharp_matches_excel_oxfunc_differs = ($ledger | Where-Object comparison_class -eq 'fsharp_matches_excel_oxfunc_differs' | Measure-Object).Count
    oxfunc_matches_fsharp_not_excel = ($ledger | Where-Object comparison_class -eq 'oxfunc_matches_fsharp_not_excel' | Measure-Object).Count
    all_diverge_or_inconclusive = ($ledger | Where-Object comparison_class -eq 'all_diverge_or_inconclusive' | Measure-Object).Count
}
$summary | ConvertTo-Json -Depth 4 | Set-Content -Path $SummaryOut -Encoding UTF8

Write-Output "Cases: $($summary.case_count)"
Write-Output "Aligned all three: $($summary.aligned_all_three)"
Write-Output "OxFunc matches Excel, F# differs: $($summary.oxfunc_matches_excel_fsharp_differs)"
Write-Output "F# matches Excel, OxFunc differs: $($summary.fsharp_matches_excel_oxfunc_differs)"
Write-Output "OxFunc matches F#, not Excel: $($summary.oxfunc_matches_fsharp_not_excel)"
Write-Output "All diverge or inconclusive: $($summary.all_diverge_or_inconclusive)"
