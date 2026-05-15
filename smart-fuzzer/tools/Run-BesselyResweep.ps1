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
    $RunId = "W097-R-E-bessely-cellref"
}

# W097 R-E: BESSELY cell-ref re-replay around the (2.5, 1) witness.
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

# Build a fixed band of (x, n) pairs around the witness BESSELY(2.5, 1).
# Cover both directions (low x, high x; n=0..5; tight neighborhood
# around 2.5).
$cases = New-Object System.Collections.ArrayList
$nValues = @(0, 1, 2, 3, 5, 10)
$xValuesBroad = @(0.5, 1.0, 1.5, 2.0, 2.5, 3.0, 5.0, 10.0, 20.0, 50.0, 100.0)
$xValuesTight = @(2.40, 2.45, 2.49, 2.499, 2.5, 2.501, 2.51, 2.55, 2.60)
$idx = 0
foreach ($n in $nValues) {
    foreach ($x in $xValuesBroad) {
        $caseId = "BESY-{0:D4}" -f $idx; $idx += 1
        [void]$cases.Add([ordered]@{
            case_id = $caseId
            function_id = "FUNC.BESSELY"
            function_name = "BESSELY"
            args = @([double]$x, [double]$n)
            band = "broad"
        })
    }
}
foreach ($n in @(0, 1, 2)) {
    foreach ($x in $xValuesTight) {
        $caseId = "BESY-{0:D4}" -f $idx; $idx += 1
        [void]$cases.Add([ordered]@{
            case_id = $caseId
            function_id = "FUNC.BESSELY"
            function_name = "BESSELY"
            args = @([double]$x, [double]$n)
            band = "tight_around_2_5"
        })
    }
}

$casesPath = Join-Path $caseDir "bessely-cases.jsonl"
if (Test-Path -LiteralPath $casesPath) { Remove-Item -LiteralPath $casesPath }
foreach ($c in $cases) { Add-JsonLine $casesPath $c }

# Local OxFunc evaluation through pmt_ppmt_local_eval bin (reads
# {case_id, function_id, args} JSONL and writes outcomes).
$localOutcomesPath = Join-Path $outcomeDir "local.jsonl"
if (Test-Path -LiteralPath $localOutcomesPath) { Remove-Item -LiteralPath $localOutcomesPath }
$manifest = Join-Path $RepoRoot "smart-fuzzer\tools\pmt_ppmt_local_eval\Cargo.toml"
$localWatch = [System.Diagnostics.Stopwatch]::StartNew()
& cargo run --release --quiet --manifest-path $manifest --bin pmt_ppmt_local_eval -- --cases $casesPath --out $localOutcomesPath
if ($LASTEXITCODE -ne 0) {
    throw "Local evaluator failed exit $LASTEXITCODE"
}
$localWatch.Stop()

$localById = @{}
foreach ($line in Get-Content -LiteralPath $localOutcomesPath) {
    if ([string]::IsNullOrWhiteSpace($line)) { continue }
    $o = $line | ConvertFrom-Json
    $localById[[string]$o.case_id] = $o.outcome
}

# Excel evaluation through CellRefBatch.
$cellRefCandidates = @()
foreach ($c in $cases) {
    $cellRefCandidates += @{ function_name = [string]$c.function_name; args = @($c.args) }
}
$excelWatch = [System.Diagnostics.Stopwatch]::StartNew()
$excel = Invoke-ExcelCellRefBatch -Candidates $cellRefCandidates
$excelWatch.Stop()

# Comparison
$comparisonsPath = Join-Path $comparisonDir "excel_sample_comparisons.jsonl"
if (Test-Path -LiteralPath $comparisonsPath) { Remove-Item -LiteralPath $comparisonsPath }
$matches = 0
$drifts = 0
$errors = 0
$blocked = 0
for ($i = 0; $i -lt $cases.Count; $i++) {
    $case = $cases[$i]
    $local = $localById[[string]$case.case_id]
    $excelOutcome = if ($excel.outcomes.Count -gt $i) { $excel.outcomes[$i] } else { $null }
    $result = "blocked"
    $absDelta = $null; $ulp = $null
    if ($null -eq $local -or $null -eq $excelOutcome) { $blocked += 1 }
    elseif ([string]$local.kind -eq "number" -and [string]$excelOutcome.kind -eq "number") {
        $lv = [double]$local.value; $ev = [double]$excelOutcome.value
        $absDelta = [Math]::Abs($lv - $ev)
        if ([string]$local.bits_hex -eq [string]$excelOutcome.bits_hex) {
            $matches += 1; $result = "exact_typed_bit_match"
        } else {
            $ulp = Get-UlpDistance $lv $ev
            $drifts += 1; $result = "known_residual_bessely_drift"
        }
    } elseif ([string]$local.kind -eq [string]$excelOutcome.kind -and [string]$local.digest_payload -eq [string]$excelOutcome.digest_payload) {
        $matches += 1; $result = "exact_typed_bit_match"
    } else {
        $errors += 1; $result = "kind_drift"
    }
    Add-JsonLine $comparisonsPath ([ordered]@{
        schema_version = "oxfunc.smart_fuzzer.bessely_excel_comparison.v0"
        case_id = [string]$case.case_id
        x = [double]$case.args[0]
        n = [double]$case.args[1]
        band = [string]$case.band
        comparison_result = $result
        abs_delta = $absDelta
        ulp_distance = $ulp
        local_outcome = $local
        excel_outcome = $excelOutcome
    })
}

Write-JsonFile (Join-Path $runDir "rollup.json") ([ordered]@{
    schema_version = "oxfunc.smart_fuzzer.bessely_run_rollup.v0"
    run_id = $RunId
    cases = $cases.Count
    matches = $matches
    drifts = $drifts
    kind_drift = $errors
    blocked = $blocked
    excel_input_plumbing = "cell_value2"
    excel_environment = $excel.environment
    local_wall_seconds = $localWatch.Elapsed.TotalSeconds
    excel_wall_seconds = $excelWatch.Elapsed.TotalSeconds
})

Write-Host "Run: $RunId; cases: $($cases.Count); matches: $matches; drifts: $drifts; kind drift: $errors; blocked: $blocked"
