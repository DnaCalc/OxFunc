[CmdletBinding(PositionalBinding = $false)]
param(
    [string] $RepoRoot,
    [string] $RunId,
    [int] $CaseCount = 5000000,
    [int] $Seed = 17,
    [int] $CandidateLimit = 800
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

if ([string]::IsNullOrWhiteSpace($RepoRoot)) {
    $scriptRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
    $RepoRoot = (Resolve-Path (Join-Path $scriptRoot "..\..")).Path
}
$RepoRoot = [System.IO.Path]::GetFullPath($RepoRoot)
if ([string]::IsNullOrWhiteSpace($RunId)) {
    $RunId = (Get-Date).ToUniversalTime().ToString("yyyyMMddTHHmmssZ") + "-broad-scalar"
}

$runDir = Join-Path $RepoRoot ("smart-fuzzer\runs\" + $RunId)
$comparisonDir = Join-Path $runDir "comparisons"
$failureDir = Join-Path $runDir "failure_packets"
$logDir = Join-Path $runDir "logs"
New-Item -ItemType Directory -Force -Path $runDir, $comparisonDir, $failureDir, $logDir | Out-Null

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
function Get-MaxCandidateArity {
    param([object[]] $Candidates)
    $maxArity = 0
    foreach ($c in $Candidates) {
        $n = if ($null -eq $c.args) { 0 } else { @($c.args).Count }
        if ($n -gt $maxArity) { $maxArity = $n }
    }
    return [Math]::Max(1, $maxArity)
}
function Get-CandidateColumnLetter {
    param([int] $ZeroBasedColIndex)
    # 0..25 -> A..Z; 26..51 -> AA..AZ. We never go past ~10 args here.
    if ($ZeroBasedColIndex -lt 26) {
        return [char]([byte][char]'A' + $ZeroBasedColIndex)
    }
    $hi = [int]([Math]::Floor($ZeroBasedColIndex / 26)) - 1
    $lo = $ZeroBasedColIndex % 26
    return ([char]([byte][char]'A' + $hi)) + ([char]([byte][char]'A' + $lo))
}
function New-ArgValueArray {
    param([object[]] $Candidates, [int] $MaxArity)
    # Column C..(C+MaxArity-1). Cells holding raw f64 doubles via Value2.
    $array = New-Object "object[,]" $Candidates.Count, $MaxArity
    for ($row = 0; $row -lt $Candidates.Count; $row++) {
        $args = @($Candidates[$row].args)
        for ($col = 0; $col -lt $MaxArity; $col++) {
            if ($col -lt $args.Count) {
                # Cast through [double] to ensure VT_R8 marshalling and bit-exact storage.
                $array[$row, $col] = [double] $args[$col]
            } else {
                # Padding cells stay empty so trailing args do not leak.
                $array[$row, $col] = $null
            }
        }
    }
    return ,$array
}
function New-CellRefFormulaArray {
    param([object[]] $Candidates, [int] $MaxArity, [int] $ArgStartColZeroIndexed)
    # Build formulas of the form "=FNAME(<col1><row>, <col2><row>, ...)" referencing
    # the per-row argument cells. For variadic shapes the candidate.args length wins.
    $array = New-Object "object[,]" $Candidates.Count, 1
    for ($row = 0; $row -lt $Candidates.Count; $row++) {
        $excelRow = $row + 1
        $cand = $Candidates[$row]
        $args = @($cand.args)
        $name = [string] $cand.function_name
        $refs = @()
        for ($col = 0; $col -lt $args.Count; $col++) {
            $letter = Get-CandidateColumnLetter ($ArgStartColZeroIndexed + $col)
            $refs += "$letter$excelRow"
        }
        $array[$row, 0] = "=$name($($refs -join ','))"
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
    # Cell-ref plumbing variant. Numeric arguments are written to cells C..(C+maxArity-1)
    # via Range.Value2 batch (bit-exact f64 round-trip), then the formula in column A
    # references those cells. This eliminates the formula-literal-text encoding-drift
    # class. See smart-fuzzer/planning/EXCEL_RUNNER_PLUMBING_NOTE.md for the full rule.
    param([object[]] $Candidates)
    $excel = $null
    $workbook = $null
    $worksheet = $null
    $formulaRange = $null
    $errorRange = $null
    $argRange = $null
    try {
        $excel = New-Object -ComObject Excel.Application
        try { $excel.Visible = $false } catch {}
        try { $excel.DisplayAlerts = $false } catch {}
        try { $excel.ScreenUpdating = $false } catch {}
        try { $excel.EnableEvents = $false } catch {}
        $workbook = $excel.Workbooks.Add()
        $worksheet = $workbook.Worksheets.Item(1)
        try { $worksheet.Columns.Item(1).ColumnWidth = 36 } catch {}

        $maxArity = Get-MaxCandidateArity $Candidates
        $argStartColZeroIndexed = 2  # A=formula, B=errorType, C..=args

        # Step 1: write input doubles via Value2 (exact f64 round-trip).
        $argAnchor = $worksheet.Range((Get-CandidateColumnLetter $argStartColZeroIndexed) + "1")
        $argRange = $argAnchor.Resize($Candidates.Count, $maxArity)
        Release-ComObject $argAnchor
        $argRange.Value2 = New-ArgValueArray $Candidates $maxArity

        # Step 2: write formulas referencing those cells, plus error-type companion.
        $formulaAnchor = $worksheet.Range("A1")
        $formulaRange = $formulaAnchor.Resize($Candidates.Count, 1)
        Release-ComObject $formulaAnchor
        $errorAnchor = $worksheet.Range("B1")
        $errorRange = $errorAnchor.Resize($Candidates.Count, 1)
        Release-ComObject $errorAnchor
        $formulaRange.Formula2 = New-CellRefFormulaArray $Candidates $maxArity $argStartColZeroIndexed
        $errorRange.Formula2 = New-ErrorTypeFormulaArray $Candidates.Count

        # Step 3: calculate the formula columns. The arg cells are static values.
        $calcAnchor = $worksheet.Range("A1")
        $calcRange = $calcAnchor.Resize($Candidates.Count, 2)
        Release-ComObject $calcAnchor
        [void] $calcRange.Calculate()
        Release-ComObject $calcRange

        $values = $formulaRange.Value2
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
                excel_input_plumbing = "cell_value2"
            }
        }
    }
    catch {
        return [ordered]@{ blocked = $true; blocker = $_.Exception.Message; outcomes = @(); environment = @{} }
    }
    finally {
        Release-ComObject $formulaRange
        Release-ComObject $errorRange
        Release-ComObject $argRange
        if ($null -ne $workbook) { try { $workbook.Close($false) } catch {} }
        if ($null -ne $excel) { try { $excel.Quit() } catch {} }
        Release-ComObject $worksheet
        Release-ComObject $workbook
        Release-ComObject $excel
        [System.GC]::Collect()
        [System.GC]::WaitForPendingFinalizers()
    }
}

# Statistical-exactness functions covered by BUG-FUNC-021 etc.
$KnownResidualFunctions = New-Object 'System.Collections.Generic.HashSet[string]'
@(
    "FUNC.BETADIST","FUNC.BETAINV","FUNC.CHIDIST","FUNC.CHIINV",
    "FUNC.FDIST","FUNC.FINV","FUNC.GAMMADIST","FUNC.GAMMAINV",
    "FUNC.HYPGEOMDIST","FUNC.NEGBINOMDIST","FUNC.NORMSDIST","FUNC.NORMSINV",
    "FUNC.TDIST","FUNC.TINV","FUNC.PERCENTRANK","FUNC.CONFIDENCE.T","FUNC.Z.TEST",
    "FUNC.BESSELY","FUNC.MINVERSE","FUNC.PMT","FUNC.PPMT","FUNC.IPMT"
) | ForEach-Object { [void]$KnownResidualFunctions.Add($_) }

# Approx ULP distance for two finite f64 values. Uses the standard
# sign-magnitude-to-biased mapping over [Int64.MinValue, Int64.MaxValue].
function Get-UlpDistance {
    param([double] $A, [double] $B)
    if ([double]::IsNaN($A) -or [double]::IsNaN($B)) { return [double]::PositiveInfinity }
    if ([double]::IsInfinity($A) -or [double]::IsInfinity($B)) { return [double]::PositiveInfinity }
    $aBits = [System.BitConverter]::DoubleToInt64Bits($A)
    $bBits = [System.BitConverter]::DoubleToInt64Bits($B)
    # Map signed-magnitude bit pattern to a monotone biased Int64 ordering.
    if ($aBits -lt 0) { $aBits = [int64]9223372036854775807 - $aBits }
    else { $aBits = $aBits - [int64]0 }
    if ($bBits -lt 0) { $bBits = [int64]9223372036854775807 - $bBits }
    else { $bBits = $bBits - [int64]0 }
    $diff = $aBits - $bBits
    if ($diff -lt 0) { $diff = -$diff }
    return [double] $diff
}

$repoRootForRust = $RepoRoot
$exeRel = "smart-fuzzer\tools\pmt_ppmt_local_eval\target\release\broad_scalar_explorer.exe"
$exePath = Join-Path $repoRootForRust $exeRel

$localWatch = [System.Diagnostics.Stopwatch]::StartNew()
& $exePath --run-dir $runDir --cases $CaseCount --seed $Seed --candidate-limit $CandidateLimit
if ($LASTEXITCODE -ne 0) {
    throw "broad_scalar_explorer failed with exit code $LASTEXITCODE"
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
$ulpResidualKnown = 0
$ulpResidualUnknown = 0
$unexpected = 0
$blocked = 0

for ($i = 0; $i -lt $candidates.Count; $i++) {
    $candidate = $candidates[$i]
    $local = $candidate.local_outcome
    $excelOutcome = if ($excel.outcomes.Count -gt $i) { $excel.outcomes[$i] } else { $null }
    $result = "blocked"
    $absDelta = $null
    $ulp = $null
    if ($null -eq $excelOutcome) {
        $blocked += 1
    }
    elseif ([string]$local.kind -eq "number" -and [string]$excelOutcome.kind -eq "number") {
        $lv = [double]$local.value
        $ev = [double]$excelOutcome.value
        $absDelta = [Math]::Abs($lv - $ev)
        if ([string]$local.bits_hex -eq [string]$excelOutcome.bits_hex) {
            $matches += 1
            $result = "exact_typed_bit_match"
        }
        elseif ($lv -eq 0.0 -and $ev -eq 0.0) {
            $matches += 1
            $result = "match_signed_zero"
        }
        else {
            # Inputs are bit-exact via cell_value2 plumbing, so any numeric
            # difference here is a kernel-side drift, not an encoding artefact.
            $ulp = Get-UlpDistance $lv $ev
            $isKnownStatFamily = $KnownResidualFunctions.Contains([string]$candidate.function_id)
            if ($isKnownStatFamily) {
                $ulpResidualKnown += 1
                $result = "known_residual_numeric_drift"
            }
            else {
                $unexpected += 1
                $result = "unexpected_mismatch"
            }
        }
    }
    elseif ([string]$local.kind -eq [string]$excelOutcome.kind -and [string]$local.digest_payload -eq [string]$excelOutcome.digest_payload) {
        $matches += 1
        $result = "exact_typed_bit_match"
    }
    elseif ([string]$local.kind -eq "error" -and [string]$excelOutcome.kind -eq "error") {
        if ([string]$local.code -eq [string]$excelOutcome.code) {
            $matches += 1
            $result = "exact_typed_bit_match"
        } else {
            $unexpected += 1
            $result = "unexpected_error_code_drift"
        }
    }
    else {
        $unexpected += 1
        $result = "unexpected_kind_drift"
    }

    $comparison = [ordered]@{
        schema_version = "oxfunc.smart_fuzzer.broad_scalar_excel_comparison.v0"
        case_id = [string] $candidate.case_id
        function_id = [string] $candidate.function_id
        function_name = [string] $candidate.function_name
        formula_text = [string] $candidate.formula_text
        comparison_result = $result
        abs_delta = $absDelta
        ulp_distance = $ulp
        coverage_buckets = $candidate.coverage_buckets
        selection_reason = [string] $candidate.selection_reason
        local_outcome = $local
        excel_outcome = $excelOutcome
    }
    Add-JsonLine $comparisonsPath $comparison
    if ($result -eq "unexpected_mismatch" -or $result -eq "unexpected_error_code_drift" -or $result -eq "unexpected_kind_drift") {
        Write-JsonFile (Join-Path $failureDir ([string]$candidate.case_id + ".json")) $comparison
    }
}

$localRollup = Get-Content -LiteralPath (Join-Path $runDir "local_rollup.json") -Raw | ConvertFrom-Json
$rollup = [ordered]@{
    schema_version = "oxfunc.smart_fuzzer.broad_scalar_run_rollup.v0"
    run_id = $RunId
    generated = $CaseCount
    local_evaluated = $CaseCount
    excel_sampled = $candidates.Count
    excel_blocked = [bool] $excel.blocked
    excel_input_plumbing = "cell_value2"
    matches = $matches
    expected_known_deviations = $ulpResidualKnown
    unexpected_mismatches = $unexpected
    blocked = $blocked
    local_cases_per_second = $localRollup.local_cases_per_second
    local_wall_seconds = $localRollup.local_wall_seconds
    wrapper_local_wall_seconds = $localWatch.Elapsed.TotalSeconds
    excel_wall_seconds = $excelWatch.Elapsed.TotalSeconds
    excel_environment = $excel.environment
    comparison_ref = "comparisons/excel_sample_comparisons.jsonl"
}
if ($excel.blocked) { $rollup["excel_blocker"] = $excel.blocker }
Write-JsonFile (Join-Path $runDir "rollup.json") $rollup

Write-Host "Run: $RunId"
Write-Host "Run directory: $runDir"
Write-Host "Generated/local evaluated: $CaseCount"
Write-Host "Excel sampled: $($candidates.Count)"
Write-Host "Matches: $matches; known stat residuals: $ulpResidualKnown; unexpected: $unexpected; blocked: $blocked"
