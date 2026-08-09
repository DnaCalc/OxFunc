# Clean-room CONVERT plumbing discriminator.
#
# Writes exact adjacent-bit binary64 values through the same bulk object[,]
# Range.Value2 path as Run-W109BulkBatch, immediately reads the argument cells
# back, and records both direct-reference and CONVERT identity results.  This
# distinguishes arithmetic staging from any host-side argument-cell mutation.

[CmdletBinding()]
param(
    [Parameter(Mandatory)] [string] $OutPath
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Get-BitsHex([double] $Value) {
    $bits = [BitConverter]::ToUInt64([BitConverter]::GetBytes($Value), 0)
    return ('0x{0:x16}' -f $bits)
}

function From-BitsHex([string] $Raw) {
    $bits = [uint64]::Parse($Raw.Substring(2), [Globalization.NumberStyles]::HexNumber)
    return [BitConverter]::ToDouble([BitConverter]::GetBytes($bits), 0)
}

function Release-Com([object] $Object) {
    if ($null -ne $Object -and [Runtime.InteropServices.Marshal]::IsComObject($Object)) {
        [void] [Runtime.InteropServices.Marshal]::FinalReleaseComObject($Object)
    }
}

function Array-Value([object] $Values, [int] $Row, [int] $Column) {
    if ($Values -is [Array]) {
        return $Values.GetValue(
            $Values.GetLowerBound(0) + $Row,
            $Values.GetLowerBound(1) + $Column
        )
    }
    return $Values
}

$before = @(Get-Process -Name EXCEL -ErrorAction SilentlyContinue).Count
if ($before -ne 0) {
    throw "Serialized COM lane is not free: EXCEL_PROCESS_COUNT_BEFORE=$before"
}

$excel = $null
$workbook = $null
$worksheet = $null
$cells = $null
$inputRange = $null
$referenceRange = $null
$convertRange = $null
$baseIdentityRange = $null
$environment = $null
$rows = @(
    [ordered]@{ bits = '0x3c2fffffffffffff'; from = 'm';  to = 'm' },
    [ordered]@{ bits = '0x3c30000000000001'; from = 'm';  to = 'm' },
    [ordered]@{ bits = '0x3fefffffffffffff'; from = 'm';  to = 'm' },
    [ordered]@{ bits = '0x3ff0000000000000'; from = 'm';  to = 'm' },
    [ordered]@{ bits = '0x3ff0000000000001'; from = 'm';  to = 'm' },
    [ordered]@{ bits = '0x3fffffffffffffff'; from = 'm';  to = 'm' },
    [ordered]@{ bits = '0x4000000000000001'; from = 'm';  to = 'm' },

    # Retired-v2 CONVERT kill and exact adjacent-bit controls.  The direct
    # Value2 readback decides whether the discrepancy belongs to the cell-input
    # seam; requested CONVERT and m->m isolate prefix and core staging.
    [ordered]@{ bits = '0x457bc2d00cc56eaf'; from = 'nm'; to = 'Pm' },
    [ordered]@{ bits = '0x457bc2d00cc56eb0'; from = 'nm'; to = 'Pm' },
    [ordered]@{ bits = '0x457bc2d00cc56eb1'; from = 'nm'; to = 'Pm' },
    [ordered]@{ bits = '0x457bc2d00cc56eb2'; from = 'nm'; to = 'Pm' },
    [ordered]@{ bits = '0x457bc2d00cc56eb3'; from = 'nm'; to = 'Pm' },
    [ordered]@{ bits = '0x457bc2d00cc56eb4'; from = 'nm'; to = 'Pm' },
    [ordered]@{ bits = '0x457bc2d00cc56eb5'; from = 'nm'; to = 'Pm' },
    [ordered]@{ bits = '0xc57bc2d00cc56eaf'; from = 'nm'; to = 'Pm' },
    [ordered]@{ bits = '0xc57bc2d00cc56eb2'; from = 'nm'; to = 'Pm' },
    [ordered]@{ bits = '0xc57bc2d00cc56eb5'; from = 'nm'; to = 'Pm' },
    [ordered]@{ bits = '0x4570000000000000'; from = 'nm'; to = 'Pm' }
)
$records = @()

try {
    $excel = New-Object -ComObject Excel.Application
    $excel.Visible = $false
    $excel.DisplayAlerts = $false
    $excel.ScreenUpdating = $false
    $excel.EnableEvents = $false
    $workbook = $excel.Workbooks.Add()
    $workbook.PrecisionAsDisplayed = $false
    $worksheet = $workbook.Worksheets.Item(1)
    $cells = $worksheet.Cells
    $environment = [ordered]@{
        excel_version = [string] $excel.Version
        excel_build = $(try { [string] $excel.Build } catch { $null })
        workbook_compatibility = $(try { [string] $workbook.CompatibilityVersion } catch { 'unknown' })
        excel_operating_system = $(try { [string] $excel.OperatingSystem } catch { 'unknown' })
        excel_input_plumbing = 'cell_value2_bulk_with_argument_readback'
    }

    # Columns: exact numeric input, literal from-unit, literal to-unit,
    # direct reference, requested CONVERT, base m->m identity.
    $matrix = New-Object 'object[,]' $rows.Count, 3
    for ($row = 0; $row -lt $rows.Count; $row++) {
        $matrix[$row, 0] = From-BitsHex $rows[$row].bits
        $matrix[$row, 1] = [string] $rows[$row].from
        $matrix[$row, 2] = [string] $rows[$row].to
    }
    $topLeft = $cells.Item(1, 1)
    $bottomRight = $cells.Item($rows.Count, 3)
    $inputRange = $worksheet.Range($topLeft, $bottomRight)
    Release-Com $topLeft
    Release-Com $bottomRight
    $inputRange.Value2 = $matrix

    # Read immediately, before formulas or calculation can touch workbook state.
    $readback = $inputRange.Value2
    $topLeft = $cells.Item(1, 4)
    $bottomRight = $cells.Item($rows.Count, 4)
    $referenceRange = $worksheet.Range($topLeft, $bottomRight)
    Release-Com $topLeft
    Release-Com $bottomRight
    $topLeft = $cells.Item(1, 5)
    $bottomRight = $cells.Item($rows.Count, 5)
    $convertRange = $worksheet.Range($topLeft, $bottomRight)
    Release-Com $topLeft
    Release-Com $bottomRight
    $topLeft = $cells.Item(1, 6)
    $bottomRight = $cells.Item($rows.Count, 6)
    $baseIdentityRange = $worksheet.Range($topLeft, $bottomRight)
    Release-Com $topLeft
    Release-Com $bottomRight
    $referenceRange.Formula2R1C1 = '=RC[-3]'
    $convertRange.Formula2R1C1 = '=CONVERT(RC[-4],RC[-3],RC[-2])'
    $baseIdentityRange.Formula2R1C1 = '=CONVERT(RC[-5],"m","m")'
    [void] $excel.Calculate()
    $referenceValues = $referenceRange.Value2
    $convertValues = $convertRange.Value2
    $baseIdentityValues = $baseIdentityRange.Value2

    for ($row = 0; $row -lt $rows.Count; $row++) {
        $readValue = [double] (Array-Value $readback $row 0)
        $referenceValue = [double] (Array-Value $referenceValues $row 0)
        $convertValue = [double] (Array-Value $convertValues $row 0)
        $baseIdentityValue = [double] (Array-Value $baseIdentityValues $row 0)
        $records += [ordered]@{
            requested_bits = $rows[$row].bits
            from_unit = $rows[$row].from
            to_unit = $rows[$row].to
            argument_value2_readback_bits = Get-BitsHex $readValue
            direct_reference_bits = Get-BitsHex $referenceValue
            requested_convert_bits = Get-BitsHex $convertValue
            base_m_identity_bits = Get-BitsHex $baseIdentityValue
        }
    }
}
finally {
    if ($null -ne $workbook) { try { $workbook.Close($false) } catch {} }
    if ($null -ne $excel) { try { $excel.Quit() } catch {} }
    Release-Com $baseIdentityRange
    Release-Com $convertRange
    Release-Com $referenceRange
    Release-Com $inputRange
    Release-Com $cells
    Release-Com $worksheet
    Release-Com $workbook
    Release-Com $excel
    [GC]::Collect()
    [GC]::WaitForPendingFinalizers()
}

$after = -1
for ($poll = 0; $poll -lt 20; $poll++) {
    $after = @(Get-Process -Name EXCEL -ErrorAction SilentlyContinue).Count
    if ($after -eq 0) { break }
    Start-Sleep -Milliseconds 200
}

$artifact = [ordered]@{
    schema_version = 'w109.convert.value2_argument_readback.v2'
    captured_utc = [DateTime]::UtcNow.ToString('o')
    excel_process_count_before = $before
    excel_process_count_after = $after
    environment = $environment
    rows = $records
}
$parent = Split-Path -Parent $OutPath
if (-not [string]::IsNullOrWhiteSpace($parent)) {
    [void] (New-Item -ItemType Directory -Force -Path $parent)
}
$artifact | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $OutPath -Encoding utf8
$artifact | ConvertTo-Json -Depth 8

if ($after -ne 0) {
    throw "Excel did not exit cleanly: EXCEL_PROCESS_COUNT_AFTER=$after"
}
