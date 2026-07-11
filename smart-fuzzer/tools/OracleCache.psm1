# OracleCache.psm1
#
# Persistent, deduplicating Excel oracle for the W109 active model-discovery
# campaign. Wraps CellRefBatch.psm1 (bit-exact cell Value2 plumbing) with an
# append-only JSONL cache so that NO EXCEL QUESTION IS EVER ASKED TWICE.
#
# Owning workset: W109.
#
# Cache layout:
#   <CacheRoot>/env.json                    - pinned oracle environment
#                                             (excel_build, excel_version, cpu_id)
#   <CacheRoot>/build-<build>/<FUNC>.jsonl  - one shard per function, one JSON
#                                             record per answered question
#
# Record schema (one per line):
#   { schema_version, key, function_name, args, result_index?, outcome,
#     environment, recorded_utc }
#   `args` are stored with full bit fidelity: scalars carry bits_hex, matrices
#   carry row-major bits_hex lists.
#
# Public API:
#   Initialize-OracleCache -CacheRoot <path> [-Environment <hashtable>]
#     Loads (or creates) env.json and primes the in-memory index. Pass
#     -Environment to pin a synthetic environment (tests); otherwise the first
#     live batch records the real one.
#
#   Get-OracleCacheKey -Request <object>
#     Canonical cache key for a request:
#       FUNC|<argkey>|...|@r,c
#     scalar  -> f64 bits hex (0x................)
#     logical -> b:TRUE / b:FALSE
#     matrix  -> m<rows>x<cols>:sha256-16 of the row-major bits-hex join
#     result_index (matrix candidates) -> trailing @r,c (default @1,1 when any
#     matrix arg is present).
#
#   Get-OracleAnswers -Requests <object[]> [-CacheRoot <path>] [-BatchSize <n>]
#                     [-Invoker <scriptblock>]
#     Answers every request, serving cached rows first and batching only the
#     misses through Invoke-ExcelCellRefBatch (or -Invoker, for tests).
#     Duplicate requests inside one call are asked at most once. Returns
#     [ordered]@{ blocked; blocker?; answers } with answers parallel to
#     Requests: [ordered]@{ key; outcome; from_cache }.
#
#   Get-OracleCacheStats
#     [ordered]@{ cache_root; environment; functions; records; hits; misses }
#     for the current session.
#
# Requests use the CellRefBatch candidate shape:
#   function_name : string   (Excel function name, no leading "=")
#   args          : object[] (f64 | bool | @{kind="matrix"; rows; cols; values})
#   result_index  : optional @(r, c) for matrix candidates

Set-StrictMode -Version Latest

Import-Module (Join-Path $PSScriptRoot "CellRefBatch.psm1") -Force

$script:SchemaVersion = "oracle-cache-v1"
$script:CacheRoot = $null
$script:Environment = $null
$script:Index = @{}          # key -> outcome (ordered hashtable)
$script:LoadedShards = @{}   # function name -> $true once its shard is loaded
$script:Hits = 0
$script:Misses = 0

function _Get-DefaultCacheRoot {
    $repo = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path
    return Join-Path $repo "smart-fuzzer/cache/oracle"
}

function _Get-CpuId {
    try {
        $name = (Get-CimInstance Win32_Processor | Select-Object -First 1).Name
        if ([string]::IsNullOrWhiteSpace($name)) { return "unknown-cpu" }
        return ($name -replace '\s+', ' ').Trim()
    } catch {
        return "unknown-cpu"
    }
}

function _Sha256Hex16 {
    param([Parameter(Mandatory)] [string] $Text)
    $sha = [System.Security.Cryptography.SHA256]::Create()
    try {
        $bytes = $sha.ComputeHash([System.Text.Encoding]::UTF8.GetBytes($Text))
        return ([System.BitConverter]::ToString($bytes) -replace '-', '').Substring(0, 16).ToLowerInvariant()
    } finally {
        $sha.Dispose()
    }
}

function _Get-RequestProperty {
    param([object] $Request, [string] $Name)
    if ($Request -is [System.Collections.IDictionary]) {
        if ($Request.Contains($Name)) { return $Request[$Name] }
        return $null
    }
    $prop = $Request.PSObject.Properties[$Name]
    if ($null -ne $prop) { return $prop.Value }
    return $null
}

