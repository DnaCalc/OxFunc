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
#                                             (version/build/bitness/workbook
#                                              compatibility/cpu identity)
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
#     live environment handshake records the real one.
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
#     Before loading shards, calls the invoker with an empty candidate array
#     and validates version/build/bitness/workbook compatibility against
#     env.json. A mismatch or incomplete legacy manifest fails closed.
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
$script:EnvironmentSchemaVersion = "oracle-environment-v2"
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
        $name = (Get-CimInstance Win32_Processor -ErrorAction Stop | Select-Object -First 1).Name
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

function _Get-EnvironmentProperty {
    param([object] $Environment, [string] $Name)
    if ($null -eq $Environment) { return $null }
    if ($Environment -is [System.Collections.IDictionary]) {
        if ($Environment.Contains($Name)) { return $Environment[$Name] }
        return $null
    }
    $prop = $Environment.PSObject.Properties[$Name]
    if ($null -ne $prop) { return $prop.Value }
    return $null
}

function _Normalize-ExcelBitness {
    param([object] $Bitness, [object] $OperatingSystem)
    $text = [string] $Bitness
    if ([string]::IsNullOrWhiteSpace($text) -or $text -eq "unknown") { $text = [string] $OperatingSystem }
    if ($text -match '(?i)64[ -]?bit|\bx64\b|amd64') { return "64-bit" }
    if ($text -match '(?i)32[ -]?bit|\bx86\b') { return "32-bit" }
    return $(if ([string]::IsNullOrWhiteSpace($text)) { "" } else { $text.Trim() })
}

function _Normalize-Environment {
    param([object] $Environment, [switch] $AddCpuId)
    $os = [string] (_Get-EnvironmentProperty $Environment "excel_operating_system")
    if ([string]::IsNullOrWhiteSpace($os)) {
        $os = [string] (_Get-EnvironmentProperty $Environment "operating_system")
    }
    $compatibility = [string] (_Get-EnvironmentProperty $Environment "workbook_compatibility")
    if ([string]::IsNullOrWhiteSpace($compatibility)) {
        $compatibility = [string] (_Get-EnvironmentProperty $Environment "workbook_compatibility_version")
    }
    $cpuId = [string] (_Get-EnvironmentProperty $Environment "cpu_id")
    if ($AddCpuId -and [string]::IsNullOrWhiteSpace($cpuId)) { $cpuId = _Get-CpuId }

    $normalized = [ordered]@{
        environment_schema = $script:EnvironmentSchemaVersion
        excel_version = ([string] (_Get-EnvironmentProperty $Environment "excel_version")).Trim()
        excel_build = ([string] (_Get-EnvironmentProperty $Environment "excel_build")).Trim()
        excel_bitness = _Normalize-ExcelBitness `
            -Bitness (_Get-EnvironmentProperty $Environment "excel_bitness") `
            -OperatingSystem $os
        workbook_compatibility = $compatibility.Trim()
        cpu_id = $(if ([string]::IsNullOrWhiteSpace($cpuId)) { "unknown-cpu" } else { $cpuId.Trim() })
    }
    $plumbing = [string] (_Get-EnvironmentProperty $Environment "excel_input_plumbing")
    if (-not [string]::IsNullOrWhiteSpace($plumbing)) { $normalized["excel_input_plumbing"] = $plumbing.Trim() }
    if (-not [string]::IsNullOrWhiteSpace($os)) { $normalized["excel_operating_system"] = $os.Trim() }
    return $normalized
}

function _Get-MissingEnvironmentFields {
    param([object] $Environment)
    $missing = New-Object 'System.Collections.Generic.List[string]'
    foreach ($field in @("excel_version", "excel_build", "excel_bitness", "workbook_compatibility")) {
        $value = [string] (_Get-EnvironmentProperty $Environment $field)
        if ([string]::IsNullOrWhiteSpace($value) -or $value -eq "unknown") { [void]$missing.Add($field) }
    }
    return $missing.ToArray()
}

