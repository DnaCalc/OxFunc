[CmdletBinding(PositionalBinding = $false)]
param(
    # Probe batch emitted by `calc_graph_racer distinguish` (ProbeBatch JSON).
    [Parameter(Mandatory)] [string] $Batch,
    # Output path: an answered WitnessSet JSON for `calc_graph_racer eliminate`.
    [Parameter(Mandatory)] [string] $Out,
    [string] $CacheRoot = $null,
    # Slice size handed to the bulk engine per Get-OracleAnswers batch (cached path).
    [int] $BatchSize = 20000,
    # Max probes written into one worksheet fill before looping (bulk engine).
    [int] $ChunkSize = 20000,
    # Validation / fresh-compute mode: bypass cache READS *and* WRITES entirely.
    # Every probe is (re)computed live in Excel, so the produced WitnessSet is a
    # pure function of the engine — this is the bit-identity validation gate.
    [switch] $NoCache,
    # Test seam: forwarded in place of the live bulk Excel engine.
    [scriptblock] $Invoker = $null
)

# ==========================================================================
# W109 BULK recalc-sheet capture engine (fast live-Excel oracle).
#
# Same CLI as Run-W109ProbeBatch.ps1 (-Batch/-Out/-CacheRoot) plus -NoCache.
# Reads the racer's distinguishing-probe batch, answers every SCALAR probe
# through a bulk recalc sheet (one workbook, one Application.Calculate per
# ~20k-row chunk), and writes the answers back in the racer's WitnessSet
# format:
#
#   { "function": "...", "witnesses": [ { "id", "args", "expected_bits" } ] }
#
# It shares ONE persistent OracleCache with Run-W109ProbeBatch.ps1: cache
# reads dedup work, cache writes are byte-identical (routed through the same
# OracleCache append path), so the two engines are interchangeable.
#
# --------------------------------------------------------------------------
# THE SCALAR-PATH BIT-IDENTITY INVARIANT (why this file exists, do not break):
#
#   Argument doubles MUST reach the Excel function through CELL REFERENCES
#   whose cells were written via Range.Value2 = <f64>. They must NEVER be
#   serialized into formula text as decimal literals.
#
#   Excel's formula parser is not correctly-rounded for long decimal literals
#   (see planning/EXCEL_RUNNER_PLUMBING_NOTE.md). "=ERF.PRECISE(0.1234567...)"
#   can silently evaluate the function at a NEIGHBOURING f64, so the captured
#   bits describe the wrong input. Value2 round-trips every double bit-exactly.
#   This engine therefore:
#     * writes all argument tuples as ONE 2D object[,] Value2 assignment
#       (rows = probes, cols = arity), and
#     * fills the result column with ONE shared RELATIVE R1C1 formula
#       =FUNC(RC[-arity],...,RC[-1]) that only ever REFERENCES those cells.
#   No argument double is ever formatted into the formula string.
#
#   Speed comes from amortising COM round-trips (one Value2 block write, one
#   formula-fill, one Application.Calculate, one Value2 block read per chunk)
#   and reusing a single Excel instance across every chunk and batch -- NOT
#   from cheapening the input plumbing. The plumbing stays bit-exact.
#
# Scope: SCALAR args only (covers the W109 batteries). Matrix-arg probes are
# detected and delegated to the legacy Invoke-ExcelCellRefBatch engine so the
# answer stays correct; the bulk fast path never touches them.
# ==========================================================================

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$scriptRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
Import-Module (Join-Path $scriptRoot "OracleCache.psm1") -Force
Import-Module (Join-Path $scriptRoot "CellRefBatch.psm1") -Force

# --- xlEnum constants (avoid loading the interop assembly) ---------------
$script:xlCalculationManual = -4135
$script:xlCalculationAutomatic = -4105