function _Get-MatrixBitsRowMajor {
    # Returns the matrix values as a flat row-major string[] of bits-hex,
    # tolerating the same three shapes CellRefBatch accepts (2D, jagged, flat).
    param([object] $MatrixArg)
    $rows = [int] $MatrixArg.rows
    $cols = [int] $MatrixArg.cols
    $vals = $MatrixArg.values
    $out = New-Object 'System.Collections.Generic.List[string]'
    if ($vals -is [System.Array] -and $vals.Rank -eq 2) {
        for ($i = 0; $i -lt $rows; $i++) {
            for ($j = 0; $j -lt $cols; $j++) {
                [void]$out.Add((Get-F64BitsHex ([double] $vals[$i, $j])))
            }
        }
    } elseif (@($vals).Count -eq $rows -and ($vals[0] -is [System.Array] -or $vals[0] -is [System.Collections.IList])) {
        for ($i = 0; $i -lt $rows; $i++) {
            $rowVals = @($vals[$i])
            for ($j = 0; $j -lt $cols; $j++) {
                [void]$out.Add((Get-F64BitsHex ([double] $rowVals[$j])))
            }
        }
    } else {
        $flat = @($vals)
        for ($i = 0; $i -lt ($rows * $cols); $i++) {
            [void]$out.Add((Get-F64BitsHex ([double] $flat[$i])))
        }
    }
    return ,$out.ToArray()
}

function Get-OracleCacheKey {
    [CmdletBinding()]
    param([Parameter(Mandatory)] [object] $Request)
    $name = ([string](_Get-RequestProperty $Request "function_name")).ToUpperInvariant()
    if ([string]::IsNullOrWhiteSpace($name)) {
        throw "OracleCache: request has no function_name"
    }
    $parts = New-Object 'System.Collections.Generic.List[string]'
    [void]$parts.Add($name)
    $hasMatrix = $false
    foreach ($a in @((_Get-RequestProperty $Request "args"))) {
        if ($a -is [System.Collections.IDictionary] -and $a.Contains("kind") -and $a.kind -eq "matrix") {
            $hasMatrix = $true
            $bits = _Get-MatrixBitsRowMajor $a
            $digest = _Sha256Hex16 ($bits -join ',')
            [void]$parts.Add("m$([int]$a.rows)x$([int]$a.cols):$digest")
        } elseif ($a -is [bool]) {
            [void]$parts.Add("b:$(([string]$a).ToUpperInvariant())")
        } else {
            [void]$parts.Add((Get-F64BitsHex ([double] $a)))
        }
    }
    if ($hasMatrix) {
        $ri = @(1, 1)
        $requested = _Get-RequestProperty $Request "result_index"
        if ($null -ne $requested) {
            $riList = @($requested)
            if ($riList.Count -ge 2) { $ri = @([int]$riList[0], [int]$riList[1]) }
        }
        [void]$parts.Add("@$($ri[0]),$($ri[1])")
    }
    return ($parts -join '|')
}

function _Get-EnvDirName {
    param([object] $Environment)
    $build = [string] $Environment.excel_build
    if ([string]::IsNullOrWhiteSpace($build)) { $build = "unknown" }
    return "build-$build"
}

function _Get-ShardPath {
    param([string] $FunctionName)
    $dir = Join-Path $script:CacheRoot (_Get-EnvDirName $script:Environment)
    return Join-Path $dir ("{0}.jsonl" -f $FunctionName.ToUpperInvariant())
}

function _Load-Shard {
    param([string] $FunctionName)
    $fn = $FunctionName.ToUpperInvariant()
    if ($script:LoadedShards.ContainsKey($fn)) { return }
    $script:LoadedShards[$fn] = $true
    $path = _Get-ShardPath $fn
    if (-not (Test-Path $path)) { return }
    foreach ($line in [System.IO.File]::ReadLines($path)) {
        if ([string]::IsNullOrWhiteSpace($line)) { continue }
        $rec = $line | ConvertFrom-Json
        if (-not $script:Index.ContainsKey($rec.key)) {
            $script:Index[$rec.key] = $rec.outcome
        }
    }
}

