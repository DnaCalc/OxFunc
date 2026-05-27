[CmdletBinding(PositionalBinding = $false)]
param(
    [string] $RepoRoot,
    [string] $OutputCsv,
    [string] $OutputMarkdown
)

# Build a per-surface OxFunc-vs-Excel status map across the 517 in-scope
# functions and operators (CHARTER.md §4.1).
#
# Sources joined:
#   1. docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1.csv
#        — the 534 published rows (511 functions + 23 operators).
#   2. docs/function-lane/W50_DEFERRED_CURRENT_VERSION_INVENTORY.csv
#        — the 17 deferred (service/cube/pivot/TRANSLATE) rows to exclude.
#   3. docs/bugs/BUG_STREAM_REGISTER.csv plus per-stream curated function
#        lists below — for open BUG-FUNC-* streams and severity hints.
#   4. smart-fuzzer/runs/*/rollup.json (array_rollup schema only) — for
#        observed per-function coverage and classification distribution.
#
# Per-surface status (one of):
#   deferred              — in W050 deferred list.
#   structural_bug_open   — covered by an open BUG-FUNC stream tagged
#                           structural (kind/error/shape/array-lift).
#   numeric_drift_open    — covered by an open BUG-FUNC stream tagged
#                           numeric (any ULP magnitude — still a bug,
#                           CHARTER §4.1).
#   bit_exact_observed    — appears in ≥1 array_rollup run and never
#                           classified as anything but exact bit match.
#   mixed_or_open         — appears in runs and has non-match rows but
#                           no open BUG-FUNC stream — needs triage.
#   unswept               — does not appear in any array_rollup run and
#                           no open BUG-FUNC stream targets it.
#
# The map is reproducible: regenerate by re-running this script. It is
# not semantic authority and should not be treated as evidence of
# bit-exact parity beyond the rows actually observed.

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

if ([string]::IsNullOrWhiteSpace($RepoRoot)) {
    $scriptRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
    $RepoRoot = (Resolve-Path (Join-Path $scriptRoot "..\..")).Path
}
$RepoRoot = [System.IO.Path]::GetFullPath($RepoRoot)

if ([string]::IsNullOrWhiteSpace($OutputCsv)) {
    $OutputCsv = Join-Path $RepoRoot "smart-fuzzer\cache\function-status-map-v0.csv"
}
if ([string]::IsNullOrWhiteSpace($OutputMarkdown)) {
    $OutputMarkdown = Join-Path $RepoRoot "smart-fuzzer\planning\FUNCTION_STATUS_MAP.md"
}

