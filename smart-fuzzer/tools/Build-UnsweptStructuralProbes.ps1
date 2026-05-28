[CmdletBinding(PositionalBinding = $false)]
param(
    [string] $RepoRoot,
    [string] $StatusMapCsv,
    [string] $OutputPath,
    [int]    $MaxArgsToFill = 3
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
$unsweptNames = New-Object 'System.Collections.Generic.HashSet[string]'
foreach ($r in $statusRows) {
    if ([string]$r.status -eq "unswept") { [void]$unsweptNames.Add([string]$r.canonical_surface_name) }
}
Write-Host "Unswept surfaces in status map: $($unsweptNames.Count)"

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
    "LAMBDA","MAKEARRAY","BYROW","BYCOL","MAP","REDUCE","SCAN",
    "GROUPBY","PIVOTBY","ISOMITTED",
    "JIS","PHONETIC","HYPERLINK"
) | ForEach-Object { [void]$ExcludedSurfaces.Add($_) }

$cases = New-Object 'System.Collections.Generic.List[object]'
$skipped = New-Object 'System.Collections.Generic.List[object]'
$byCategoryTranche = @{}

$seq = 0
$sortedNames = @($unsweptNames) | Sort-Object
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
    $functionId = [string]$surf.surface_id

    if ($entryKind -ne "built_in_function") {
        $skipped.Add([ordered]@{ name = $name; reason = "not_a_function ($entryKind)" }) | Out-Null
        continue
    }
    if ($ExcludedSurfaces.Contains($name)) {
        $skipped.Add([ordered]@{ name = $name; reason = "excluded_lambda_locale_or_host" }) | Out-Null
        continue
    }
    if ($prep -ne "ValuesOnlyPreAdapter") {
        $skipped.Add([ordered]@{ name = $name; reason = "arg_prep_profile=$prep (needs ref-aware generator)" }) | Out-Null
        continue
    }
    if ($det -ne "Deterministic") {
        $skipped.Add([ordered]@{ name = $name; reason = "determinism=$det" }) | Out-Null
        continue
    }
    if ($vol -ne "NonVolatile") {
        $skipped.Add([ordered]@{ name = $name; reason = "volatility=$vol" }) | Out-Null
        continue
    }

    $arityMin = Get-ArityInt ([string]$surf.arity.min) 1
    $arityMax = Get-ArityInt ([string]$surf.arity.max) $arityMin
    # Number of required positions to fill for a baseline call.
    $fill = [Math]::Max(0, $arityMin)
    if ($fill -gt $MaxArgsToFill) { $fill = $MaxArgsToFill }

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
            if ($i -eq 0 -and $null -ne $probe.arg0) { $argVec += $probe.arg0 }
            else { $argVec += (New-NumberArg $baselineNumber) }
        }
        $tokens = @()
        foreach ($a in $argVec) { $tokens += (New-ArgFormulaToken $a) }
        $bareName = $name
        $formula = "=$bareName(" + ($tokens -join ",") + ")"

        $seq += 1
        $caseId = ("unswept-{0:D5}-{1}-{2}" -f $seq, ($name.ToLower() -replace '[^a-z0-9]','_'), $probe.tag)
        $case = [ordered]@{
            schema_version = "oxfunc.smart_fuzzer.scenario_seed_case.v0"
            run_id = "assigned_by_runner"
            tranche_id = "unswept-structural-probes-v0"
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
        $trKey = "unswept-structural-" + (($category.ToLower() -replace '[^a-z0-9]+','-').Trim('-'))
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
    tranche_id = "unswept-structural-probes-v0"
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
