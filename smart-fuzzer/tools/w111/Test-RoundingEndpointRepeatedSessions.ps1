[CmdletBinding()]
param([string]$OutPath = 'smart-fuzzer/runs/w111-rounding-endpoint-refinement-20260929/repeated-session-controls.json')
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
    [ordered]@{ managed_type = $Value.GetType().FullName; text = [Convert]::ToString($Value, [Globalization.CultureInfo]::InvariantCulture); bits = $(if ($Value -is [double]) { Get-F64BitsHex $Value } else { $null }); managed_vartype = [string][Microsoft.VisualBasic.Information]::VarType($Value) }
}
$cases = [Collections.Generic.List[object]]::new()
foreach ($fn in @('ROUND','ROUNDDOWN','ROUNDUP','TRUNC')) {
    foreach ($bits in @('0x7fefffffffffffff','0xffefffffffffffff','0x0010000000000000','0x8010000000000000','0x3ff4000000000000')) {
        foreach ($count in @(-308,0,323)) { $cases.Add([ordered]@{ function = $fn; bits = $bits; count = $count }) }
    }
}
$records = [Collections.Generic.List[object]]::new(); $sessions = [Collections.Generic.List[object]]::new()
for ($session = 1; $session -le 2; $session++) {
    if (@(Get-Process EXCEL -ErrorAction SilentlyContinue).Count -ne 0) { throw 'Serialized Excel lane is occupied' }
    $excel = $null; $book = $null; $sheet = $null; $range = $null
    try {
        $excel = New-Object -ComObject Excel.Application
        $excel.Visible = $false; $excel.DisplayAlerts = $false; $excel.EnableEvents = $false; $excel.ScreenUpdating = $false
        $book = $excel.Workbooks.Add(); $book.PrecisionAsDisplayed = $false; $excel.Calculation = -4135
        $sessionRecord = [ordered]@{ session = $session; process_ids = @(Get-Process EXCEL | ForEach-Object Id); excel_version = [string]$excel.Version; excel_build = [string]$excel.Build; workbook_compatibility = $(try { [string]$book.CompatibilityVersion } catch { 'unavailable' }); precision_as_displayed = [bool]$book.PrecisionAsDisplayed; operating_system = [string]$excel.OperatingSystem }
        $sessions.Add($sessionRecord); $block = 0
        foreach ($format in @('General','0.000000000000000E+00')) {
            foreach ($calculate in @('Calculate','CalculateFull')) {
                foreach ($formulaMode in @('Formula2','Formula2R1C1')) {
                    $sheet = $book.Worksheets.Add(); $sheet.Name = "probe$block"; $block++
                    $range = $sheet.Range("A1:G$($cases.Count)"); $range.NumberFormat = $format; $range.ColumnWidth = 48; Release-Com $range; $range = $null
                    $values = New-Object 'object[,]' $cases.Count, 2; $main = New-Object 'object[,]' $cases.Count, 1; $controls = New-Object 'object[,]' $cases.Count, 4
                    for ($i = 0; $i -lt $cases.Count; $i++) {
                        $row = $i+1; $case = $cases[$i]; $values[$i,0] = From-Bits $case.bits; $values[$i,1] = [double]$case.count
                        $main[$i,0] = "=$($case.function)(A$row,B$row)"
                        $controls[$i,0] = "=ISNUMBER(C$row)"; $controls[$i,1] = "=ISERROR(C$row)"; $controls[$i,2] = "=TYPE(C$row)"; $controls[$i,3] = "=C$row+0"
                    }
                    $range = $sheet.Range("A1:B$($cases.Count)"); $range.Value2 = $values; $initial = $range.Value2; Release-Com $range; $range = $null
                    $range = $sheet.Range("C1:C$($cases.Count)")
                    if ($formulaMode -eq 'Formula2') { $range.Formula2 = $main } else {
                        foreach ($fn in @('ROUND','ROUNDDOWN','ROUNDUP','TRUNC')) {
                            $indices = @(for ($i = 0; $i -lt $cases.Count; $i++) { if ($cases[$i].function -eq $fn) { $i+1 } })
                            $group = $sheet.Range("C$($indices[0]):C$($indices[-1])")
                            try { $group.Formula2R1C1 = "=$fn(RC[-2],RC[-1])" } finally { Release-Com $group }
                        }
                    }
                    if ($calculate -eq 'Calculate') { $excel.Calculate() } else { $excel.CalculateFull() }
                    $beforeDependents = $range.Value2; Release-Com $range; $range = $null
                    $range = $sheet.Range("D1:G$($cases.Count)"); $range.Formula2 = $controls; Release-Com $range; $range = $null
                    if ($calculate -eq 'Calculate') { $excel.Calculate() } else { $excel.CalculateFull() }
                    $range = $sheet.Range("A1:G$($cases.Count)"); $observed = $range.Value2
                    for ($i = 0; $i -lt $cases.Count; $i++) {
                        $inputBefore = Get-F64BitsHex ([double]$initial.GetValue($i+1,1)); $inputAfter = Get-F64BitsHex ([double]$observed.GetValue($i+1,1))
                        $countBefore = Get-F64BitsHex ([double]$initial.GetValue($i+1,2)); $countAfter = Get-F64BitsHex ([double]$observed.GetValue($i+1,2)); $countExpected = Get-F64BitsHex ([double]$cases[$i].count)
                        $cell = $sheet.Cells.Item($i+1,3)
                        try {
                            $records.Add([ordered]@{ session = $session; sheet = [string]$sheet.Name; row = $i+1; format = $format; calculate = $calculate; formula_mode = $formulaMode; source = $cases[$i]; input_before = $inputBefore; input_after = $inputAfter; count_before = $countBefore; count_after = $countAfter; ingress_exact = ($inputBefore -ceq $cases[$i].bits -and $inputAfter -ceq $cases[$i].bits -and $countBefore -ceq $countExpected -and $countAfter -ceq $countExpected); before_dependents = Describe-Value ($beforeDependents.GetValue($i+1,1)); after_dependents = Describe-Value ($observed.GetValue($i+1,3)); scalar_value2 = Describe-Value $cell.Value2; displayed_text = [string]$cell.Text; isnumber = Describe-Value ($observed.GetValue($i+1,4)); iserror = Describe-Value ($observed.GetValue($i+1,5)); type = Describe-Value ($observed.GetValue($i+1,6)); add_zero = Describe-Value ($observed.GetValue($i+1,7)) })
                        } finally { Release-Com $cell }
                    }
                    Release-Com $range; $range = $null; Release-Com $sheet; $sheet = $null
                }
            }
        }
        foreach ($record in $records) {
            if ($record.session -ne $session) { continue }
            $sheet = $book.Worksheets.Item($record.sheet); $cell = $sheet.Cells.Item($record.row,3)
            try { $record['after_all_recalculations'] = Describe-Value $cell.Value2 } finally { Release-Com $cell; Release-Com $sheet; $sheet = $null }
        }
    } finally {
        if ($null -ne $book) { try { $book.Close($false) } catch {} }
        if ($null -ne $excel) { try { $excel.Quit() } catch {} }
        foreach ($object in @($range,$sheet,$book,$excel)) { Release-Com $object }
        [GC]::Collect(); [GC]::WaitForPendingFinalizers()
    }
    for ($i = 0; $i -lt 50 -and @(Get-Process EXCEL -ErrorAction SilentlyContinue).Count -ne 0; $i++) { Start-Sleep -Milliseconds 100 }
}
if (-not [IO.Path]::IsPathRooted($OutPath)) { $OutPath = Join-Path $repo $OutPath }
[void][IO.Directory]::CreateDirectory((Split-Path $OutPath -Parent))
[ordered]@{ schema_version = 'w111.rounding_endpoint_repeated_sessions.v1'; captured_utc = [DateTime]::UtcNow.ToString('o'); runner_sha256 = (Get-FileHash -LiteralPath $PSCommandPath -Algorithm SHA256).Hash.ToLowerInvariant(); sessions = $sessions.ToArray(); managed_vartype_note = 'Describes the returned managed object, not an inspected native VARIANT buffer.'; records = $records.ToArray() } | ConvertTo-Json -Depth 14 | Set-Content -LiteralPath $OutPath -Encoding utf8
Write-Output "Rounding repetition: $($records.Count) observations in two independent sessions; $OutPath"