# --------------------------------------------------------------------------
# Curated per-stream function lists.
#
# For each non-closed BUG-FUNC-* stream we list the canonical surface
# names it covers and a severity tag (structural vs numeric). For broad
# streams (BUG-FUNC-021 statistical, BUG-FUNC-027 broad scalar) the list
# is the explicit witness families recorded in the stream record. Where
# a stream is handed_off to a sibling repo (OxFml seam, publication
# seam) we still record it but note ownership_class = seam.
#
# Severity tags echo CHARTER §4.1:
#   "structural" — kind / error_code / shape / array-lift / publication.
#   "numeric"    — float drift (1 or >1 ULP — both bugs).
# --------------------------------------------------------------------------
$StreamCoverage = @(
    @{ stream = "BUG-FUNC-001"; severity = "structural"; ownership = "seam";    surfaces = @() },
    @{ stream = "BUG-FUNC-002"; severity = "structural"; ownership = "seam";    surfaces = @() },
    @{ stream = "BUG-FUNC-003"; severity = "structural"; ownership = "seam";    surfaces = @("OP_UNION_REF","AREAS","INDEX") },
    @{ stream = "BUG-FUNC-004"; severity = "structural"; ownership = "oxfunc"; surfaces = @("SWITCH","DAVERAGE","DCOUNT","DCOUNTA","DGET","DMAX","DMIN","DPRODUCT","DSTDEV","DSTDEVP","DSUM","DVAR","DVARP","COUNTIF","COUNTIFS","SUMIF","SUMIFS","AVERAGEIF","AVERAGEIFS","MAXIFS","MINIFS") },
    @{ stream = "BUG-FUNC-009"; severity = "numeric";    ownership = "oxfunc"; surfaces = @("RATE") },
    @{ stream = "BUG-FUNC-015"; severity = "numeric";    ownership = "oxfunc"; surfaces = @("PMT","PPMT","IPMT") },
    @{ stream = "BUG-FUNC-018"; severity = "structural"; ownership = "oxfunc"; surfaces = @("BINOMDIST","NORMDIST","COMPLEX","DOLLARFR","SWITCH","IFS","ADDRESS") },
    @{ stream = "BUG-FUNC-021"; severity = "numeric";    ownership = "oxfunc"; surfaces = @(
        # Legacy statistical names
        "BETADIST","BETAINV","CHIDIST","CHIINV","FDIST","FINV","GAMMADIST","GAMMAINV",
        "HYPGEOMDIST","NEGBINOMDIST","NORMSDIST","NORMSINV","TDIST","TINV",
        "PERCENTRANK","CONFIDENCE.T","Z.TEST","KURT","SKEW","SKEW.P",
        # Modern statistical names (sibling kernels often share drift surface)
        "BETA.DIST","BETA.INV","CHISQ.DIST","CHISQ.DIST.RT","CHISQ.INV","CHISQ.INV.RT",
        "F.DIST","F.DIST.RT","F.INV","F.INV.RT","GAMMA.DIST","GAMMA.INV",
        "HYPGEOM.DIST","NEGBINOM.DIST","NORM.S.DIST","NORM.S.INV","T.DIST","T.DIST.2T","T.DIST.RT","T.INV","T.INV.2T",
        "PERCENTRANK.EXC","PERCENTRANK.INC","CONFIDENCE","CONFIDENCE.NORM"
    ) },
    @{ stream = "BUG-FUNC-023"; severity = "numeric";    ownership = "oxfunc"; surfaces = @() },
    @{ stream = "BUG-FUNC-024"; severity = "numeric";    ownership = "oxfunc"; surfaces = @("BESSELY") },
    @{ stream = "BUG-FUNC-025"; severity = "numeric";    ownership = "oxfunc"; surfaces = @("MINVERSE") },
    @{ stream = "BUG-FUNC-026"; severity = "structural"; ownership = "seam";    surfaces = @("TAKE") },
    @{ stream = "BUG-FUNC-027"; severity = "numeric";    ownership = "oxfunc"; surfaces = @("GAMMA","GAMMALN","SINH","COSH","POWER","PERMUTATIONA","FISHERINV","MROUND","MOD","TAN","SIN","COS","COT","SEC","CSC","TANH","COTH","SECH","CSCH","ATANH","ACOTH","ACOSH","ATAN2","COMBIN","COMBINA") }
)

# --------------------------------------------------------------------------
# Load the snapshot.
# --------------------------------------------------------------------------
$snapshotPath = Join-Path $RepoRoot "docs\function-lane\OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1.csv"
$snapshot = Import-Csv -LiteralPath $snapshotPath
Write-Host "Loaded snapshot: $($snapshot.Count) rows from $snapshotPath"

# --------------------------------------------------------------------------
# Load the W050 deferred list.
# --------------------------------------------------------------------------
$deferredPath = Join-Path $RepoRoot "docs\function-lane\W50_DEFERRED_CURRENT_VERSION_INVENTORY.csv"
$deferredRows = Import-Csv -LiteralPath $deferredPath
$deferredSet = New-Object 'System.Collections.Generic.HashSet[string]'
foreach ($r in $deferredRows) { [void]$deferredSet.Add([string]$r.entry_name) }
Write-Host "Loaded W050 deferred: $($deferredSet.Count) rows"

# --------------------------------------------------------------------------
# Load BUG-FUNC stream register and select non-closed streams.
# --------------------------------------------------------------------------
$streamRegisterPath = Join-Path $RepoRoot "docs\bugs\BUG_STREAM_REGISTER.csv"
$streamRows = Import-Csv -LiteralPath $streamRegisterPath
$openStreamStatus = @{}
foreach ($s in $streamRows) {
    if ([string]$s.status -eq "closed") { continue }
    $openStreamStatus[[string]$s.bug_id] = [string]$s.status
}
Write-Host "Open / handed_off / validated_local streams: $($openStreamStatus.Count)"

