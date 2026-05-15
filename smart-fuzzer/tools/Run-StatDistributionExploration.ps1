[CmdletBinding(PositionalBinding = $false)]
param(
    [string] $RepoRoot,
    [string] $RunId,
    [int] $CaseCount = 1000000,
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
    $RunId = (Get-Date).ToUniversalTime().ToString("yyyyMMddTHHmmssZ") + "-stat-distribution"
}

# W097 R-D: cell-ref Excel comparator for the BUG-FUNC-021 statistical
# distribution surface. Local evaluation comes from the
# stat_distribution_explorer Rust binary; Excel comparison goes through
# the shared CellRefBatch.psm1 module so numeric inputs are bit-exact.
$cellRefModulePath = Join-Path $RepoRoot "smart-fuzzer\tools\CellRefBatch.psm1"
Import-Module $cellRefModulePath -Force

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

$repoRootForRust = $RepoRoot
$exeRel = "smart-fuzzer\tools\pmt_ppmt_local_eval\target\release\stat_distribution_explorer.exe"
$exePath = Join-Path $repoRootForRust $exeRel
if (-not (Test-Path -LiteralPath $exePath)) {
    throw "stat_distribution_explorer.exe not built: $exePath. Run: cargo build --release --manifest-path smart-fuzzer\tools\pmt_ppmt_local_eval\Cargo.toml --bin stat_distribution_explorer"
}

$localWatch = [System.Diagnostics.Stopwatch]::StartNew()
& $exePath --run-dir $runDir --cases $CaseCount --seed $Seed --candidate-limit $CandidateLimit
if ($LASTEXITCODE -ne 0) {
    throw "stat_distribution_explorer failed with exit code $LASTEXITCODE"
}
$localWatch.Stop()

$candidatePath = Join-Path $runDir "candidates\excel_candidates.jsonl"
$candidates = @(Get-Content -LiteralPath $candidatePath | Where-Object { -not [string]::IsNullOrWhiteSpace($_) } | ForEach-Object { $_ | ConvertFrom-Json })

# Convert each Rust-emitted candidate to a CellRefBatch input.
# args_typed: list of {kind, value/values, rows, cols}.
function ConvertTo-CellRefArgs {
    param([object] $Candidate)
    $list = @()
    foreach ($a in @($Candidate.args_typed)) {
        switch ($a.kind) {
            "number"  { $list += [double]$a.value; break }
            "logical" { $list += [bool]$a.value; break }
            "matrix"  {
                $list += @{ kind = "matrix"; rows = [int]$a.rows; cols = [int]$a.cols; values = @($a.values | ForEach-Object { [double]$_ }) }
                break
            }
            default   { throw "unknown args_typed kind: $($a.kind)" }
        }
    }
    return ,$list
}

$cellRefCandidates = @()
foreach ($c in $candidates) {
    $cellRefCandidates += @{
        function_name = [string] $c.function_name
        args = (ConvertTo-CellRefArgs $c)
    }
}

$excelWatch = [System.Diagnostics.Stopwatch]::StartNew()
$excel = Invoke-ExcelCellRefBatch -Candidates $cellRefCandidates
$excelWatch.Stop()

$comparisonsPath = Join-Path $comparisonDir "excel_sample_comparisons.jsonl"
if (Test-Path -LiteralPath $comparisonsPath) { Remove-Item -LiteralPath $comparisonsPath }

$matches = 0
$knownStat = 0
$unexpected = 0
$blocked = 0

# Functions covered by BUG-FUNC-021 for the "known residual" classification.
$bugFunc021Set = New-Object 'System.Collections.Generic.HashSet[string]'
@(
    "FUNC.BETADIST","FUNC.BETAINV","FUNC.CHIDIST","FUNC.CHIINV",
    "FUNC.FDIST","FUNC.FINV","FUNC.GAMMADIST","FUNC.GAMMAINV",
    "FUNC.HYPGEOMDIST","FUNC.NEGBINOMDIST","FUNC.NORMSDIST","FUNC.NORMSINV",
    "FUNC.TDIST","FUNC.TINV","FUNC.PERCENTRANK","FUNC.CONFIDENCE.T","FUNC.Z.TEST",
    "FUNC.KURT","FUNC.SKEW","FUNC.SKEW.P"
) | ForEach-Object { [void]$bugFunc021Set.Add($_) }

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
            $ulp = Get-UlpDistance $lv $ev
            if ($bugFunc021Set.Contains([string]$candidate.function_id)) {
                $knownStat += 1
                $result = "known_residual_stat_drift"
            } else {
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
        schema_version = "oxfunc.smart_fuzzer.stat_distribution_excel_comparison.v0"
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
    schema_version = "oxfunc.smart_fuzzer.stat_distribution_run_rollup.v0"
    run_id = $RunId
    generated = $CaseCount
    local_evaluated = $CaseCount
    excel_sampled = $candidates.Count
    excel_blocked = [bool] $excel.blocked
    excel_input_plumbing = "cell_value2"
    matches = $matches
    known_stat_drift = $knownStat
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
Write-Host "Matches: $matches; known stat drift: $knownStat; unexpected: $unexpected; blocked: $blocked"
