[CmdletBinding(PositionalBinding = $false)]
param()

# In-process self-test for OracleCache.psm1: key canonicalization, dedup
# ("no Excel question is ever asked twice"), persistence round-trip, and
# from_cache flags. Excel COM is NOT exercised — a scriptblock invoker stands
# in for Invoke-ExcelCellRefBatch, and the cache root is a temp directory.

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$scriptRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
Import-Module (Join-Path $scriptRoot "OracleCache.psm1") -Force

# Local bits helper: OracleCache's nested CellRefBatch import is module-scoped,
# so the mock invoker cannot rely on Get-F64BitsHex being visible here.
function Get-TestBitsHex {
    param([Parameter(Mandatory)] [double] $Value)
    $bits = [System.BitConverter]::ToUInt64([System.BitConverter]::GetBytes($Value), 0)
    return ("0x{0:x16}" -f $bits)
}

$failures = New-Object 'System.Collections.Generic.List[string]'
$passes = 0
$script:totalCases = 0

function Assert-Equal {
    param([string] $Label, $Expected, $Actual)
    $script:totalCases += 1
    if (-not ($Expected -is [string])) { $Expected = [string]$Expected }
    if (-not ($Actual -is [string])) { $Actual = [string]$Actual }
    if ($Expected -eq $Actual) {
        $script:passes += 1
        Write-Host "  PASS  $Label"
    } else {
        $script:failures.Add("$Label : expected '$Expected' got '$Actual'") | Out-Null
        Write-Host "  FAIL  $Label : expected '$Expected' got '$Actual'" -ForegroundColor Red
    }
}

$tempRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("oracle-cache-test-" + [Guid]::NewGuid().ToString("N"))
$testEnv = [ordered]@{
    excel_version = "16.0"
    excel_build = "test"
    excel_bitness = "64-bit"
    workbook_compatibility = "test"
    cpu_id = "test-cpu"
}

