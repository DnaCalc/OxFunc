[CmdletBinding()]
param(
    [string]$OutPath = "smart-fuzzer/cache/w111-typed-fixture-ingress-20260929.json"
)

# Live, serialized smoke check of the actual generic runner fixture writer.
# Root owns the Excel lane; this script must only run after that lane is free.
Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$repo = (Resolve-Path (Join-Path $PSScriptRoot "../../..")).Path
$runner = Join-Path $repo "smart-fuzzer/tools/Run-ArraySupportTranche.ps1"
$tokens = $null; $parseErrors = $null
$ast = [Management.Automation.Language.Parser]::ParseFile($runner, [ref]$tokens, [ref]$parseErrors)
if ($parseErrors.Count) { throw "runner parse errors: $parseErrors" }
# Load definitions without running the runner's top-level case/Excel workflow.
foreach ($definition in $ast.FindAll({ param($node) $node -is [Management.Automation.Language.FunctionDefinitionAst] }, $false)) {
    . ([scriptblock]::Create($definition.Extent.Text))
}

$before = @(Get-Process EXCEL -ErrorAction SilentlyContinue).Count
if ($before -ne 0) { throw "Serialized Excel lane is busy: $before existing EXCEL processes" }
$excel = $null; $workbook = $null; $worksheet = $null; $cell = $null
$records = [Collections.Generic.List[object]]::new()
$environment = $null
$inputs = @(
    @{ kind = "text"; value = "2" },
    @{ kind = "text"; value = "00123" },
    @{ kind = "text"; value = "1E10" },
    @{ kind = "text"; value = "01/02/2020" },
    @{ kind = "text"; value = "=1+1" },
    @{ kind = "text"; value = "'apostrophe" },
    @{ kind = "text"; value = "" },
    @{ kind = "number"; value = 0.30000000000000004 },
    @{ kind = "text"; value = "FALSE" },
    @{ kind = "logical"; value = $false },
    @{ kind = "logical"; value = $true },
    @{ kind = "empty_cell" },
    @{ kind = "error"; code = "NA" },
    @{ kind = "error"; code = "Div0" },
    @{ kind = "error"; code = "Value" },
    @{ kind = "error"; code = "Num" },
    @{ kind = "text"; value = "" },
    @{ kind = "number"; value = 42.0 },
    @{ kind = "empty_cell" }
)
try {
    $excel = New-Object -ComObject Excel.Application
    $excel.Visible = $false; $excel.DisplayAlerts = $false
    $excel.ScreenUpdating = $false; $excel.EnableEvents = $false
    $workbook = $excel.Workbooks.Add()
    $workbook.PrecisionAsDisplayed = $false
    $worksheet = $workbook.Worksheets.Item(1)
    $cell = $worksheet.Range("A1")
    $environment = [ordered]@{
        excel_version = [string]$excel.Version; excel_build = [string]$excel.Build
        workbook_compatibility_version = $(try { [string]$workbook.CompatibilityVersion } catch { "unavailable" })
        operating_system = [string]$excel.OperatingSystem
        workbook_date_system = if ([bool]$workbook.Date1904) { "1904" } else { "1900" }
        precision_as_displayed = [bool]$workbook.PrecisionAsDisplayed
    }
    foreach ($value in $inputs) {
        $status = "match"; $message = $null
        try { Set-ExcelCellFromTypedValue $cell ([pscustomobject]$value) }
        catch { $status = "ingress_mismatch"; $message = $_.Exception.Message }
        $observed = $cell.Value2
        $isBlank = [bool]$worksheet.Evaluate("ISBLANK(A1)")
        $isText = [bool]$worksheet.Evaluate("ISTEXT(A1)")
        if (($value.kind -eq "empty_cell" -and -not $isBlank) -or
            ($value.kind -eq "text" -and ($isBlank -or -not $isText))) {
            $status = "ingress_mismatch"; $message = "ISBLANK/ISTEXT disagrees with fixture kind"
        }
        [void]$records.Add([ordered]@{
            index = $records.Count; input = $value; status = $status; message = $message
            observed_type = if ($null -eq $observed) { "null" } else { $observed.GetType().Name }
            observed_value = $observed
            observed_bits = if ($observed -is [double]) { Get-DoubleBitsHex $observed } else { $null }
            is_blank = $isBlank; is_text = $isText; has_formula = [bool]$cell.HasFormula
            number_format = [string]$cell.NumberFormat; prefix_character = [string]$cell.PrefixCharacter
        })
    }
} finally {
    if ($null -ne $workbook) { try { $workbook.Close($false) } catch {} }
    if ($null -ne $excel) { try { $excel.Quit() } catch {} }
    foreach ($object in @($cell, $worksheet, $workbook, $excel)) {
        if ($null -ne $object -and [Runtime.InteropServices.Marshal]::IsComObject($object)) {
            [void][Runtime.InteropServices.Marshal]::FinalReleaseComObject($object)
        }
    }
    [GC]::Collect(); [GC]::WaitForPendingFinalizers()
}
$after = @(Get-Process EXCEL -ErrorAction SilentlyContinue).Count
$mismatches = @($records | Where-Object status -ne "match").Count
$output = [ordered]@{
    schema_version = "w111.typed_fixture_ingress.v1"; captured_utc = [DateTime]::UtcNow.ToString("o")
    runner_sha256 = (Get-FileHash -LiteralPath $runner -Algorithm SHA256).Hash.ToLowerInvariant()
    environment = $environment; excel_process_count_before = $before; excel_process_count_after = $after
    cases = $records.Count; mismatches = $mismatches; records = $records.ToArray()
}
if (-not [IO.Path]::IsPathRooted($OutPath)) { $OutPath = Join-Path $repo $OutPath }
$outDir = Split-Path $OutPath -Parent
[void][IO.Directory]::CreateDirectory($outDir)
$output | ConvertTo-Json -Depth 12 | Set-Content -LiteralPath $OutPath -Encoding utf8
Write-Host "Typed fixture ingress: $($records.Count) rows, $mismatches mismatches; Excel processes $before -> $after; $OutPath"
if ($mismatches -ne 0) { exit 1 }
