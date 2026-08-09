[CmdletBinding(PositionalBinding = $false)]
param(
    [Parameter(Mandatory)] [string] $Helper,
    [Parameter(Mandatory)] [string] $Out,
    [string] $LowBits = "0x3f1af6e2eb1c432d",
    [string] $HighBits = "0x3f1afee2eb1c432d"
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Convert-HexToUInt64([string] $Text) {
    return [Convert]::ToUInt64($Text.Substring(2), 16)
}

function Format-Bits([UInt64] $Bits) {
    return "0x{0:x16}" -f $Bits
}

function Release-Com([object] $Value) {
    if ($null -ne $Value -and [Runtime.InteropServices.Marshal]::IsComObject($Value)) {
        [void][Runtime.InteropServices.Marshal]::FinalReleaseComObject($Value)
    }
}

$helperPath = (Resolve-Path -LiteralPath $Helper).Path
$outPath = [IO.Path]::GetFullPath((Join-Path (Get-Location) $Out))
$outDir = Split-Path -Parent $outPath
[IO.Directory]::CreateDirectory($outDir) | Out-Null

$before = @(Get-Process EXCEL -ErrorAction SilentlyContinue).Count
if ($before -ne 0) {
    throw "Serialized COM precondition failed: $before EXCEL process(es) already running"
}

$excel = $null
$workbook = $null
$sheet = $null
$argCell = $null
$resultCell = $null
$steps = New-Object 'System.Collections.Generic.List[object]'
$environment = $null
$collapsed = $false
$low = Convert-HexToUInt64 $LowBits
$high = Convert-HexToUInt64 $HighBits

try {
    $excel = New-Object -ComObject Excel.Application
    $excel.Visible = $false
    $excel.DisplayAlerts = $false
    $excel.ScreenUpdating = $false
    $excel.EnableEvents = $false
    $workbook = $excel.Workbooks.Add()
    $excel.Calculation = -4135 # xlCalculationManual
    $sheet = $workbook.Worksheets.Item(1)
    $argCell = $sheet.Cells.Item(1, 1)
    $resultCell = $sheet.Cells.Item(1, 2)
    $resultCell.Formula2 = "=ATANH(A1)"

    $environment = [ordered]@{
        excel_version = [string] $excel.Version
        excel_build = [string] $excel.Build
        excel_operating_system = [string] $excel.OperatingSystem
        workbook_compatibility = [string] $workbook.CompatibilityVersion
        date_1904 = [bool] $workbook.Date1904
        country_setting = [string] $excel.International(2) # xlCountrySetting
        decimal_separator = [string] $excel.International(3) # xlDecimalSeparator
        cpu = [string] $env:PROCESSOR_IDENTIFIER
        input_plumbing = "Range.Value2 cell reference"
        formula_api = "Formula2"
        cache = "none"
    }
    if ($environment.workbook_compatibility -ne "2") {
        throw "Unexpected CompatibilityVersion $($environment.workbook_compatibility)"
    }

    for ($iteration = 0; $iteration -lt 64 -and ($high - $low) -gt 1; $iteration++) {
        $line = & $helperPath (Format-Bits $low) (Format-Bits $high)
        if ($LASTEXITCODE -ne 0) {
            throw "Boundary helper failed at iteration $iteration"
        }
        $probe = $line | ConvertFrom-Json
        if ($probe.PSObject.Properties.Name -contains "collapsed" -and $probe.collapsed) {
            $collapsed = $true
            break
        }
        $bits = Convert-HexToUInt64 $probe.input_bits
        if ($bits -le $low -or $bits -ge $high) {
            throw "Boundary helper returned an out-of-range input"
        }

        $argCell.Value2 = [BitConverter]::UInt64BitsToDouble($bits)
        $resultCell.Dirty()
        $excel.Calculate()
        $value = [double] $resultCell.Value2
        $actual = Format-Bits ([BitConverter]::DoubleToUInt64Bits($value))
        $route = if ($actual -eq $probe.cubic_bits) {
            $low = $bits
            "cubic"
        } elseif ($actual -eq $probe.ratio_bits) {
            $high = $bits
            "ratio"
        } else {
            throw "Excel returned $actual, neither cubic $($probe.cubic_bits) nor ratio $($probe.ratio_bits)"
        }
        $steps.Add([ordered]@{
            iteration = $iteration
            input_bits = $probe.input_bits
            cubic_bits = $probe.cubic_bits
            ratio_bits = $probe.ratio_bits
            excel_bits = $actual
            route = $route
            low_after = Format-Bits $low
            high_after = Format-Bits $high
        })
    }
}
finally {
    if ($null -ne $workbook) {
        try { $workbook.Close($false) } catch { }
    }
    if ($null -ne $excel) {
        try { $excel.Quit() } catch { }
    }
    Release-Com $resultCell
    Release-Com $argCell
    Release-Com $sheet
    Release-Com $workbook
    Release-Com $excel
    [GC]::Collect()
    [GC]::WaitForPendingFinalizers()
    [GC]::Collect()
    [GC]::WaitForPendingFinalizers()
}

$deadline = [DateTime]::UtcNow.AddSeconds(15)
do {
    $after = @(Get-Process EXCEL -ErrorAction SilentlyContinue).Count
    if ($after -eq 0) { break }
    Start-Sleep -Milliseconds 100
} while ([DateTime]::UtcNow -lt $deadline)

$artifact = [ordered]@{
    schema_version = "w109-atanh-boundary-bisection-v1"
    function = "ATANH"
    row_id = "G4-02"
    captured_utc = [DateTime]::UtcNow.ToString("o")
    environment = $environment
    serialization = [ordered]@{ before_excel = $before; after_excel = $after }
    initial_low_bits = $LowBits
    initial_high_bits = $HighBits
    final_cubic_discriminator_bits = Format-Bits $low
    final_ratio_discriminator_bits = Format-Bits $high
    adjacent_discriminator_bits = (($high - $low) -eq 1)
    candidate_equivalence_stopped_bisection = $collapsed
    steps = $steps
}
$artifact | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $outPath -Encoding UTF8
Write-Host "ATANH boundary: cubic $(Format-Bits $low) | ratio $(Format-Bits $high) | gap $($high-$low) bit(s)"
Write-Host "wrote $outPath"
if ($after -ne 0) {
    throw "Excel teardown did not reach zero processes (after=$after)"
}