# --------------------------------------------------------------------------
# Build per-surface bug overlay from the curated coverage map.
# --------------------------------------------------------------------------
$bugBySurface = @{}
foreach ($entry in $StreamCoverage) {
    $stream = [string]$entry.stream
    if (-not $openStreamStatus.ContainsKey($stream)) { continue }
    foreach ($surf in @($entry.surfaces)) {
        $key = [string]$surf
        if (-not $bugBySurface.ContainsKey($key)) {
            $bugBySurface[$key] = New-Object 'System.Collections.Generic.List[object]'
        }
        [void]$bugBySurface[$key].Add([ordered]@{
            stream = $stream
            status = $openStreamStatus[$stream]
            severity = [string]$entry.severity
            ownership = [string]$entry.ownership
        })
    }
}
Write-Host "Surfaces with open bug stream link: $($bugBySurface.Count)"

# --------------------------------------------------------------------------
# Aggregate per-function coverage from rollups (array_rollup schema only).
# --------------------------------------------------------------------------
$rollups = Get-ChildItem (Join-Path $RepoRoot "smart-fuzzer\runs") -Filter rollup.json -Recurse
$coverage = @{}
$arrayRollupCount = 0
foreach ($r in $rollups) {
    $d = Get-Content $r.FullName -Raw | ConvertFrom-Json
    $schema = [string]$d.schema_version
    if ($schema -ne "oxfunc.smart_fuzzer.array_rollup.v0" -and $schema -ne "oxfunc.smart_fuzzer.array_rollup.v1") {
        continue
    }
    $arrayRollupCount += 1
    $runId = [string]$d.run_id
    $runDate = [string]$d.generated_utc
    if ([string]::IsNullOrWhiteSpace($runDate)) {
        $runDate = $r.LastWriteTimeUtc.ToString("o")
    }
    if (-not ($d.PSObject.Properties.Name -contains "by_function")) { continue }
    foreach ($prop in $d.by_function.PSObject.Properties) {
        $fnRaw = [string]$prop.Name
        $fn = $fnRaw -replace '^FUNC\.', ''
        if (-not $coverage.ContainsKey($fn)) {
            $coverage[$fn] = [ordered]@{
                runs_seen = 0
                last_seen_run = $null
                last_seen_date = $null
                last_seen_classifications = [ordered]@{}
                last_seen_non_match = $false
            }
        }
        $rec = $coverage[$fn]
        $rec.runs_seen += 1
        # Keep only the MOST RECENT observation's classification distribution per surface.
        # Older runs may reflect pre-fix state; the most recent run is the closest signal we
        # have to the live OxFunc behavior.
        if ($null -eq $rec.last_seen_date -or $runDate -gt $rec.last_seen_date) {
            $rec.last_seen_date = $runDate
            $rec.last_seen_run = $runId
            $newDist = [ordered]@{}
            $hasNonMatch = $false
            foreach ($cprop in $prop.Value.PSObject.Properties) {
                $cls = [string]$cprop.Name
                $cnt = [int]$cprop.Value
                $newDist[$cls] = $cnt
                if ($cls -ne "exact_typed_bit_match") { $hasNonMatch = $true }
            }
            $rec.last_seen_classifications = $newDist
            $rec.last_seen_non_match = $hasNonMatch
        }
    }
}
Write-Host "array_rollup runs scanned: $arrayRollupCount"
Write-Host "Surfaces with at least one observation: $(([int]$coverage.Keys.Count))"

