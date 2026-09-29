[CmdletBinding()]
param(
    [string]$JudgePath = "smart-fuzzer/runs/w111-broad-20260929/judge-progress.json",
    [string]$OutPath = "smart-fuzzer/cache/w111-numeric-value2-ingress-20260929.json",
    [int]$MaxInputs = 4096
)

# Read-only with respect to existing evidence. Answers a separate plumbing
# question: do source bits survive Value2 storage and a direct cell reference?
# No input, expected result, or judgement is normalized or overwritten.
Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$repo = (Resolve-Path (Join-Path $PSScriptRoot "../../..")).Path
Import-Module (Join-Path $repo "smart-fuzzer/tools/CellRefBatch.psm1") -Force
function Resolve-InputPath([string]$Path) {
    if ([IO.Path]::IsPathRooted($Path)) { return $Path }
    return Join-Path $repo $Path
}
function From-Bits([string]$Bits) {
    $u = [uint64]::Parse($Bits.Substring(2), [Globalization.NumberStyles]::HexNumber)
    return [BitConverter]::ToDouble([BitConverter]::GetBytes($u), 0)
}
function Cell-Value($Values, [int]$Row, [int]$Col = 0) {
    if ($Values -is [Array]) { return $Values.GetValue($Values.GetLowerBound(0) + $Row, $Values.GetLowerBound(1) + $Col) }
    return $Values
}
function Release-Com($Object) {
    if ($null -ne $Object -and [Runtime.InteropServices.Marshal]::IsComObject($Object)) {
        [void][Runtime.InteropServices.Marshal]::FinalReleaseComObject($Object)
    }
}