# --- Persistent bulk Excel session (reused across every chunk & batch) ----
$script:BulkExcel = $null
$script:BulkWorkbook = $null
$script:BulkWorksheet = $null
$script:BulkEnvironment = $null
$script:BulkStartupSeconds = 0.0    # one-off Excel instance creation cost
$script:BulkComputeSeconds = 0.0    # fill + calc + read (excludes startup)
$script:BulkProbeCount = 0          # scalar probes actually computed live

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
            $items = @($a)
            if ($items.Count -gt 0 -and ($items[0] -is [System.Array] -or $items[0] -is [System.Collections.IList])) {
                $rows = $items.Count
                $cols = @($items[0]).Count
                $values = @()
                foreach ($row in $items) {
                    $values += @(@($row) | ForEach-Object { ConvertFrom-BitsHex ([string]$_) })
                }
                [void]$args.Add(@{ kind = "matrix"; rows = $rows; cols = $cols; values = $values })
            } else {
                $values = @($items | ForEach-Object { ConvertFrom-BitsHex ([string]$_) })
                [void]$args.Add(@{ kind = "matrix"; rows = $values.Count; cols = 1; values = $values })
            }
        } else {
            [void]$args.Add((ConvertFrom-BitsHex ([string]$a)))
        }
    }
    $request = @{ function_name = $FunctionName; args = $args.ToArray() }
    $hasRi = $false
    if ($Probe -is [System.Collections.IDictionary]) {
        $hasRi = $Probe.Contains("result_index")
    } else {
        $hasRi = ($Probe.PSObject.Properties.Name -contains "result_index")
    }
    if ($hasRi -and $null -ne $Probe.result_index) {
        $request.result_index = @($Probe.result_index | ForEach-Object { [int]$_ })
    }
    return $request
}

function _Rel {
    param([object] $Object)
    if ($null -ne $Object -and [System.Runtime.InteropServices.Marshal]::IsComObject($Object)) {
        [void] [System.Runtime.InteropServices.Marshal]::FinalReleaseComObject($Object)
    }
}

function _Candidate-HasMatrix {
    param([object] $Candidate)
    foreach ($a in @($Candidate.args)) {
        if ($a -is [System.Collections.IDictionary] -and $a.Contains("kind") -and $a.kind -eq "matrix") { return $true }
    }
    return $false
}

function _Get-CellFromColumn {
    # Value2 of a single-column, multi-row range is a 1-based object[R,1];
    # for a single row it collapses to a scalar. Mirror CellRefBatch's reader.
    param([object] $Values, [int] $RowIndex)
    if ($Values -is [System.Array]) {
        $lower0 = $Values.GetLowerBound(0)
        $lower1 = $Values.GetLowerBound(1)
        return $Values.GetValue($lower0 + $RowIndex, $lower1)
    }
    return $Values
}

function _Ensure-BulkExcel {
    if ($null -ne $script:BulkExcel) { return }
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    $script:BulkExcel = New-Object -ComObject Excel.Application
    try { $script:BulkExcel.Visible = $false } catch {}
    try { $script:BulkExcel.DisplayAlerts = $false } catch {}
    try { $script:BulkExcel.ScreenUpdating = $false } catch {}
    try { $script:BulkExcel.EnableEvents = $false } catch {}
    $script:BulkWorkbook = $script:BulkExcel.Workbooks.Add()
    # Manual calc so writing formulas never triggers a per-write recalc; we
    # fire exactly one Application.Calculate per chunk.
    try { $script:BulkExcel.Calculation = $script:xlCalculationManual } catch {}
    # Guard the "precision as displayed" trap: with it on, cell reads would be
    # rounded to the displayed digits and lose the low bits we are capturing.
    try { $script:BulkWorkbook.PrecisionAsDisplayed = $false } catch {}
    $script:BulkWorksheet = $script:BulkWorkbook.Worksheets.Item(1)
    $script:BulkEnvironment = [ordered]@{
        excel_version = [string] $script:BulkExcel.Version
        excel_build = $(try { [string] $script:BulkExcel.Build } catch { $null })
        workbook_compatibility = $(try { [string] $script:BulkWorkbook.CompatibilityVersion } catch { "unknown" })
        excel_input_plumbing = "cell_value2_bulk"
    }
    $sw.Stop()
    $script:BulkStartupSeconds += $sw.Elapsed.TotalSeconds
}

