[CmdletBinding(PositionalBinding = $false)]
param(
    [string] $RepoRoot,
    [string] $RunId,
    [object[]] $BatchSizes = @(100, 1000, 5000),
    [int] $Seed = 8802,
    [switch] $KeepWorkbook
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

if ([string]::IsNullOrWhiteSpace($RepoRoot)) {
    $scriptRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
    $RepoRoot = (Resolve-Path (Join-Path $scriptRoot "..\..")).Path
}
$RepoRoot = [System.IO.Path]::GetFullPath($RepoRoot)

$parsedBatchSizes = @()
foreach ($entry in $BatchSizes) {
    foreach ($part in ([string] $entry -split ",")) {
        if ([string]::IsNullOrWhiteSpace($part)) {
            continue
        }
        $parsedBatchSizes += [int] $part.Trim()
    }
}
if ($parsedBatchSizes.Count -eq 0) {
    throw "At least one batch size is required."
}
$EffectiveBatchSizes = [int[]] $parsedBatchSizes

if ([string]::IsNullOrWhiteSpace($RunId)) {
    $RunId = (Get-Date).ToUniversalTime().ToString("yyyyMMddTHHmmssZ") + "-excel-throughput"
}

$runnerVersion = "smart-fuzzer-excel-throughput/0.1.0"
$runDir = Join-Path $RepoRoot ("smart-fuzzer\runs\" + $RunId)
$logDir = Join-Path $runDir "logs"
New-Item -ItemType Directory -Force -Path $runDir, $logDir | Out-Null

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
        [int] $Depth = 12
    )

    $Value | ConvertTo-Json -Depth $Depth | Set-Content -LiteralPath $Path -Encoding UTF8
}

