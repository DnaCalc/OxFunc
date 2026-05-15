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

# Cell-ref Excel comparator plumbing lives in the shared CellRefBatch
# module (W097 R-B). The runner now sources Invoke-ExcelCellRefBatch,
# Get-F64BitsHex, ConvertTo-ExcelOutcome, Get-UlpDistance from there.
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

# Statistical-exactness functions covered by BUG-FUNC-021 etc.
$KnownResidualFunctions = New-Object 'System.Collections.Generic.HashSet[string]'
@(
    "FUNC.BETADIST","FUNC.BETAINV","FUNC.CHIDIST","FUNC.CHIINV",
    "FUNC.FDIST","FUNC.FINV","FUNC.GAMMADIST","FUNC.GAMMAINV",
    "FUNC.HYPGEOMDIST","FUNC.NEGBINOMDIST","FUNC.NORMSDIST","FUNC.NORMSINV",
    "FUNC.TDIST","FUNC.TINV","FUNC.PERCENTRANK","FUNC.CONFIDENCE.T","FUNC.Z.TEST",
    "FUNC.BESSELY","FUNC.MINVERSE","FUNC.PMT","FUNC.PPMT","FUNC.IPMT"
) | ForEach-Object { [void]$KnownResidualFunctions.Add($_) }

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
$excel = Invoke-ExcelCellRefBatch -Candidates $candidates
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