# --------------------------------------------------------------------------
# Compose the per-surface status row.
# --------------------------------------------------------------------------
$rows = New-Object 'System.Collections.Generic.List[object]'
try {
foreach ($s in $snapshot) {
    $name = [string]$s.canonical_surface_name
    $kind = [string]$s.entry_kind
    $category = [string]$s.category
    $isDeferred = $deferredSet.Contains($name)

    $bugInfo = $null
    if ($bugBySurface.ContainsKey($name)) { $bugInfo = $bugBySurface[$name] }

    $cov = $null
    if ($coverage.ContainsKey($name)) { $cov = $coverage[$name] }

    $status = "unswept"
    $statusReason = ""
    if ($isDeferred) {
        $status = "deferred"
        $statusReason = "in W050 deferred inventory"
    } elseif ($null -ne $bugInfo) {
        $structural = $false; $numeric = $false
        foreach ($b in $bugInfo) {
            if ($b.severity -eq "structural") { $structural = $true }
            if ($b.severity -eq "numeric")    { $numeric = $true }
        }
        if ($structural) { $status = "structural_bug_open" }
        elseif ($numeric) { $status = "numeric_drift_open" }
        $streamNames = New-Object 'System.Collections.Generic.List[string]'
        foreach ($b in $bugInfo) { [void]$streamNames.Add([string]$b.stream) }
        $statusReason = "open streams: " + ($streamNames -join ",")
    } elseif ($null -ne $cov) {
        if ($cov.last_seen_non_match) {
            $status = "mixed_or_open"
            $statusReason = "most recent run had non-match rows without linked stream"
        } else {
            $status = "bit_exact_observed"
            $statusReason = "most recent run was all bit-exact (across $($cov.runs_seen) array_rollup runs)"
        }
    } else {
        $status = "unswept"
        $statusReason = "no array_rollup run observation"
    }

    $aggregatedStr = ""
    if ($null -ne $cov) {
        $parts = New-Object 'System.Collections.Generic.List[string]'
        foreach ($kvp in $cov.last_seen_classifications.GetEnumerator()) {
            [void]$parts.Add("$($kvp.Key)=$($kvp.Value)")
        }
        $aggregatedStr = $parts -join ";"
    }
    $streamStr = ""
    if ($null -ne $bugInfo) {
        $sp = New-Object 'System.Collections.Generic.List[string]'
        foreach ($b in $bugInfo) {
            [void]$sp.Add("$($b.stream)($($b.severity)/$($b.status)/$($b.ownership))")
        }
        $streamStr = $sp -join "; "
    }

    $rows.Add([ordered]@{
        canonical_surface_name = $name
        entry_kind = $kind
        category = $category
        status = $status
        status_reason = $statusReason
        runs_seen = if ($null -ne $cov) { $cov.runs_seen } else { 0 }
        last_seen_run = if ($null -ne $cov) { $cov.last_seen_run } else { "" }
        last_seen_date = if ($null -ne $cov) { $cov.last_seen_date } else { "" }
        last_run_classifications = $aggregatedStr
        linked_open_streams = $streamStr
    }) | Out-Null
}

} catch {
  Write-Host "FAILED at script line $($_.InvocationInfo.ScriptLineNumber) col $($_.InvocationInfo.OffsetInLine)"
  Write-Host "  Message: $($_.Exception.Message)"
  Write-Host "  Line:    $($_.InvocationInfo.Line)"
  Write-Host "  Surface: $name"
  throw
}

# Write CSV.
$rows | ForEach-Object { [pscustomobject]$_ } | Export-Csv -LiteralPath $OutputCsv -NoTypeInformation -Encoding UTF8
Write-Host "CSV written: $OutputCsv"

# --------------------------------------------------------------------------
# Markdown summary.
# --------------------------------------------------------------------------
$totalRows = $rows.Count
$tally = @{}
foreach ($r in $rows) {
    if (-not $tally.ContainsKey($r.status)) { $tally[$r.status] = 0 }
    $tally[$r.status] += 1
}

$mdLines = New-Object 'System.Collections.Generic.List[string]'
[void]$mdLines.Add("# OxFunc Function Status Map")
[void]$mdLines.Add("")
[void]$mdLines.Add("Generated: $((Get-Date).ToUniversalTime().ToString('o'))")
[void]$mdLines.Add("")
[void]$mdLines.Add("This map is the reproducible derived view of where each of the published OxFunc surfaces stands against the bit-exact Excel parity goal (CHARTER.md §4.1). Rebuild with ``smart-fuzzer/tools/Build-FunctionStatusMap.ps1``.")
[void]$mdLines.Add("")
[void]$mdLines.Add("Inputs joined:")
[void]$mdLines.Add("")
[void]$mdLines.Add("1. ``docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1.csv``")
[void]$mdLines.Add("2. ``docs/function-lane/W50_DEFERRED_CURRENT_VERSION_INVENTORY.csv``")
[void]$mdLines.Add("3. ``docs/bugs/BUG_STREAM_REGISTER.csv`` plus the curated stream→surface map in this script")
[void]$mdLines.Add("4. ``smart-fuzzer/runs/*/rollup.json`` (array_rollup schema)")
[void]$mdLines.Add("")
[void]$mdLines.Add("## Tally")
[void]$mdLines.Add("")
[void]$mdLines.Add("| Status | Count | Meaning |")
[void]$mdLines.Add("| --- | ---: | --- |")
$statusOrder = @("deferred","structural_bug_open","numeric_drift_open","mixed_or_open","bit_exact_observed","unswept")
$statusMeaning = @{
    "deferred"            = "in W050; not part of the 517 in-scope rows"
    "structural_bug_open" = "open BUG-FUNC stream with structural severity (kind/error/shape/array-lift)"
    "numeric_drift_open"  = "open BUG-FUNC stream with numeric drift severity (1 or >1 ULP)"
    "mixed_or_open"       = "covered by runs and produced non-match rows but no linked stream"
    "bit_exact_observed"  = "covered by ≥1 array_rollup run and never produced a non-match row"
    "unswept"             = "never observed in an array_rollup run and no open stream targets it"
}
foreach ($k in $statusOrder) {
    $count = if ($tally.ContainsKey($k)) { $tally[$k] } else { 0 }
    [void]$mdLines.Add("| ``$k`` | $count | $($statusMeaning[$k]) |")
}
[void]$mdLines.Add("| **total** | $totalRows | published snapshot rows |")
[void]$mdLines.Add("")