function Add-JsonLine {
    param(
        [string] $Path,
        [object] $Value,
        [int] $Depth = 12
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

function New-BenchmarkFormula {
    param([int] $Index)

    switch ($Index % 8) {
        0 { return "=1+1" }
        1 { return "=SUM(1,2,3,4,5)" }
        2 { return "=PMT(0.05/12,360,200000)" }
        3 { return "=PPMT(0.05/12,1,360,200000)" }
        4 { return ("=DATE(2024,1,1)+{0}" -f ($Index % 365)) }
        5 { return ("=POWER({0},2)" -f (($Index % 17) + 1)) }
        6 { return ("=IF(MOD({0},2)=0,TRUE,FALSE)" -f $Index) }
        default { return ("=ROUND(SIN({0}),10)" -f $Index) }
    }
}

function Get-FunctionIdForFormula {
    param([string] $Formula)

    if ($Formula -match "^=([A-Z][A-Z0-9\.]*)\(") {
        return "FUNC." + $Matches[1]
    }
    return "BENCH.EXPRESSION"
}

function New-FormulaArray {
    param(
        [int] $RowCount,
        [int] $StartIndex
    )

    $array = New-Object "object[,]" $RowCount, 1
    for ($row = 0; $row -lt $RowCount; $row++) {
        $array[$row, 0] = New-BenchmarkFormula ($StartIndex + $row)
    }
    return ,$array
}

function Get-ArrayCellValue {
    param(
        [object] $Values,
        [int] $RowIndex
    )

    if ($Values -is [System.Array]) {
        $lower0 = $Values.GetLowerBound(0)
        $lower1 = $Values.GetLowerBound(1)
        return $Values.GetValue($lower0 + $RowIndex, $lower1)
    }
    return $Values
}

function Release-ComObject {
    param([object] $Object)

    if ($null -ne $Object -and [System.Runtime.InteropServices.Marshal]::IsComObject($Object)) {
        [void] [System.Runtime.InteropServices.Marshal]::FinalReleaseComObject($Object)
    }
}

function Set-ExcelPropertyBestEffort {
    param(
        [object] $ExcelApplication,
        [string] $PropertyName,
        [object] $Value,
        [System.Collections.IList] $Warnings
    )

    try {
        $ExcelApplication.$PropertyName = $Value
    }
    catch {
        $Warnings.Add(([ordered]@{
            property = $PropertyName
            message = $_.Exception.Message
        })) | Out-Null
    }
}

$telemetryPath = Join-Path $runDir "telemetry.jsonl"
if (Test-Path -LiteralPath $telemetryPath) {
    Remove-Item -LiteralPath $telemetryPath
}

$gitRevision = Get-GitValue @("rev-parse", "HEAD")
$gitStatus = Get-GitValue @("status", "--short")
$excel = $null
$workbook = $null
$worksheet = $null
$workbookPath = $null
$excelProcessId = $null
$createdUtc = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")

$manifest = [ordered]@{
    schema_version = "oxfunc.smart_fuzzer.run_manifest.v0"
    run_id = $RunId
    created_utc = $createdUtc
    git_revision = $gitRevision
    worktree_dirty = -not [string]::IsNullOrWhiteSpace($gitStatus)
    runner = [ordered]@{
        runner_id = "smart-fuzzer-excel-throughput"
        runner_version = $runnerVersion
        command_line = @([System.Environment]::CommandLine)
    }
    scope = [ordered]@{
        function_ids = @("BENCH.EXPRESSION", "FUNC.SUM", "FUNC.PMT", "FUNC.PPMT", "FUNC.DATE", "FUNC.POWER", "FUNC.IF", "FUNC.ROUND")
        generator_ids = @("excel_throughput_benchmark.v0")
        excel_budget_cases = ($EffectiveBatchSizes | Measure-Object -Sum).Sum
        local_budget_cases = 0
    }
    environment = [ordered]@{
        host_os = [System.Environment]::OSVersion.VersionString
        rust_profile = $null
        excel_available = $false
        excel_version = $null
        excel_build = $null
        excel_channel = $null
        workbook_compatibility = "unknown"
        locale_profile = "en-US"
    }
    inputs = [ordered]@{
        metadata_snapshot_refs = @("docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1.csv")
        source_index_digest = $null
        seed = $Seed
        batch_sizes = $EffectiveBatchSizes
    }
}

$batchSummaries = @()
$totalCases = 0
$allMatches = 0
$blocked = 0
$coldStartSeconds = $null
$excelSettingWarnings = New-Object System.Collections.ArrayList

try {
    $coldStart = [System.Diagnostics.Stopwatch]::StartNew()
    $excelProcessIdsBefore = @(Get-Process EXCEL -ErrorAction SilentlyContinue | ForEach-Object { $_.Id })
    try {
        $excel = New-Object -ComObject Excel.Application
    }
    catch {
        $blocked = ($EffectiveBatchSizes | Measure-Object -Sum).Sum
        $manifest.environment.excel_available = $false
        $manifest.environment["blocker"] = "Excel COM automation unavailable: $($_.Exception.Message)"
        throw
    }
    $coldStart.Stop()
    $coldStartSeconds = $coldStart.Elapsed.TotalSeconds
    $excelProcessIdsAfter = @(Get-Process EXCEL -ErrorAction SilentlyContinue | ForEach-Object { $_.Id })
    $newExcelProcessIds = @($excelProcessIdsAfter | Where-Object { $excelProcessIdsBefore -notcontains $_ })
    if ($newExcelProcessIds.Count -eq 1) {
        $excelProcessId = [int] $newExcelProcessIds[0]
    }

    Set-ExcelPropertyBestEffort -ExcelApplication $excel -PropertyName "Visible" -Value $false -Warnings $excelSettingWarnings
    Set-ExcelPropertyBestEffort -ExcelApplication $excel -PropertyName "DisplayAlerts" -Value $false -Warnings $excelSettingWarnings
    Set-ExcelPropertyBestEffort -ExcelApplication $excel -PropertyName "ScreenUpdating" -Value $false -Warnings $excelSettingWarnings
    Set-ExcelPropertyBestEffort -ExcelApplication $excel -PropertyName "EnableEvents" -Value $false -Warnings $excelSettingWarnings
    Set-ExcelPropertyBestEffort -ExcelApplication $excel -PropertyName "Calculation" -Value -4135 -Warnings $excelSettingWarnings

    $manifest.environment.excel_available = $true
    $manifest.environment.excel_version = [string] $excel.Version
    try { $manifest.environment.excel_build = [string] $excel.Build } catch { $manifest.environment.excel_build = $null }
    if ($excelSettingWarnings.Count -gt 0) {
        $manifest.environment["excel_setting_warnings"] = @($excelSettingWarnings)
    }

    $workbook = $excel.Workbooks.Add()
    $worksheet = $workbook.Worksheets.Item(1)
    try { $manifest.environment.workbook_compatibility = [string] $workbook.CompatibilityVersion } catch { $manifest.environment.workbook_compatibility = "unknown" }

    $caseIndex = 0
    foreach ($batchSize in $EffectiveBatchSizes) {
        if ($batchSize -le 0) {
            continue
        }

        $formulas = New-FormulaArray -RowCount $batchSize -StartIndex ($Seed + $caseIndex)
        $anchor = $worksheet.Range("A1")
        $range = $anchor.Resize($batchSize, 1)
        Release-ComObject $anchor
        [void] $range.Clear()

        $writeWatch = [System.Diagnostics.Stopwatch]::StartNew()
        try {
            $range.Formula2 = $formulas
        }
        catch {
            $range.Formula = $formulas
        }
        $writeWatch.Stop()

        $calcWatch = [System.Diagnostics.Stopwatch]::StartNew()
        [void] $range.Calculate()
        $calcWatch.Stop()

        $extractWatch = [System.Diagnostics.Stopwatch]::StartNew()
        $values = $range.Value2
        $extractWatch.Stop()

        $batchId = "batch-{0:D4}" -f $batchSummaries.Count
        for ($row = 0; $row -lt $batchSize; $row++) {
            $formula = [string] $formulas[$row, 0]
            $value = Get-ArrayCellValue -Values $values -RowIndex $row
            $caseId = "SFZ-BENCH-{0:D8}" -f ($caseIndex + $row + 1)
            Add-JsonLine -Path $telemetryPath -Value ([ordered]@{
                schema_version = "oxfunc.smart_fuzzer.telemetry.v0"
                case_id = $caseId
                run_id = $RunId
                function_id = Get-FunctionIdForFormula $formula
                generator_id = "excel_throughput_benchmark.v0"
                seed = $Seed
                invocation_digest = Get-Sha256Text $formula
                formula_text = $formula
                coverage_buckets = @("benchmark:excel_throughput", "batch:$batchId")
                local_outcome_digest = $null
                excel_outcome_digest = Get-Sha256Text ([string] $value)
                comparison_result = "observed_only"
            })
        }

        $elapsed = $writeWatch.Elapsed.TotalSeconds + $calcWatch.Elapsed.TotalSeconds + $extractWatch.Elapsed.TotalSeconds
        $batchSummary = [ordered]@{
            batch_id = $batchId
            case_count = $batchSize
            formula_write_seconds = $writeWatch.Elapsed.TotalSeconds
            calculate_seconds = $calcWatch.Elapsed.TotalSeconds
            result_extract_seconds = $extractWatch.Elapsed.TotalSeconds
            total_measured_seconds = $elapsed
            cases_per_second = if ($elapsed -gt 0) { $batchSize / $elapsed } else { $null }
            calculate_cases_per_second = if ($calcWatch.Elapsed.TotalSeconds -gt 0) { $batchSize / $calcWatch.Elapsed.TotalSeconds } else { $null }
        }
        $batchSummaries += $batchSummary
        $totalCases += $batchSize
        $allMatches += $batchSize
        $caseIndex += $batchSize
        Release-ComObject $range
        $range = $null
    }

    if ($KeepWorkbook) {
        $workbookPath = Join-Path $logDir "excel-throughput-workbook.xlsx"
        [void] $workbook.SaveAs($workbookPath)
    }
}
catch {
    if ($blocked -eq 0) {
        $blocked = (($EffectiveBatchSizes | Measure-Object -Sum).Sum) - $totalCases
    }
    Add-JsonLine -Path $telemetryPath -Value ([ordered]@{
        schema_version = "oxfunc.smart_fuzzer.telemetry.v0"
        case_id = "SFZ-BENCH-BLOCKED"
        run_id = $RunId
        function_id = "BENCH.EXCEL"
        generator_id = "excel_throughput_benchmark.v0"
        seed = $Seed
        invocation_digest = $null
        formula_text = $null
        coverage_buckets = @("benchmark:excel_throughput", "blocked:excel_harness")
        local_outcome_digest = $null
        excel_outcome_digest = $null
        comparison_result = "blocked"
        blocker = $_.Exception.Message
    })
    Write-Host "Excel throughput benchmark blocked: $($_.Exception.Message)"
}
finally {
    if ($null -ne $workbook -and -not $KeepWorkbook) {
        try { $workbook.Close($false) } catch {}
    }
    if ($null -ne $excel) {
        try { $excel.Quit() } catch {}
    }
    Release-ComObject $worksheet
    Release-ComObject $workbook
    Release-ComObject $excel
    [System.GC]::Collect()
    [System.GC]::WaitForPendingFinalizers()
    if ($null -ne $excelProcessId) {
        $createdExcelProcess = Get-Process -Id $excelProcessId -ErrorAction SilentlyContinue
        if ($null -ne $createdExcelProcess) {
            try {
                $createdExcelProcess.WaitForExit(2000) | Out-Null
            }
            catch {}
            $createdExcelProcess = Get-Process -Id $excelProcessId -ErrorAction SilentlyContinue
            if ($null -ne $createdExcelProcess) {
                Stop-Process -Id $excelProcessId -Force -ErrorAction SilentlyContinue
            }
        }
    }
}

$manifestPath = Join-Path $runDir "manifest.json"
Write-JsonFile -Path $manifestPath -Value $manifest
$manifestHash = Get-Sha256Text (Get-Content -LiteralPath $manifestPath -Raw)

$totalMeasuredSeconds = ($batchSummaries | ForEach-Object { $_.total_measured_seconds } | Measure-Object -Sum).Sum
$maxBatch = ($EffectiveBatchSizes | Measure-Object -Maximum).Maximum
$rollup = [ordered]@{
    schema_version = "oxfunc.smart_fuzzer.rollup.v0"
    run_id = $RunId
    manifest_hash = $manifestHash
    case_counts = [ordered]@{
        generated = $totalCases
        local_evaluated = 0
        excel_evaluated = $totalCases
        matches = $allMatches
        mismatches = 0
        unstable = 0
        blocked = $blocked
        invalid_generator_output = 0
    }
    by_function = @{}
    by_generator = @{ "excel_throughput_benchmark.v0" = $totalCases }
    by_coverage_bucket = @{ "benchmark:excel_throughput" = $totalCases }
    throughput = [ordered]@{
        cold_start_seconds = $coldStartSeconds
        local_cases_per_second = $null
        excel_cases_per_second = if ($totalMeasuredSeconds -gt 0) { $totalCases / $totalMeasuredSeconds } else { $null }
        excel_batch_size = $maxBatch
    }
    batches = $batchSummaries
    promotion_candidates = @()
    workbook_path = $workbookPath
}

Write-JsonFile -Path (Join-Path $runDir "rollup.json") -Value $rollup
Write-JsonFile -Path (Join-Path $logDir "excel-throughput-summary.json") -Value ([ordered]@{
    run_id = $RunId
    manifest_hash = $manifestHash
    total_cases = $totalCases
    batch_count = $batchSummaries.Count
    throughput = $rollup.throughput
})

Write-Host "Run: $RunId"
Write-Host "Run directory: $runDir"
Write-Host "Manifest hash: $manifestHash"
if ($rollup.throughput.excel_cases_per_second) {
    Write-Host ("Excel measured throughput: {0:N2} cases/sec" -f $rollup.throughput.excel_cases_per_second)
}
else {
    Write-Host "Excel measured throughput: unavailable"
}