function _Close-BulkExcel {
    if ($null -ne $script:BulkWorkbook) { try { $script:BulkWorkbook.Close($false) } catch {} }
    if ($null -ne $script:BulkExcel) { try { $script:BulkExcel.Quit() } catch {} }
    _Rel $script:BulkWorksheet
    _Rel $script:BulkWorkbook
    _Rel $script:BulkExcel
    $script:BulkWorksheet = $null
    $script:BulkWorkbook = $null
    $script:BulkExcel = $null
    [System.GC]::Collect()
    [System.GC]::WaitForPendingFinalizers()
}

function _Invoke-BulkScalarChunk {
    # $Candidates: same function_name AND same scalar arity. Returns outcomes[]
    # parallel to $Candidates. Assumes _Ensure-BulkExcel already ran.
    param([object[]] $Candidates, [int] $Arity)
    $ws = $script:BulkWorksheet
    $rows = $Candidates.Count
    $resultCol = $Arity + 1
    $etCol = $Arity + 2
    # Single reusable whole-sheet Cells wrapper; released at chunk end so no
    # intermediate Range RCW leaks across chunks (keeps teardown zombie-free).
    $cells = $ws.Cells

    # Fresh slate: clear any prior chunk so stale formulas/values cannot leak
    # into this read or dirty the recalc.
    try { [void] $cells.ClearContents() } catch {}

    # 1) ONE Value2 block write of every argument tuple (rows x arity).
    $argArray = New-Object "object[,]" $rows, $Arity
    for ($r = 0; $r -lt $rows; $r++) {
        $argList = @($Candidates[$r].args)
        for ($c = 0; $c -lt $Arity; $c++) {
            $a = $argList[$c]
            if ($a -is [bool]) { $argArray[$r, $c] = [bool] $a }
            else { $argArray[$r, $c] = [double] $a }
        }
    }
    $tl = $cells.Item(1, 1)
    $br = $cells.Item($rows, $Arity)
    $argRange = $ws.Range($tl, $br)
    _Rel $tl; _Rel $br
    $argRange.Value2 = $argArray

    # 2) ONE relative R1C1 formula fill for the whole result column. Every
    #    arg is a RELATIVE cell reference (RC[-arity]..RC[-1]) -- no argument
    #    double is ever placed in the formula text.
    $name = [string] $Candidates[0].function_name
    $refs = @()
    for ($j = 1; $j -le $Arity; $j++) { $refs += "RC[$($j - $Arity - 1)]" }
    $formula = "=$name($($refs -join ','))"
    $rtl = $cells.Item(1, $resultCol)
    $rbr = $cells.Item($rows, $resultCol)
    $resRange = $ws.Range($rtl, $rbr)
    _Rel $rtl; _Rel $rbr
    $resRange.Formula2R1C1 = $formula

    # 3) ONE relative R1C1 fill for the ERROR.TYPE companion column (drives the
    #    exact same error classification as the legacy engine).
    $etl = $cells.Item(1, $etCol)
    $ebr = $cells.Item($rows, $etCol)
    $etRange = $ws.Range($etl, $ebr)
    _Rel $etl; _Rel $ebr
    $etRange.Formula2R1C1 = '=IF(ISERROR(RC[-1]),ERROR.TYPE(RC[-1]),"")'

    # 4) ONE calculate over the whole workbook.
    [void] $script:BulkExcel.Calculate()

    # 5) ONE Value2 block read of each column.
    $resVals = $resRange.Value2
    $etVals = $etRange.Value2

    _Rel $argRange; _Rel $resRange; _Rel $etRange; _Rel $cells

    $outcomes = New-Object 'object[]' $rows
    for ($r = 0; $r -lt $rows; $r++) {
        $value = _Get-CellFromColumn $resVals $r
        $errorType = _Get-CellFromColumn $etVals $r
        $outcomes[$r] = ConvertTo-ExcelOutcome $value $errorType
    }
    return ,$outcomes
}