[void]$mdLines.Add("## Caveats")
[void]$mdLines.Add("")
[void]$mdLines.Add("1. ``bit_exact_observed`` is **not** a closure claim. It only means that across the sampled invocation rows in array_rollup runs, no non-match was seen. Unswept axes (locale, alternate version, broader array, edge values not in the manifest seeds) remain unexplored on those surfaces.")
[void]$mdLines.Add("2. The stream→surface mapping is curated from the open BUG-FUNC stream titles and W097 records. Broad streams like BUG-FUNC-021 (statistical) and BUG-FUNC-027 (broad scalar) list explicit witness families; non-listed surfaces that share a kernel family may still be affected.")
[void]$mdLines.Add("3. Run-BroadScalarExploration cycles (broad_scalar_run_rollup schema) are not summarized here — per-function detail lives in their comparisons.jsonl and failure_packets/. Their findings flow into the bug-stream column instead.")
[void]$mdLines.Add("4. The status uses the CHARTER §4.1 severity vocabulary. A ``numeric_drift_open`` surface is still a bug — ``excel_imprecision_witness`` rows remain in the numeric-drift bug count, not outside it.")
[void]$mdLines.Add("")

# Per-status detail (compact).
foreach ($k in @("structural_bug_open","numeric_drift_open","mixed_or_open","unswept","bit_exact_observed","deferred")) {
    $list = @($rows | Where-Object { $_.status -eq $k } | Sort-Object canonical_surface_name)
    [void]$mdLines.Add("## $k ($($list.Count))")
    [void]$mdLines.Add("")
    if ($list.Count -eq 0) {
        [void]$mdLines.Add("_(none)_")
        [void]$mdLines.Add("")
        continue
    }
    if ($k -eq "bit_exact_observed" -or $k -eq "unswept" -or $k -eq "deferred") {
        # Compact comma-separated list.
        $names = ($list | ForEach-Object { $_.canonical_surface_name })
        [void]$mdLines.Add(($names -join ", "))
        [void]$mdLines.Add("")
    } else {
        [void]$mdLines.Add("| Surface | Streams | Runs seen | Last seen |")
        [void]$mdLines.Add("| --- | --- | ---: | --- |")
        foreach ($r in $list) {
            $lastSeenShort = if ($r.last_seen_date) { ([string]$r.last_seen_date).Substring(0,[Math]::Min(10,([string]$r.last_seen_date).Length)) } else { "" }
            [void]$mdLines.Add("| ``$($r.canonical_surface_name)`` | $($r.linked_open_streams) | $($r.runs_seen) | $lastSeenShort |")
        }
        [void]$mdLines.Add("")
    }
}

$mdLines | Set-Content -LiteralPath $OutputMarkdown -Encoding UTF8
Write-Host "Markdown written: $OutputMarkdown"
Write-Host ""
Write-Host "Tally:"
foreach ($k in $statusOrder) {
    $count = if ($tally.ContainsKey($k)) { $tally[$k] } else { 0 }
    Write-Host "  $k`: $count"
}
Write-Host "  total: $totalRows"
