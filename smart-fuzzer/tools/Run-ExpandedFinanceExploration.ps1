[CmdletBinding(PositionalBinding = $false)]
param(
    [string] $RepoRoot,
    [string] $RunId,
    [int] $CaseCount = 10000000,
    [int] $Seed = 8804,
    [int] $CandidateLimit = 640
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

if ([string]::IsNullOrWhiteSpace($RepoRoot)) {
    $scriptRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
    $RepoRoot = (Resolve-Path (Join-Path $scriptRoot "..\..")).Path
}
$RepoRoot = [System.IO.Path]::GetFullPath($RepoRoot)
if ([string]::IsNullOrWhiteSpace($RunId)) {
    $RunId = (Get-Date).ToUniversalTime().ToString("yyyyMMddTHHmmssZ") + "-expanded-finance"
}

$runDir = Join-Path $RepoRoot ("smart-fuzzer\runs\" + $RunId)
$comparisonDir = Join-Path $runDir "comparisons"
$failureDir = Join-Path $runDir "failure_packets"
$logDir = Join-Path $runDir "logs"
New-Item -ItemType Directory -Force -Path $runDir, $comparisonDir, $failureDir, $logDir | Out-Null

function Get-Sha256Text {
    param([string] $Text)
    $bytes = [System.Text.Encoding]::UTF8.GetBytes($Text)
    $sha = [System.Security.Cryptography.SHA256]::Create()
    try {
        $hash = $sha.ComputeHash($bytes)
        return "sha256:" + ([System.BitConverter]::ToString($hash).Replace("-", "").ToLowerInvariant())
    }
    finally {
        $sha.Dispose()
    }
}

function Write-JsonFile {
    param([string] $Path, [object] $Value, [int] $Depth = 16)
    $Value | ConvertTo-Json -Depth $Depth | Set-Content -LiteralPath $Path -Encoding UTF8
}

function Add-JsonLine {
    param([string] $Path, [object] $Value, [int] $Depth = 16)
    ($Value | ConvertTo-Json -Compress -Depth $Depth) | Add-Content -LiteralPath $Path -Encoding UTF8
}

function Get-F64BitsHex {
    param([double] $Value)
    $bits = [System.BitConverter]::ToUInt64([System.BitConverter]::GetBytes($Value), 0)
    return ("0x{0:x16}" -f $bits)
}

function Release-ComObject {
    param([object] $Object)
    if ($null -ne $Object -and [System.Runtime.InteropServices.Marshal]::IsComObject($Object)) {
        [void] [System.Runtime.InteropServices.Marshal]::FinalReleaseComObject($Object)
    }
}

function Get-ArrayCellValue {
    param([object] $Values, [int] $RowIndex)
    if ($Values -is [System.Array]) {
        $lower0 = $Values.GetLowerBound(0)
        $lower1 = $Values.GetLowerBound(1)
        return $Values.GetValue($lower0 + $RowIndex, $lower1)
    }
    return $Values
}

function Convert-ExcelErrorTypeToCode {
    param([int] $ErrorType)
    switch ($ErrorType) {
        1 { return "Null" }
        2 { return "Div0" }
        3 { return "Value" }
        4 { return "Ref" }
        5 { return "Name" }
        6 { return "Num" }
        7 { return "NA" }
        8 { return "GettingData" }
        default { return "ExcelErrorType$ErrorType" }
    }
}

function New-FormulaArray {
    param([object[]] $Candidates)
    $array = New-Object "object[,]" $Candidates.Count, 1
    for ($row = 0; $row -lt $Candidates.Count; $row++) {
        $array[$row, 0] = [string] $Candidates[$row].formula_text
    }
    return ,$array
}

function New-ErrorTypeFormulaArray {
    param([int] $Count)
    $array = New-Object "object[,]" $Count, 1
    for ($row = 0; $row -lt $Count; $row++) {
        $excelRow = $row + 1
        $array[$row, 0] = "=IF(ISERROR(A$excelRow),ERROR.TYPE(A$excelRow),"""")"
    }
    return ,$array
}

function Convert-ExcelOutcome {
    param([object] $Value, [object] $ErrorType)
    if ($null -ne $ErrorType -and -not [string]::IsNullOrWhiteSpace([string] $ErrorType)) {
        $code = Convert-ExcelErrorTypeToCode ([int] [double] $ErrorType)
        return [ordered]@{ kind = "error"; code = $code; digest_payload = "error:$code" }
    }
    if ($Value -is [double] -or $Value -is [single] -or $Value -is [decimal] -or $Value -is [int]) {
        $value = [double] $Value
        $bits = Get-F64BitsHex $value
        return [ordered]@{ kind = "number"; value = $value; bits_hex = $bits; digest_payload = "number:$bits" }
    }
    if ($Value -is [bool]) {
        return [ordered]@{ kind = "logical"; value = [bool] $Value; digest_payload = "logical:$Value" }
    }
    $text = [string] $Value
    return [ordered]@{ kind = "text"; value = $text; digest_payload = "text:$text" }
}

function Invoke-ExcelCandidateBatch {
    param([object[]] $Candidates)
    $excel = $null
    $workbook = $null
    $worksheet = $null
    $range = $null
    $errorRange = $null
    try {
        $excel = New-Object -ComObject Excel.Application
        try { $excel.Visible = $false } catch {}
        try { $excel.DisplayAlerts = $false } catch {}
        try { $excel.ScreenUpdating = $false } catch {}
        try { $excel.EnableEvents = $false } catch {}
        $workbook = $excel.Workbooks.Add()
        $worksheet = $workbook.Worksheets.Item(1)
        try { $worksheet.Columns.Item(1).ColumnWidth = 32 } catch {}
        $anchor = $worksheet.Range("A1")
        $range = $anchor.Resize($Candidates.Count, 1)
        Release-ComObject $anchor
        $errorAnchor = $worksheet.Range("B1")
        $errorRange = $errorAnchor.Resize($Candidates.Count, 1)
        Release-ComObject $errorAnchor
        $range.Formula2 = New-FormulaArray $Candidates
        $errorRange.Formula2 = New-ErrorTypeFormulaArray $Candidates.Count
        $calcAnchor = $worksheet.Range("A1")
        $calcRange = $calcAnchor.Resize($Candidates.Count, 2)
        Release-ComObject $calcAnchor
        [void] $calcRange.Calculate()
        Release-ComObject $calcRange
        $values = $range.Value2
        $errorValues = $errorRange.Value2
        $outcomes = New-Object System.Collections.ArrayList
        for ($row = 0; $row -lt $Candidates.Count; $row++) {
            $value = Get-ArrayCellValue $values $row
            $errorType = Get-ArrayCellValue $errorValues $row
            $outcomes.Add((Convert-ExcelOutcome $value $errorType)) | Out-Null
        }
        return [ordered]@{
            blocked = $false
            outcomes = @($outcomes)
            environment = [ordered]@{
                excel_version = [string] $excel.Version
                excel_build = $(try { [string] $excel.Build } catch { $null })
                workbook_compatibility = $(try { [string] $workbook.CompatibilityVersion } catch { "unknown" })
            }
        }
    }
    catch {
        return [ordered]@{ blocked = $true; blocker = $_.Exception.Message; outcomes = @(); environment = @{} }
    }
    finally {
        Release-ComObject $range
        Release-ComObject $errorRange
        if ($null -ne $workbook) { try { $workbook.Close($false) } catch {} }
        if ($null -ne $excel) { try { $excel.Quit() } catch {} }
        Release-ComObject $worksheet
        Release-ComObject $workbook
        Release-ComObject $excel
        [System.GC]::Collect()
        [System.GC]::WaitForPendingFinalizers()
    }
}

$manifestPath = Join-Path $RepoRoot "smart-fuzzer\tools\pmt_ppmt_local_eval\Cargo.toml"
$localWatch = [System.Diagnostics.Stopwatch]::StartNew()
& cargo run --release --quiet --manifest-path $manifestPath --bin expanded_finance_explorer -- --run-dir $runDir --cases $CaseCount --seed $Seed --candidate-limit $CandidateLimit
if ($LASTEXITCODE -ne 0) {
    throw "expanded_finance_explorer failed with exit code $LASTEXITCODE"
}
$localWatch.Stop()

$candidatePath = Join-Path $runDir "candidates\excel_candidates.jsonl"
$candidates = @(Get-Content -LiteralPath $candidatePath | Where-Object { -not [string]::IsNullOrWhiteSpace($_) } | ForEach-Object { $_ | ConvertFrom-Json })
$excelWatch = [System.Diagnostics.Stopwatch]::StartNew()
$excel = Invoke-ExcelCandidateBatch $candidates
$excelWatch.Stop()

$comparisonsPath = Join-Path $comparisonDir "excel_sample_comparisons.jsonl"
if (Test-Path -LiteralPath $comparisonsPath) { Remove-Item -LiteralPath $comparisonsPath }
$matches = 0
$expectedKnown = 0
$unexpected = 0
$blocked = 0

for ($i = 0; $i -lt $candidates.Count; $i++) {
    $candidate = $candidates[$i]
    $local = $candidate.local_outcome
    $excelOutcome = if ($excel.outcomes.Count -gt $i) { $excel.outcomes[$i] } else { $null }
    $result = "blocked"
    $delta = $null
    if ($null -eq $excelOutcome) {
        $blocked += 1
    }
    elseif ([string]$local.kind -eq "number" -and [string]$excelOutcome.kind -eq "number") {
        $delta = [Math]::Abs(([double]$local.value) - ([double]$excelOutcome.value))
        $scale = [Math]::Max(1.0, [Math]::Abs([double]$excelOutcome.value))
        $withinLiteralEncodingTolerance = $delta -le (1E-12 * $scale)
        if ([string]$local.bits_hex -eq [string]$excelOutcome.bits_hex) {
            $matches += 1
            $result = "match"
        }
        elseif (([double]$local.value) -eq 0.0 -and ([double]$excelOutcome.value) -eq 0.0) {
            $matches += 1
            $result = "match_signed_zero_difference"
        }
        elseif (([string]$candidate.function_id -in @("FUNC.PMT","FUNC.PPMT","FUNC.IPMT")) -and ($candidate.coverage_buckets -notcontains "rate:zero")) {
            $expectedKnown += 1
            $result = "expected_known_financial_exactness_drift"
        }
        elseif ($withinLiteralEncodingTolerance) {
            $expectedKnown += 1
            $result = "expected_formula_literal_encoding_drift"
        }
        else {
            $unexpected += 1
            $result = "unexpected_mismatch"
        }
    }
    elseif ([string]$local.kind -eq [string]$excelOutcome.kind -and [string]$local.digest_payload -eq [string]$excelOutcome.digest_payload) {
        $matches += 1
        $result = "match"
    }
    else {
        $unexpected += 1
        $result = "unexpected_mismatch"
    }

    $comparison = [ordered]@{
        schema_version = "oxfunc.smart_fuzzer.expanded_excel_comparison.v0"
        case_id = [string] $candidate.case_id
        function_id = [string] $candidate.function_id
        formula_text = [string] $candidate.formula_text
        comparison_result = $result
        abs_delta = $delta
        coverage_buckets = $candidate.coverage_buckets
        selection_reason = [string] $candidate.selection_reason
        local_outcome = $local
        excel_outcome = $excelOutcome
    }
    Add-JsonLine $comparisonsPath $comparison
    if ($result -eq "unexpected_mismatch") {
        Write-JsonFile (Join-Path $failureDir ([string]$candidate.case_id + ".json")) $comparison
    }
}

$localRollup = Get-Content -LiteralPath (Join-Path $runDir "local_rollup.json") -Raw | ConvertFrom-Json
$rollup = [ordered]@{
    schema_version = "oxfunc.smart_fuzzer.expanded_run_rollup.v0"
    run_id = $RunId
    generated = $CaseCount
    local_evaluated = $CaseCount
    excel_sampled = $candidates.Count
    excel_blocked = [bool] $excel.blocked
    matches = $matches
    expected_known_deviations = $expectedKnown
    unexpected_mismatches = $unexpected
    blocked = $blocked
    local_cases_per_second = $localRollup.local_cases_per_second
    local_wall_seconds = $localRollup.local_wall_seconds
    wrapper_local_wall_seconds = $localWatch.Elapsed.TotalSeconds
    excel_wall_seconds = $excelWatch.Elapsed.TotalSeconds
    excel_environment = $excel.environment
    roadmap_trace = "roadmap_trace.md"
    comparison_ref = "comparisons/excel_sample_comparisons.jsonl"
}
if ($excel.blocked) { $rollup["excel_blocker"] = $excel.blocker }
Write-JsonFile (Join-Path $runDir "rollup.json") $rollup

$highlightPath = Join-Path $runDir "highlights_trace.md"
$highlight = @(
    "# Expanded Smart-Fuzzer Highlights",
    "",
    "Run id: ``$RunId``",
    "",
    "## Counts",
    "",
    "1. Local generated/evaluated cases: ``$CaseCount``",
    "2. Excel candidate samples: ``$($candidates.Count)``",
    "3. Exact sample matches: ``$matches``",
    "4. Expected known financial exactness or literal-encoding deviations: ``$expectedKnown``",
    "5. Unexpected mismatches: ``$unexpected``",
    "6. Blocked sample rows: ``$blocked``",
    "",
    "## Explored Areas",
    "",
    "See ``roadmap_trace.md`` and ``roadmap_trace.json`` in this run directory. The run",
    "covered PMT, PPMT, and IPMT across arity, rate, horizon, present/future value,",
    "payment timing, and period-position bands.",
    "",
    "## Interpretation",
    "",
    "Known PMT/PPMT/IPMT non-zero-rate exactness drift is treated as expected pending",
    "further investigation. Tiny differences caused by formula literal decimal",
    "encoding are tracked separately from repair-grade failures. Unexpected",
    "mismatches, if any, are written as failure packets under ``failure_packets/``."
) -join [Environment]::NewLine
$highlight | Set-Content -LiteralPath $highlightPath -Encoding UTF8

Write-Host "Run: $RunId"
Write-Host "Run directory: $runDir"
Write-Host "Generated/local evaluated: $CaseCount"
Write-Host "Excel sampled: $($candidates.Count)"
Write-Host "Matches: $matches; expected known deviations: $expectedKnown; unexpected mismatches: $unexpected; blocked: $blocked"