function Invoke-ExcelBulkBatch {
    # Invoker contract identical to Invoke-ExcelCellRefBatch:
    #   returns [ordered]@{ blocked; blocker?; outcomes; environment }
    # with outcomes parallel to $Candidates. Reuses the persistent session
    # (does NOT quit Excel -- the caller tears it down once at the end).
    [CmdletBinding()]
    param([Parameter(Mandatory)] [object[]] $Candidates)

    if ($Candidates.Count -eq 0) {
        return [ordered]@{ blocked = $false; outcomes = @(); environment = @{ excel_input_plumbing = "cell_value2_bulk" } }
    }

    try {
        $outcomes = New-Object 'object[]' $Candidates.Count

        # Partition: scalar (fast bulk path) vs matrix (delegate to legacy).
        $scalarIdx = New-Object 'System.Collections.Generic.List[int]'
        $matrixIdx = New-Object 'System.Collections.Generic.List[int]'
        for ($i = 0; $i -lt $Candidates.Count; $i++) {
            if (_Candidate-HasMatrix $Candidates[$i]) { [void]$matrixIdx.Add($i) } else { [void]$scalarIdx.Add($i) }
        }

        # --- Matrix probes: delegate to the legacy per-cell engine ----------
        if ($matrixIdx.Count -gt 0) {
            $matrixCands = @($matrixIdx | ForEach-Object { $Candidates[$_] })
            $legacy = Invoke-ExcelCellRefBatch -Candidates $matrixCands
            if ($legacy.blocked) {
                return [ordered]@{ blocked = $true; blocker = "matrix-delegate: $($legacy.blocker)"; outcomes = @(); environment = $legacy.environment }
            }
            $legacyOutcomes = @($legacy.outcomes)
            for ($k = 0; $k -lt $matrixIdx.Count; $k++) { $outcomes[$matrixIdx[$k]] = $legacyOutcomes[$k] }
        }

        # --- Scalar probes: group by (function_name, arity), chunk, bulk-run -
        if ($scalarIdx.Count -gt 0) {
            _Ensure-BulkExcel
            $groups = [ordered]@{}   # "name|arity" -> List[int]
            foreach ($i in $scalarIdx) {
                $arity = @($Candidates[$i].args).Count
                $gkey = "$([string]$Candidates[$i].function_name)|$arity"
                if (-not $groups.Contains($gkey)) { $groups[$gkey] = New-Object 'System.Collections.Generic.List[int]' }
                [void]$groups[$gkey].Add($i)
            }
            foreach ($gkey in $groups.Keys) {
                $idxList = $groups[$gkey]
                $arity = [int]($gkey.Split('|')[-1])
                $cursor = 0
                while ($cursor -lt $idxList.Count) {
                    $take = [Math]::Min($ChunkSize, $idxList.Count - $cursor)
                    $sliceIdx = @($idxList[$cursor..($cursor + $take - 1)])
                    $sliceCands = @($sliceIdx | ForEach-Object { $Candidates[$_] })
                    $sw = [System.Diagnostics.Stopwatch]::StartNew()
                    $chunkOutcomes = _Invoke-BulkScalarChunk -Candidates $sliceCands -Arity $arity
                    $sw.Stop()
                    $script:BulkComputeSeconds += $sw.Elapsed.TotalSeconds
                    $script:BulkProbeCount += $take
                    for ($k = 0; $k -lt $sliceIdx.Count; $k++) { $outcomes[$sliceIdx[$k]] = $chunkOutcomes[$k] }
                    $cursor += $take
                }
            }
        }

        return [ordered]@{
            blocked = $false
            outcomes = @($outcomes)
            environment = $(if ($null -ne $script:BulkEnvironment) { $script:BulkEnvironment } else { [ordered]@{ excel_input_plumbing = "cell_value2_bulk" } })
        }
    }
    catch {
        $where = $_.ScriptStackTrace -replace "`r?`n", " | "
        return [ordered]@{ blocked = $true; blocker = "$($_.Exception.Message) @ $where"; outcomes = @(); environment = @{ excel_input_plumbing = "cell_value2_bulk" } }
    }
}

