[CmdletBinding(PositionalBinding = $false)]
param(
    [string]   $RepoRoot,
    [string]   $StatusMapCsv,
    [string]   $OutputPath,
    [int]      $MaxArgsToFill = 10,
    # Which status-map buckets to target. Default is the original `unswept`
    # set; pass e.g. `scalar_swept_only` to add structural axes to surfaces
    # the broad-scalar numeric runner has already touched but the structural
    # (array/error/blank/coercion) axes have not.
    [string[]] $IncludeStatuses = @("unswept"),
    # Optional explicit surface allowlist (canonical names). When non-empty,
    # only these surfaces are emitted, still subject to the structural-validity
    # filters below. Lets a caller drive a narrow, named tranche.
    [string[]] $OnlySurfaces = @(),
    # Tranche / case-id identity, so a narrow re-run does not collide with the
    # default `unswept-structural-probes-v0` cache and run lineage.
    [string]   $TrancheId = "unswept-structural-probes-v0",
    [string]   $CaseIdPrefix = "unswept"
)

# Synthesize structural-axis probe cases for the unswept value-taking
# surfaces in the function status map. The goal is the structural bug
# class (CHARTER §4.1): does OxFunc agree with Excel on kind / error /
# shape / array-lift / coercion for adversarial inputs, regardless of
# whether the numeric result is "correct".
#
# Scope of this generator (first cut):
#   - surfaces currently classed `unswept` in the status map,
#   - whose dimension-inventory arg_preparation_profile is
#     `ValuesOnlyPreAdapter` (take values, not references),
#   - that are Deterministic + NonVolatile (so a bit-exact comparison
#     is meaningful — RAND/NOW/TODAY/volatile are skipped).
#
# Reference-taking surfaces (lookup/database/OFFSET/INDEX, profile
# `RefsVisibleInAdapter`) are recorded as skipped and left for a
# later reference-aware generator.
#
# Each probe emits an equivalent (formula_text, args) pair so the local
# Rust evaluator and the Excel COM runner see the same invocation. All
# literals are short and bit-exact-safe (CHARTER §4.1 plumbing rule);
# array probes use short integer constants.

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

if ([string]::IsNullOrWhiteSpace($RepoRoot)) {
    $scriptRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
    $RepoRoot = (Resolve-Path (Join-Path $scriptRoot "..\..")).Path
}
$RepoRoot = [System.IO.Path]::GetFullPath($RepoRoot)
if ([string]::IsNullOrWhiteSpace($StatusMapCsv)) {
    $StatusMapCsv = Join-Path $RepoRoot "smart-fuzzer\cache\function-status-map-v0.csv"
}
if ([string]::IsNullOrWhiteSpace($OutputPath)) {
    $OutputPath = Join-Path $RepoRoot "smart-fuzzer\cache\unswept-structural-probes-v0.json"
}

$statusRows = Import-Csv -LiteralPath $StatusMapCsv
$onlySet = New-Object 'System.Collections.Generic.HashSet[string]'
foreach ($o in $OnlySurfaces) { if (-not [string]::IsNullOrWhiteSpace($o)) { [void]$onlySet.Add($o.Trim()) } }
$targetNames = New-Object 'System.Collections.Generic.HashSet[string]'
foreach ($r in $statusRows) {
    $st = [string]$r.status
    $nm = [string]$r.canonical_surface_name
    if ($IncludeStatuses -notcontains $st) { continue }
    if ($onlySet.Count -gt 0 -and -not $onlySet.Contains($nm)) { continue }
    [void]$targetNames.Add($nm)
}
Write-Host "Target surfaces (statuses [$($IncludeStatuses -join ',')]$(if($onlySet.Count){"; allowlist of $($onlySet.Count)"})): $($targetNames.Count)"

$di = Get-Content -Raw (Join-Path $RepoRoot "smart-fuzzer\cache\dimension-inventory-v0.json") | ConvertFrom-Json
$byName = @{}
foreach ($s in $di.surfaces) { $byName[[string]$s.canonical_surface_name] = $s }

function Get-ArityInt {
    param([string] $Raw, [int] $Default)
    if ([string]::IsNullOrWhiteSpace($Raw)) { return $Default }
    $n = 0
    if ([int]::TryParse($Raw, [ref]$n)) { return $n }
    return $Default
}

function New-NumberArg { param([double]$V) return [ordered]@{ kind = "number"; value = $V } }

function New-ArgFormulaToken {
    param($Arg)
    switch ([string]$Arg.kind) {
        "number"  { return [string]$Arg.value }
        "text"    { return '"' + ([string]$Arg.value) + '"' }
        "logical" { if ($Arg.value) { return "TRUE" } else { return "FALSE" } }
        "error"   { return "NA()" }
        "array"   {
            $rowsTokens = @()
            foreach ($row in $Arg.rows) {
                $cellTokens = @()
                foreach ($cell in $row) { $cellTokens += [string]$cell.value }
                $rowsTokens += ($cellTokens -join ",")
            }
            return "{" + ($rowsTokens -join ";") + "}"
        }
        default { return [string]$Arg.value }
    }
}