function _Get-EnvironmentValidationError {
    param([object] $PinnedEnvironment, [object] $ActiveEnvironment)
    $pinnedSchema = [string] (_Get-EnvironmentProperty $PinnedEnvironment "environment_schema")
    if (-not $pinnedSchema.Equals($script:EnvironmentSchemaVersion, [StringComparison]::OrdinalIgnoreCase)) {
        $displaySchema = $(if ([string]::IsNullOrWhiteSpace($pinnedSchema)) { "missing" } else { $pinnedSchema })
        return "OracleCache: pinned env.json has legacy/unsupported environment schema '$displaySchema'; it cannot be safely validated. Use a new CacheRoot and recapture under $($script:EnvironmentSchemaVersion)."
    }
    $pinned = _Normalize-Environment -Environment $PinnedEnvironment
    $active = _Normalize-Environment -Environment $ActiveEnvironment -AddCpuId
    $pinnedMissing = @(_Get-MissingEnvironmentFields $pinned)
    if ($pinnedMissing.Count -gt 0) {
        return "OracleCache: pinned env.json is legacy/incomplete (missing $($pinnedMissing -join ', ')); it cannot be safely validated. Use a new CacheRoot and recapture under $($script:EnvironmentSchemaVersion)."
    }
    $activeMissing = @(_Get-MissingEnvironmentFields $active)
    if ($activeMissing.Count -gt 0) {
        return "OracleCache: active Excel environment is incomplete (missing $($activeMissing -join ', ')); refusing cache reads and writes because identity cannot be validated."
    }

    $mismatches = New-Object 'System.Collections.Generic.List[string]'
    foreach ($field in @("excel_version", "excel_build", "excel_bitness", "workbook_compatibility")) {
        $p = ([string] (_Get-EnvironmentProperty $pinned $field)).Trim()
        $a = ([string] (_Get-EnvironmentProperty $active $field)).Trim()
        if (-not $p.Equals($a, [StringComparison]::OrdinalIgnoreCase)) {
            [void]$mismatches.Add("$field pinned='$p' active='$a'")
        }
    }
    $pinnedCpu = ([string] (_Get-EnvironmentProperty $pinned "cpu_id")).Trim()
    $activeCpu = ([string] (_Get-EnvironmentProperty $active "cpu_id")).Trim()
    if ($pinnedCpu -and $activeCpu -and $pinnedCpu -ne "unknown-cpu" -and $activeCpu -ne "unknown-cpu" -and
        -not $pinnedCpu.Equals($activeCpu, [StringComparison]::OrdinalIgnoreCase)) {
        [void]$mismatches.Add("cpu_id pinned='$pinnedCpu' active='$activeCpu'")
    }
    if ($mismatches.Count -gt 0) {
        return "OracleCache: active Excel environment does not match pinned env.json: $($mismatches -join '; '). Refusing cache reads and writes; use the matching Excel profile or a new CacheRoot."
    }
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
        $provided = _Normalize-Environment -Environment $Environment -AddCpuId
        $providedMissing = @(_Get-MissingEnvironmentFields $provided)
        if ($providedMissing.Count -gt 0) {
            throw "OracleCache: supplied environment is incomplete (missing $($providedMissing -join ', '))"
        }
        if (Test-Path $envPath) {
            $existing = Get-Content $envPath -Raw | ConvertFrom-Json
            $validationError = _Get-EnvironmentValidationError -PinnedEnvironment $existing -ActiveEnvironment $provided
            if ($null -ne $validationError) { throw $validationError }
            $script:Environment = _Normalize-Environment -Environment $existing
        } else {
            $script:Environment = $provided
            ($provided | ConvertTo-Json -Depth 4) | Set-Content -Path $envPath -Encoding utf8NoBOM
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
    $pinned = _Normalize-Environment -Environment $BatchEnvironment -AddCpuId
    $missing = @(_Get-MissingEnvironmentFields $pinned)
    if ($missing.Count -gt 0) {
        throw "OracleCache: cannot pin incomplete active environment (missing $($missing -join ', '))"
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
        [scriptblock] $Invoker = $null,
        # Optional test/embedding seam. Production callers omit this so the
        # invoker is called with an empty candidate array to discover the live
        # environment before ANY cache shard is loaded.
        [object] $ActiveEnvironment = $null
    )
    if ($null -ne $CacheRoot -and $CacheRoot -ne "") {
        if ($script:CacheRoot -ne $CacheRoot) { Initialize-OracleCache -CacheRoot $CacheRoot | Out-Null }
    } elseif ($null -eq $script:CacheRoot) {
        Initialize-OracleCache | Out-Null
    }
    if ($null -eq $Invoker) {
        $Invoker = { param($candidates) Invoke-ExcelCellRefBatch -Candidates $candidates }
    }

    # Cache identity is validated before shard loading, hit classification, or
    # append. An empty invoker call is an environment handshake, not an Excel
    # worksheet question. This intentionally starts Excel even for an all-hit
    # run: serving an unvalidated hit is the corruption mode this guard closes.
    if ($null -eq $ActiveEnvironment) {
        try {
            $environmentProbe = & $Invoker ([object[]]::new(0))
        } catch {
            return [ordered]@{
                blocked = $true
                blocker = "OracleCache: environment probe failed before cache access: $($_.Exception.Message)"
                answers = @()
            }
        }
        if ($null -eq $environmentProbe) {
            return [ordered]@{ blocked = $true; blocker = "OracleCache: environment probe returned no result"; answers = @() }
        }
        if ($environmentProbe.blocked) {
            return [ordered]@{ blocked = $true; blocker = "OracleCache: environment probe blocked: $($environmentProbe.blocker)"; answers = @() }
        }
        $ActiveEnvironment = $environmentProbe.environment
    }
    $normalizedActive = _Normalize-Environment -Environment $ActiveEnvironment -AddCpuId
    if ($null -eq $script:Environment) {
        try {
            _Pin-EnvironmentFromBatch $normalizedActive
        } catch {
            return [ordered]@{ blocked = $true; blocker = $_.Exception.Message; answers = @() }
        }
    } else {
        $validationError = _Get-EnvironmentValidationError `
            -PinnedEnvironment $script:Environment -ActiveEnvironment $normalizedActive
        if ($null -ne $validationError) {
            return [ordered]@{ blocked = $true; blocker = $validationError; answers = @() }
        }
        $script:Environment = _Normalize-Environment -Environment $script:Environment
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
            $batchValidationError = _Get-EnvironmentValidationError `
                -PinnedEnvironment $script:Environment -ActiveEnvironment $batch.environment
            if ($null -ne $batchValidationError) {
                return [ordered]@{
                    blocked = $true
                    blocker = "OracleCache: invoker environment changed during capture. $batchValidationError"
                    answers = @()
                }
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
    return [ordered]@{
        blocked = $false
        answers = $answers.ToArray()
        # Provenance describes the active runner/session, while env.json and
        # cache records retain the pinned identity. Plumbing may legitimately
        # differ between the interchangeable cell-ref and bulk runners.
        environment = $normalizedActive
    }
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