# ==========================================================================
# Main
# ==========================================================================
$batchDoc = Get-Content $Batch -Raw | ConvertFrom-Json
$probes = @($batchDoc.probes)
$functionName = [string]$batchDoc.function

if ($probes.Count -eq 0) {
    Write-Host "probe batch is empty; nothing to ask"
    $answered = [ordered]@{ function = $functionName; witnesses = @() }
    $outDir = Split-Path -Parent $Out
    if ($outDir -and -not (Test-Path $outDir)) { New-Item -ItemType Directory -Force -Path $outDir | Out-Null }
    $answered | ConvertTo-Json -Depth 16 | Set-Content -Path $Out -Encoding utf8NoBOM
    exit 0
}

$requests = @($probes | ForEach-Object { ConvertTo-OracleRequest -FunctionName $functionName -Probe $_.probe })

if ($null -eq $Invoker) {
    $Invoker = { param($candidates) Invoke-ExcelBulkBatch -Candidates $candidates }
}

$outcomesByIndex = $null
$totalSw = [System.Diagnostics.Stopwatch]::StartNew()
try {
    if ($NoCache) {
        # Fresh-compute / validation mode: no cache read, no cache write.
        $bulk = & $Invoker $requests
        if ($bulk.blocked) { Write-Error "bulk engine blocked: $($bulk.blocker)"; exit 1 }
        $outcomesByIndex = @($bulk.outcomes)
        if ($outcomesByIndex.Count -ne $requests.Count) {
            Write-Error "bulk engine returned $($outcomesByIndex.Count) outcomes for $($requests.Count) probes"; exit 1
        }
    } else {
        # Cached mode: dedup via OracleCache, misses computed by the bulk
        # invoker, answers persisted into the SHARED cache.
        $oracleArgs = @{ Requests = $requests; BatchSize = $BatchSize; Invoker = $Invoker }
        if ($CacheRoot) { $oracleArgs.CacheRoot = $CacheRoot }
        $result = Get-OracleAnswers @oracleArgs
        if ($result.blocked) { Write-Error "oracle blocked: $($result.blocker)"; exit 1 }
        $outcomesByIndex = @($result.answers | ForEach-Object { $_.outcome })
    }
}
finally {
    _Close-BulkExcel
}
$totalSw.Stop()

$witnesses = New-Object 'System.Collections.Generic.List[object]'
for ($i = 0; $i -lt $probes.Count; $i++) {
    $outcome = $outcomesByIndex[$i]
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

# --- Throughput report ----------------------------------------------------
$computeSec = $script:BulkComputeSeconds
$startupSec = $script:BulkStartupSeconds
$liveProbes = $script:BulkProbeCount
$ppsCompute = if ($computeSec -gt 0) { [Math]::Round($liveProbes / $computeSec, 1) } else { 0 }
$ppsWithStartup = if (($computeSec + $startupSec) -gt 0) { [Math]::Round($liveProbes / ($computeSec + $startupSec), 1) } else { 0 }

if (-not $NoCache) {
    $stats = Get-OracleCacheStats
    Write-Host ("answered {0} probes for {1} (cache hits {2}, misses {3}) -> {4}" -f `
        $probes.Count, $functionName, $stats.hits, $stats.misses, $Out)
} else {
    Write-Host ("answered {0} probes for {1} (NoCache: all live) -> {2}" -f $probes.Count, $functionName, $Out)
}
Write-Host ("BULK throughput: {0} live probes | compute {1:n2}s ({2}/s) | +excel startup {3:n2}s ({4}/s incl startup) | total wall {5:n2}s" -f `
    $liveProbes, $computeSec, $ppsCompute, $startupSec, $ppsWithStartup, $totalSw.Elapsed.TotalSeconds)
