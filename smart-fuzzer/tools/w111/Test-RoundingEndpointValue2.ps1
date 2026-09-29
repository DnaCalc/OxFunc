[CmdletBinding()]
param([string]$OutPath = 'smart-fuzzer/runs/w111-rounding-endpoint-refinement-20260929/worksheet-controls.json')

# Separate public worksheet/COM observations. Root executes in its serialized
# Excel lane. No raw nonfinite value is serialized as a JSON number.
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$repo = (Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
Import-Module (Join-Path $repo 'smart-fuzzer/tools/CellRefBatch.psm1') -Force
function Release-Com($Object) {
    if ($null -ne $Object -and [Runtime.InteropServices.Marshal]::IsComObject($Object)) {
        [void][Runtime.InteropServices.Marshal]::FinalReleaseComObject($Object)
    }
}
function From-Bits([string]$Bits) {
    $raw = [uint64]::Parse($Bits.Substring(2), [Globalization.NumberStyles]::HexNumber)
    [BitConverter]::ToDouble([BitConverter]::GetBytes($raw), 0)
}
function Describe-Value($Value) {
    if ($null -eq $Value) { return [ordered]@{ managed_type = 'null' } }
    [ordered]@{
        managed_type = $Value.GetType().FullName
        value_as_string = [Convert]::ToString($Value, [Globalization.CultureInfo]::InvariantCulture)
        numeric_bits = $(if ($Value -is [double]) { Get-F64BitsHex $Value } else { $null })
        finite = $(if ($Value -is [double]) { [double]::IsFinite($Value) } else { $null })
        managed_vartype = [string][Microsoft.VisualBasic.Information]::VarType($Value)
    }
}
$before = @(Get-Process EXCEL -ErrorAction SilentlyContinue).Count
if ($before -ne 0) { throw "Serialized Excel lane busy: $before Excel processes" }
$excel = $null; $book = $null; $sheet = $null; $range = $null
$records = [Collections.Generic.List[object]]::new()
$cases = [Collections.Generic.List[object]]::new()
foreach ($fn in @('ROUND','ROUNDDOWN','ROUNDUP','TRUNC')) {
    foreach ($bits in @('0x7fefffffffffffff','0xffefffffffffffff','0x7feffffffffffff7','0x0010000000000000','0x8010000000000000','0x0010000000000008','0x3ff3c083126e978d')) {
        foreach ($count in @(-500,-308,-307,-1,0,308,323,32768)) {
            $cases.Add([ordered]@{ function = $fn; bits = $bits; count = $count })
        }
    }
}
$environment = $null
try {
    $excel = New-Object -ComObject Excel.Application
    $excel.Visible = $false; $excel.DisplayAlerts = $false; $excel.EnableEvents = $false; $excel.ScreenUpdating = $false
    $book = $excel.Workbooks.Add(); $book.PrecisionAsDisplayed = $false
    $excel.Calculation = -4135; $sheet = $book.Worksheets.Item(1)
    $environment = [ordered]@{ excel_version = [string]$excel.Version; excel_build = [string]$excel.Build; operating_system = [string]$excel.OperatingSystem; workbook_compatibility = $(try { [string]$book.CompatibilityVersion } catch { 'unavailable' }); precision_as_displayed = [bool]$book.PrecisionAsDisplayed }
    $values = New-Object 'object[,]' $cases.Count, 2
    $formulas = New-Object 'object[,]' $cases.Count, 9
    for ($i = 0; $i -lt $cases.Count; $i++) {
        $row = $i + 1; $case = $cases[$i]; $call = "$($case.function)(A$row,B$row)"
        $values[$i,0] = From-Bits $case.bits; $values[$i,1] = [double]$case.count
        $formulas[$i,0] = "=$call"
        $formulas[$i,1] = "=ISNUMBER(C$row)"
        $formulas[$i,2] = "=ISERROR(C$row)"
        $formulas[$i,3] = "=TYPE(C$row)"
        $formulas[$i,4] = "=IFERROR(C$row,`"caught-error`")"
        $formulas[$i,5] = "=C$row+0"
        $formulas[$i,6] = "=C$row=C$row"
        $formulas[$i,7] = "=ISNUMBER($call)"
        $formulas[$i,8] = "=ISERROR($call)"
    }
    $range = $sheet.Range("A1:B$($cases.Count)"); $range.Value2 = $values
    $readback = $range.Value2; Release-Com $range; $range = $null
    for ($i = 0; $i -lt $cases.Count; $i++) {
        $sourceBits = Get-F64BitsHex ([double]$readback.GetValue($i+1,1))
        if ($sourceBits -cne $cases[$i].bits) { throw "Input ingress mismatch at row $($i+1)" }
    }
    $range = $sheet.Range("C1:K$($cases.Count)"); $range.Formula2 = $formulas
    $range.ColumnWidth = 32
    $excel.CalculateFull()
    $observed = $range.Value2
    for ($i = 0; $i -lt $cases.Count; $i++) {
        $cell = $sheet.Cells.Item($i+1,3)
        try {
            $details = [ordered]@{}
            $names = @('result','isnumber_reference','iserror_reference','type_reference','iferror_reference','add_zero_reference','self_equal_reference','isnumber_nested','iserror_nested')
            for ($j = 0; $j -lt $names.Count; $j++) { $details[$names[$j]] = Describe-Value ($observed.GetValue($i+1,$j+1)) }
            $records.Add([ordered]@{ source = $cases[$i]; input_storage_exact = $true; formula = [string]$cell.Formula2; displayed_text = [string]$cell.Text; scalar_value2 = Describe-Value $cell.Value2; scalar_value = Describe-Value $cell.Value; bulk_results = $details })
        } finally { Release-Com $cell }
    }
} finally {
    if ($null -ne $book) { try { $book.Close($false) } catch {} }
    if ($null -ne $excel) { try { $excel.Quit() } catch {} }
    foreach ($object in @($range,$sheet,$book,$excel)) { Release-Com $object }
    [GC]::Collect(); [GC]::WaitForPendingFinalizers()
}
if (-not [IO.Path]::IsPathRooted($OutPath)) { $OutPath = Join-Path $repo $OutPath }
[void][IO.Directory]::CreateDirectory((Split-Path $OutPath -Parent))
[ordered]@{ schema_version = 'w111.rounding_endpoint_worksheet_controls.v1'; captured_utc = [DateTime]::UtcNow.ToString('o'); runner_sha256 = (Get-FileHash -LiteralPath $PSCommandPath -Algorithm SHA256).Hash.ToLowerInvariant(); environment = $environment; managed_vartype_note = 'VarType describes the managed Value2 return; it does not inspect a native VARIANT buffer.'; excel_processes_before = $before; excel_processes_after = @(Get-Process EXCEL -ErrorAction SilentlyContinue).Count; records = $records.ToArray() } | ConvertTo-Json -Depth 15 | Set-Content -LiteralPath $OutPath -Encoding utf8
Write-Output "Rounding worksheet controls: $($records.Count) rows; $OutPath"
