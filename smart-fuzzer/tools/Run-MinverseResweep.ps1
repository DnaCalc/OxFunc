[CmdletBinding(PositionalBinding = $false)]
param(
    [string] $RepoRoot,
    [string] $RunId,
    [int] $RandomSeed = 17
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

if ([string]::IsNullOrWhiteSpace($RepoRoot)) {
    $scriptRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
    $RepoRoot = (Resolve-Path (Join-Path $scriptRoot "..\..")).Path
}
$RepoRoot = [System.IO.Path]::GetFullPath($RepoRoot)
if ([string]::IsNullOrWhiteSpace($RunId)) {
    $RunId = "W097-R-F-minverse-cellref"
}

# W097 R-F: MINVERSE cell-ref re-replay over 2x2/3x3/4x4 random and
# structured matrices. The local OxFunc evaluator emits per-cell
# outcomes; the cell-ref Excel comparator wraps each cell in
# INDEX(MINVERSE(<range>), r, c) so the bit-exact comparison is
# scalar-by-scalar.
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

# Deterministic pseudo-random for matrix entries.
# Use .NET System.Random with a fixed seed to keep generation
# reproducible without manual modular arithmetic (PowerShell uint64
# does not wrap on overflow).
$script:rng = [System.Random]::new([int]$RandomSeed)
function Get-NextRandomDouble {
    return [double]$script:rng.NextDouble()
}

function New-MatrixCase {
    param([string] $CaseId, [int] $N, [string] $Kind, [double[]] $Values)
    return [ordered]@{
        case_id = $CaseId
        function_id = "FUNC.MINVERSE"
        function_name = "MINVERSE"
        n = $N
        kind = $Kind
        args_typed = @(@{ kind = "matrix"; rows = $N; cols = $N; values = @($Values) })
    }
}

function New-RandomMatrix {
    param([int] $N)
    $vals = @()
    for ($i = 0; $i -lt $N * $N; $i++) {
        $v = (Get-NextRandomDouble) * 18.0 - 9.0   # [-9, +9]
        $vals += [Math]::Round($v, 4)
    }
    return ,@($vals)
}

function New-DiagonallyDominantMatrix {
    param([int] $N)
    $vals = New-Object 'double[]' ($N * $N)
    for ($i = 0; $i -lt $N; $i++) {
        for ($j = 0; $j -lt $N; $j++) {
            if ($i -eq $j) { $vals[$i * $N + $j] = ($N + 1.0) + (Get-NextRandomDouble) }
            else { $vals[$i * $N + $j] = (Get-NextRandomDouble) - 0.5 }
        }
    }
    return ,@([double[]]$vals)
}

# Build the case set: structured + random matrices for n=2,3,4.
$cases = New-Object System.Collections.ArrayList
$caseIdx = 0
function Next-CaseId { $script:caseIdx = $script:caseIdx + 1; return ("MIV-{0:D4}" -f $script:caseIdx) }

# Witness from BUG-FUNC-025: =MINVERSE({1,2;3,4})
[void]$cases.Add((New-MatrixCase (Next-CaseId) 2 "witness_BUG_FUNC_025" @(1.0, 2.0, 3.0, 4.0)))

# 2x2 fixed structured cases
[void]$cases.Add((New-MatrixCase (Next-CaseId) 2 "identity"        @(1.0, 0.0, 0.0, 1.0)))
[void]$cases.Add((New-MatrixCase (Next-CaseId) 2 "diag_2_3"        @(2.0, 0.0, 0.0, 3.0)))
[void]$cases.Add((New-MatrixCase (Next-CaseId) 2 "rotation_45"     @(0.7071067811865476, -0.7071067811865475, 0.7071067811865475, 0.7071067811865476)))
[void]$cases.Add((New-MatrixCase (Next-CaseId) 2 "near_singular"   @(1.0, 1.0, 1.0, 1.0000001)))
[void]$cases.Add((New-MatrixCase (Next-CaseId) 2 "negative_offdiag" @(2.0, -1.0, -1.0, 2.0)))

# 2x2 random
for ($i = 0; $i -lt 8; $i++) {
    [void]$cases.Add((New-MatrixCase (Next-CaseId) 2 "random" (New-RandomMatrix 2)))
}

# 3x3 structured
[void]$cases.Add((New-MatrixCase (Next-CaseId) 3 "identity_3"      @(1.0,0.0,0.0, 0.0,1.0,0.0, 0.0,0.0,1.0)))
[void]$cases.Add((New-MatrixCase (Next-CaseId) 3 "diag_2_3_5"      @(2.0,0.0,0.0, 0.0,3.0,0.0, 0.0,0.0,5.0)))
[void]$cases.Add((New-MatrixCase (Next-CaseId) 3 "tridiag"         @(2.0,-1.0,0.0, -1.0,2.0,-1.0, 0.0,-1.0,2.0)))
$h3_third = 1.0 / 3.0
$hilbert3Values = @(
    1.0,        0.5,        $h3_third,
    0.5,        $h3_third,  0.25,
    $h3_third,  0.25,       0.2
)
[void]$cases.Add((New-MatrixCase (Next-CaseId) 3 "hilbert_3" $hilbert3Values))

# 3x3 random and diagonally-dominant
for ($i = 0; $i -lt 8; $i++) {
    [void]$cases.Add((New-MatrixCase (Next-CaseId) 3 "random" (New-RandomMatrix 3)))
}
for ($i = 0; $i -lt 4; $i++) {
    [void]$cases.Add((New-MatrixCase (Next-CaseId) 3 "diag_dominant" (New-DiagonallyDominantMatrix 3)))
}

# 4x4 structured
[void]$cases.Add((New-MatrixCase (Next-CaseId) 4 "identity_4" @(1,0,0,0, 0,1,0,0, 0,0,1,0, 0,0,0,1)))
[void]$cases.Add((New-MatrixCase (Next-CaseId) 4 "diag_4"     @(2,0,0,0, 0,3,0,0, 0,0,5,0, 0,0,0,7)))
# 4x4 hilbert (precompute reciprocals to avoid PS infix division parse weirdness)
$h_third = 1.0 / 3.0
$h_fifth = 0.2
$h_sixth = 1.0 / 6.0
$h_seventh = 1.0 / 7.0
$hilbert4Values = @(
    1.0,        0.5,        $h_third,   0.25,
    0.5,        $h_third,   0.25,       $h_fifth,
    $h_third,   0.25,       $h_fifth,   $h_sixth,
    0.25,       $h_fifth,   $h_sixth,   $h_seventh
)
[void]$cases.Add((New-MatrixCase (Next-CaseId) 4 "hilbert_4" $hilbert4Values))

# 4x4 random and diagonally-dominant
for ($i = 0; $i -lt 8; $i++) {
    [void]$cases.Add((New-MatrixCase (Next-CaseId) 4 "random" (New-RandomMatrix 4)))
}
for ($i = 0; $i -lt 4; $i++) {
    [void]$cases.Add((New-MatrixCase (Next-CaseId) 4 "diag_dominant" (New-DiagonallyDominantMatrix 4)))
}

$casesPath = Join-Path $caseDir "minverse-cases.jsonl"
if (Test-Path -LiteralPath $casesPath) { Remove-Item -LiteralPath $casesPath }
foreach ($c in $cases) { Add-JsonLine $casesPath $c }

Write-Host "Generated $($cases.Count) MINVERSE cases."

# Local eval through matrix_local_eval. Returns per-cell outcomes.
$localOutcomesPath = Join-Path $outcomeDir "local.jsonl"
if (Test-Path -LiteralPath $localOutcomesPath) { Remove-Item -LiteralPath $localOutcomesPath }
$exePath = Join-Path $RepoRoot "smart-fuzzer\tools\pmt_ppmt_local_eval\target\release\matrix_local_eval.exe"
if (-not (Test-Path -LiteralPath $exePath)) { throw "matrix_local_eval.exe not built: $exePath" }
$localWatch = [System.Diagnostics.Stopwatch]::StartNew()
& $exePath --cases $casesPath --out $localOutcomesPath
if ($LASTEXITCODE -ne 0) { throw "matrix_local_eval failed exit $LASTEXITCODE" }
$localWatch.Stop()

# Index per-case local outcome by case_id, then per-cell by (row,col).
$localById = @{}
foreach ($line in Get-Content -LiteralPath $localOutcomesPath) {
    if ([string]::IsNullOrWhiteSpace($line)) { continue }
    $o = $line | ConvertFrom-Json
    $cellMap = @{}
    foreach ($cell in @($o.cells)) {
        $cellMap[("{0}_{1}" -f $cell.row, $cell.col)] = $cell
    }
    $localById[[string]$o.case_id] = @{ rows = $o.rows; cols = $o.cols; cells = $cellMap }
}

# For Excel: build one cell-ref candidate per (case, row, col) cell of
# the result matrix. Each candidate is INDEX(MINVERSE(<matrix>), r, c).
$cellRefCandidates = @()
$cellRefIndex = @()  # parallel array: each entry maps back to (case_id, row, col)
foreach ($c in $cases) {
    $local = $localById[[string]$c.case_id]
    if ($null -eq $local) { continue }
    for ($r = 0; $r -lt $local.rows; $r++) {
        for ($cc = 0; $cc -lt $local.cols; $cc++) {
            $cellRefCandidates += @{
                function_name = [string]$c.function_name
                args = @(@{ kind = "matrix"; rows = $c.args_typed[0].rows; cols = $c.args_typed[0].cols; values = @($c.args_typed[0].values) })
                result_index = @(($r + 1), ($cc + 1))
            }
            $cellRefIndex += @{ case_id = [string]$c.case_id; row = $r; col = $cc }
        }
    }
}

Write-Host "Built $($cellRefCandidates.Count) per-cell Excel comparison candidates. Running cell-ref batch..."
$excelWatch = [System.Diagnostics.Stopwatch]::StartNew()
$excel = Invoke-ExcelCellRefBatch -Candidates $cellRefCandidates
$excelWatch.Stop()

# Compare per-cell.
$comparisonsPath = Join-Path $comparisonDir "excel_sample_comparisons.jsonl"
if (Test-Path -LiteralPath $comparisonsPath) { Remove-Item -LiteralPath $comparisonsPath }

$matches = 0
$drifts = 0
$kindDrift = 0
$blocked = 0
$perCaseStats = @{}

for ($i = 0; $i -lt $cellRefCandidates.Count; $i++) {
    $idx = $cellRefIndex[$i]
    $local = $localById[$idx.case_id]
    $localCell = $local.cells[("{0}_{1}" -f $idx.row, $idx.col)]
    $excelOutcome = if ($excel.outcomes.Count -gt $i) { $excel.outcomes[$i] } else { $null }
    $result = "blocked"; $absDelta = $null; $ulp = $null
    if ($null -eq $excelOutcome) { $blocked += 1 }
    elseif ([string]$localCell.kind -eq "number" -and [string]$excelOutcome.kind -eq "number") {
        $lv = [double]$localCell.value; $ev = [double]$excelOutcome.value
        $absDelta = [Math]::Abs($lv - $ev)
        if ([string]$localCell.bits_hex -eq [string]$excelOutcome.bits_hex) {
            $matches += 1; $result = "exact_typed_bit_match"
        } elseif ($lv -eq 0.0 -and $ev -eq 0.0) {
            $matches += 1; $result = "match_signed_zero"
        } else {
            $ulp = Get-UlpDistance $lv $ev
            $drifts += 1; $result = "known_residual_minverse_drift"
        }
    } elseif ([string]$localCell.digest_payload -eq [string]$excelOutcome.digest_payload) {
        $matches += 1; $result = "exact_typed_bit_match"
    } else {
        $kindDrift += 1; $result = "kind_drift"
    }

    if (-not $perCaseStats.ContainsKey($idx.case_id)) { $perCaseStats[$idx.case_id] = @{ match = 0; drift = 0; kind = 0; blocked = 0; ulp_max = 0.0 } }
    $stats = $perCaseStats[$idx.case_id]
    if ($result -eq "exact_typed_bit_match" -or $result -eq "match_signed_zero") { $stats.match += 1 }
    elseif ($result -eq "known_residual_minverse_drift") {
        $stats.drift += 1
        if ($null -ne $ulp -and -not [double]::IsInfinity($ulp) -and $ulp -gt $stats.ulp_max) { $stats.ulp_max = $ulp }
    } elseif ($result -eq "kind_drift") { $stats.kind += 1 }
    else { $stats.blocked += 1 }

    Add-JsonLine $comparisonsPath ([ordered]@{
        schema_version = "oxfunc.smart_fuzzer.minverse_excel_comparison.v0"
        case_id = [string]$idx.case_id
        cell_row = [int]$idx.row
        cell_col = [int]$idx.col
        comparison_result = $result
        abs_delta = $absDelta
        ulp_distance = $ulp
        local_outcome = $localCell
        excel_outcome = $excelOutcome
    })
}

Write-JsonFile (Join-Path $runDir "rollup.json") ([ordered]@{
    schema_version = "oxfunc.smart_fuzzer.minverse_run_rollup.v0"
    run_id = $RunId
    cases = $cases.Count
    total_cells_compared = $cellRefCandidates.Count
    matches = $matches
    drifts = $drifts
    kind_drift = $kindDrift
    blocked = $blocked
    excel_input_plumbing = "cell_value2_matrix"
    excel_environment = $excel.environment
    local_wall_seconds = $localWatch.Elapsed.TotalSeconds
    excel_wall_seconds = $excelWatch.Elapsed.TotalSeconds
})

# Per-case summary log to make scanning easier.
$perCaseSummaryPath = Join-Path $runDir "per_case_summary.md"
$lines = @("# W097 R-F per-case MINVERSE summary", "")
$lines += "| Case ID | n | Kind | match | drift | kind_drift | max ULP |"
$lines += "| --- | --- | --- | ---: | ---: | ---: | ---: |"
foreach ($c in $cases) {
    $s = $perCaseStats[[string]$c.case_id]
    $lines += ("| {0} | {1} | {2} | {3} | {4} | {5} | {6:E2} |" -f $c.case_id, $c.n, $c.kind, $s.match, $s.drift, $s.kind, $s.ulp_max)
}
Set-Content -LiteralPath $perCaseSummaryPath -Encoding UTF8 -Value ($lines -join [Environment]::NewLine)

Write-Host "Run: $RunId; cases: $($cases.Count); cells: $($cellRefCandidates.Count); matches: $matches; drifts: $drifts; kind drift: $kindDrift; blocked: $blocked"