$selected = [ordered]@{}
foreach ($bits in @(
    '0x0000000000000000', '0x8000000000000000',
    '0x0000000000000001', '0x8000000000000001',
    '0x0000000000000002', '0x8000000000000002',
    '0x0008000000000000', '0x8008000000000000',
    '0x000fffffffffffff', '0x800fffffffffffff',
    '0x0010000000000000', '0x8010000000000000',
    '0x0010000000000001', '0x8010000000000001',
    '0x3fb999999999999a', '0x3fd3333333333334',
    '0x3fefffffffffffff', '0x3ff0000000000000', '0x3ff0000000000001',
    '0x433fffffffffffff', '0x4340000000000000', '0x7fefffffffffffff', '0xffefffffffffffff'
)) { $selected[$bits] = [Collections.Generic.List[string]]::new() }
$JudgePath = Resolve-InputPath $JudgePath
$judgeHash = $null
if (Test-Path -LiteralPath $JudgePath) {
    $judgeHash = (Get-FileHash -LiteralPath $JudgePath -Algorithm SHA256).Hash.ToLowerInvariant()
    $reports = Get-Content -LiteralPath $JudgePath -Raw | ConvertFrom-Json
    foreach ($report in $reports) {
        foreach ($miss in $report.misses) {
            foreach ($raw in $miss.args) {
                $bits = ([string]$raw).ToLowerInvariant()
                if (-not $selected.Contains($bits) -and $selected.Count -lt $MaxInputs) {
                    $selected[$bits] = [Collections.Generic.List[string]]::new()
                }
                if ($selected.Contains($bits) -and $selected[$bits].Count -lt 5) {
                    $witnessRef = "$($report.function_id)/$($miss.id)"
                    if (-not $selected[$bits].Contains($witnessRef)) { [void]$selected[$bits].Add($witnessRef) }
                }
            }
        }
    }
}
$bitsList = @($selected.Keys)
$before = @(Get-Process EXCEL -ErrorAction SilentlyContinue).Count
if ($before -ne 0) { throw "Serialized Excel lane is busy: $before existing EXCEL processes" }
$excel = $null; $workbook = $null; $worksheet = $null
$inputRange = $null; $referenceRange = $null; $absRange = $null; $refTypeRange = $null
$environment = $null; $records = @()
try {
    $excel = New-Object -ComObject Excel.Application
    $excel.Visible = $false; $excel.DisplayAlerts = $false; $excel.EnableEvents = $false; $excel.ScreenUpdating = $false
    $workbook = $excel.Workbooks.Add()
    $workbook.PrecisionAsDisplayed = $false
    $excel.Calculation = -4135
    $worksheet = $workbook.Worksheets.Item(1)
    $environment = [ordered]@{
        excel_version = [string]$excel.Version; excel_build = [string]$excel.Build
        workbook_compatibility_version = $(try { [string]$workbook.CompatibilityVersion } catch { "unavailable" })
        excel_operating_system = [string]$excel.OperatingSystem
        excel_channel = "unavailable_not_queried"
        workbook_date_system = if ([bool]$workbook.Date1904) { "1904" } else { "1900" }
        precision_as_displayed = [bool]$workbook.PrecisionAsDisplayed
        excel_input_plumbing = "bulk_value2_with_immediate_and_postcalc_readback"
    }
    $rows = $bitsList.Count
    $inputs = New-Object 'object[,]' $rows, 1
    for ($i = 0; $i -lt $rows; $i++) { $inputs[$i,0] = From-Bits $bitsList[$i] }
    $inputRange = $worksheet.Range("A1:A$rows")
    $inputRange.Value2 = $inputs
    $immediate = $inputRange.Value2
    $referenceRange = $worksheet.Range("B1:B$rows")
    $referenceRange.Formula2R1C1 = '=RC[-1]'
    $absRange = $worksheet.Range("C1:C$rows")
    $absRange.Formula2R1C1 = '=ABS(RC[-2])'
    $refTypeRange = $worksheet.Range("D1:D$rows")
    $refTypeRange.Formula2R1C1 = '=ISREF(RC[-3])'
    $excel.Calculate()
    $postcalc = $inputRange.Value2; $references = $referenceRange.Value2; $absolute = $absRange.Value2; $referenceKinds = $refTypeRange.Value2
    for ($i = 0; $i -lt $rows; $i++) {
        $sourceBits = $bitsList[$i]
        $immediateValue = Cell-Value $immediate $i
        $postcalcValue = Cell-Value $postcalc $i
        $referenceValue = Cell-Value $references $i
        $absValue = Cell-Value $absolute $i
        $ib = if ($immediateValue -is [double]) { Get-F64BitsHex $immediateValue } else { $null }
        $pb = if ($postcalcValue -is [double]) { Get-F64BitsHex $postcalcValue } else { $null }
        $rb = if ($referenceValue -is [double]) { Get-F64BitsHex $referenceValue } else { $null }
        $ab = if ($absValue -is [double]) { Get-F64BitsHex $absValue } else { $null }
        $records += [ordered]@{
            source_bits = $sourceBits; related_witnesses = $selected[$sourceBits].ToArray()
            immediate_value2_bits = $ib; postcalc_value2_bits = $pb; direct_reference_bits = $rb; abs_result_bits = $ab
            input_storage_exact = ($sourceBits -eq $ib -and $sourceBits -eq $pb)
            stored_to_reference_exact = ($pb -eq $rb)
            isref_on_argument_cell = [bool](Cell-Value $referenceKinds $i)
        }
    }
} finally {
    if ($null -ne $workbook) { try { $workbook.Close($false) } catch {} }
    if ($null -ne $excel) { try { $excel.Quit() } catch {} }
    foreach ($object in @($inputRange, $referenceRange, $absRange, $refTypeRange, $worksheet, $workbook, $excel)) { Release-Com $object }
    [GC]::Collect(); [GC]::WaitForPendingFinalizers()
}
$after = @(Get-Process EXCEL -ErrorAction SilentlyContinue).Count
$storageChanges = @($records | Where-Object {-not $_.input_storage_exact}).Count
$referenceChanges = @($records | Where-Object {-not $_.stored_to_reference_exact}).Count
$doc = [ordered]@{
    schema_version = "w111.numeric_value2_ingress.v1"; captured_utc = [DateTime]::UtcNow.ToString("o")
    source_judge_path = $JudgePath; source_judge_sha256 = $judgeHash
    runner_sha256 = (Get-FileHash -LiteralPath $PSCommandPath -Algorithm SHA256).Hash.ToLowerInvariant()
    environment = $environment; excel_process_count_before = $before; excel_process_count_after = $after
    rows = $records.Count; source_to_storage_changes = $storageChanges; storage_to_reference_changes = $referenceChanges
    records = $records
}
$OutPath = Resolve-InputPath $OutPath
[void][IO.Directory]::CreateDirectory((Split-Path $OutPath -Parent))
$doc | ConvertTo-Json -Depth 12 | Set-Content -LiteralPath $OutPath -Encoding utf8
Write-Host "Numeric ingress: $($records.Count) inputs; source/storage changes=$storageChanges; storage/reference changes=$referenceChanges; Excel $before -> $after; $OutPath"
