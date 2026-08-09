[CmdletBinding(PositionalBinding = $false)]
param()

# End-to-end smoke of the W109 scheduler loop with NO Excel:
#   calc_graph_racer distinguish  ->  Run-W109ProbeBatch.ps1 (mock invoker)
#   ->  calc_graph_racer eliminate
# A fake worksheet function DOUBLE(x) = 2*x answers the probes; two candidate
# graphs (x*2 and x*3) disagree, and the loop must kill x*3.

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$scriptRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
$racerDir = Join-Path $scriptRoot "calc_graph_racer"
$work = Join-Path ([System.IO.Path]::GetTempPath()) ("w109-loop-test-" + [Guid]::NewGuid().ToString("N"))
New-Item -ItemType Directory -Force -Path $work | Out-Null

function Bits([double]$v) {
    "0x{0:x16}" -f [System.BitConverter]::ToUInt64([System.BitConverter]::GetBytes($v), 0)
}

$failures = New-Object 'System.Collections.Generic.List[string]'
function Assert-True([string]$Label, [bool]$Cond) {
    if ($Cond) { Write-Host "  PASS  $Label" }
    else { $script:failures.Add($Label) | Out-Null; Write-Host "  FAIL  $Label" -ForegroundColor Red }
}

try {
    # --- candidate set: DOUBLE(x) as x*2 (right) and x*3 (wrong) ---
    $candidates = @(
        [ordered]@{
            id = "times-2"; description = "x * 2 strict"
            graph = [ordered]@{
                nodes = @(
                    [ordered]@{ op = [ordered]@{ arg = 0 }; model = "strict" },
                    [ordered]@{ op = [ordered]@{ const = [ordered]@{ f64 = [ordered]@{ bits_hex = (Bits 2.0) } } }; model = "strict" },
                    [ordered]@{ op = [ordered]@{ mul = @(0, 1) }; model = "strict" }
                )
                output = 2
            }
        },
        [ordered]@{
            id = "times-3"; description = "x * 3 strict"
            graph = [ordered]@{
                nodes = @(
                    [ordered]@{ op = [ordered]@{ arg = 0 }; model = "strict" },
                    [ordered]@{ op = [ordered]@{ const = [ordered]@{ f64 = [ordered]@{ bits_hex = (Bits 3.0) } } }; model = "strict" },
                    [ordered]@{ op = [ordered]@{ mul = @(0, 1) }; model = "strict" }
                )
                output = 2
            }
        }
    )
    $candidatesPath = Join-Path $work "candidates.json"
    ConvertTo-Json $candidates -Depth 16 | Set-Content $candidatesPath -Encoding utf8NoBOM

    # --- probe pool: x=0 is zero-information (2x == 3x), the others split ---
    $pool = @(
        [ordered]@{ id = "p-zero"; args = @((Bits 0.0)) },
        [ordered]@{ id = "p-one-and-half"; args = @((Bits 1.5)) },
        [ordered]@{ id = "p-pi-ish"; args = @((Bits 3.14159)) }
    )
    $poolPath = Join-Path $work "pool.json"
    ConvertTo-Json $pool -Depth 8 | Set-Content $poolPath -Encoding utf8NoBOM

    # --- distinguish (offline) ---
    $batchPath = Join-Path $work "batch.json"
    cargo run --quiet --bin calc_graph_racer --manifest-path (Join-Path $racerDir "Cargo.toml") -- distinguish `
        --candidates $candidatesPath --pool $poolPath `
        --function DOUBLE --row-id TEST-00 --top 10 --out $batchPath | Out-Host
    if ($LASTEXITCODE -ne 0) { throw "distinguish failed" }
    $batch = Get-Content $batchPath -Raw | ConvertFrom-Json
    Assert-True "zero-information probe dropped" (@($batch.probes).Count -eq 2)
    Assert-True "disagreeing probes surfaced" ((@($batch.probes) | ForEach-Object { $_.probe.id }) -notcontains "p-zero")

    # --- answer through the driver with a mock DOUBLE(x)=2x invoker ---
    $answersPath = Join-Path $work "answers.json"
    $cacheRoot = Join-Path $work "oracle-cache"
    $mock = {
        param($candidatesIn)
        $outcomes = @(@($candidatesIn) | ForEach-Object {
            $v = [double]@($_.args)[0] * 2.0
            $bits = "0x{0:x16}" -f [System.BitConverter]::ToUInt64([System.BitConverter]::GetBytes($v), 0)
            [ordered]@{ kind = "number"; value = $v; bits_hex = $bits; digest_payload = "number:$bits" }
        })
        [ordered]@{ blocked = $false; outcomes = $outcomes;
            environment = [ordered]@{
                excel_version = "16.0"
                excel_build = "mock"
                excel_bitness = "64-bit"
                workbook_compatibility = "2"
                cpu_id = "test-cpu"
                excel_input_plumbing = "mock"
            } }
    }
    & (Join-Path $scriptRoot "Run-W109ProbeBatch.ps1") -Batch $batchPath -Out $answersPath -CacheRoot $cacheRoot -Invoker $mock
    $answers = Get-Content $answersPath -Raw | ConvertFrom-Json
    Assert-True "answers written for both probes" (@($answers.witnesses).Count -eq 2)
    Assert-True "answer bits are DOUBLE(1.5)=3" ((@($answers.witnesses) | Where-Object { $_.id -eq "p-one-and-half" }).expected_bits -eq (Bits 3.0))
    Assert-True "capture provenance added without changing witnesses" ($answers.capture_provenance.schema_version -eq "w109-capture-provenance-v1")
    Assert-True "capture identifies cache mode" ($answers.capture_provenance.oracle_cache.mode -eq "cache")
    Assert-True "capture records Excel build" ($answers.capture_provenance.environment.excel_build -eq "mock")
    Assert-True "capture records Excel bitness" ($answers.capture_provenance.environment.excel_bitness -eq "64-bit")
    Assert-True "capture records workbook compatibility" ($answers.capture_provenance.environment.workbook_compatibility -eq "2")
    Assert-True "capture records probe runner version" ($answers.capture_provenance.runner.version -eq "w109-probe-batch-v2")

    # The bulk runner's no-cache sign-off path must persist the same optional
    # provenance envelope while leaving the WitnessSet array shape unchanged.
    $bulkAnswersPath = Join-Path $work "answers-bulk-no-cache.json"
    & (Join-Path $scriptRoot "Run-W109BulkBatch.ps1") `
        -Batch $batchPath -Out $bulkAnswersPath -NoCache -Invoker $mock
    $bulkAnswers = Get-Content $bulkAnswersPath -Raw | ConvertFrom-Json
    Assert-True "bulk no-cache preserves witnesses" (@($bulkAnswers.witnesses).Count -eq 2)
    Assert-True "bulk capture identifies no-cache mode" ($bulkAnswers.capture_provenance.oracle_cache.mode -eq "no_cache")
    Assert-True "bulk capture has no cache root" ($null -eq $bulkAnswers.capture_provenance.oracle_cache.root)
    Assert-True "bulk capture records Excel identity" (
        $bulkAnswers.capture_provenance.environment.excel_build -eq "mock" -and
        $bulkAnswers.capture_provenance.environment.excel_bitness -eq "64-bit" -and
        $bulkAnswers.capture_provenance.environment.workbook_compatibility -eq "2")
    Assert-True "bulk capture records runner version" ($bulkAnswers.capture_provenance.runner.version -eq "w109-bulk-batch-v2")

    $bulkCachedPath = Join-Path $work "answers-bulk-cached.json"
    & (Join-Path $scriptRoot "Run-W109BulkBatch.ps1") `
        -Batch $batchPath -Out $bulkCachedPath -CacheRoot $cacheRoot -Invoker $mock
    $bulkCached = Get-Content $bulkCachedPath -Raw | ConvertFrom-Json
    Assert-True "bulk cached path reuses witness array" (@($bulkCached.witnesses).Count -eq 2)
    Assert-True "bulk cached provenance records hits" (
        $bulkCached.capture_provenance.oracle_cache.mode -eq "cache" -and
        $bulkCached.capture_provenance.oracle_cache.hits -eq 2 -and
        $bulkCached.capture_provenance.oracle_cache.misses -eq 0)

    # --- eliminate ---
    $survivorsPath = Join-Path $work "survivors.json"
    $eliminatedPath = Join-Path $work "eliminated.jsonl"
    cargo run --quiet --bin calc_graph_racer --manifest-path (Join-Path $racerDir "Cargo.toml") -- eliminate `
        --candidates $candidatesPath --answers $answersPath `
        --survivors $survivorsPath --eliminated $eliminatedPath | Out-Host
    if ($LASTEXITCODE -ne 0) { throw "eliminate failed" }
    $alive = Get-Content $survivorsPath -Raw | ConvertFrom-Json
    Assert-True "one survivor" (@($alive).Count -eq 1)
    Assert-True "survivor is x*2" (@($alive)[0].id -eq "times-2")
    Assert-True "kill ledger written" (Test-Path $eliminatedPath)
    $kill = Get-Content $eliminatedPath | Select-Object -First 1 | ConvertFrom-Json
    Assert-True "kill names x*3" ($kill.candidate_id -eq "times-3")
    Assert-True "kill carries the witness bits" ($kill.expected_bits -eq (Bits 3.0))
}
finally {
    if (Test-Path $work) { Remove-Item -Recurse -Force $work }
}

Write-Host ""
if ($failures.Count -gt 0) {
    Write-Host "FAILURES: $($failures -join '; ')" -ForegroundColor Red
    exit 1
}
Write-Host "scheduler loop smoke: all green"
exit 0