function _Describe-ArgsForRecord {
    param([object] $Request)
    $described = New-Object 'System.Collections.Generic.List[object]'
    foreach ($a in @((_Get-RequestProperty $Request "args"))) {
        if ($a -is [System.Collections.IDictionary] -and $a.Contains("kind") -and $a.kind -eq "matrix") {
            [void]$described.Add([ordered]@{
                kind = "matrix"
                rows = [int]$a.rows
                cols = [int]$a.cols
                bits_hex_row_major = @(_Get-MatrixBitsRowMajor $a)
            })
        } elseif ($a -is [bool]) {
            [void]$described.Add([ordered]@{ kind = "logical"; value = [bool]$a })
        } else {
            $d = [double] $a
            [void]$described.Add([ordered]@{ kind = "number"; value = $d; bits_hex = (Get-F64BitsHex $d) })
        }
    }
    return ,$described.ToArray()
}

function _Append-ShardRecords {
    # $Entries: list of @{ request; key; outcome }
    param([object[]] $Entries)
    $byFunction = @{}
    foreach ($e in $Entries) {
        $fn = ([string](_Get-RequestProperty $e.request "function_name")).ToUpperInvariant()
        if (-not $byFunction.ContainsKey($fn)) {
            $byFunction[$fn] = New-Object 'System.Collections.Generic.List[object]'
        }
        [void]$byFunction[$fn].Add($e)
    }
    $stamp = [DateTime]::UtcNow.ToString("o")
    foreach ($fn in $byFunction.Keys) {
        $path = _Get-ShardPath $fn
        $dir = Split-Path -Parent $path
        if (-not (Test-Path $dir)) { New-Item -ItemType Directory -Force -Path $dir | Out-Null }
        $lines = New-Object 'System.Collections.Generic.List[string]'
        foreach ($e in $byFunction[$fn]) {
            $record = [ordered]@{
                schema_version = $script:SchemaVersion
                key = $e.key
                function_name = $fn
                args = @(_Describe-ArgsForRecord $e.request)
                outcome = $e.outcome
                environment = $script:Environment
                recorded_utc = $stamp
            }
            $ri = _Get-RequestProperty $e.request "result_index"
            if ($null -ne $ri) { $record["result_index"] = @($ri) }
            [void]$lines.Add(($record | ConvertTo-Json -Depth 16 -Compress))
        }
        [System.IO.File]::AppendAllLines($path, $lines)
    }
}

function Initialize-OracleCache {
    [CmdletBinding()]
    param(
        [string] $CacheRoot = (_Get-DefaultCacheRoot),
        [object] $Environment = $null
    )
    $script:CacheRoot = $CacheRoot
    $script:Index = @{}
    $script:LoadedShards = @{}
    $script:Hits = 0
    $script:Misses = 0
    if (-not (Test-Path $CacheRoot)) { New-Item -ItemType Directory -Force -Path $CacheRoot | Out-Null }
    $envPath = Join-Path $CacheRoot "env.json"
    if ($null -ne $Environment) {
        $script:Environment = $Environment
        if (-not (Test-Path $envPath)) {
            ($Environment | ConvertTo-Json -Depth 4) | Set-Content -Path $envPath -Encoding utf8NoBOM
        }
    } elseif (Test-Path $envPath) {
        $script:Environment = Get-Content $envPath -Raw | ConvertFrom-Json
    } else {
        $script:Environment = $null   # pinned by the first live batch
    }
    return [ordered]@{
        cache_root = $script:CacheRoot
        environment = $script:Environment
    }
}

function _Pin-EnvironmentFromBatch {
    param([object] $BatchEnvironment)
    $pinned = [ordered]@{
        excel_version = [string] $BatchEnvironment.excel_version
        excel_build = [string] $BatchEnvironment.excel_build
        cpu_id = (_Get-CpuId)
    }
    $script:Environment = $pinned
    $envPath = Join-Path $script:CacheRoot "env.json"
    ($pinned | ConvertTo-Json -Depth 4) | Set-Content -Path $envPath -Encoding utf8NoBOM
}

