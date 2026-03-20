[CmdletBinding()]
param(
    [string]$OxFuncOut = ".tmp/w37-xirr-large-root-oxfunc-results.csv",
    [string]$ExcelOut = ".tmp/w37-xirr-large-root-excel-results.csv",
    [string]$MergedOut = ".tmp/w37-xirr-large-root-results.csv"
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
    $asString = [string]$Value
    $parsed = 0.0
    if ([double]::TryParse($asString, [System.Globalization.NumberStyles]::Float, [System.Globalization.CultureInfo]::InvariantCulture, [ref]$parsed)) {
        return $parsed.ToString("G17", [System.Globalization.CultureInfo]::InvariantCulture)
    }
    return $asString
}

function Get-ExcelCellObservable {
    param($Range)
    $text = [string]$Range.Text
    if ($text.StartsWith('#')) { return $text }
    Normalize-Observable $Range.Value2
}

Ensure-ParentDirectory $OxFuncOut
Ensure-ParentDirectory $ExcelOut
Ensure-ParentDirectory $MergedOut

$oxRows = @(cargo run --manifest-path crates/oxfunc_core/Cargo.toml --example w37_xirr_large_root_probe | ConvertFrom-Csv)
$oxRows | Export-Csv -Path $OxFuncOut -NoTypeInformation -Encoding UTF8

$excel = New-Object -ComObject Excel.Application
$excel.Visible = $false
$excel.DisplayAlerts = $false
$wb = $excel.Workbooks.Add()
$ws = $wb.Worksheets.Item(1)

try {
    $ws.Range('B1').Value2 = 15108163.3840923
    $ws.Range('B2').Value2 = -75382259.6628424
    $ws.Range('C1').Formula = '=DATE(2000,2,29)'
    $ws.Range('C2').Formula = '=DATE(2000,3,31)'

    $guesses = @('0.0001', '0.01', '0.1', '1', '10', '100', '1000')
    $excelRows = @()
    for ($i = 0; $i -lt $guesses.Count; $i++) {
        $row = $i + 1
        $guess = $guesses[$i]
        $caseId = "xirr_large_root_guess_$guess"
        $ws.Cells.Item($row, 1).Formula = "=XIRR(B1:B2,C1:C2,$guess)"
        $excel.CalculateFull()
        $excelRows += [pscustomobject]@{
            case_id = $caseId
            guess = $guess
            source = 'excel'
            observable = Get-ExcelCellObservable $ws.Cells.Item($row, 1)
        }
    }
    @($excelRows) | Export-Csv -Path $ExcelOut -NoTypeInformation -Encoding UTF8
}
finally {
    $wb.Close($false)
    $excel.Quit()
    [System.Runtime.Interopservices.Marshal]::ReleaseComObject($ws) | Out-Null
    [System.Runtime.Interopservices.Marshal]::ReleaseComObject($wb) | Out-Null
    [System.Runtime.Interopservices.Marshal]::ReleaseComObject($excel) | Out-Null
}

$excelByCase = @{}
foreach ($row in $excelRows) {
    $excelByCase[$row.case_id] = $row
}

$merged = @(foreach ($row in $oxRows) {
    $excelRow = $excelByCase[$row.case_id]
    $oxObservable = Normalize-Observable $row.observable
    $excelObservable = Normalize-Observable $excelRow.observable
    [pscustomobject]@{
        case_id = $row.case_id
        guess = Normalize-Observable $row.guess
        oxfunc_observable = $oxObservable
        excel_observable = $excelObservable
        matches_expected = ($oxObservable -eq $excelObservable)
    }
})

$merged | Export-Csv -Path $MergedOut -NoTypeInformation -Encoding UTF8
$matchCount = @($merged | Where-Object { $_.matches_expected -eq $true -or $_.matches_expected -eq 'True' }).Count
Write-Output "Rows: $($merged.Count)"
Write-Output "Matches: $matchCount"
