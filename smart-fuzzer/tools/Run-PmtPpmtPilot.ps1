[CmdletBinding(PositionalBinding = $false)]
param(
    [string] $RepoRoot,
    [string] $RunId,
    [int] $Seed = 8803,
    [switch] $KeepWorkbook
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

if ([string]::IsNullOrWhiteSpace($RepoRoot)) {
    $scriptRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
    $RepoRoot = (Resolve-Path (Join-Path $scriptRoot "..\..")).Path
}
$RepoRoot = [System.IO.Path]::GetFullPath($RepoRoot)

if ([string]::IsNullOrWhiteSpace($RunId)) {
    $RunId = (Get-Date).ToUniversalTime().ToString("yyyyMMddTHHmmssZ") + "-pmt-ppmt-pilot"
}

# Cell-ref Excel comparator plumbing lives in the shared CellRefBatch
# module (W097 R-B). Numeric inputs are written via Range.Value2 so the
# formula parser does not introduce any input-side encoding drift.
$cellRefModulePath = Join-Path $RepoRoot "smart-fuzzer\tools\CellRefBatch.psm1"
Import-Module $cellRefModulePath -Force

$runnerVersion = "smart-fuzzer-pmt-ppmt-pilot/0.2.0-cellref"
$runDir = Join-Path $RepoRoot ("smart-fuzzer\runs\" + $RunId)
$caseDir = Join-Path $runDir "cases"
$outcomeDir = Join-Path $runDir "outcomes"
$comparisonDir = Join-Path $runDir "comparisons"
$failureDir = Join-Path $runDir "failure_packets"
$logDir = Join-Path $runDir "logs"
New-Item -ItemType Directory -Force -Path $runDir, $caseDir, $outcomeDir, $comparisonDir, $failureDir, $logDir | Out-Null

if ($KeepWorkbook) {
    Write-Warning "-KeepWorkbook is ignored by the cell-ref batch runner; no workbook artifact is emitted."
}

function Get-GitValue {
    param([string[]] $GitArgs)

    try {
        $value = (& git @GitArgs 2>$null)
        if ($LASTEXITCODE -ne 0) {
            return $null
        }
        return (($value -join "`n").Trim())
    }
    catch {
        return $null
    }
}

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
    param(
        [string] $Path,
        [object] $Value,
        [int] $Depth = 16
    )

    $Value | ConvertTo-Json -Depth $Depth | Set-Content -LiteralPath $Path -Encoding UTF8
}

function Add-JsonLine {
    param(
        [string] $Path,
        [object] $Value,
        [int] $Depth = 16
    )

    $line = $Value | ConvertTo-Json -Compress -Depth $Depth
    for ($attempt = 1; $attempt -le 10; $attempt++) {
        try {
            $line | Add-Content -LiteralPath $Path -Encoding UTF8
            return
        }
        catch [System.IO.IOException] {
            if ($attempt -eq 10) {
                throw
            }
            Start-Sleep -Milliseconds (50 * $attempt)
        }
    }
}

function Convert-ExpressionToDouble {
    param([string] $Expression)

    $culture = [System.Globalization.CultureInfo]::InvariantCulture
    $parts = $Expression -split "/"
    if ($parts.Count -eq 2) {
        return [double]::Parse($parts[0].Trim(), $culture) / [double]::Parse($parts[1].Trim(), $culture)
    }
    if ($parts.Count -gt 2) {
        throw "Unsupported numeric expression: $Expression"
    }
    return [double]::Parse($Expression.Trim(), $culture)
}

function Get-OutcomeDigest {
    param([object] $Outcome)

    return Get-Sha256Text ([string] $Outcome.digest_payload)
}

function Add-PilotCase {
    param(
        [System.Collections.IList] $Cases,
        [string] $CaseId,
        [string] $FunctionName,
        [string[]] $ArgExpressions,
        [string[]] $CoverageBuckets
    )

    $argValues = @()
    $argSources = @()
    foreach ($expr in $ArgExpressions) {
        $value = Convert-ExpressionToDouble $expr
        $argValues += $value
        $argSources += [ordered]@{
            expression = $expr
            value = $value
        }
    }

    $case = [ordered]@{
        schema_version = "oxfunc.smart_fuzzer.case.v0"
        case_id = $CaseId
        function_id = "FUNC.$FunctionName"
        function_name = $FunctionName
        generator_id = "pmt_ppmt_pilot.v0"
        seed = $Seed
        formula_text = ("={0}({1})" -f $FunctionName, ($ArgExpressions -join ","))
        args = $argValues
        arg_sources = $argSources
        coverage_buckets = $CoverageBuckets
    }
    $Cases.Add($case) | Out-Null
}

function New-PilotCases {
    $cases = New-Object System.Collections.ArrayList

    Add-PilotCase $cases "SFZ-PMT-0001" "PMT" @("0.05/12", "360", "200000") @("function:PMT", "arity:3", "rate:monthly_5pct", "known_witness:ftc_0377")
    Add-PilotCase $cases "SFZ-PMT-0002" "PMT" @("0", "12", "1200") @("function:PMT", "arity:3", "rate:zero")
    Add-PilotCase $cases "SFZ-PMT-0003" "PMT" @("0", "12", "1200", "100", "1") @("function:PMT", "arity:5", "rate:zero", "type:beginning")
    Add-PilotCase $cases "SFZ-PMT-0004" "PMT" @("0.08/12", "10", "10000") @("function:PMT", "arity:3", "w24_seed:pmt_sample")
    Add-PilotCase $cases "SFZ-PMT-0005" "PMT" @("0.05/12", "360", "200000", "0", "1") @("function:PMT", "arity:5", "type:beginning", "rate:monthly_5pct")
    Add-PilotCase $cases "SFZ-PMT-0006" "PMT" @("0.05/12", "360", "-200000") @("function:PMT", "arity:3", "pv:negative", "rate:monthly_5pct")
    Add-PilotCase $cases "SFZ-PMT-0007" "PMT" @("0.05/12", "359", "200000") @("function:PMT", "arity:3", "neighborhood:known_nper_minus_1")
    Add-PilotCase $cases "SFZ-PMT-0008" "PMT" @("0.05/12", "361", "200000") @("function:PMT", "arity:3", "neighborhood:known_nper_plus_1")
    Add-PilotCase $cases "SFZ-PMT-0009" "PMT" @("0.050000000000001/12", "360", "200000") @("function:PMT", "arity:3", "neighborhood:rate_plus")
    Add-PilotCase $cases "SFZ-PMT-0010" "PMT" @("0.049999999999999/12", "360", "200000") @("function:PMT", "arity:3", "neighborhood:rate_minus")
    Add-PilotCase $cases "SFZ-PMT-0011" "PMT" @("1E-9", "1000", "100000") @("function:PMT", "arity:3", "rate:tiny_positive")
    Add-PilotCase $cases "SFZ-PMT-0012" "PMT" @("-0.01/12", "12", "1000") @("function:PMT", "arity:3", "rate:negative")
    Add-PilotCase $cases "SFZ-PMT-0013" "PMT" @("1", "2", "100") @("function:PMT", "arity:3", "rate:large")
    Add-PilotCase $cases "SFZ-PMT-0014" "PMT" @("0.03/4", "16", "5000", "1000", "0") @("function:PMT", "arity:5", "fv:nonzero")

    Add-PilotCase $cases "SFZ-PPMT-0001" "PPMT" @("0.05/12", "1", "360", "200000") @("function:PPMT", "arity:4", "rate:monthly_5pct", "known_witness:ftc_0391")
    Add-PilotCase $cases "SFZ-PPMT-0002" "PPMT" @("0.10/12", "1", "36", "8000") @("function:PPMT", "arity:4", "w24_seed:ppmt_sample")
    Add-PilotCase $cases "SFZ-PPMT-0003" "PPMT" @("0", "1", "12", "1200") @("function:PPMT", "arity:4", "rate:zero")
    Add-PilotCase $cases "SFZ-PPMT-0004" "PPMT" @("0", "12", "12", "1200", "100", "1") @("function:PPMT", "arity:6", "rate:zero", "type:beginning")
    Add-PilotCase $cases "SFZ-PPMT-0005" "PPMT" @("0.05/12", "180", "360", "200000") @("function:PPMT", "arity:4", "period:middle", "rate:monthly_5pct")
    Add-PilotCase $cases "SFZ-PPMT-0006" "PPMT" @("0.05/12", "360", "360", "200000") @("function:PPMT", "arity:4", "period:last", "rate:monthly_5pct")
    Add-PilotCase $cases "SFZ-PPMT-0007" "PPMT" @("0.05/12", "1", "360", "200000", "0", "1") @("function:PPMT", "arity:6", "type:beginning", "rate:monthly_5pct")
    Add-PilotCase $cases "SFZ-PPMT-0008" "PPMT" @("0.05/12", "0", "360", "200000") @("function:PPMT", "arity:4", "period:zero_invalid")
    Add-PilotCase $cases "SFZ-PPMT-0009" "PPMT" @("0.05/12", "361", "360", "200000") @("function:PPMT", "arity:4", "period:past_nper_invalid")
    Add-PilotCase $cases "SFZ-PPMT-0010" "PPMT" @("0.050000000000001/12", "1", "360", "200000") @("function:PPMT", "arity:4", "neighborhood:rate_plus")
    Add-PilotCase $cases "SFZ-PPMT-0011" "PPMT" @("0.049999999999999/12", "1", "360", "200000") @("function:PPMT", "arity:4", "neighborhood:rate_minus")
    Add-PilotCase $cases "SFZ-PPMT-0012" "PPMT" @("1E-9", "1", "1000", "100000") @("function:PPMT", "arity:4", "rate:tiny_positive")
    Add-PilotCase $cases "SFZ-PPMT-0013" "PPMT" @("-0.01/12", "1", "12", "1000") @("function:PPMT", "arity:4", "rate:negative")
    Add-PilotCase $cases "SFZ-PPMT-0014" "PPMT" @("0.03/4", "4", "16", "5000", "1000", "0") @("function:PPMT", "arity:6", "fv:nonzero")

    return @($cases)
}

function Invoke-ExcelEvaluation {
    param(
        [object[]] $Cases
    )

    $excelCandidates = @()
    foreach ($case in $Cases) {
        $excelCandidates += @{ function_name = [string] $case.function_name; args = @($case.args) }
    }
    $batch = Invoke-ExcelCellRefBatch -Candidates $excelCandidates

    $outcomes = New-Object System.Collections.ArrayList
    if ($batch.blocked) {
        return [ordered]@{
            blocked = $true
            blocker = [string] $batch.blocker
            environment = $batch.environment
            outcomes = @()
        }
    }
    for ($i = 0; $i -lt $Cases.Count; $i++) {
        $case = $Cases[$i]
        $outcome = $batch.outcomes[$i]
        $excelErrorTypeMirror = $null
        if ($outcome.kind -eq "error") { $excelErrorTypeMirror = $outcome.code }
        $outcomes.Add(([ordered]@{
            schema_version = "oxfunc.smart_fuzzer.excel_outcome.v0"
            case_id = [string] $case.case_id
            function_id = [string] $case.function_id
            evaluator_id = "excel.com.cellref_batch/0.2.0"
            execution_status = "ok"
            formula_text = [string] $case.formula_text
            excel_error_type = $excelErrorTypeMirror
            outcome = $outcome
        })) | Out-Null
    }
    return [ordered]@{
        blocked = $false
        blocker = $null
        environment = $batch.environment
        outcomes = @($outcomes)
    }
}

$casesPath = Join-Path $caseDir "pmt-ppmt-cases.jsonl"
$localOutcomesPath = Join-Path $outcomeDir "local.jsonl"
$excelOutcomesPath = Join-Path $outcomeDir "excel.jsonl"
$comparisonsPath = Join-Path $comparisonDir "comparisons.jsonl"
$telemetryPath = Join-Path $runDir "telemetry.jsonl"
foreach ($path in @($casesPath, $localOutcomesPath, $excelOutcomesPath, $comparisonsPath, $telemetryPath)) {
    if (Test-Path -LiteralPath $path) {
        Remove-Item -LiteralPath $path
    }
}

$gitRevision = Get-GitValue @("rev-parse", "HEAD")
$gitStatus = Get-GitValue @("status", "--short")
$createdUtc = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")
$cases = New-PilotCases
foreach ($case in $cases) {
    Add-JsonLine -Path $casesPath -Value $case
}

$localWatch = [System.Diagnostics.Stopwatch]::StartNew()
$localEvaluatorManifest = Join-Path $RepoRoot "smart-fuzzer\tools\pmt_ppmt_local_eval\Cargo.toml"
& cargo run --quiet --manifest-path $localEvaluatorManifest --bin pmt_ppmt_local_eval -- --cases $casesPath --out $localOutcomesPath
if ($LASTEXITCODE -ne 0) {
    throw "Local PMT/PPMT evaluator failed with exit code $LASTEXITCODE"
}
$localWatch.Stop()

$excelWatch = [System.Diagnostics.Stopwatch]::StartNew()
$excelResult = Invoke-ExcelEvaluation -Cases $cases
$excelWatch.Stop()

$localById = @{}
foreach ($line in Get-Content -LiteralPath $localOutcomesPath) {
    if ([string]::IsNullOrWhiteSpace($line)) {
        continue
    }
    $outcome = $line | ConvertFrom-Json
    $localById[[string] $outcome.case_id] = $outcome
}

$excelById = @{}
foreach ($outcome in $excelResult.outcomes) {
    $excelById[[string] $outcome.case_id] = $outcome
    Add-JsonLine -Path $excelOutcomesPath -Value $outcome
}

$matches = 0
$mismatches = 0
$blocked = 0
$failurePackets = New-Object System.Collections.ArrayList
$comparisonWatch = [System.Diagnostics.Stopwatch]::StartNew()
foreach ($case in $cases) {
    $caseId = [string] $case.case_id
    $local = $localById[$caseId]
    $excelOutcomeRecord = $excelById[$caseId]
    $comparisonResult = "blocked"
    $absDelta = $null
    $withinAbs1e9 = $null
    $localKind = $null
    $excelKind = $null
    $localBits = $null
    $excelBits = $null
    $localDigest = $null
    $excelDigest = $null

    if ($null -eq $local -or $null -eq $excelOutcomeRecord) {
        $blocked += 1
    }
    else {
        $localKind = [string] $local.outcome.kind
        $excelKind = [string] $excelOutcomeRecord.outcome.kind
        $localDigest = Get-OutcomeDigest $local.outcome
        $excelDigest = Get-OutcomeDigest $excelOutcomeRecord.outcome
        if ($localKind -eq "number" -and $excelKind -eq "number") {
            $localBits = [string] $local.outcome.bits_hex
            $excelBits = [string] $excelOutcomeRecord.outcome.bits_hex
            $absDelta = [Math]::Abs(([double] $local.outcome.value) - ([double] $excelOutcomeRecord.outcome.value))
            $withinAbs1e9 = $absDelta -le 1E-9
            if ($localBits -eq $excelBits) {
                $comparisonResult = "match"
                $matches += 1
            }
            else {
                $comparisonResult = "mismatch_numeric_bits"
                $mismatches += 1
            }
        }
        elseif ($localKind -eq "error" -and $excelKind -eq "error") {
            if ([string] $local.outcome.code -eq [string] $excelOutcomeRecord.outcome.code) {
                $comparisonResult = "match"
                $matches += 1
            }
            else {
                $comparisonResult = "mismatch_error_code"
                $mismatches += 1
            }
        }
        else {
            if ($local.outcome.digest_payload -eq $excelOutcomeRecord.outcome.digest_payload) {
                $comparisonResult = "match"
                $matches += 1
            }
            else {
                $comparisonResult = "mismatch_kind_or_payload"
                $mismatches += 1
            }
        }
    }

    $comparison = [ordered]@{
        schema_version = "oxfunc.smart_fuzzer.comparison.v0"
        case_id = $caseId
        function_id = [string] $case.function_id
        formula_text = [string] $case.formula_text
        comparison_result = $comparisonResult
        local_kind = $localKind
        excel_kind = $excelKind
        local_bits_hex = $localBits
        excel_bits_hex = $excelBits
        abs_delta = $absDelta
        within_abs_1e_9 = $withinAbs1e9
        coverage_buckets = $case.coverage_buckets
    }
    Add-JsonLine -Path $comparisonsPath -Value $comparison
    Add-JsonLine -Path $telemetryPath -Value ([ordered]@{
        schema_version = "oxfunc.smart_fuzzer.telemetry.v0"
        case_id = $caseId
        run_id = $RunId
        function_id = [string] $case.function_id
        generator_id = "pmt_ppmt_pilot.v0"
        seed = $Seed
        invocation_digest = Get-Sha256Text ([string] $case.formula_text)
        coverage_buckets = $case.coverage_buckets
        local_outcome_digest = $localDigest
        excel_outcome_digest = $excelDigest
        comparison_result = $comparisonResult
    })

    if ($comparisonResult -like "mismatch*") {
        $packetPath = Join-Path $failureDir ($caseId + ".json")
        Write-JsonFile -Path $packetPath -Value ([ordered]@{
            schema_version = "oxfunc.smart_fuzzer.failure_packet.v0"
            run_id = $RunId
            case = $case
            local_outcome = $local
            excel_outcome = $excelOutcomeRecord
            comparison = $comparison
        })
        $failurePackets.Add($packetPath.Replace($RepoRoot + "\", "")) | Out-Null
    }
}
$comparisonWatch.Stop()

$byFunction = @{}
$byCoverageBucket = @{}
foreach ($case in $cases) {
    $functionId = [string] $case.function_id
    if (-not $byFunction.ContainsKey($functionId)) {
        $byFunction[$functionId] = 0
    }
    $byFunction[$functionId] += 1
    foreach ($bucket in $case.coverage_buckets) {
        $bucketKey = [string] $bucket
        if (-not $byCoverageBucket.ContainsKey($bucketKey)) {
            $byCoverageBucket[$bucketKey] = 0
        }
        $byCoverageBucket[$bucketKey] += 1
    }
}

$manifest = [ordered]@{
    schema_version = "oxfunc.smart_fuzzer.run_manifest.v0"
    run_id = $RunId
    created_utc = $createdUtc
    git_revision = $gitRevision
    worktree_dirty = -not [string]::IsNullOrWhiteSpace($gitStatus)
    runner = [ordered]@{
        runner_id = "smart-fuzzer-pmt-ppmt-pilot"
        runner_version = $runnerVersion
        command_line = @([System.Environment]::CommandLine)
    }
    scope = [ordered]@{
        function_ids = @("FUNC.PMT", "FUNC.PPMT")
        generator_ids = @("pmt_ppmt_pilot.v0")
        excel_budget_cases = $cases.Count
        local_budget_cases = $cases.Count
    }
    environment = [ordered]@{
        host_os = [System.Environment]::OSVersion.VersionString
        rust_profile = "cargo run default dev profile for helper"
        excel = $excelResult.environment
        excel_input_plumbing = "cell_value2"
        locale_profile = "en-US"
    }
    inputs = [ordered]@{
        metadata_snapshot_refs = @("docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1.csv")
        source_index_digest = $null
        seed = $Seed
        generator_policy = "fixed PMT/PPMT witness, neighborhood, arity, rate, period, fv/type, and invalid-period rows"
    }
}

$manifestPath = Join-Path $runDir "manifest.json"
Write-JsonFile -Path $manifestPath -Value $manifest
$manifestHash = Get-Sha256Text (Get-Content -LiteralPath $manifestPath -Raw)

$rollup = [ordered]@{
    schema_version = "oxfunc.smart_fuzzer.rollup.v0"
    run_id = $RunId
    manifest_hash = $manifestHash
    case_counts = [ordered]@{
        generated = $cases.Count
        local_evaluated = $localById.Count
        excel_evaluated = $excelById.Count
        matches = $matches
        mismatches = $mismatches
        unstable = 0
        blocked = $blocked
        invalid_generator_output = 0
    }
    by_function = $byFunction
    by_generator = @{ "pmt_ppmt_pilot.v0" = $cases.Count }
    by_coverage_bucket = $byCoverageBucket
    throughput = [ordered]@{
        local_cases_per_second = if ($localWatch.Elapsed.TotalSeconds -gt 0) { $localById.Count / $localWatch.Elapsed.TotalSeconds } else { $null }
        excel_cases_per_second = if ($excelWatch.Elapsed.TotalSeconds -gt 0) { $excelById.Count / $excelWatch.Elapsed.TotalSeconds } else { $null }
        comparison_cases_per_second = if ($comparisonWatch.Elapsed.TotalSeconds -gt 0) { $cases.Count / $comparisonWatch.Elapsed.TotalSeconds } else { $null }
        local_wall_seconds = $localWatch.Elapsed.TotalSeconds
        excel_wall_seconds = $excelWatch.Elapsed.TotalSeconds
        comparison_wall_seconds = $comparisonWatch.Elapsed.TotalSeconds
    }
    promotion_candidates = @($failurePackets)
    workbook_path = $null
}
if ($excelResult.blocked) {
    $rollup["blocker"] = $excelResult.blocker
}

Write-JsonFile -Path (Join-Path $runDir "rollup.json") -Value $rollup
Write-JsonFile -Path (Join-Path $logDir "pmt-ppmt-pilot-summary.json") -Value ([ordered]@{
    run_id = $RunId
    manifest_hash = $manifestHash
    total_cases = $cases.Count
    matches = $matches
    mismatches = $mismatches
    blocked = $blocked
    promotion_candidates = @($failurePackets)
})

Write-Host "Run: $RunId"
Write-Host "Run directory: $runDir"
Write-Host "Manifest hash: $manifestHash"
Write-Host ("Cases: {0}; matches: {1}; mismatches: {2}; blocked: {3}" -f $cases.Count, $matches, $mismatches, $blocked)
if ($failurePackets.Count -gt 0) {
    Write-Host "Failure packets:"
    foreach ($packet in $failurePackets) {
        Write-Host ("  {0}" -f $packet)
    }
}