function Get-OracleAnswers {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)] [object[]] $Requests,
        [string] $CacheRoot = $null,
        [int] $BatchSize = 5000,
        [scriptblock] $Invoker = $null
    )
    if ($null -ne $CacheRoot -and $CacheRoot -ne "") {
        if ($script:CacheRoot -ne $CacheRoot) { Initialize-OracleCache -CacheRoot $CacheRoot | Out-Null }
    } elseif ($null -eq $script:CacheRoot) {
        Initialize-OracleCache | Out-Null
    }
    if ($null -eq $Invoker) {
        $Invoker = { param($candidates) Invoke-ExcelCellRefBatch -Candidates $candidates }
    }

    # 1. Key every request; load shards lazily per function (only when the
    #    environment is already pinned — without a pinned env there is no shard).
    $keys = New-Object 'object[]' $Requests.Count
    for ($i = 0; $i -lt $Requests.Count; $i++) {
        $keys[$i] = Get-OracleCacheKey -Request $Requests[$i]
        if ($null -ne $script:Environment) {
            _Load-Shard ([string](_Get-RequestProperty $Requests[$i] "function_name"))
        }
    }

    # 2. Split hits from misses, deduplicating misses by key.
    $wasHit = New-Object 'bool[]' $Requests.Count
    $missByKey = [ordered]@{}   # key -> first request index
    for ($i = 0; $i -lt $Requests.Count; $i++) {
        if ($script:Index.ContainsKey($keys[$i])) {
            $script:Hits++
            $wasHit[$i] = $true
        } else {
            $script:Misses++
            if (-not $missByKey.Contains($keys[$i])) { $missByKey[$keys[$i]] = $i }
        }
    }

    # 3. Ask Excel only the deduplicated misses, in batches.
    if ($missByKey.Count -gt 0) {
        $missKeys = @($missByKey.Keys)
        $cursor = 0
        while ($cursor -lt $missKeys.Count) {
            $take = [Math]::Min($BatchSize, $missKeys.Count - $cursor)
            $slice = @($missKeys[$cursor..($cursor + $take - 1)])
            $candidates = @($slice | ForEach-Object { $Requests[$missByKey[$_]] })
            $batch = & $Invoker $candidates
            if ($batch.blocked) {
                return [ordered]@{ blocked = $true; blocker = $batch.blocker; answers = @() }
            }
            if ($null -eq $script:Environment) {
                _Pin-EnvironmentFromBatch $batch.environment
                # Now that the env is pinned, existing shards become reachable;
                # they cannot contain these keys (we just missed), so no reload
                # of already-answered functions is needed for this batch.
            }
            $outcomes = @($batch.outcomes)
            if ($outcomes.Count -ne $slice.Count) {
                return [ordered]@{
                    blocked = $true
                    blocker = "OracleCache: invoker returned $($outcomes.Count) outcomes for $($slice.Count) candidates"
                    answers = @()
                }
            }
            $newEntries = New-Object 'System.Collections.Generic.List[object]'
            for ($j = 0; $j -lt $slice.Count; $j++) {
                $script:Index[$slice[$j]] = $outcomes[$j]
                [void]$newEntries.Add(@{ request = $candidates[$j]; key = $slice[$j]; outcome = $outcomes[$j] })
            }
            _Append-ShardRecords $newEntries.ToArray()
            $cursor += $take
        }
    }

    # 4. Assemble answers in request order.
    $answers = New-Object 'System.Collections.Generic.List[object]'
    for ($i = 0; $i -lt $Requests.Count; $i++) {
        [void]$answers.Add([ordered]@{
            key = $keys[$i]
            outcome = $script:Index[$keys[$i]]
            from_cache = $wasHit[$i]
        })
    }
    return [ordered]@{ blocked = $false; answers = $answers.ToArray() }
}

function Get-OracleCacheStats {
    [CmdletBinding()]
    param()
    return [ordered]@{
        cache_root = $script:CacheRoot
        environment = $script:Environment
        functions = @($script:LoadedShards.Keys)
        records = $script:Index.Count
        hits = $script:Hits
        misses = $script:Misses
    }
}

Export-ModuleMember -Function `
    Initialize-OracleCache, `
    Get-OracleCacheKey, `
    Get-OracleAnswers, `
    Get-OracleCacheStats
