[CmdletBinding(PositionalBinding = $false)]
param(
    # Probe batch emitted by `calc_graph_racer distinguish` (ProbeBatch JSON).
    [Parameter(Mandatory)] [string] $Batch,
    # Output path: an answered WitnessSet JSON for `calc_graph_racer eliminate`.
    [Parameter(Mandatory)] [string] $Out,
    [string] $CacheRoot = $null,
    [int] $BatchSize = 5000,
    # Test seam: forwarded to Get-OracleAnswers in place of live Excel COM.
    [scriptblock] $Invoker = $null
)

# W109 scheduler <-> Excel bridge. Reads the racer's distinguishing-probe
# batch, answers every probe through the persistent OracleCache (so re-runs
# and overlapping batches never re-ask Excel), and writes the answers back in
# the racer's WitnessSet format:
#
#   { "function": "...", "witnesses": [ { "id", "args", "expected_bits" } ] }
#
# Probe args are exact-bits hex; scalars go through cell Value2 plumbing as
# doubles, arrays as single-column matrix ranges. Non-numeric Excel outcomes
# are recorded as "error:<code>" / "text:..." / "logical:..." in
# expected_bits — the racer treats those rows as unanswerable for numeric
# candidates (they never kill), and they stay visible for structural triage.

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$scriptRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
Import-Module (Join-Path $scriptRoot "OracleCache.psm1") -Force

function ConvertFrom-BitsHex {
    param([Parameter(Mandatory)] [string] $Hex)
    if (-not ($Hex -match '^0x[0-9a-fA-F]{16}$')) {
        throw "bad bits hex '$Hex'"
    }
    $bits = [uint64]::Parse($Hex.Substring(2), [Globalization.NumberStyles]::HexNumber)
    return [System.BitConverter]::ToDouble([System.BitConverter]::GetBytes($bits), 0)
}

function ConvertTo-OracleRequest {
    param([Parameter(Mandatory)] [string] $FunctionName, [Parameter(Mandatory)] [object] $Probe)
    $args = New-Object 'System.Collections.Generic.List[object]'
    foreach ($a in @($Probe.args)) {
        if ($a -is [System.Array] -or $a -is [System.Collections.IList]) {
            $values = @(@($a) | ForEach-Object { ConvertFrom-BitsHex ([string]$_) })
            [void]$args.Add(@{ kind = "matrix"; rows = $values.Count; cols = 1; values = $values })
        } else {
            [void]$args.Add((ConvertFrom-BitsHex ([string]$a)))
        }
    }
    return @{ function_name = $FunctionName; args = $args.ToArray() }
}

$batchDoc = Get-Content $Batch -Raw | ConvertFrom-Json
$probes = @($batchDoc.probes)
if ($probes.Count -eq 0) {
    Write-Host "probe batch is empty; nothing to ask"
    $answered = [ordered]@{ function = [string]$batchDoc.function; witnesses = @() }
    $answered | ConvertTo-Json -Depth 16 | Set-Content -Path $Out -Encoding utf8NoBOM
    exit 0
}

$functionName = [string]$batchDoc.function
$requests = @($probes | ForEach-Object { ConvertTo-OracleRequest -FunctionName $functionName -Probe $_.probe })

$oracleArgs = @{ Requests = $requests; BatchSize = $BatchSize }
if ($CacheRoot) { $oracleArgs.CacheRoot = $CacheRoot }
if ($null -ne $Invoker) { $oracleArgs.Invoker = $Invoker }
$result = Get-OracleAnswers @oracleArgs
if ($result.blocked) {
    Write-Error "oracle blocked: $($result.blocker)"
    exit 1
}

$witnesses = New-Object 'System.Collections.Generic.List[object]'
for ($i = 0; $i -lt $probes.Count; $i++) {
    $outcome = $result.answers[$i].outcome
    $expected = switch ([string]$outcome.kind) {
        "number" { [string]$outcome.bits_hex }
        "error" { "error:$($outcome.code)" }
        "logical" { "logical:$($outcome.value)" }
        default { "text:$($outcome.value)" }
    }
    [void]$witnesses.Add([ordered]@{
        id = [string]$probes[$i].probe.id
        args = $probes[$i].probe.args
        expected_bits = $expected
    })
}

$answered = [ordered]@{
    function = $functionName
    witnesses = $witnesses.ToArray()
}
$outDir = Split-Path -Parent $Out
if ($outDir -and -not (Test-Path $outDir)) { New-Item -ItemType Directory -Force -Path $outDir | Out-Null }
$answered | ConvertTo-Json -Depth 16 | Set-Content -Path $Out -Encoding utf8NoBOM

$stats = Get-OracleCacheStats
Write-Host ("answered {0} probes for {1} (cache hits {2}, misses {3}) -> {4}" -f `
    $probes.Count, $functionName, $stats.hits, $stats.misses, $Out)