try {
    Write-Host "Get-OracleCacheKey"
    Initialize-OracleCache -CacheRoot $tempRoot -Environment $testEnv | Out-Null
    $manifest = Get-Content (Join-Path $tempRoot "env.json") -Raw | ConvertFrom-Json
    Assert-Equal "manifest schema upgraded" "oracle-environment-v2" $manifest.environment_schema
    Assert-Equal "manifest records bitness" "64-bit" $manifest.excel_bitness
    Assert-Equal "manifest records workbook compatibility" "test" $manifest.workbook_compatibility

    # Scalar canonicalization: key is function + f64 bits, not decimal text.
    $k1 = Get-OracleCacheKey -Request @{ function_name = "sin"; args = @(1.5) }
    Assert-Equal "scalar key uses bits hex" "SIN|0x3ff8000000000000" $k1
    $k2 = Get-OracleCacheKey -Request @{ function_name = "SIN"; args = @(1.5) }
    Assert-Equal "function name case-insensitive" $k1 $k2
    $k3 = Get-OracleCacheKey -Request @{ function_name = "SIN"; args = @(1.5000000000000002) }
    Assert-Equal "adjacent f64 gets a different key" $false ($k1 -eq $k3)

    # Negative zero and positive zero are distinct bit patterns.
    $kz1 = Get-OracleCacheKey -Request @{ function_name = "ABS"; args = @(0.0) }
    $kz2 = Get-OracleCacheKey -Request @{ function_name = "ABS"; args = @(-0.0) }
    Assert-Equal "+0 and -0 keys differ" $false ($kz1 -eq $kz2)

    # Logical args.
    $kb = Get-OracleCacheKey -Request @{ function_name = "PMT"; args = @(0.05, 10.0, -1000.0, 0.0, $true) }
    Assert-Equal "logical arg key" "PMT|0x3fa999999999999a|0x4024000000000000|0xc08f400000000000|0x0000000000000000|b:TRUE" $kb

    # Matrix args: shape + row-major bits digest, default result_index @1,1.
    $m = @{ kind = "matrix"; rows = 2; cols = 2; values = @(@(1.0, 2.0), @(3.0, 4.0)) }
    $km1 = Get-OracleCacheKey -Request @{ function_name = "MINVERSE"; args = @($m) }
    Assert-Equal "matrix key has shape prefix" $true ($km1 -like "MINVERSE|m2x2:*@1,1")
    # Same values flat row-major = same key; transposed values = different key.
    $mFlat = @{ kind = "matrix"; rows = 2; cols = 2; values = @(1.0, 2.0, 3.0, 4.0) }
    $km2 = Get-OracleCacheKey -Request @{ function_name = "MINVERSE"; args = @($mFlat) }
    Assert-Equal "jagged and flat row-major agree" $km1 $km2
    $mT = @{ kind = "matrix"; rows = 2; cols = 2; values = @(1.0, 3.0, 2.0, 4.0) }
    $km3 = Get-OracleCacheKey -Request @{ function_name = "MINVERSE"; args = @($mT) }
    Assert-Equal "transposed matrix differs" $false ($km1 -eq $km3)
    # result_index distinguishes cells of one spilled result.
    $km4 = Get-OracleCacheKey -Request @{ function_name = "MINVERSE"; args = @($m); result_index = @(2, 1) }
    Assert-Equal "result_index in key" $true ($km4 -like "MINVERSE|m2x2:*@2,1")
    Assert-Equal "result_index changes key" $false ($km1 -eq $km4)

    Write-Host "Get-OracleAnswers dedup + persistence"
    # Mock invoker: answers f(x) = bits-tagged echo, counts every candidate row.
    $script:askedRows = 0
    $script:askedBatches = 0
    $mockInvoker = {
        param($candidates)
        $candidateCount = @($candidates).Count
        $script:askedRows += $candidateCount
        if ($candidateCount -gt 0) { $script:askedBatches += 1 }
        $outcomes = @(@($candidates) | ForEach-Object {
            $v = [double] @($_.args)[0] * 2.0
            [ordered]@{ kind = "number"; value = $v; bits_hex = (Get-TestBitsHex $v); digest_payload = "number:$(Get-TestBitsHex $v)" }
        })
        return [ordered]@{
            blocked = $false
            outcomes = $outcomes
            environment = [ordered]@{
                excel_version = "16.0"
                excel_build = "test"
                excel_bitness = "64-bit"
                workbook_compatibility = "test"
                cpu_id = "test-cpu"
                excel_input_plumbing = "mock"
            }
        }
    }

    # Six requests, three distinct: only three rows may reach the invoker.
    $reqs = @(
        @{ function_name = "DOUBLE"; args = @(1.0) },
        @{ function_name = "DOUBLE"; args = @(2.0) },
        @{ function_name = "DOUBLE"; args = @(1.0) },
        @{ function_name = "DOUBLE"; args = @(3.0) },
        @{ function_name = "DOUBLE"; args = @(2.0) },
        @{ function_name = "DOUBLE"; args = @(1.0) }
    )
    $r1 = Get-OracleAnswers -Requests $reqs -Invoker $mockInvoker
    Assert-Equal "not blocked" $false $r1.blocked
    Assert-Equal "answers parallel to requests" 6 (@($r1.answers).Count)
    Assert-Equal "duplicates asked once (rows)" 3 $script:askedRows
    Assert-Equal "answer order preserved (r0)" 2 $r1.answers[0].outcome.value
    Assert-Equal "answer order preserved (r3)" 6 $r1.answers[3].outcome.value
    Assert-Equal "duplicate got same outcome" $r1.answers[0].outcome.bits_hex $r1.answers[5].outcome.bits_hex
    Assert-Equal "first ask not from_cache" $false $r1.answers[0].from_cache

    # Same requests again: zero invoker traffic, all from cache.
    $r2 = Get-OracleAnswers -Requests $reqs -Invoker $mockInvoker
    Assert-Equal "second call asks nothing" 3 $script:askedRows
    Assert-Equal "second call all from_cache" $true ((@($r2.answers) | ForEach-Object { $_.from_cache }) -notcontains $false)
    Assert-Equal "second call outcome stable" $r1.answers[1].outcome.bits_hex $r2.answers[1].outcome.bits_hex

    # Persistence: a fresh module session must serve everything from disk.
    Initialize-OracleCache -CacheRoot $tempRoot | Out-Null
    $r3 = Get-OracleAnswers -Requests $reqs -Invoker $mockInvoker
    Assert-Equal "fresh session reads shard (no asks)" 3 $script:askedRows
    Assert-Equal "fresh session outcome bits stable" $r1.answers[0].outcome.bits_hex $r3.answers[0].outcome.bits_hex
    $stats = Get-OracleCacheStats
    Assert-Equal "stats records" 3 $stats.records

    # Batch splitting: 5 new distinct requests at BatchSize 2 -> 3 batches.
    $script:askedRows = 0
    $script:askedBatches = 0
    $reqsB = @(10.0, 11.0, 12.0, 13.0, 14.0 | ForEach-Object { @{ function_name = "DOUBLE"; args = @($_) } })
    $rB = Get-OracleAnswers -Requests $reqsB -Invoker $mockInvoker -BatchSize 2
    Assert-Equal "batched rows" 5 $script:askedRows
    Assert-Equal "batched batches" 3 $script:askedBatches
    Assert-Equal "batched answers" 5 (@($rB.answers).Count)

    # Blocked invoker propagates without caching anything.
    $blockedInvoker = { param($candidates) [ordered]@{ blocked = $true; blocker = "no excel"; outcomes = @(); environment = @{} } }
    $rBlocked = Get-OracleAnswers -Requests @(@{ function_name = "DOUBLE"; args = @(99.0) }) -Invoker $blockedInvoker
    Assert-Equal "blocked propagates" $true $rBlocked.blocked
    $script:askedRows = 0
    $rRetry = Get-OracleAnswers -Requests @(@{ function_name = "DOUBLE"; args = @(99.0) }) -Invoker $mockInvoker
    Assert-Equal "blocked row was not cached" 1 $script:askedRows

    Write-Host "Environment mismatch fails closed"
    $mismatchInvoker = {
        param($candidates)
        [ordered]@{
            blocked = $false
            outcomes = @()
            environment = [ordered]@{
                excel_version = "16.0"
                excel_build = "other-build"
                excel_bitness = "64-bit"
                workbook_compatibility = "test"
                cpu_id = "test-cpu"
                excel_input_plumbing = "mock"
            }
        }
    }
    $shardPath = Join-Path $tempRoot "build-test\DOUBLE.jsonl"
    $linesBeforeMismatch = @(Get-Content $shardPath).Count
    $rMismatch = Get-OracleAnswers -Requests @(@{ function_name = "DOUBLE"; args = @(1.0) }) -Invoker $mismatchInvoker
    Assert-Equal "mismatched cache use is blocked" $true $rMismatch.blocked
    Assert-Equal "mismatch names Excel build" $true ($rMismatch.blocker -like "*excel_build pinned='test' active='other-build'*")
    Assert-Equal "mismatch does not append shard" $linesBeforeMismatch @(Get-Content $shardPath).Count

    # Legacy manifests lack bitness and CompatibilityVersion. They are not
    # silently upgraded because those axes cannot be reconstructed for old rows.
    $legacyRoot = Join-Path $tempRoot "legacy"
    New-Item -ItemType Directory -Force -Path $legacyRoot | Out-Null
    [ordered]@{ excel_version = "16.0"; excel_build = "test"; cpu_id = "test-cpu" } |
        ConvertTo-Json | Set-Content (Join-Path $legacyRoot "env.json") -Encoding utf8NoBOM
    Initialize-OracleCache -CacheRoot $legacyRoot | Out-Null
    $rLegacy = Get-OracleAnswers -Requests @(@{ function_name = "DOUBLE"; args = @(1.0) }) -Invoker $mockInvoker
    Assert-Equal "legacy manifest fails closed" $true $rLegacy.blocked
    Assert-Equal "legacy failure gives migration action" $true ($rLegacy.blocker -like "*legacy/unsupported*new CacheRoot*")

    $schemaLessRoot = Join-Path $tempRoot "schema-less"
    New-Item -ItemType Directory -Force -Path $schemaLessRoot | Out-Null
    $testEnv | ConvertTo-Json | Set-Content (Join-Path $schemaLessRoot "env.json") -Encoding utf8NoBOM
    Initialize-OracleCache -CacheRoot $schemaLessRoot | Out-Null
    $rSchemaLess = Get-OracleAnswers -Requests @(@{ function_name = "DOUBLE"; args = @(1.0) }) -Invoker $mockInvoker
    Assert-Equal "schema-less manifest fails closed" $true $rSchemaLess.blocked
    Assert-Equal "schema-less failure names required schema" $true ($rSchemaLess.blocker -like "*environment schema 'missing'*oracle-environment-v2*")
}
finally {
    if (Test-Path $tempRoot) { Remove-Item -Recurse -Force $tempRoot }
}

Write-Host ""
Write-Host "$passes/$script:totalCases passed"
if ($failures.Count -gt 0) {
    Write-Host "FAILURES:" -ForegroundColor Red
    $failures | ForEach-Object { Write-Host "  $_" -ForegroundColor Red }
    exit 1
}
exit 0