# A short, bit-exact-safe baseline scalar used to fill required positions.
$baselineNumber = 2

# Surfaces a naive scalar/array probe cannot test honestly, so excluded
# to keep the sweep signal clean:
#   - LAMBDA-family / callable-helper functions need a lambda fixture
#     (deferred per W089); a scalar probe is generator-invalid.
#   - Locale/host-dependent functions resolve differently per build
#     (JIS -> #NAME? in this build; PHONETIC/HYPERLINK need host state).
# ASC/DBCS are kept: they are genuine BUG-FUNC-028 coercion witnesses.
$ExcludedSurfaces = New-Object 'System.Collections.Generic.HashSet[string]'
@(
    "LAMBDA","LET","MAKEARRAY","BYROW","BYCOL","MAP","REDUCE","SCAN",
    "GROUPBY","PIVOTBY","ISOMITTED",
    "JIS","PHONETIC","HYPERLINK",
    # Stochastic surfaces with blank determinism metadata that slip past
    # the determinism filter; they cannot bit-match Excel per-draw.
    "RAND","RANDARRAY","RANDBETWEEN"
) | ForEach-Object { [void]$ExcludedSurfaces.Add($_) }

$cases = New-Object 'System.Collections.Generic.List[object]'
$skipped = New-Object 'System.Collections.Generic.List[object]'
$byCategoryTranche = @{}

$seq = 0
$sortedNames = @($targetNames) | Sort-Object
foreach ($name in $sortedNames) {
  try {
    $surf = $byName[$name]
    if ($null -eq $surf) {
        $skipped.Add([ordered]@{ name = $name; reason = "not_in_dimension_inventory" }) | Out-Null
        continue
    }
    $fs = $surf.function_surface
    $prep = [string]$fs.arg_preparation_profile
    $det = [string]$fs.determinism_class
    $vol = [string]$fs.volatility_class
    $entryKind = [string]$fs.entry_kind
    $category = [string]$surf.category

    # A canonical_surface_name may bundle aliases (e.g. "FIND, FINDB").
    # Use the first alias as the Excel formula name and the local
    # function id, so the formula is valid and dispatch resolves.
    $bareName = ($name -split ',')[0].Trim()
    $functionId = "FUNC.$bareName"

    if ($entryKind -ne "built_in_function") {
        $skipped.Add([ordered]@{ name = $name; reason = "not_a_function ($entryKind)" }) | Out-Null
        continue
    }
    if ($ExcludedSurfaces.Contains($name) -or $ExcludedSurfaces.Contains($bareName)) {
        $skipped.Add([ordered]@{ name = $name; reason = "excluded_lambda_locale_or_host" }) | Out-Null
        continue
    }
    # Reference-taking surfaces need a fixture-aware generator; skip here.
    if ($prep -eq "RefsVisibleInAdapter") {
        $skipped.Add([ordered]@{ name = $name; reason = "arg_prep_profile=RefsVisibleInAdapter (needs ref-aware generator)" }) | Out-Null
        continue
    }
    # ValuesOnlyPreAdapter and empty/unknown profiles are attempted; an
    # unknown profile that turns out to need references will surface as
    # harness_blocked rather than a false bug.
    #
    # Determinism/volatility are filtered by KNOWN-bad rather than
    # required-good, because many curated rows leave these fields blank.
    # A blank (unknown) field is attempted; only explicitly stochastic /
    # time / event / volatile surfaces are excluded (they cannot bit-match).
    $nonDeterministic = @("PseudoRandom","TimeDependent","ExternalEventDependent")
    if ($nonDeterministic -contains $det) {
        $skipped.Add([ordered]@{ name = $name; reason = "determinism=$det" }) | Out-Null
        continue
    }
    if ($vol -eq "VolatileContextual" -or $vol -eq "Volatile") {
        $skipped.Add([ordered]@{ name = $name; reason = "volatility=$vol" }) | Out-Null
        continue
    }

    $arityMin = Get-ArityInt ([string]$surf.arity.min) 1
    $arityMax = Get-ArityInt ([string]$surf.arity.max) $arityMin
    # Fill the full required arity (capped only to avoid pathological
    # variadic blowups). Under-filling produced invalid Excel calls that
    # the harness flagged as blocked rather than testing the function.
    $fill = [Math]::Max(0, $arityMin)
    if ($fill -gt $MaxArgsToFill) { $fill = $MaxArgsToFill }

    # Type-aware baseline: text-category functions take a text first
    # argument; everything else takes a short number. Later positions
    # are short numbers (most are scalar/position-like).
    $isTextCategory = ($category -eq "Text functions")
    $baselineArg0 = if ($isTextCategory) { [ordered]@{ kind = "text"; value = "abc" } } else { New-NumberArg $baselineNumber }

    # Probe definitions: each replaces position 0 (when present) with a
    # structural stressor; baseline keeps the all-number vector.
    $probes = New-Object 'System.Collections.Generic.List[object]'
    [void]$probes.Add(@{ tag = "baseline_scalar"; arg0 = $null })
    if ($fill -ge 1) {
        [void]$probes.Add(@{ tag = "arg0_array_lift";  arg0 = [ordered]@{ kind = "array"; rows = @(@([ordered]@{kind="number";value=2}), @([ordered]@{kind="number";value=3})) } })
        [void]$probes.Add(@{ tag = "arg0_error_na";    arg0 = [ordered]@{ kind = "error"; code = "NA" } })
        [void]$probes.Add(@{ tag = "arg0_empty_text";  arg0 = [ordered]@{ kind = "text"; value = "" } })
        [void]$probes.Add(@{ tag = "arg0_text_number"; arg0 = [ordered]@{ kind = "text"; value = "2" } })
        [void]$probes.Add(@{ tag = "arg0_logical";     arg0 = [ordered]@{ kind = "logical"; value = $true } })
    }

    foreach ($probe in $probes) {
        $argVec = @()
        for ($i = 0; $i -lt $fill; $i++) {
            if ($i -eq 0) {
                if ($null -ne $probe.arg0) { $argVec += $probe.arg0 }
                else { $argVec += $baselineArg0 }
            } else {
                $argVec += (New-NumberArg $baselineNumber)
            }
        }
        $tokens = @()
        foreach ($a in $argVec) { $tokens += (New-ArgFormulaToken $a) }
        $formula = "=$bareName(" + ($tokens -join ",") + ")"

        $seq += 1
        $caseId = ("{0}-{1:D5}-{2}-{3}" -f $CaseIdPrefix, $seq, ($name.ToLower() -replace '[^a-z0-9]','_'), $probe.tag)
        $case = [ordered]@{
            schema_version = "oxfunc.smart_fuzzer.scenario_seed_case.v0"
            run_id = "assigned_by_runner"
            tranche_id = $TrancheId
            case_id = $caseId
            function_id = $functionId
            canonical_surface_name = $name
            case_tag = "structural_probe:$($probe.tag)"
            axis = "unswept_structural_probe"
            expected_probe_class = "structural_axis"
            formula_text = $formula
            args = $argVec
            category = $category
            blocked_or_deferred_lanes = @()
            known_deviation_tags = @()
        }
        $cases.Add($case) | Out-Null
        $trKey = "$CaseIdPrefix-structural-" + (($category.ToLower() -replace '[^a-z0-9]+','-').Trim('-'))
        if (-not $byCategoryTranche.ContainsKey($trKey)) {
            $byCategoryTranche[$trKey] = New-Object 'System.Collections.Generic.List[string]'
        }
        $byCategoryTranche[$trKey].Add($caseId) | Out-Null
    }
  } catch {
    $skipped.Add([ordered]@{ name = $name; reason = "generator_error: $($_.Exception.Message)" }) | Out-Null
  }
}

