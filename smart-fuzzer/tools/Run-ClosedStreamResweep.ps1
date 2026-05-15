[CmdletBinding(PositionalBinding = $false)]
param(
    [string] $RepoRoot,
    [string] $RunId
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

if ([string]::IsNullOrWhiteSpace($RepoRoot)) {
    $scriptRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
    $RepoRoot = (Resolve-Path (Join-Path $scriptRoot "..\..")).Path
}
$RepoRoot = [System.IO.Path]::GetFullPath($RepoRoot)
if ([string]::IsNullOrWhiteSpace($RunId)) {
    $RunId = "W097-R-GH-closed-streams-cellref"
}

# W097 R-G / R-H: confirm closed BUG-FUNC-005 / -013 / -014 streams
# under cell-ref Excel input plumbing. Each witness goes through the
# shared CellRefBatch.psm1 module; OxFunc local results come from
# matrix_local_eval (which handles scalar, logical, and matrix args).
$cellRefModulePath = Join-Path $RepoRoot "smart-fuzzer\tools\CellRefBatch.psm1"
Import-Module $cellRefModulePath -Force

$runDir = Join-Path $RepoRoot ("smart-fuzzer\runs\" + $RunId)
$caseDir = Join-Path $runDir "cases"
$outcomeDir = Join-Path $runDir "outcomes"
$comparisonDir = Join-Path $runDir "comparisons"
New-Item -ItemType Directory -Force -Path $runDir, $caseDir, $outcomeDir, $comparisonDir | Out-Null

function Add-JsonLine {
    param([string] $Path, [object] $Value, [int] $Depth = 16)
    ($Value | ConvertTo-Json -Compress -Depth $Depth) | Add-Content -LiteralPath $Path -Encoding UTF8
}
function Write-JsonFile {
    param([string] $Path, [object] $Value, [int] $Depth = 16)
    $Value | ConvertTo-Json -Depth $Depth | Set-Content -LiteralPath $Path -Encoding UTF8
}

# Build the case set across the three closed streams.
$cases = New-Object System.Collections.ArrayList
$idx = 0
function Next-CaseId { $script:idx = $script:idx + 1; return ("CLR-{0:D4}" -f $script:idx) }

# -------- BUG-FUNC-005 (POWER zero-to-zero and adjacency) --------
[void]$cases.Add([ordered]@{ case_id = (Next-CaseId); stream = "BUG-FUNC-005"; function_id = "FUNC.POWER"; function_name = "POWER"; args_typed = @(@{kind="number";value=0.0}, @{kind="number";value=0.0}); expected_excel_text = "#NUM!" })
[void]$cases.Add([ordered]@{ case_id = (Next-CaseId); stream = "BUG-FUNC-005"; function_id = "FUNC.POWER"; function_name = "POWER"; args_typed = @(@{kind="number";value=0.0}, @{kind="number";value=-1.0}); expected_excel_text = "#DIV/0!" })
[void]$cases.Add([ordered]@{ case_id = (Next-CaseId); stream = "BUG-FUNC-005"; function_id = "FUNC.POWER"; function_name = "POWER"; args_typed = @(@{kind="number";value=0.0}, @{kind="number";value=1.0}); expected_excel_text = "0" })
[void]$cases.Add([ordered]@{ case_id = (Next-CaseId); stream = "BUG-FUNC-005"; function_id = "FUNC.POWER"; function_name = "POWER"; args_typed = @(@{kind="number";value=1.0}, @{kind="number";value=0.0}); expected_excel_text = "1" })

# -------- BUG-FUNC-013 (NORM.* exact-value accuracy) --------
[void]$cases.Add([ordered]@{ case_id = (Next-CaseId); stream = "BUG-FUNC-013"; function_id = "FUNC.NORM.DIST"; function_name = "NORM.DIST"; args_typed = @(@{kind="number";value=0.0}, @{kind="number";value=0.0}, @{kind="number";value=1.0}, @{kind="logical";value=$true}); expected_excel_text = "0.5" })
[void]$cases.Add([ordered]@{ case_id = (Next-CaseId); stream = "BUG-FUNC-013"; function_id = "FUNC.NORM.INV"; function_name = "NORM.INV"; args_typed = @(@{kind="number";value=0.975}, @{kind="number";value=0.0}, @{kind="number";value=1.0}); expected_excel_text = "1.9599639845400536" })
[void]$cases.Add([ordered]@{ case_id = (Next-CaseId); stream = "BUG-FUNC-013"; function_id = "FUNC.NORMSDIST"; function_name = "NORMSDIST"; args_typed = @(@{kind="number";value=0.0}); expected_excel_text = "0.5" })
[void]$cases.Add([ordered]@{ case_id = (Next-CaseId); stream = "BUG-FUNC-013"; function_id = "FUNC.NORMSINV"; function_name = "NORMSINV"; args_typed = @(@{kind="number";value=0.975}); expected_excel_text = "1.9599639845400536" })
[void]$cases.Add([ordered]@{ case_id = (Next-CaseId); stream = "BUG-FUNC-013"; function_id = "FUNC.NORM.S.DIST"; function_name = "NORM.S.DIST"; args_typed = @(@{kind="number";value=0.0}, @{kind="logical";value=$true}); expected_excel_text = "0.5" })
[void]$cases.Add([ordered]@{ case_id = (Next-CaseId); stream = "BUG-FUNC-013"; function_id = "FUNC.NORM.S.INV"; function_name = "NORM.S.INV"; args_typed = @(@{kind="number";value=0.975}); expected_excel_text = "1.9599639845400536" })
[void]$cases.Add([ordered]@{ case_id = (Next-CaseId); stream = "BUG-FUNC-013"; function_id = "FUNC.GAUSS"; function_name = "GAUSS"; args_typed = @(@{kind="number";value=1.0}); expected_excel_text = "Z.TEST tighten witness" })
[void]$cases.Add([ordered]@{ case_id = (Next-CaseId); stream = "BUG-FUNC-013"; function_id = "FUNC.PHI"; function_name = "PHI"; args_typed = @(@{kind="number";value=0.0}); expected_excel_text = "PHI(0)" })
[void]$cases.Add([ordered]@{ case_id = (Next-CaseId); stream = "BUG-FUNC-013"; function_id = "FUNC.ERF"; function_name = "ERF"; args_typed = @(@{kind="number";value=1.0}); expected_excel_text = "ERF(1)" })
[void]$cases.Add([ordered]@{ case_id = (Next-CaseId); stream = "BUG-FUNC-013"; function_id = "FUNC.ERFC"; function_name = "ERFC"; args_typed = @(@{kind="number";value=1.0}); expected_excel_text = "ERFC(1)" })

# -------- BUG-FUNC-014 (XIRR solver-precision) --------
[void]$cases.Add([ordered]@{
    case_id = (Next-CaseId); stream = "BUG-FUNC-014"; function_id = "FUNC.XIRR"; function_name = "XIRR"
    args_typed = @(
        @{kind="matrix"; rows=1; cols=5; values=@(-10000.0, 2750.0, 4250.0, 3250.0, 2750.0)},
        @{kind="matrix"; rows=1; cols=5; values=@(44927.0, 45108.0, 45292.0, 45473.0, 45658.0)}
    )
    expected_excel_text = "0.24449183344840997"
})

$casesPath = Join-Path $caseDir "closed-stream-cases.jsonl"
if (Test-Path -LiteralPath $casesPath) { Remove-Item -LiteralPath $casesPath }
foreach ($c in $cases) { Add-JsonLine $casesPath $c }
Write-Host "Generated $($cases.Count) closed-stream cases."

# Local OxFunc evaluation through matrix_local_eval (handles scalar / logical / matrix).
$localOutcomesPath = Join-Path $outcomeDir "local.jsonl"
if (Test-Path -LiteralPath $localOutcomesPath) { Remove-Item -LiteralPath $localOutcomesPath }
$exePath = Join-Path $RepoRoot "smart-fuzzer\tools\pmt_ppmt_local_eval\target\release\matrix_local_eval.exe"
if (-not (Test-Path -LiteralPath $exePath)) { throw "matrix_local_eval.exe not built: $exePath" }
$localWatch = [System.Diagnostics.Stopwatch]::StartNew()
& $exePath --cases $casesPath --out $localOutcomesPath
if ($LASTEXITCODE -ne 0) { throw "matrix_local_eval failed exit $LASTEXITCODE" }
$localWatch.Stop()

$localById = @{}
foreach ($line in Get-Content -LiteralPath $localOutcomesPath) {
    if ([string]::IsNullOrWhiteSpace($line)) { continue }
    $o = $line | ConvertFrom-Json
    # Closed-stream cases all have scalar outputs (1x1 result), so use cells[0].
    $localById[[string]$o.case_id] = $o.cells[0]
}

# Build cell-ref candidates (one per case).
$cellRefCandidates = @()
foreach ($c in $cases) {
    $argList = @()
    foreach ($a in @($c.args_typed)) {
        switch ($a.kind) {
            "number"  { $argList += [double]$a.value; break }
            "logical" { $argList += [bool]$a.value; break }
            "matrix"  {
                $argList += @{ kind = "matrix"; rows = [int]$a.rows; cols = [int]$a.cols; values = @($a.values | ForEach-Object { [double]$_ }) }
                break
            }
            default   { throw "unknown args_typed kind: $($a.kind)" }
        }
    }
    $cellRefCandidates += @{ function_name = [string]$c.function_name; args = $argList }
}

$excelWatch = [System.Diagnostics.Stopwatch]::StartNew()
$excel = Invoke-ExcelCellRefBatch -Candidates $cellRefCandidates
$excelWatch.Stop()

# Comparison
$comparisonsPath = Join-Path $comparisonDir "excel_sample_comparisons.jsonl"
if (Test-Path -LiteralPath $comparisonsPath) { Remove-Item -LiteralPath $comparisonsPath }
$matches = 0; $drifts = 0; $kindDrift = 0; $blocked = 0
$perStream = @{}
for ($i = 0; $i -lt $cases.Count; $i++) {
    $case = $cases[$i]
    $local = $localById[[string]$case.case_id]
    $excelOutcome = if ($excel.outcomes.Count -gt $i) { $excel.outcomes[$i] } else { $null }
    $result = "blocked"; $absDelta = $null; $ulp = $null
    if ($null -eq $local -or $null -eq $excelOutcome) { $blocked += 1 }
    elseif ([string]$local.kind -eq "number" -and [string]$excelOutcome.kind -eq "number") {
        $lv = [double]$local.value; $ev = [double]$excelOutcome.value
        $absDelta = [Math]::Abs($lv - $ev)
        if ([string]$local.bits_hex -eq [string]$excelOutcome.bits_hex) {
            $matches += 1; $result = "exact_typed_bit_match"
        } elseif ($lv -eq 0.0 -and $ev -eq 0.0) {
            $matches += 1; $result = "match_signed_zero"
        } else {
            $ulp = Get-UlpDistance $lv $ev
            $drifts += 1; $result = "regression_or_drift"
        }
    } elseif ([string]$local.kind -eq "error" -and [string]$excelOutcome.kind -eq "error") {
        if ([string]$local.error_code -eq [string]$excelOutcome.code) {
            $matches += 1; $result = "exact_typed_bit_match"
        } else {
            $kindDrift += 1; $result = "error_code_mismatch"
        }
    } else {
        $kindDrift += 1; $result = "kind_drift"
    }

    if (-not $perStream.ContainsKey($case.stream)) { $perStream[$case.stream] = @{ total = 0; match = 0; drift = 0; kind = 0; blocked = 0 } }
    $perStream[$case.stream].total += 1
    if ($result -eq "exact_typed_bit_match" -or $result -eq "match_signed_zero") { $perStream[$case.stream].match += 1 }
    elseif ($result -eq "regression_or_drift") { $perStream[$case.stream].drift += 1 }
    elseif ($result -eq "kind_drift" -or $result -eq "error_code_mismatch") { $perStream[$case.stream].kind += 1 }
    else { $perStream[$case.stream].blocked += 1 }

    Add-JsonLine $comparisonsPath ([ordered]@{
        schema_version = "oxfunc.smart_fuzzer.closed_stream_excel_comparison.v0"
        case_id = [string]$case.case_id
        stream = [string]$case.stream
        function_id = [string]$case.function_id
        function_name = [string]$case.function_name
        comparison_result = $result
        abs_delta = $absDelta
        ulp_distance = $ulp
        local_outcome = $local
        excel_outcome = $excelOutcome
        expected_excel_text = [string]$case.expected_excel_text
    })
}

Write-JsonFile (Join-Path $runDir "rollup.json") ([ordered]@{
    schema_version = "oxfunc.smart_fuzzer.closed_stream_run_rollup.v0"
    run_id = $RunId
    cases = $cases.Count
    matches = $matches
    drifts = $drifts
    kind_drift = $kindDrift
    blocked = $blocked
    excel_input_plumbing = "cell_value2"
    per_stream = $perStream
    excel_environment = $excel.environment
    local_wall_seconds = $localWatch.Elapsed.TotalSeconds
    excel_wall_seconds = $excelWatch.Elapsed.TotalSeconds
})

Write-Host "Run: $RunId"
Write-Host ("  Total: {0} | matches: {1} | drifts: {2} | kind drift: {3} | blocked: {4}" -f $cases.Count, $matches, $drifts, $kindDrift, $blocked)
foreach ($k in $perStream.Keys) {
    $s = $perStream[$k]
    Write-Host ("  {0,-15}: total={1,-3} match={2,-3} drift={3,-3} kind={4,-3} blocked={5,-3}" -f $k, $s.total, $s.match, $s.drift, $s.kind, $s.blocked)
}