$tranches = New-Object 'System.Collections.Generic.List[object]'
foreach ($k in ($byCategoryTranche.Keys | Sort-Object)) {
    $tranches.Add([ordered]@{ tranche_id = $k; case_ids = $byCategoryTranche[$k].ToArray() }) | Out-Null
}
$surfacesCovered = (@($cases | ForEach-Object { $_.canonical_surface_name } | Sort-Object -Unique)).Count

$out = [ordered]@{
    schema_version = "oxfunc.smart_fuzzer.scenario_seed_case_set.v0"
    authority = "non_semantic_exploration_input"
    generated_utc = (Get-Date).ToUniversalTime().ToString("o")
    generator = "Build-UnsweptStructuralProbes.ps1"
    tranche_id = $TrancheId
    comparison_policy = "exact_typed_bit_match_no_tolerance"
    cases = $cases.ToArray()
    tranches = $tranches.ToArray()
    skipped = $skipped.ToArray()
    summary = [ordered]@{
        case_count = $cases.Count
        tranche_count = $tranches.Count
        skipped_count = $skipped.Count
        surfaces_covered = $surfacesCovered
    }
}
$out | ConvertTo-Json -Depth 12 | Set-Content -LiteralPath $OutputPath -Encoding UTF8

Write-Host "Cases:    $($cases.Count)"
Write-Host "Surfaces: $((@($cases | ForEach-Object { $_.canonical_surface_name } | Sort-Object -Unique)).Count)"
Write-Host "Tranches: $($tranches.Count)"
Write-Host "Skipped:  $($skipped.Count)"
Write-Host "Output:   $OutputPath"
Write-Host ""
Write-Host "Skip reasons:"
$skipped | Group-Object reason | Sort-Object Count -Descending | Select-Object -First 12 | ForEach-Object { "  {0,4}  {1}" -f $_.Count, $_.Name }
